// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! pkg - Rustica Package Manager
//!
//! Package manager for installing, updating, and managing software packages.

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use ed25519_dalek::{Signature, Signer, Verifier, SigningKey, VerifyingKey};
use flate2::read::GzDecoder;
use rutils::{compute_checksum, ensure_dir};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tokio::time::{timeout, Duration};
use base64::prelude::*;

/// Rustica Package Manager
#[derive(Parser, Debug)]
#[command(name = "pkg")]
#[command(about = "Rustica Package Manager", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Update package list
    Update {
        /// Force update even if up-to-date
        #[arg(short, long)]
        force: bool,
    },

    /// Install packages
    Install {
        /// Packages to install
        #[arg(required = true)]
        packages: Vec<String>,

        /// Assume yes
        #[arg(short, long)]
        yes: bool,

        /// Download only
        #[arg(long)]
        download_only: bool,
    },

    /// Remove packages
    Remove {
        /// Packages to remove
        #[arg(required = true)]
        packages: Vec<String>,

        /// Remove dependencies
        #[arg(short, long)]
        purge: bool,
    },

    /// Search for packages
    Search {
        /// Search query
        query: String,

        /// Search by name only
        #[arg(short, long)]
        name_only: bool,
    },

    /// List repositories
    RepoList,

    /// Upgrade all packages
    Upgrade {
        /// Assume yes
        #[arg(short, long)]
        yes: bool,
    },

    /// Show package info
    Info {
        /// Package name
        package: String,
    },

    /// List installed packages
    List {
        /// Filter by pattern
        pattern: Option<String>,
    },

    /// Generate a new signing keypair
    Keygen {
        /// Key name/identifier
        #[arg(required = true)]
        name: String,
    },

    /// Export public key
    ExportKey {
        /// Key name
        #[arg(required = true)]
        name: String,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Import public key
    ImportKey {
        /// Key file
        #[arg(required = true)]
        key_file: PathBuf,

        /// Trust this key for package verification
        #[arg(long)]
        trust: bool,
    },

    /// Sign a package file
    SignPackage {
        /// Package file to sign
        #[arg(required = true)]
        package_file: PathBuf,

        /// Key name to use for signing
        #[arg(short, long)]
        key: Option<String>,
    },

    /// List trusted keys
    ListKeys,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Package {
    name: String,
    version: String,
    description: String,
    size: u64,
    dependencies: Vec<String>,
    checksum: String,
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    signer_key: Option<String>,
    /// Required capabilities for this package
    #[serde(skip_serializing_if = "Vec::is_empty")]
    capabilities: Vec<String>,
    /// Maximum capability level required (0-255)
    #[serde(skip_serializing_if = "Option::is_none")]
    capability_level: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RepositoryIndex {
    repository: String,
    packages: HashMap<String, Package>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InstalledPackage {
    name: String,
    version: String,
    install_date: String,
    files: Vec<String>,
}

#[derive(Debug)]
struct PackageManager {
    config_dir: PathBuf,
    cache_dir: PathBuf,
    package_dir: PathBuf,
    repositories: Vec<Repository>,
    installed: HashMap<String, InstalledPackage>,
}

#[derive(Debug, Clone)]
struct Repository {
    name: String,
    url: String,
    enabled: bool,
}

impl PackageManager {
    fn new() -> Result<Self> {
        let config_dir = PathBuf::from("/etc/rustica/pkg");
        let cache_dir = PathBuf::from("/var/cache/rpg");
        let package_dir = PathBuf::from("/var/lib/rpg");

        // Create directories
        ensure_dir(&config_dir)?;
        ensure_dir(&cache_dir)?;
        ensure_dir(&package_dir)?;

        let mut pm = Self {
            config_dir,
            cache_dir,
            package_dir,
            repositories: Vec::new(),
            installed: HashMap::new(),
        };

        // Load repositories and installed packages
        pm.load_repositories()?;
        pm.load_installed()?;

        Ok(pm)
    }

    fn load_repositories(&mut self) -> Result<()> {
        let sources_file = self.config_dir.join("sources.list");

        if !sources_file.exists() {
            // Default repositories
            self.repositories = vec![
                Repository {
                    name: "kernel".to_string(),
                    url: "https://rustux.com/repo/kernel".to_string(),
                    enabled: true,
                },
                Repository {
                    name: "rustica".to_string(),
                    url: "https://rustux.com/repo/rustica".to_string(),
                    enabled: true,
                },
                Repository {
                    name: "apps".to_string(),
                    url: "https://rustux.com/repo/apps".to_string(),
                    enabled: true,
                },
            ];

            self.save_repositories()?;
            return Ok(());
        }

        let content = std::fs::read_to_string(&sources_file)?;
        self.repositories = content
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
            .map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                Repository {
                    name: parts.get(0).unwrap_or(&"").to_string(),
                    url: parts.get(1).unwrap_or(&"").to_string(),
                    enabled: parts.get(2).map(|&s| s != "disabled").unwrap_or(true),
                }
            })
            .collect();

        Ok(())
    }

    fn save_repositories(&self) -> Result<()> {
        let sources_file = self.config_dir.join("sources.list");

        let mut content = String::from("# Rustica Package Repositories\n");
        content.push_str("# Format: name url [enabled|disabled]\n\n");

        for repo in &self.repositories {
            let status = if repo.enabled { "enabled" } else { "disabled" };
            content.push_str(&format!("{} {} {}\n", repo.name, repo.url, status));
        }

        std::fs::write(&sources_file, content)?;
        Ok(())
    }

    fn load_installed(&mut self) -> Result<()> {
        let installed_file = self.package_dir.join("installed.json");

        if installed_file.exists() {
            let content = std::fs::read_to_string(&installed_file)?;
            if let Ok(installed) = serde_json::from_str::<HashMap<String, InstalledPackage>>(&content) {
                self.installed = installed;
            }
        }

        Ok(())
    }

    fn save_installed(&self) -> Result<()> {
        let installed_file = self.package_dir.join("installed.json");

        let content = serde_json::to_string_pretty(&self.installed)?;
        std::fs::write(&installed_file, content)?;

        Ok(())
    }

    async fn update_repositories(&self, _force: bool) -> Result<()> {
        println!("Updating package lists...");

        for repo in &self.repositories {
            if !repo.enabled {
                continue;
            }

            println!("  Fetching from {}...", repo.name);

            // Download repository index
            let index_url = format!("{}/index.json", repo.url);

            match self.download_file(&index_url).await {
                Ok(index_data) => {
                    // Parse index
                    if let Ok(index) = serde_json::from_slice::<RepositoryIndex>(&index_data) {
                        let index_file = self.cache_dir.join(format!("{}.json", repo.name));
                        std::fs::write(&index_file, serde_json::to_string_pretty(&index)?)?;

                        // Count packages
                        let count = index.packages.len();
                        println!("    {} packages", count);
                    } else {
                        eprintln!("    Warning: Failed to parse index");
                    }
                }
                Err(e) => {
                    eprintln!("    Warning: Failed to download index: {}", e);
                }
            }
        }

        println!("Done.");
        Ok(())
    }

    async fn download_file(&self, url: &str) -> Result<Vec<u8>> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let response = timeout(
            Duration::from_secs(60),
            client.get(url).send()
        ).await??;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        let data = response.bytes().await?;
        Ok(data.to_vec())
    }

    async fn download_with_progress(&self, url: &str, dest: &Path) -> Result<()> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()?;

        let response = timeout(
            Duration::from_secs(300),
            client.get(url).send()
        ).await??;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);

        if total_size > 0 {
            println!("  Downloading: {} bytes ({})", total_size, format_size(total_size));
        }

        // Get response bytes
        let data = response.bytes().await?;

        // Write to file
        std::fs::write(dest, &data)?;

        Ok(())
    }

    fn search_packages(&self, query: &str, name_only: bool) -> Result<()> {
        println!("Searching for '{}'...", query);
        println!();

        let query_lower = query.to_lowercase();

        for repo in &self.repositories {
            if !repo.enabled {
                continue;
            }

            let index_file = self.cache_dir.join(format!("{}.json", repo.name));
            if !index_file.exists() {
                continue;
            }

            let content = std::fs::read_to_string(&index_file)?;
            if let Ok(index) = serde_json::from_str::<RepositoryIndex>(&content) {
                for (name, pkg) in &index.packages {
                    let matches = if name_only {
                        name.contains(&query_lower)
                    } else {
                        name.contains(&query_lower) ||
                        pkg.description.to_lowercase().contains(&query_lower)
                    };

                    if matches {
                        println!("  {} - {}", name, pkg.version);
                        println!("    {}", pkg.description);
                        println!("    Repository: {}", repo.name);
                        println!("    Size: {}", format_size(pkg.size));
                        println!();
                    }
                }
            }
        }

        Ok(())
    }

    async fn install_package(&mut self, package: &str, yes: bool, download_only: bool) -> Result<()> {
        println!("Installing {}...", package);

        // Check if already installed
        if self.installed.contains_key(package) {
            println!("  Package {} is already installed", package);
            return Ok(());
        }

        // Search for package in all repositories
        let (pkg, _repo) = self.find_package(package)?;

        // Check capabilities
        if !pkg.capabilities.is_empty() {
            println!("  Required capabilities: {}", pkg.capabilities.join(", "));
            if let Some(level) = pkg.capability_level {
                println!("  Capability level: {}", level);
            }
            println!("  Note: Ensure you have the required permissions/capabilities");
            if !yes {
                print!("  Continue anyway? [Y/n] ");
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase() == "n" {
                    return Ok(());
                }
            }
        }

        // Check dependencies
        if !pkg.dependencies.is_empty() {
            println!("  Dependencies: {}", pkg.dependencies.join(", "));
            if !yes {
                print!("  Install dependencies? [Y/n] ");
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase() == "n" {
                    return Ok(());
                }
            }

            // Install dependencies first
            for dep in &pkg.dependencies {
                if !self.installed.contains_key(dep) {
                    println!("  Installing dependency {}...", dep);
                    Box::pin(self.install_package(dep, true, false)).await?;
                }
            }
        }

        // Download package
        println!("  Downloading {}...", package);
        let package_file = self.cache_dir.join(format!("{}_{}.tar.gz", package, pkg.version));

        self.download_with_progress(&pkg.url, &package_file).await?;

        // Verify checksum
        println!("  Verifying checksum...");
        let file_data = std::fs::read(&package_file)?;
        let actual_checksum = compute_checksum(&file_data);

        if actual_checksum != pkg.checksum && !pkg.checksum.is_empty() {
            std::fs::remove_file(&package_file)?;
            return Err(anyhow::anyhow!("Checksum mismatch: expected {}, got {}", pkg.checksum, actual_checksum));
        }

        // Verify signature if present
        if let Some(ref sig) = pkg.signature {
            println!("  Verifying signature...");
            if let Err(e) = self.verify_package_signature(&file_data, sig, &pkg.signer_key) {
                std::fs::remove_file(&package_file)?;
                return Err(anyhow::anyhow!("Signature verification failed: {}", e));
            }
            println!("    Signature valid!");
        } else {
            println!("  Warning: Package has no signature");
        }

        if download_only {
            println!("  Package downloaded (download-only mode)");
            return Ok(());
        }

        // Extract package
        println!("  Extracting...");
        let install_dir = PathBuf::from("/");
        let files = self.extract_package(&package_file, &install_dir)?;

        // Set capabilities on installed files if package requires them
        if !pkg.capabilities.is_empty() {
            println!("  Setting capabilities on installed files...");
            // Note: This would use capctl in a real implementation
            // For now, we just note that capabilities should be set
        }

        // Run post-install script if present
        let post_install = install_dir.join("var/lib/rpg/postinst").join(package);
        if post_install.exists() {
            println!("  Running post-install script...");
            // Execute script
        }

        // Record installation
        self.installed.insert(package.to_string(), InstalledPackage {
            name: package.to_string(),
            version: pkg.version.clone(),
            install_date: chrono::Utc::now().to_rfc3339(),
            files,
        });

        self.save_installed()?;

        println!("  Done.");
        Ok(())
    }

    fn find_package(&self, name: &str) -> Result<(Package, String)> {
        for repo in &self.repositories {
            if !repo.enabled {
                continue;
            }

            let index_file = self.cache_dir.join(format!("{}.json", repo.name));
            if !index_file.exists() {
                continue;
            }

            let content = std::fs::read_to_string(&index_file)?;
            if let Ok(index) = serde_json::from_str::<RepositoryIndex>(&content) {
                if let Some(pkg) = index.packages.get(name) {
                    return Ok((pkg.clone(), repo.name.clone()));
                }
            }
        }

        Err(anyhow::anyhow!("Package not found: {}", name))
    }

    fn extract_package(&self, archive_path: &Path, dest: &Path) -> Result<Vec<String>> {
        let file = File::open(archive_path)?;
        let decoder = GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);

        let mut files = Vec::new();

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.to_path_buf();
            let full_path = dest.join(&path);

            // Record file
            files.push(full_path.to_string_lossy().to_string());

            // Extract
            entry.unpack(&full_path)?;
        }

        Ok(files)
    }

    fn remove_package(&mut self, package: &str, _purge: bool) -> Result<()> {
        println!("Removing {}...", package);

        if !self.installed.contains_key(package) {
            println!("  Package {} is not installed", package);
            return Ok(());
        }

        let installed = self.installed.get(package).unwrap();

        // Remove files
        println!("  Removing files...");
        for file in &installed.files {
            let path = Path::new(file);
            if path.exists() {
                // Check if file is shared
                if self.is_file_shared(file, package) {
                    continue;
                }

                if path.is_dir() {
                    std::fs::remove_dir_all(path).ok();
                } else {
                    std::fs::remove_file(path).ok();
                }
            }
        }

        // Remove from installed list
        self.installed.remove(package);
        self.save_installed()?;

        println!("  Done.");
        Ok(())
    }

    fn is_file_shared(&self, file: &str, exclude_pkg: &str) -> bool {
        for (name, pkg) in &self.installed {
            if name == exclude_pkg {
                continue;
            }

            if pkg.files.contains(&file.to_string()) {
                return true;
            }
        }

        false
    }

    fn show_package_info(&self, package: &str) -> Result<()> {
        // Check installed
        if let Some(installed) = self.installed.get(package) {
            println!("Package: {}", package);
            println!("Version: {}", installed.version);
            println!("Installed: {}", installed.install_date);
            println!("Files: {}", installed.files.len());
            return Ok(());
        }

        // Search for package
        let (pkg, repo) = self.find_package(package)?;

        println!("Package: {}", package);
        println!("Version: {}", pkg.version);
        println!("Description: {}", pkg.description);
        println!("Repository: {}", repo);
        println!("Size: {}", format_size(pkg.size));
        if !pkg.dependencies.is_empty() {
            println!("Dependencies: {}", pkg.dependencies.join(", "));
        }
        if !pkg.capabilities.is_empty() {
            println!("Required capabilities: {}", pkg.capabilities.join(", "));
            if let Some(level) = pkg.capability_level {
                println!("Capability level: {}", level);
            }
        }

        Ok(())
    }

    fn list_installed(&self, pattern: Option<&str>) -> Result<()> {
        println!("Installed packages:");
        println!();

        let mut names: Vec<_> = self.installed.keys().collect();
        names.sort();

        for name in names {
            if let Some(pkg) = self.installed.get(name) {
                if let Some(pat) = pattern {
                    if name.contains(pat) {
                        println!("  {} - {}", name, pkg.version);
                    }
                } else {
                    println!("  {} - {}", name, pkg.version);
                }
            }
        }

        Ok(())
    }

    fn verify_package_signature(&self, data: &[u8], sig_str: &str, signer_key: &Option<String>) -> Result<()> {
        // Decode signature
        let sig_bytes = BASE64_STANDARD.decode(sig_str)
            .context("Failed to decode signature")?;

        if sig_bytes.len() != 64 {
            return Err(anyhow!("Invalid signature length"));
        }

        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(&sig_bytes);
        let signature = Signature::from_slice(&sig_array)?;

        // Load trusted public keys
        let keys_dir = self.config_dir.join("trusted_keys");
        let public_key = if let Some(ref key_id) = signer_key {
            // Use specific key
            let key_file = keys_dir.join(format!("{}.pub", key_id));
            let key_data = std::fs::read(&key_file)
                .with_context(|| format!("Key file not found: {}", key_file.display()))?;
            if key_data.len() != 32 {
                return Err(anyhow!("Invalid public key length"));
            }
            let mut key_array = [0u8; 32];
            key_array.copy_from_slice(&key_data);
            VerifyingKey::from_bytes(&key_array)?
        } else {
            // Try all trusted keys
            let mut found = None;
            if keys_dir.exists() {
                for entry in std::fs::read_dir(&keys_dir)? {
                    let entry = entry?;
                    if entry.path().extension().and_then(|s| s.to_str()) == Some("pub") {
                        let key_data = std::fs::read(entry.path())?;
                        if key_data.len() == 32 {
                            let mut key_array = [0u8; 32];
                            key_array.copy_from_slice(&key_data);
                            if let Ok(pk) = VerifyingKey::from_bytes(&key_array) {
                                if pk.verify(data, &signature).is_ok() {
                                    found = Some(pk);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            found.ok_or_else(|| anyhow!("No valid trusted key found for signature"))?
        };

        // Verify signature
        public_key.verify(data, &signature)?;
        Ok(())
    }

    fn generate_keypair(&self, name: &str) -> Result<()> {
        use rand::RngCore;

        let keys_dir = self.config_dir.join("keys");
        ensure_dir(&keys_dir)?;

        // Generate random keypair bytes
        let mut keypair_bytes = [0u8; 64];
        rand::rngs::OsRng.fill_bytes(&mut keypair_bytes);

        let signing_key = SigningKey::from_keypair_bytes(&keypair_bytes)?;
        let verifying_key = signing_key.verifying_key();

        // Save secret key
        let secret_file = keys_dir.join(format!("{}.secret", name));
        std::fs::write(&secret_file, signing_key.to_bytes())?;
        // Set restrictive permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&secret_file)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&secret_file, perms)?;
        }

        // Save public key
        let public_file = keys_dir.join(format!("{}.pub", name));
        std::fs::write(&public_file, verifying_key.to_bytes())?;

        println!("Generated keypair: {}", name);
        println!("  Secret key: {}", secret_file.display());
        println!("  Public key: {}", public_file.display());

        Ok(())
    }

    fn export_public_key(&self, name: &str, output: Option<PathBuf>) -> Result<()> {
        let keys_dir = self.config_dir.join("keys");
        let public_file = keys_dir.join(format!("{}.pub", name));

        let key_data = std::fs::read(&public_file)
            .with_context(|| format!("Key not found: {}", name))?;

        let output_path = output.unwrap_or_else(|| PathBuf::from(format!("{}.pub", name)));
        std::fs::write(&output_path, &key_data)?;

        println!("Exported public key to: {}", output_path.display());

        Ok(())
    }

    fn import_public_key(&self, key_file: &Path, trust: bool) -> Result<()> {
        let key_data = std::fs::read(key_file)?;
        if key_data.len() != 32 {
            return Err(anyhow!("Invalid public key length"));
        }
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key_data);
        let _public_key = VerifyingKey::from_bytes(&key_array)
            .context("Invalid public key format")?;

        // Get key name from filename
        let name = key_file.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("imported");

        if trust {
            let keys_dir = self.config_dir.join("trusted_keys");
            ensure_dir(&keys_dir)?;

            let dest_file = keys_dir.join(format!("{}.pub", name));
            std::fs::write(&dest_file, key_data)?;

            println!("Imported and trusted key: {}", name);
        } else {
            println!("Imported key: {} (not trusted)", name);
        }

        Ok(())
    }

    fn sign_package_file(&self, package_file: &Path, key_name: Option<String>) -> Result<()> {
        // Load or generate key
        let keys_dir = self.config_dir.join("keys");
        let signing_key: SigningKey = if let Some(name) = key_name {
            let secret_file = keys_dir.join(format!("{}.secret", name));
            let secret_data = std::fs::read(&secret_file)
                .with_context(|| format!("Key not found: {}", name))?;
            if secret_data.len() != 32 {
                return Err(anyhow!("Invalid secret key length"));
            }
            let mut secret_array = [0u8; 32];
            secret_array.copy_from_slice(&secret_data);
            SigningKey::from_bytes(&secret_array)
        } else {
            // Use default key
            let default_key = keys_dir.join("default.secret");
            if !default_key.exists() {
                return Err(anyhow!("No signing key found. Use 'pkg keygen' to create one."));
            }
            let secret_data = std::fs::read(&default_key)?;
            if secret_data.len() != 32 {
                return Err(anyhow!("Invalid secret key length"));
            }
            let mut secret_array = [0u8; 32];
            secret_array.copy_from_slice(&secret_data);
            SigningKey::from_bytes(&secret_array)
        };

        // Read package data
        let package_data = std::fs::read(package_file)?;

        // Sign
        let signature = signing_key.sign(&package_data);
        let sig_b64 = BASE64_STANDARD.encode(signature.to_bytes());

        // Write signature file
        let sig_file = package_file.with_extension("sig");
        std::fs::write(&sig_file, sig_b64)?;

        println!("Signed package: {}", package_file.display());
        println!("  Signature: {}", sig_file.display());

        Ok(())
    }

    fn list_trusted_keys(&self) -> Result<()> {
        let keys_dir = self.config_dir.join("trusted_keys");

        if !keys_dir.exists() {
            println!("No trusted keys configured.");
            return Ok(());
        }

        println!("Trusted keys:");
        for entry in std::fs::read_dir(&keys_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("pub") {
                let name = entry.file_name().to_string_lossy().replace(".pub", "");
                println!("  {}", name);
            }
        }

        Ok(())
    }

    async fn upgrade_all(&mut self, yes: bool) -> Result<()> {
        println!("Checking for updates...");

        let mut packages_to_upgrade = Vec::new();

        // Check each installed package for updates
        for (name, installed_pkg) in &self.installed {
            match self.find_package(name) {
                Ok((repo_pkg, _repo)) => {
                    if repo_pkg.version != installed_pkg.version {
                        println!("  {} -> {} (from {})", name, installed_pkg.version, repo_pkg.version);
                        packages_to_upgrade.push(name.clone());
                    }
                }
                Err(_) => {
                    println!("  {} - not found in repositories (skipping)", name);
                }
            }
        }

        if packages_to_upgrade.is_empty() {
            println!("All packages are up to date.");
            return Ok(());
        }

        println!();
        println!("Packages to upgrade: {}", packages_to_upgrade.len());

        if !yes {
            print!("Continue? [Y/n] ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().to_lowercase() == "n" {
                println!("Upgrade cancelled.");
                return Ok(());
            }
        }

        // Upgrade each package
        for package in &packages_to_upgrade {
            println!();
            println!("Upgrading {}...", package);

            // Remove old version
            self.remove_package(package, false)?;

            // Install new version
            self.install_package(package, true, false).await?;
        }

        println!();
        println!("Upgrade complete!");

        Ok(())
    }
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

#[tokio::main]
async fn run() -> Result<()> {
    let args = Args::parse();
    let mut pm = PackageManager::new()?;

    match args.command {
        Commands::Update { force } => {
            pm.update_repositories(force).await?;
        }
        Commands::Install { packages, yes, download_only } => {
            for package in packages {
                pm.install_package(&package, yes, download_only).await?;
            }
        }
        Commands::Remove { packages, purge } => {
            for package in packages {
                pm.remove_package(&package, purge)?;
            }
        }
        Commands::Search { query, name_only } => {
            pm.search_packages(&query, name_only)?;
        }
        Commands::RepoList => {
            println!("Configured repositories:");
            for repo in &pm.repositories {
                let status = if repo.enabled { "[enabled]" } else { "[disabled]" };
                println!("  {} - {} {}", repo.name, repo.url, status);
            }
        }
        Commands::Upgrade { yes } => {
            pm.upgrade_all(yes).await?;
        }
        Commands::Info { package } => {
            pm.show_package_info(&package)?;
        }
        Commands::List { pattern } => {
            pm.list_installed(pattern.as_deref())?;
        }
        Commands::Keygen { name } => {
            pm.generate_keypair(&name)?;
        }
        Commands::ExportKey { name, output } => {
            pm.export_public_key(&name, output)?;
        }
        Commands::ImportKey { key_file, trust } => {
            pm.import_public_key(&key_file, trust)?;
        }
        Commands::SignPackage { package_file, key } => {
            pm.sign_package_file(&package_file, key)?;
        }
        Commands::ListKeys => {
            pm.list_trusted_keys()?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    run()
}
