// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! sbctl - Secure Boot Control Utility
//!
//! Manage secure boot keys, sign kernels and bootloaders, and manage
//! the UEFI secure boot database for Rustica OS.

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Secure Boot Control Utility
#[derive(Parser, Debug)]
#[command(name = "rustux-sbctl")]
#[command(author = "The Rustux Authors")]
#[command(version = "0.1.0")]
#[command(about = "Secure boot control utility", long_about = None)]
struct SbctlArgs {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create new secure boot keys
    CreateKeys {
        /// Directory to store keys
        #[arg(short, long, default_value = "/var/lib/sbctl/keys")]
        directory: PathBuf,
    },

    /// Sign a binary file (kernel, bootloader, etc.)
    Sign {
        /// Binary file to sign
        #[arg(required = true)]
        binary: PathBuf,

        /// Output file (default: add .signed suffix)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Keys directory
        #[arg(short, long, default_value = "/var/lib/sbctl/keys")]
        keys_dir: PathBuf,
    },

    /// Verify a signed binary
    Verify {
        /// Signed binary to verify
        #[arg(required = true)]
        binary: PathBuf,

        /// Keys directory
        #[arg(short, long, default_value = "/var/lib/sbctl/keys")]
        keys_dir: PathBuf,
    },

    /// List all keys
    ListKeys {
        /// Keys directory
        #[arg(short, long, default_value = "/var/lib/sbctl/keys")]
        keys_dir: PathBuf,
    },

    /// Export key to ESP (EFI System Partition)
    ExportKey {
        /// Key type (db, KEK, pk)
        #[arg(required = true)]
        key_type: String,

        /// ESP mount point
        #[arg(short, long, default_value = "/boot/efi")]
        esp: PathBuf,
    },

    /// Sign all kernels in /boot
    SignKernels {
        /// Keys directory
        #[arg(short, long, default_value = "/var/lib/sbctl/keys")]
        keys_dir: PathBuf,

        /// Boot directory
        #[arg(short, long, default_value = "/boot")]
        boot_dir: PathBuf,
    },

    /// Show secure boot status
    Status,

    /// Generate UUID for database entries
    GenerateUuid,
}

/// Key types for secure boot
#[derive(Debug, Clone)]
enum KeyType {
    PK,  // Platform Key (highest authority)
    KEK, // Key Exchange Key
    DB,  // Database key (for allowed signatures)
}

impl KeyType {
    fn from_str(s: &str) -> Result<KeyType> {
        match s.to_uppercase().as_str() {
            "PK" => Ok(KeyType::PK),
            "KEK" => Ok(KeyType::KEK),
            "DB" => Ok(KeyType::DB),
            _ => Err(anyhow!("Invalid key type: {}", s)),
        }
    }

    fn description(&self) -> &str {
        match self {
            KeyType::PK => "Platform Key - highest authority in secure boot",
            KeyType::KEK => "Key Exchange Key - signs database updates",
            KeyType::DB => "Database Key - signs allowed bootloaders/kernels",
        }
    }
}

fn main() -> Result<()> {
    let args = SbctlArgs::parse();

    match args.command {
        Commands::CreateKeys { directory } => {
            create_keys(&directory)?;
        }
        Commands::Sign { binary, output, keys_dir } => {
            sign_binary(&binary, output.as_deref(), &keys_dir)?;
        }
        Commands::Verify { binary, keys_dir } => {
            verify_binary(&binary, &keys_dir)?;
        }
        Commands::ListKeys { keys_dir } => {
            list_keys(&keys_dir)?;
        }
        Commands::ExportKey { key_type, esp } => {
            export_key(&key_type, &esp)?;
        }
        Commands::SignKernels { keys_dir, boot_dir } => {
            sign_kernels(&keys_dir, &boot_dir)?;
        }
        Commands::Status => {
            show_status()?;
        }
        Commands::GenerateUuid => {
            generate_uuid()?;
        }
    }

    Ok(())
}

