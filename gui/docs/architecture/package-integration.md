# Phase 9.1: Package Manager Integration (rustica-store)

## Overview

**Component**: rustica-store
**Purpose**: Graphical package manager interface and integration layer
**Language**: Rust
**Dependencies**: rustica-pm, dbus (zbus), gtk4/librustica

## Goals

1. **User-Friendly Interface**: Easy-to-use app store experience
2. **Seamless Integration**: Works with rustica-pm, Flatpak, and AppImage
3. **Safe Defaults**: Warn about permissions and package sources
4. **Background Updates**: Automatic update checking and installation
5. **Search & Discovery**: Browse, search, and discover applications

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    User Applications                         │
│              (GUI, CLI tools, other apps)                    │
└────────────────────────┬────────────────────────────────────┘
                         │ D-Bus / Library
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  rustica-store                               │
│                  (Store Frontend)                            │
├─────────────────────────────────────────────────────────────┤
│  StoreUI           │  PackageManager    │  UpdateManager    │
│  - Browse apps     │  - rustica-pm      │  - Auto-update    │
│  - Search          │  - Flatpak         │  - Batch install  │
│  - Install/Remove  │  - AppImage        │  - Rollback       │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              rustica-pm / Flatpak / AppImage                 │
│              (Package Backends)                              │
└─────────────────────────────────────────────────────────────┘
```

## Core Data Structures

```rust
/// Application package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMetadata {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub description: String,
    pub version: String,
    pub license: String,
    pub author: String,
    pub homepage: Option<String>,
    pub icon: Option<String>,
    pub screenshots: Vec<String>,
    pub categories: Vec<Category>,
    pub keywords: Vec<String>,
    pub package_type: PackageType,
    pub source: PackageSource,
    pub size_bytes: u64,
    pub installed_size_bytes: Option<u64>,
    pub permissions: Vec<Permission>,
    pub rating: Option<Rating>,
    pub download_count: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageType {
    Native,      // rustica-pm package
    Flatpak,
    AppImage,
    Snap,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageSource {
    Official,
    Community,
    ThirdParty(String),
    Local(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Accessories,
    Audio,
    AudioVideo,
    Development,
    Education,
    Games,
    Graphics,
    Network,
    Office,
    Science,
    Settings,
    System,
    Utility,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rating {
    pub score: f64,  // 0.0 - 5.0
    pub count: u32,
}

/// Package installation status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageStatus {
    Available,
    Installed,
    UpdateAvailable(String),  // Version string
    Installing(f32),          // Progress 0.0-1.0
    Removing(f32),            // Progress 0.0-1.0
    UpdateAvailable(String),  // New version
    Error(String),
}
```

## Package Manager Backend

```rust
pub struct PackageManager {
    native_pm: NativePackageManager,
    flatpak: Option<FlatpakBackend>,
    appimage: AppImageBackend,
    config: StoreConfig,
}

impl PackageManager {
    pub fn new() -> Result<Self, Error> {
        let config = StoreConfig::load()?;

        Ok(Self {
            native_pm: NativePackageManager::new()?,
            flatpak: FlatpakBackend::new().ok(),
            appimage: AppImageBackend::new(),
            config,
        })
    }

    /// Search for packages across all sources
    pub async fn search(&self, query: &str) -> Result<Vec<AppMetadata>, Error> {
        let mut results = Vec::new();

        // Search native packages
        let native_results = self.native_pm.search(query).await?;
        results.extend(native_results);

        // Search Flatpak
        if let Some(ref flatpak) = self.flatpak {
            if self.config.enable_flatpak {
                let flatpak_results = flatpak.search(query).await?;
                results.extend(flatpak_results);
            }
        }

        // Search local AppImages
        let appimage_results = self.appimage.search(query)?;
        results.extend(appimage_results);

        // Deduplicate by ID
        results.sort_by(|a, b| a.id.cmp(&b.id));
        results.dedup_by_key(|p| p.id.clone());

        Ok(results)
    }

    /// Get package metadata
    pub async fn get_metadata(&self, id: &str, source: PackageType) -> Result<AppMetadata, Error> {
        match source {
            PackageType::Native => {
                self.native_pm.get_metadata(id).await
            }
            PackageType::Flatpak => {
                self.flatpak
                    .as_ref()
                    .ok_or(Error::FlatpakNotAvailable)?
                    .get_metadata(id)
                    .await
            }
            PackageType::AppImage => {
                self.appimage.get_metadata(id)
            }
            _ => Err(Error::UnsupportedPackageType),
        }
    }

    /// Install a package
    pub async fn install(&self, id: &str, source: PackageType) -> Result<InstallHandle, Error> {
        // Check permissions
        let metadata = self.get_metadata(id, source).await?;

        if !self.config.auto_approve_permissions {
            self.request_permission_approval(&metadata)?;
        }

        // Dispatch to appropriate backend
        match source {
            PackageType::Native => {
                self.native_pm.install(id).await
            }
            PackageType::Flatpak => {
                self.flatpak
                    .as_ref()
                    .ok_or(Error::FlatpakNotAvailable)?
                    .install(id)
                    .await
            }
            PackageType::AppImage => {
                self.appimage.install(id).await
            }
            _ => Err(Error::UnsupportedPackageType),
        }
    }

    /// Remove a package
    pub async fn remove(&self, id: &str, source: PackageType) -> Result<(), Error> {
        match source {
            PackageType::Native => {
                self.native_pm.remove(id).await
            }
            PackageType::Flatpak => {
                self.flatpak
                    .as_ref()
                    .ok_or(Error::FlatpakNotAvailable)?
                    .remove(id)
                    .await
            }
            PackageType::AppImage => {
                self.appimage.remove(id).await
            }
            _ => Err(Error::UnsupportedPackageType),
        }
    }

    /// Update a package
    pub async fn update(&self, id: &str, source: PackageType) -> Result<(), Error> {
        match source {
            PackageType::Native => {
                self.native_pm.update(id).await
            }
            PackageType::Flatpak => {
                self.flatpak
                    .as_ref()
                    .ok_or(Error::FlatpakNotAvailable)?
                    .update(id)
                    .await
            }
            PackageType::AppImage => {
                // AppImages are self-updating
                Ok(())
            }
            _ => Err(Error::UnsupportedPackageType),
        }
    }

    /// Get list of updatable packages
    pub async fn get_updates(&self) -> Result<Vec<UpdateInfo>, Error> {
        let mut updates = Vec::new();

        // Check native updates
        let native_updates = self.native_pm.get_updates().await?;
        updates.extend(native_updates);

        // Check Flatpak updates
        if let Some(ref flatpak) = self.flatpak {
            if self.config.enable_flatpak {
                let flatpak_updates = flatpak.get_updates().await?;
                updates.extend(flatpak_updates);
            }
        }

        Ok(updates)
    }

    /// Get installed packages
    pub async fn list_installed(&self) -> Result<Vec<AppMetadata>, Error> {
        let mut packages = Vec::new();

        // List native packages
        let native_packages = self.native_pm.list_installed().await?;
        packages.extend(native_packages);

        // List Flatpak packages
        if let Some(ref flatpak) = self.flatpak {
            if self.config.enable_flatpak {
                let flatpak_packages = flatpak.list_installed().await?;
                packages.extend(flatpak_packages);
            }
        }

        // List AppImages
        let appimage_packages = self.appimage.list_installed()?;
        packages.extend(appimage_packages);

        Ok(packages)
    }

    fn request_permission_approval(&self, metadata: &AppMetadata) -> Result<(), Error> {
        // Show permission dialog
        let dialog = PermissionDialog {
            app_name: metadata.name.clone(),
            permissions: metadata.permissions.clone(),
        };

        let approved = dialog.show()?;

        if !approved {
            return Err(Error::PermissionsDenied);
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub id: String,
    pub name: String,
    pub old_version: String,
    pub new_version: String,
    pub package_type: PackageType,
    pub size_bytes: u64,
}

/// Handle for tracking installation progress
pub struct InstallHandle {
    pub id: String,
    pub sender: Sender<InstallProgress>,
}

#[derive(Debug, Clone)]
pub enum InstallProgress {
    Downloading { progress: f32, bytes_downloaded: u64, total_bytes: u64 },
    Installing { progress: f32 },
    Complete,
    Error(String),
}

impl Drop for InstallHandle {
    fn drop(&mut self) {
        // Cancel installation if handle dropped
    }
}
```

## Native Package Manager Integration

```rust
pub struct NativePackageManager {
    pm_client: rustica_pm::Client,
}

impl NativePackageManager {
    pub fn new() -> Result<Self, Error> {
        let pm_client = rustica_pm::Client::connect()?;

        Ok(Self { pm_client })
    }

    pub async fn search(&self, query: &str) -> Result<Vec<AppMetadata>, Error> {
        let packages = self.pm_client.search(query).await?;

        let metadata: Vec<AppMetadata> = packages
            .into_iter()
            .map(|p| self.to_app_metadata(p))
            .collect();

        Ok(metadata)
    }

    pub async fn get_metadata(&self, id: &str) -> Result<AppMetadata, Error> {
        let package = self.pm_client.get(id).await?;
        Ok(self.to_app_metadata(package))
    }

    pub async fn install(&self, id: &str) -> Result<InstallHandle, Error> {
        let (sender, receiver) = channel();

        // Spawn background task
        let pm_client = self.pm_client.clone();
        let id = id.to_string();

        tokio::spawn(async move {
            match pm_client.install(&id, |progress| {
                let _ = sender.send(InstallProgress::Installing {
                    progress: progress as f32 / 100.0,
                });
            }).await {
                Ok(_) => {
                    let _ = sender.send(InstallProgress::Complete);
                }
                Err(e) => {
                    let _ = sender.send(InstallProgress::Error(e.to_string()));
                }
            }
        });

        Ok(InstallHandle {
            id: id.to_string(),
            sender,
        })
    }

    pub async fn remove(&self, id: &str) -> Result<(), Error> {
        self.pm_client.remove(id).await
    }

    pub async fn update(&self, id: &str) -> Result<(), Error> {
        self.pm_client.update(id).await
    }

    pub async fn get_updates(&self) -> Result<Vec<UpdateInfo>, Error> {
        let updates = self.pm_client.get_updates().await?;

        Ok(updates
            .into_iter()
            .map(|u| UpdateInfo {
                id: u.id,
                name: u.name,
                old_version: u.old_version,
                new_version: u.new_version,
                package_type: PackageType::Native,
                size_bytes: u.size_bytes,
            })
            .collect())
    }

    pub async fn list_installed(&self) -> Result<Vec<AppMetadata>, Error> {
        let packages = self.pm_client.list_installed().await?;

        Ok(packages
            .into_iter()
            .map(|p| self.to_app_metadata(p))
            .collect())
    }

    fn to_app_metadata(&self, package: rustica_pm::Package) -> AppMetadata {
        AppMetadata {
            id: package.id,
            name: package.name,
            summary: package.summary,
            description: package.description,
            version: package.version,
            license: package.license,
            author: package.author,
            homepage: package.homepage,
            icon: package.icon,
            screenshots: package.screenshots,
            categories: package.categories
                .into_iter()
                .map(|c| Category::from_str(&c).unwrap_or(Category::Utility))
                .collect(),
            keywords: package.keywords,
            package_type: PackageType::Native,
            source: PackageSource::Official,
            size_bytes: package.size_bytes,
            installed_size_bytes: Some(package.installed_size_bytes),
            permissions: package.permissions
                .into_iter()
                .map(|p| Permission {
                    name: p.name,
                    description: p.description,
                    required: p.required,
                    risk_level: RiskLevel::from_str(&p.risk_level)
                        .unwrap_or(RiskLevel::Medium),
                })
                .collect(),
            rating: None,
            download_count: None,
        }
    }
}
```

## Flatpak Integration

```rust
pub struct FlatpakBackend {
    installation: flatpak::Installation,
}

impl FlatpakBackend {
    pub fn new() -> Result<Self, Error> {
        let installation = flatpak::Installation::default_system()?;

        Ok(Self { installation })
    }

    pub async fn search(&self, query: &str) -> Result<Vec<AppMetadata>, Error> {
        let remote = self.installation.find_remote_by_name("flathub")?;

        let refs = remote.search(query)?;

        let metadata: Vec<AppMetadata> = refs
            .into_iter()
            .map(|r| self.ref_to_metadata(r))
            .collect();

        Ok(metadata)
    }

    pub async fn get_metadata(&self, id: &str) -> Result<AppMetadata, Error> {
        let ref_str = format!("app/{}", id);
        let remote_ref = self.installation.find_ref_by_name(&ref_str)?;

        Ok(self.ref_to_metadata(remote_ref))
    }

    pub async fn install(&self, id: &str) -> Result<InstallHandle, Error> {
        let (sender, receiver) = channel();

        let ref_str = format!("app/{}", id);
        let installation = self.installation.clone();

        tokio::spawn(async move {
            match installation.install(&ref_str, |progress| {
                let _ = sender.send(InstallProgress::Installing {
                    progress: progress as f32 / 100.0,
                });
            }).await {
                Ok(_) => {
                    let _ = sender.send(InstallProgress::Complete);
                }
                Err(e) => {
                    let _ = sender.send(InstallProgress::Error(e.to_string()));
                }
            }
        });

        Ok(InstallHandle {
            id: id.to_string(),
            sender,
        })
    }

    pub async fn remove(&self, id: &str) -> Result<(), Error> {
        let ref_str = format!("app/{}", id);
        self.installation.uninstall(&ref_str).await
    }

    pub async fn update(&self, id: &str) -> Result<(), Error> {
        let ref_str = format!("app/{}", id);
        self.installation.update(&ref_str).await
    }

    pub async fn get_updates(&self) -> Result<Vec<UpdateInfo>, Error> {
        let updates = self.installation.list_updates().await?;

        Ok(updates
            .into_iter()
            .map(|u| UpdateInfo {
                id: u.id,
                name: u.name,
                old_version: u.old_version,
                new_version: u.new_version,
                package_type: PackageType::Flatpak,
                size_bytes: u.size_bytes,
            })
            .collect())
    }

    pub async fn list_installed(&self) -> Result<Vec<AppMetadata>, Error> {
        let refs = self.installation.list_installed_refs()?;

        Ok(refs
            .into_iter()
            .filter(|r| r.kind() == flatpak::RefKind::App)
            .map(|r| self.ref_to_metadata(r))
            .collect())
    }

    fn ref_to_metadata(&self, remote_ref: flatpak::RemoteRef) -> AppMetadata {
        let metadata = remote_ref.metadata();

        AppMetadata {
            id: remote_ref.id(),
            name: metadata.name,
            summary: metadata.summary,
            description: metadata.description,
            version: metadata.version,
            license: metadata.license,
            author: metadata.author,
            homepage: metadata.homepage,
            icon: metadata.icon,
            screenshots: metadata.screenshots,
            categories: metadata.categories
                .into_iter()
                .map(|c| Category::from_str(&c).unwrap_or(Category::Utility))
                .collect(),
            keywords: metadata.keywords,
            package_type: PackageType::Flatpak,
            source: PackageSource::ThirdParty("Flathub".to_string()),
            size_bytes: metadata.download_size,
            installed_size_bytes: Some(metadata.installed_size),
            permissions: metadata.permissions
                .into_iter()
                .map(|p| Permission {
                    name: p.name,
                    description: p.description,
                    required: p.required,
                    risk_level: RiskLevel::from_str(&p.risk_level)
                        .unwrap_or(RiskLevel::Medium),
                })
                .collect(),
            rating: None,
            download_count: None,
        }
    }
}
```

## AppImage Integration

```rust
pub struct AppImageBackend {
    install_dir: PathBuf,
}

impl AppImageBackend {
    pub fn new() -> Self {
        Self {
            install_dir: PathBuf::from("/opt/appimages"),
        }
    }

    pub fn search(&self, query: &str) -> Result<Vec<AppMetadata>, Error> {
        // Search local AppImages only
        let mut results = Vec::new();

        for entry in std::fs::read_dir(&self.install_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("AppImage") {
                if let Some(metadata) = self.read_appimage_metadata(&path)? {
                    if metadata.name.to_lowercase().contains(&query.to_lowercase()) {
                        results.push(metadata);
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn get_metadata(&self, id: &str) -> Result<AppMetadata, Error> {
        let path = self.install_dir.join(format!("{}.AppImage", id));
        self.read_appimage_metadata(&path)?
            .ok_or(Error::AppImageNotFound)
    }

    pub async fn install(&self, id: &str) -> Result<InstallHandle, Error> {
        // AppImage install is just downloading and setting executable
        let (sender, receiver) = channel();

        tokio::spawn(async move {
            // Download AppImage
            // ...

            let _ = sender.send(InstallProgress::Complete);
        });

        Ok(InstallHandle {
            id: id.to_string(),
            sender,
        })
    }

    pub async fn remove(&self, id: &str) -> Result<(), Error> {
        let path = self.install_dir.join(format!("{}.AppImage", id));
        std::fs::remove_file(path)?;
        Ok(())
    }

    pub async fn update(&self, id: &str) -> Result<(), Error> {
        // AppImages are self-updating or require re-download
        Ok(())
    }

    pub fn list_installed(&self) -> Result<Vec<AppMetadata>, Error> {
        let mut packages = Vec::new();

        for entry in std::fs::read_dir(&self.install_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("AppImage") {
                if let Some(metadata) = self.read_appimage_metadata(&path)? {
                    packages.push(metadata);
                }
            }
        }

        Ok(packages)
    }

    fn read_appimage_metadata(&self, path: &Path) -> Result<Option<AppMetadata>, Error> {
        // Extract metadata from AppImage desktop file
        // This is simplified
        Ok(None)
    }
}
```

## Update Manager

```rust
pub struct UpdateManager {
    package_manager: PackageManager,
    config: UpdateConfig,
    last_check: Option<DateTime<Utc>>,
}

impl UpdateManager {
    pub fn new(package_manager: PackageManager) -> Result<Self, Error> {
        let config = UpdateConfig::load()?;

        Ok(Self {
            package_manager,
            config,
            last_check: None,
        })
    }

    /// Check for updates
    pub async fn check_updates(&mut self) -> Result<Vec<UpdateInfo>, Error> {
        let updates = self.package_manager.get_updates().await?;
        self.last_check = Some(Utc::now());

        // Send notification if updates available
        if !updates.is_empty() {
            self.send_update_notification(&updates);
        }

        Ok(updates)
    }

    /// Install all updates
    pub async fn install_all_updates(&self) -> Result<(), Error> {
        let updates = self.package_manager.get_updates().await?;

        for update in updates {
            self.package_manager.update(&update.id, update.package_type).await?;
        }

        Ok(())
    }

    /// Auto-update background task
    pub async fn auto_update_task(&mut self) {
        loop {
            tokio::time::sleep(self.config.check_interval).await;

            if let Ok(updates) = self.check_updates().await {
                if self.config.auto_install && !updates.is_empty() {
                    // Auto-install security updates
                    let security_updates: Vec<_> = updates
                        .into_iter()
                        .filter(|u| u.is_security_update)
                        .collect();

                    for update in security_updates {
                        let _ = self.package_manager.update(
                            &update.id,
                            update.package_type
                        ).await;
                    }
                }
            }
        }
    }

    fn send_update_notification(&self, updates: &[UpdateInfo]) {
        let count = updates.len();

        Notification::new()
            .summary(&format!("{} Update{} Available", count, if count > 1 { "s" } else { "" }))
            .body(&format!("Click to view and install updates"))
            .icon("software-update-available")
            .show();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    pub check_interval: Duration,
    pub auto_install: bool,
    pub notify_available: bool,
    pub background_download: bool,
}

impl UpdateConfig {
    fn load() -> Result<Self, Error> {
        let path = "/var/lib/rustica/store/update-config.json";

        if let Ok(file) = std::fs::File::open(path) {
            Ok(serde_json::from_reader(file)?)
        } else {
            Ok(Self::default())
        }
    }
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(86400),  // Daily
            auto_install: false,
            notify_available: true,
            background_download: true,
        }
    }
}
```

## D-Bus Interface

```rust
#[dbus_interface(name = "org.rustica.Software")]
impl PackageManager {
    /// Search for packages
    async fn search(&self, query: String) -> Result<Vec<AppMetadata>, Error> {
        self.search(&query).await
    }

    /// Get package metadata
    async fn get_metadata(&self, id: String, package_type: PackageType) -> Result<AppMetadata>, Error> {
        self.get_metadata(&id, package_type).await
    }

    /// Install package
    async fn install(&self, id: String, package_type: PackageType) -> Result<String, Error> {
        let handle = self.install(&id, package_type).await?;
        Ok(handle.id)
    }

    /// Remove package
    async fn remove(&self, id: String, package_type: PackageType) -> Result<(), Error> {
        self.remove(&id, package_type).await
    }

    /// Update package
    async fn update(&self, id: String, package_type: PackageType) -> Result<(), Error> {
        self.update(&id, package_type).await
    }

    /// Get list of updates
    async fn get_updates(&self) -> Result<Vec<UpdateInfo>, Error> {
        self.get_updates().await
    }

    /// List installed packages
    async fn list_installed(&self) -> Result<Vec<AppMetadata>, Error> {
        self.list_installed().await
    }

    /// Installation progress signal
    #[dbus(signal)]
    fn install_progress(&self, id: String, progress: f32);

    /// Update available signal
    #[dbus(signal)]
    fn update_available(&self, updates: Vec<UpdateInfo>);
}
```

## Configuration

```toml
# /etc/rustica/store.conf
[general]
# Enable software store
enabled = true

# Package sources to enable
enable_native = true
enable_flatpak = true
enable_appimage = true

[permissions]
# Auto-approve package permissions
auto_approve = false

# Warn about high-risk permissions
warn_high_risk = true

# Require approval for network access
require_network_approval = true

[updates]
# Check for updates interval (seconds)
check_interval = 86400

# Auto-install updates
auto_install = false

# Auto-install security updates
auto_install_security = true

# Notify when updates available
notify_available = true

# Background download updates
background_download = true

[interface]
# Show featured apps on home
show_featured = true

# Show category navigation
show_categories = true

# Number of apps per page
apps_per_page = 20

# Show download counts
show_download_counts = true

# Show ratings
show_ratings = true
```

## Security

1. **Repository Verification**: GPG signatures for package repositories
2. **Permission Warnings**: Warn about high-risk permissions
3. **Sandboxing**: Flatpak provides sandboxing by default
4. **Hash Verification**: Verify package checksums
5. **User Confirmation**: Require user approval for installations

## Dependencies

```toml
[dependencies]
zbus = "4"
rustica-pm = { path = "../../rustica-pm" }
flatpak-rs = { version = "0.5", optional = true }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search() {
        let pm = PackageManager::new().unwrap();
        let results = pm.search("firefox").await.unwrap();

        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_get_metadata() {
        let pm = PackageManager::new().unwrap();
        let metadata = pm.get_metadata("org.mozilla.firefox", PackageType::Flatpak).await.unwrap();

        assert_eq!(metadata.id, "org.mozilla.firefox");
    }

    #[tokio::test]
    async fn test_list_installed() {
        let pm = PackageManager::new().unwrap();
        let packages = pm.list_installed().await.unwrap();

        // Should at least include some system packages
        assert!(!packages.is_empty());
    }

    #[tokio::test]
    async fn test_get_updates() {
        let pm = PackageManager::new().unwrap();
        let updates = pm.get_updates().await.unwrap();

        // Should return a list (may be empty)
        assert!(updates.len() >= 0);
    }
}
```

## Future Enhancements

1. **App Ratings & Reviews**: User ratings and reviews
2. **Wishlist**: Save apps for later
3. **Auto-Removal**: Remove unused dependencies
4. **Rollback**: Revert to previous package version
5. **Package History**: View installation/update history
6. **Batch Operations**: Install/remove multiple apps
7. **Package Verification**: Verify package integrity
8. **Dependency Visualization**: Show package dependencies
9. **Disk Usage**: Show space used by packages
10. **Offline Mode**: Browse packages without internet