fn create_keys(directory: &Path) -> Result<()> {
    println!("Creating secure boot keys in {}...", directory.display());

    // Create directory
    fs::create_dir_all(directory)
        .context("Failed to create keys directory")?;

    // Check if keys already exist
    let pk_key = directory.join("PK.key");
    let pk_cert = directory.join("PK.crt");

    if pk_key.exists() && pk_cert.exists() {
        println!("  Keys already exist!");
        println!("  To recreate, remove the existing keys first.");
        return Ok(());
    }

    // Create PK (Platform Key)
    println!("  Creating Platform Key (PK)...");
    create_key(directory, "PK", 3650)?;

    // Create KEK (Key Exchange Key)
    println!("  Creating Key Exchange Key (KEK)...");
    create_key(directory, "KEK", 3650)?;

    // Create DB (Database Key)
    println!("  Creating Database Key (DB)...");
    create_key(directory, "DB", 3650)?;

    // Create GUID for database entries
    println!("  Generating GUID...");
    let guid_path = directory.join("GUID");
    if !guid_path.exists() {
        let guid = generate_uuid_value()?;
        fs::write(&guid_path, &guid)?;
        println!("    GUID: {}", guid);
    }

    println!();
    println!("Keys created successfully!");
    println!();
    println!("Next steps:");
    println!("  1. Enroll the keys in your firmware setup:");
    println!("     rustux-sbctl export-key PK /boot/efi");
    println!("     rustux-sbctl export-key KEK /boot/efi");
    println!("     rustux-sbctl export-key DB /boot/efi");
    println!("  2. Sign your kernels and bootloaders:");
    println!("     rustux-sbctl sign-kernels");

    Ok(())
}

fn create_key(directory: &Path, name: &str, days: u32) -> Result<()> {
    let key_file = directory.join(format!("{}.key", name));
    let cert_file = directory.join(format!("{}.crt", name));
    let der_file = directory.join(format!("{}.der", name));

    // Generate private key
    let key_status = Command::new("openssl")
        .args([
            "genrsa", "-out", &key_file.to_string_lossy(), "4096"
        ])
        .status()?;

    if !key_status.success() {
        return Err(anyhow!("Failed to generate private key"));
    }

    // Create certificate config
    let config_file = directory.join(format!("{}.cnf", name));
    let config_content = format!(
        "[req]\n\
         default_bits = 4096\n\
         distinguished_name = req_distinguished_name\n\
         x509_extensions = v3_ca\n\
         prompt = no\n\
         \n\
         [req_distinguished_name]\n\
         C = US\n\
         ST = State\n\
         L = City\n\
         O = Rustica\n\
         OU = Secure Boot\n\
         CN = {}\n\
         \n\
         [v3_ca]\n\
         subjectKeyIdentifier = hash\n\
         authorityKeyIdentifier = keyid:always,issuer\n\
         basicConstraints = critical,CA:TRUE\n\
         keyUsage = critical,digitalSignature,keyCertSign\n\
         {}",
        name,
        if name == "PK" {
            "extendedKeyUsage = 1.3.6.1.4.1.311.10.3.1\n" // Code signing for PK
        } else {
            ""
        }
    );

    fs::write(&config_file, config_content)?;

    // Generate self-signed certificate
    let cert_status = Command::new("openssl")
        .args([
            "req", "-new", "-x509",
            "-key", &key_file.to_string_lossy(),
            "-out", &cert_file.to_string_lossy(),
            "-days", &days.to_string(),
            "-config", &config_file.to_string_lossy(),
        ])
        .status()?;

    if !cert_status.success() {
        return Err(anyhow!("Failed to generate certificate"));
    }

    // Convert to DER format for UEFI
    let der_status = Command::new("openssl")
        .args([
            "x509", "-in", &cert_file.to_string_lossy(),
            "-out", &der_file.to_string_lossy(),
            "-outform", "DER",
        ])
        .status()?;

    if !der_status.success() {
        return Err(anyhow!("Failed to convert certificate to DER"));
    }

    // Set restrictive permissions on private key
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&key_file)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&key_file, perms)?;
    }

    println!("    Created: {}", key_file.display());
    println!("    Created: {}", cert_file.display());

    Ok(())
}

fn sign_binary(binary: &Path, output: Option<&Path>, keys_dir: &Path) -> Result<()> {
    if !binary.exists() {
        return Err(anyhow!("Binary not found: {}", binary.display()));
    }

    let db_key = keys_dir.join("DB.key");
    let db_cert = keys_dir.join("DB.crt");

    if !db_key.exists() || !db_cert.exists() {
        return Err(anyhow!(
            "DB key not found. Run 'rustux-sbctl create-keys' first."
        ));
    }

    let default_output = binary.with_extension(
        format!("{}.signed", binary.extension().and_then(|s| s.to_str()).unwrap_or("bin"))
    );
    let output_path = output.unwrap_or(&default_output);

    println!("Signing {}...", binary.display());
    println!("  Output: {}", output_path.display());

    // Use sbsign to sign the binary
    let sign_status = Command::new("sbsign")
        .args([
            "--key", &db_key.to_string_lossy(),
            "--cert", &db_cert.to_string_lossy(),
            "--output", &output_path.to_string_lossy(),
            &binary.to_string_lossy(),
        ])
        .status();

    match sign_status {
        Ok(status) if status.success() => {
            println!("  Signed successfully!");
            Ok(())
        }
        Ok(_) => Err(anyhow!("sbsign failed")),
        Err(_) => {
            // sbsign not available, provide instructions
            println!("  Note: sbsign not available. Binary not signed.");
            println!("  To sign manually:");
            println!("    sudo sbsign --key {} --cert {} --output {} {}",
                db_key.display(), db_cert.display(), output_path.display(), binary.display());
            Err(anyhow!("sbsign not found"))
        }
    }
}

fn verify_binary(binary: &Path, keys_dir: &Path) -> Result<()> {
    if !binary.exists() {
        return Err(anyhow!("Binary not found: {}", binary.display()));
    }

    let db_cert = keys_dir.join("DB.crt");

    if !db_cert.exists() {
        return Err(anyhow!("DB certificate not found"));
    }

    println!("Verifying {}...", binary.display());

    // Use sbverify to check signature
    let verify_status = Command::new("sbverify")
        .args([
            "--cert", &db_cert.to_string_lossy(),
            &binary.to_string_lossy(),
        ])
        .status();

    match verify_status {
        Ok(status) if status.success() => {
            println!("  Signature is VALID!");
            Ok(())
        }
        Ok(_) => Err(anyhow!("Signature is INVALID")),
        Err(_) => {
            println!("  Note: sbverify not available");
            Err(anyhow!("sbverify not found"))
        }
    }
}

fn list_keys(keys_dir: &Path) -> Result<()> {
    println!("Secure boot keys in {}:", keys_dir.display());
    println!();

    if !keys_dir.exists() {
        println!("  No keys found. Run 'rustux-sbctl create-keys' to create keys.");
        return Ok(());
    }

    let key_types = ["PK", "KEK", "DB"];

    for key_type in &key_types {
        let key_file = keys_dir.join(format!("{}.key", key_type));
        let cert_file = keys_dir.join(format!("{}.crt", key_type));

        println!("  {} ({})", key_type, KeyType::from_str(key_type)?.description());

        if key_file.exists() {
            println!("    Key: {}", key_file.display());
        } else {
            println!("    Key: MISSING");
        }

        if cert_file.exists() {
            println!("    Certificate: {}", cert_file.display());
        } else {
            println!("    Certificate: MISSING");
        }

        println!();
    }

    // Check for GUID
    let guid_file = keys_dir.join("GUID");
    if guid_file.exists() {
        let guid = fs::read_to_string(&guid_file)?;
        println!("  GUID: {}", guid.trim());
        println!();
    }

    Ok(())
}

fn export_key(key_type: &str, esp: &Path) -> Result<()> {
    let keys_dir = PathBuf::from("/var/lib/sbctl/keys");
    let key_type_upper = key_type.to_uppercase();

    let der_file = keys_dir.join(format!("{}.der", key_type_upper));

    if !der_file.exists() {
        return Err(anyhow!(
            "Key file not found: {}. Run 'rustux-sbctl create-keys' first.",
            der_file.display()
        ));
    }

    // Create EFI/keys directory in ESP
    let efi_keys_dir = esp.join("EFI").join("rustux").join("keys");
    fs::create_dir_all(&efi_keys_dir)
        .context("Failed to create EFI keys directory")?;

    let dest_file = efi_keys_dir.join(format!("{}.der", key_type_upper));

    println!("Exporting {} to ESP...", key_type_upper);
    println!("  Source: {}", der_file.display());
    println!("  Destination: {}", dest_file.display());

    fs::copy(&der_file, &dest_file)?;

    println!("  Exported successfully!");
    println!("  Note: You still need to enroll this key in your firmware setup.");

    Ok(())
}

fn sign_kernels(keys_dir: &Path, boot_dir: &Path) -> Result<()> {
    println!("Signing kernels in {}...", boot_dir.display());

    let db_key = keys_dir.join("DB.key");
    let db_cert = keys_dir.join("DB.crt");

    if !db_key.exists() || !db_cert.exists() {
        return Err(anyhow!(
            "DB key not found. Run 'rustux-sbctl create-keys' first."
        ));
    }

    // Find all kernel files
    let mut kernels_found = 0;
    let mut kernels_signed = 0;

    let entries = fs::read_dir(boot_dir)
        .context("Failed to read boot directory")?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Look for kernel files (vmlinuz-*, kernel-*, etc.)
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if name_str.starts_with("vmlinuz") || name_str.starts_with("kernel") {
                kernels_found += 1;
                println!("  Found: {}", name_str);

                let signed_path = path.with_extension(format!("{}.signed",
                    path.extension().and_then(|s| s.to_str()).unwrap_or("bin")));

                // Sign the kernel
                let sign_status = Command::new("sbsign")
                    .args([
                        "--key", &db_key.to_string_lossy(),
                        "--cert", &db_cert.to_string_lossy(),
                        "--output", &signed_path.to_string_lossy(),
                        &path.to_string_lossy(),
                    ])
                    .status();

                match sign_status {
                    Ok(status) if status.success() => {
                        kernels_signed += 1;
                        println!("    Signed: {}", signed_path.display());
                    }
                    _ => {
                        println!("    Failed to sign {}", name_str);
                    }
                }
            }
        }
    }

    println!();
    if kernels_found == 0 {
        println!("No kernels found in {}", boot_dir.display());
    } else {
        println!("Signed {}/{} kernels", kernels_signed, kernels_found);
    }

    Ok(())
}

fn show_status() -> Result<()> {
    println!("Secure Boot Status:");
    println!();

    // Check if keys exist
    let keys_dir = PathBuf::from("/var/lib/sbctl/keys");
    let keys_exist = keys_dir.exists();

    if keys_exist {
        println!("  Keys: Created");
    } else {
        println!("  Keys: Not created (run 'rustux-sbctl create-keys')");
    }

    // Check for sbverify/sbsign
    let sbsign_available = Command::new("which").arg("sbsign").output()?.status.success();
    let sbverify_available = Command::new("which").arg("sbverify").output()?.status.success();

    println!("  sbsign: {}", if sbsign_available { "Available" } else { "Not installed" });
    println!("  sbverify: {}", if sbverify_available { "Available" } else { "Not installed" });

    // Check if secure boot is enabled (read from efivars)
    let efivars = PathBuf::from("/sys/firmware/efi/efivars");
    if efivars.exists() {
        let secure_boot_var = efivars.join("SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c");

        if secure_boot_var.exists() {
            // Read the variable (second byte indicates status: 0 = disabled, 1 = enabled)
            if let Ok(data) = fs::read(&secure_boot_var) {
                if data.len() >= 2 {
                    let enabled = data.get(4).map(|&b| b == 1).unwrap_or(false);
                    println!("  Secure Boot: {}", if enabled { "Enabled" } else { "Disabled" });
                }
            }
        } else {
            println!("  Secure Boot: Not supported (no SecureBoot variable)");
        }
    } else {
        println!("  Secure Boot: Not supported (no efivars)");
    }

    Ok(())
}

fn generate_uuid() -> Result<()> {
    let uuid = generate_uuid_value()?;
    println!("{}", uuid);
    Ok(())
}

fn generate_uuid_value() -> Result<String> {
    let uuid_status = Command::new("uuidgen")
        .output()?;

    if uuid_status.status.success() {
        let uuid = String::from_utf8_lossy(&uuid_status.stdout).trim().to_string();
        Ok(uuid)
    } else {
        // Fallback: generate a simple UUID
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let uuid = format!(
            "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
            (timestamp >> 32) as u32,
            (timestamp >> 16) as u16,
            (timestamp & 0xfff) as u16,
            rand::random::<u16>(),
            (timestamp & 0xffffffffffff) as u64,
        );
        Ok(uuid)
    }
}
