# Phase 9.3: AppImage Support (rustica-appimage)

## Overview

**Component**: rustica-appimage
**Purpose**: AppImage integration for portable Linux applications
**Language**: Rust
**Dependencies:** fuse, libc, dbus (zbus)

## Goals

1. **Seamless Integration**: AppImages work like native apps
2. **Desktop Integration**: Proper menu entries and icons
3. **Auto-Integration**: Automatic desktop file integration
4. **Management UI**: Easy install/remove/update interface
5. **Sandbox Option**: Optional sandboxing via firejail/bubblewrap

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    User Applications                         │
│              (AppImage apps, native apps)                    │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  rustica-appimage                            │
│                  (AppImage Manager)                          │
├─────────────────────────────────────────────────────────────┤
│  AppImageManager     │  DesktopIntegrator│  SandboxWrapper  │
│  - Install/Remove    │  - .desktop files │  - firejail      │
│  - Register          │  - Icons          │  - bubblewrap    │
│  - Update check      │  - MIME types     │  - Permissions   │
│  - Execute           │  - Menu entries   │  - Profile mgmt  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              AppImage files (FUSE mount)                     │
└─────────────────────────────────────────────────────────────┘
```

## Core Data Structures

```rust
/// AppImage metadata
#[derive(Debug, Clone)]
pub struct AppImageMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub summary: String,
    pub description: String,
    pub icon: Option<String>,
    pub categories: Vec<Category>,
    pub keywords: Vec<String>,
    pub exec: String,
    pub file_path: PathBuf,
    pub size_bytes: u64,
    pub architecture: AppImageArch,
    pub type_: AppImageType,
    pub digest: Option<String>,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppImageArch {
    X86_64,
    AArch64,
    Armhf,
    I386,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppImageType {
    Type1,  // ISO 9660
    Type2,  // ELF
}

/// AppImage installation info
#[derive(Debug, Clone)]
pub struct AppImageInstall {
    pub metadata: AppImageMetadata,
    pub installed_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub launch_count: u32,
    pub desktop_file_path: PathBuf,
    pub sandbox_enabled: bool,
    pub sandbox_profile: Option<String>,
}
```

## AppImage Manager

```rust
pub struct AppImageManager {
    install_dir: PathBuf,
    desktop_dir: PathBuf,
    icon_dir: PathBuf,
    installed_apps: HashMap<String, AppImageInstall>,
    config: AppImageConfig,
}

impl AppImageManager {
    pub fn new() -> Result<Self, Error> {
        let config = AppImageConfig::load()?;

        // Create directories
        let install_dir = PathBuf::from("/opt/appimages");
        let desktop_dir = PathBuf::from("/usr/share/applications");
        let icon_dir = PathBuf::from("/usr/share/icons/hicolor");

        std::fs::create_dir_all(&install_dir)?;
        std::fs::create_dir_all(&desktop_dir)?;
        std::fs::create_dir_all(&icon_dir)?;

        // Load installed apps
        let installed_apps = Self::load_installed_apps(&desktop_dir)?;

        Ok(Self {
            install_dir,
            desktop_dir,
            icon_dir,
            installed_apps,
            config,
        })
    }

    /// Install AppImage from file
    pub async fn install(&mut self, path: &Path) -> Result<AppImageInstall, Error> {
        // Verify it's an AppImage
        self.verify_appimage(path)?;

        // Extract metadata
        let metadata = self.extract_metadata(path)?;

        // Copy to install directory
        let dest_path = self.install_dir.join(format!("{}.AppImage", metadata.id));
        std::fs::copy(path, &dest_path)?;

        // Make executable
        self.set_executable(&dest_path)?;

        // Integrate with desktop
        let desktop_file = self.integrate_desktop(&metadata, &dest_path)?;

        // Extract and install icon
        self.install_icon(&metadata, &dest_path)?;

        let install = AppImageInstall {
            metadata,
            installed_at: Utc::now(),
            last_used: None,
            launch_count: 0,
            desktop_file_path: desktop_file,
            sandbox_enabled: self.config.sandbox_by_default,
            sandbox_profile: None,
        };

        // Register
        self.installed_apps.insert(install.metadata.id.clone(), install.clone());

        // Save registry
        self.save_registry()?;

        Ok(install)
    }

    /// Install AppImage from URL
    pub async fn install_from_url(&mut self, url: &str) -> Result<AppImageInstall, Error> {
        // Download to temp file
        let temp_path = self.download(url).await?;

        // Install
        self.install(&temp_path).await?;

        // Clean up temp file
        std::fs::remove_file(&temp_path)?;

        // Return the installed app
        let id = Self::url_to_id(url)?;
        Ok(self.installed_apps[&id].clone())
    }

    /// Remove installed AppImage
    pub async fn remove(&mut self, id: &str) -> Result<(), Error> {
        let install = self.installed_apps.get(id)
            .ok_or(Error::NotInstalled)?;

        // Remove AppImage file
        std::fs::remove_file(&install.metadata.file_path)?;

        // Remove desktop file
        std::fs::remove_file(&install.desktop_file_path)?;

        // Remove icon
        if let Some(icon_name) = &install.metadata.icon {
            for icon_path in self.find_icon_files(icon_name) {
                let _ = std::fs::remove_file(icon_path);
            }
        }

        // Unregister
        self.installed_apps.remove(id);

        // Save registry
        self.save_registry()?;

        Ok(())
    }

    /// Launch AppImage
    pub async fn launch(&mut self, id: &str) -> Result<Process, Error> {
        let install = self.installed_apps.get(id)
            .ok_or(Error::NotInstalled)?;

        // Update usage stats
        if let Some(inst) = self.installed_apps.get_mut(id) {
            inst.last_used = Some(Utc::now());
            inst.launch_count += 1;
        }

        // Launch with optional sandbox
        if install.sandbox_enabled {
            self.launch_sandboxed(install).await
        } else {
            self.launch_direct(install).await
        }
    }

    /// Check for updates
    pub async fn check_updates(&self) -> Result<Vec<AppImageUpdate>, Error> {
        let mut updates = Vec::new();

        for (id, install) in &self.installed_apps {
            if let Some(download_url) = &self.get_download_url(id)? {
                // Check remote version
                if let Some(remote_version) = self.get_remote_version(download_url).await? {
                    if remote_version != install.metadata.version {
                        updates.push(AppImageUpdate {
                            id: id.clone(),
                            name: install.metadata.name.clone(),
                            current_version: install.metadata.version.clone(),
                            new_version: remote_version,
                            download_url: download_url.clone(),
                        });
                    }
                }
            }
        }

        Ok(updates)
    }

    /// Update AppImage
    pub async fn update(&mut self, id: &str) -> Result<(), Error> {
        let install = self.installed_apps.get(id)
            .ok_or(Error::NotInstalled)?;

        // Download new version
        let download_url = self.get_download_url(id)?
            .ok_or(Error::NoDownloadUrl)?;

        let temp_path = self.download(&download_url).await?;

        // Remove old version
        self.remove(id).await?;

        // Install new version
        self.install(&temp_path).await?;

        // Clean up temp file
        std::fs::remove_file(&temp_path)?;

        Ok(())
    }

    /// List installed AppImages
    pub fn list_installed(&self) -> Vec<AppImageInstall> {
        self.installed_apps.values().cloned().collect()
    }

    async fn download(&self, url: &str) -> Result<PathBuf, Error> {
        let response = reqwest::get(url).await?;

        let filename = url.split('/')
            .last()
            .unwrap_or("download.AppImage");

        let temp_path = std::env::temp_dir().join(filename);

        let mut file = std::fs::File::create(&temp_path)?;
        let content = response.bytes().await?;
        file.write_all(&content)?;

        Ok(temp_path)
    }

    fn verify_appimage(&self, path: &Path) -> Result<(), Error> {
        // Check if file exists
        if !path.exists() {
            return Err(Error::FileNotFound);
        }

        // Check magic bytes for AppImage
        let magic = self.read_magic_bytes(path)?;

        if magic.starts_with(b"\x7fELF") {
            // Type 2 AppImage
            Ok(())
        } else if magic.starts_with(b"\x01\x00\x00\x00") {
            // Type 1 AppImage (ISO 9660)
            Ok(())
        } else {
            Err(Error::InvalidAppImage)
        }
    }

    fn extract_metadata(&self, path: &Path) -> Result<AppImageMetadata, Error> {
        // Extract .desktop file and icon from AppImage
        let temp_dir = tempfile::tempdir()?;

        // Mount AppImage using FUSE
        let mount_point = temp_dir.path().join("mount");
        std::fs::create_dir(&mount_point)?;

        self.mount_appimage(path, &mount_point)?;

        // Read desktop file
        let desktop_file = mount_point.join("*.desktop");
        let desktop_content = std::fs::read_to_string(&desktop_file)?;

        // Parse desktop file
        let metadata = self.parse_desktop_file(&desktop_content, path)?;

        // Unmount
        self.unmount_appimage(&mount_point)?;

        Ok(metadata)
    }

    fn mount_appimage(&self, appimage: &Path, mount_point: &Path) -> Result<(), Error> {
        // Use FUSE to mount AppImage
        let output = std::process::Command::new(&appimage)
            .arg("--appimage-mount")
            .output()?;

        // Extract mount point from output
        let mount_path = String::from_utf8_lossy(&output.stdout);
        // ...

        Ok(())
    }

    fn unmount_appimage(&self, mount_point: &Path) -> Result<(), Error> {
        // Use fusermount to unmount
        std::process::Command::new("fusermount")
            .arg("-u")
            .arg(mount_point)
            .status()?;

        Ok(())
    }

    fn parse_desktop_file(&self, content: &str, file_path: &Path) -> Result<AppImageMetadata, Error> {
        let mut name = String::new();
        let mut version = String::new();
        let mut summary = String::new();
        let mut description = String::new();
        let mut icon = None;
        let mut categories = Vec::new();
        let mut keywords = Vec::new();
        let mut exec = String::new();

        // Parse .desktop file
        for line in content.lines() {
            if line.starts_with("Name=") {
                name = line[5..].to_string();
            } else if line.starts_with("Version=") {
                version = line[8..].to_string();
            } else if line.starts_with("Comment=") {
                summary = line[8..].to_string();
            } else if line.starts_with("Icon=") {
                icon = Some(line[5..].to_string());
            } else if line.starts_with("Categories=") {
                categories = line[11..].split(';')
                    .filter(|s| !s.is_empty())
                    .map(|s| Category::from_str(s).unwrap_or(Category::Utility))
                    .collect();
            } else if line.starts_with("Keywords=") {
                keywords = line[9..].split(';')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
            } else if line.starts_with("Exec=") {
                exec = line[5..].to_string();
            }
        }

        // Generate ID from name
        let id = name.to_lowercase().replace(" ", "-");

        // Detect architecture
        let magic = self.read_magic_bytes(file_path)?;
        let architecture = if magic.starts_with(b"\x7fELF\x02") {
            AppImageArch::X86_64
        } else if magic.starts_with(b"\x7fELF\x01") {
            AppImageArch::I386
        } else {
            AppImageArch::X86_64  // Default
        };

        // Detect type
        let type_ = if magic.starts_with(b"\x7fELF") {
            AppImageType::Type2
        } else {
            AppImageType::Type1
        };

        // Calculate size
        let size_bytes = std::fs::metadata(file_path)?.len();

        Ok(AppImageMetadata {
            id,
            name,
            version,
            summary,
            description: summary.clone(),
            icon,
            categories,
            keywords,
            exec,
            file_path: file_path.to_path_buf(),
            size_bytes,
            architecture,
            type_,
            digest: None,
            signature: None,
        })
    }

    fn integrate_desktop(&self, metadata: &AppImageMetadata, appimage_path: &Path) -> Result<PathBuf, Error> {
        let desktop_file_name = format!("{}.desktop", metadata.id);
        let desktop_file_path = self.desktop_dir.join(&desktop_file_name);

        // Create .desktop file
        let desktop_content = format!(
            "[Desktop Entry]
Name={}
Version={}
Comment={}
Exec={} %F
Terminal=false
Type=Application
Icon={}
Categories={}
Keywords={}
",
            metadata.name,
            metadata.version,
            metadata.summary,
            appimage_path.display(),
            metadata.icon.as_ref().unwrap_or(&metadata.id),
            metadata.categories.iter()
                .map(|c| format!("{:?}", c))
                .collect::<Vec<_>>()
                .join(";"),
            metadata.keywords.join(";")
        );

        std::fs::write(&desktop_file_path, desktop_content)?;

        Ok(desktop_file_path)
    }

    fn install_icon(&self, metadata: &AppImageMetadata, appimage_path: &Path) -> Result<(), Error> {
        if let Some(icon_name) = &metadata.icon {
            // Extract icon from AppImage
            let temp_dir = tempfile::tempdir()?;
            let mount_point = temp_dir.path().join("mount");
            std::fs::create_dir(&mount_point)?;

            self.mount_appimage(appimage_path, &mount_point)?;

            // Find icon file
            let icon_pattern = mount_point.join(format!("{}.png", icon_name));
            for icon_file in glob::glob(&format!("{}/*.png", icon_name))? {
                if let Ok(icon_path) = icon_file {
                    // Copy to icon directory
                    let dest_icon = self.icon_dir.join(icon_path.file_name().unwrap());
                    std::fs::copy(&icon_path, &dest_icon)?;
                }
            }

            self.unmount_appimage(&mount_point)?;
        }

        Ok(())
    }

    fn set_executable(&self, path: &Path) -> Result<(), Error> {
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(perms.mode() | 0o111);
        std::fs::set_permissions(path, perms)?;
        Ok(())
    }

    async fn launch_direct(&self, install: &AppImageInstall) -> Result<Process, Error> {
        let command = Command::new(&install.metadata.file_path)
            .spawn()?;

        Ok(Process::from_child(command))
    }

    async fn launch_sandboxed(&self, install: &AppImageInstall) -> Result<Process, Error> {
        // Use firejail or bubblewrap for sandboxing
        let sandbox_cmd = if self.config.use_firejail {
            vec!["firejail", "--noprofile"]
        } else {
            vec!["bwrap", "--ro-bind", "/", "/", "--dev", "/dev", "--proc", "/proc"]
        };

        let command = Command::new(sandbox_cmd[0])
            .args(&sandbox_cmd[1..])
            .arg(&install.metadata.file_path)
            .spawn()?;

        Ok(Process::from_child(command))
    }

    fn get_download_url(&self, id: &str) -> Result<Option<String>, Error> {
        // Look up download URL from registry
        let registry_path = self.install_dir.join("registry.json");

        if let Ok(file) = std::fs::File::open(&registry_path) {
            let registry: HashMap<String, AppImageRegistryEntry> =
                serde_json::from_reader(file).unwrap_or_default();

            Ok(registry.get(id).and_then(|e| e.download_url.clone()))
        } else {
            Ok(None)
        }
    }

    async fn get_remote_version(&self, url: &str) -> Result<Option<String>, Error> {
        // Fetch AppImage and check version
        // This is simplified
        Ok(None)
    }

    fn load_installed_apps(desktop_dir: &Path) -> Result<HashMap<String, AppImageInstall>, Error> {
        let mut apps = HashMap::new();

        // Scan desktop files for AppImage entries
        for entry in glob::glob(&format!("{}/*.desktop", desktop_dir.display()))? {
            if let Ok(desktop_path) = entry {
                if let Ok(content) = std::fs::read_to_string(&desktop_path) {
                    if content.contains("AppImage") {
                        // Parse and load
                        // This is simplified
                    }
                }
            }
        }

        Ok(apps)
    }

    fn save_registry(&self) -> Result<(), Error> {
        let registry_path = self.install_dir.join("registry.json");

        let registry: HashMap<String, AppImageRegistryEntry> = self.installed_apps
            .iter()
            .map(|(id, install)| {
                (id.clone(), AppImageRegistryEntry {
                    name: install.metadata.name.clone(),
                    version: install.metadata.version.clone(),
                    download_url: None,  // Would be stored during install
                })
            })
            .collect();

        let file = std::fs::File::create(&registry_path)?;
        serde_json::to_writer_pretty(file, &registry)?;

        Ok(())
    }

    fn read_magic_bytes(&self, path: &Path) -> Result<Vec<u8>, Error> {
        let mut file = std::fs::File::open(path)?;
        let mut magic = vec![0u8; 8];
        file.read_exact(&mut magic)?;
        Ok(magic)
    }

    fn url_to_id(url: &str) -> Result<String, Error> {
        // Extract ID from URL
        // This is simplified
        Ok("app-id".to_string())
    }

    fn find_icon_files(&self, icon_name: &str) -> Vec<PathBuf> {
        let mut icons = Vec::new();

        for entry in glob::glob(&format!(
            "{}/**/{}.*",
            self.icon_dir.display(),
            icon_name
        )).unwrap_or_else(|_| Box::new(std::iter::empty())) {
            if let Ok(icon_path) = entry {
                icons.push(icon_path);
            }
        }

        icons
    }
}

#[derive(Debug, Clone)]
pub struct AppImageUpdate {
    pub id: String,
    pub name: String,
    pub current_version: String,
    pub new_version: String,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppImageRegistryEntry {
    name: String,
    version: String,
    download_url: Option<String>,
}
```

## Sandbox Wrapper

```rust
pub struct SandboxWrapper {
    use_firejail: bool,
    profiles: HashMap<String, SandboxProfile>,
}

#[derive(Debug, Clone)]
pub struct SandboxProfile {
    pub name: String,
    pub allowed_dirs: Vec<PathBuf>,
    pub network_access: bool,
    pub x11: bool,
    pub wayland: bool,
    pub sound: bool,
    pub webcam: bool,
    pub custom_args: Vec<String>,
}

impl SandboxWrapper {
    pub fn new(use_firejail: bool) -> Self {
        Self {
            use_firejail,
            profiles: Self::load_profiles(),
        }
    }

    pub fn wrap_command(&self, appimage: &Path, profile: &SandboxProfile) -> Result<Vec<String>, Error> {
        let mut args = if self.use_firejail {
            self.firejail_args(profile)
        } else {
            self.bwrap_args(profile)
        };

        args.push(appimage.to_string_lossy().to_string());

        Ok(args)
    }

    fn firejail_args(&self, profile: &SandboxProfile) -> Vec<String> {
        let mut args = vec!["firejail".to_string()];

        // Profile
        if let Some(custom_profile) = &profile.name {
            args.push("--profile=".to_string() + custom_profile);
        } else {
            args.push("--noprofile".to_string());
        }

        // Directories
        for dir in &profile.allowed_dirs {
            args.push(format!("--whitelist={}", dir.display()));
        }

        // Network
        if !profile.network_access {
            args.push("--net=none".to_string());
        }

        // X11
        if profile.x11 {
            args.push("--x11".to_string());
        }

        // Sound
        if profile.sound {
            // PulseAudio
            args.push("--ignore=private-dev".to_string());
        }

        // Custom args
        args.extend(profile.custom_args.clone());

        args
    }

    fn bwrap_args(&self, profile: &SandboxProfile) -> Vec<String> {
        let mut args = vec!["bwrap".to_string()];

        // Basic setup
        args.push("--ro-bind".to_string());
        args.push("/".to_string());
        args.push("/".to_string());

        // Dev
        args.push("--dev".to_string());
        args.push("/dev".to_string());

        // Proc
        args.push("--proc".to_string());
        args.push("/proc".to_string());

        // Directories
        for dir in &profile.allowed_dirs {
            args.push("--bind".to_string());
            args.push(dir.display().to_string());
            args.push(dir.display().to_string());
        }

        // Network
        if !profile.network_access {
            args.push("--unshare-net".to_string());
        }

        // Wayland
        if profile.wayland {
            if let Ok(wayland_display) = std::env::var("WAYLAND_DISPLAY") {
                args.push("--bind".to_string());
                args.push(format!("/run/user/{}/{}", std::env::var("UID").unwrap_or("1000".to_string()), wayland_display));
                args.push(format!("/run/user/{}/{}", std::env::var("UID").unwrap_or("1000".to_string()), wayland_display));
            }
        }

        // Sound
        if profile.sound {
            if let Ok(pulse_server) = std::env::var("PULSE_SERVER") {
                args.push("--bind".to_string());
                args.push(pulse_server.clone());
                args.push(pulse_server);
            }
        }

        args
    }

    fn load_profiles() -> HashMap<String, SandboxProfile> {
        let mut profiles = HashMap::new();

        // Default restrictive profile
        profiles.insert("restrictive".to_string(), SandboxProfile {
            name: "restrictive".to_string(),
            allowed_dirs: vec![],
            network_access: false,
            x11: false,
            wayland: false,
            sound: false,
            webcam: false,
            custom_args: vec![],
        });

        // Default balanced profile
        profiles.insert("balanced".to_string(), SandboxProfile {
            name: "balanced".to_string(),
            allowed_dirs: vec![
                PathBuf::from("~/Documents"),
                PathBuf::from("~/Downloads"),
                PathBuf::from("~/Pictures"),
                PathBuf::from("~/Music"),
                PathBuf::from("~/Videos"),
            ],
            network_access: true,
            x11: false,
            wayland: true,
            sound: true,
            webcam: false,
            custom_args: vec![],
        });

        profiles
    }
}
```

## Configuration

```toml
# /etc/rustica/appimage.conf
[general]
# Enable AppImage support
enabled = true

# Installation directory
install_dir = "/opt/appimages"

# Desktop files directory
desktop_dir = "/usr/share/applications"

# Icon directory
icon_dir = "/usr/share/icons/hicolor"

[sandbox]
# Enable sandboxing by default
sandbox_by_default = false

# Use firejail (true) or bubblewrap (false)
use_firejail = true

# Default sandbox profile
default_profile = "balanced"

[updates]
# Check for updates interval (seconds)
check_interval = 604800

# Auto-update AppImages
auto_update = false

[execution]
# Set executable bit on install
set_executable = true

# Track launch count
track_launches = true

# Update last used time
update_last_used = true
```

## D-Bus Interface

```rust
#[dbus_interface(name = "org.rustica.AppImage")]
impl AppImageManager {
    /// Install AppImage from file
    async fn install(&mut self, path: String) -> Result<String, Error> {
        let path = PathBuf::from(path);
        let install = self.install(&path).await?;
        Ok(install.metadata.id)
    }

    /// Install AppImage from URL
    async fn install_from_url(&mut self, url: String) -> Result<String, Error> {
        let install = self.install_from_url(&url).await?;
        Ok(install.metadata.id)
    }

    /// Remove AppImage
    async fn remove(&mut self, id: String) -> Result<(), Error> {
        self.remove(&id).await
    }

    /// Launch AppImage
    async fn launch(&mut self, id: String) -> Result<(), Error> {
        self.launch(&id).await?;
        Ok(())
    }

    /// Check for updates
    async fn check_updates(&self) -> Result<Vec<AppImageUpdate>, Error> {
        self.check_updates().await
    }

    /// Update AppImage
    async fn update(&mut self, id: String) -> Result<(), Error> {
        self.update(&id).await
    }

    /// List installed AppImages
    fn list_installed(&self) -> Vec<AppImageInstall> {
        self.list_installed()
    }
}
```

## Dependencies

```toml
[dependencies]
zbus = "4"
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"
tempfile = "3"
glob = "0.3"
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_appimage() {
        let manager = AppImageManager::new().unwrap();

        // Test with valid AppImage (if available)
        // Test with invalid file
        let invalid_path = PathBuf::from("/etc/hosts");
        assert!(matches!(
            manager.verify_appimage(&invalid_path),
            Err(Error::InvalidAppImage)
        ));
    }

    #[tokio::test]
    async fn test_install_remove() {
        let mut manager = AppImageManager::new().unwrap();

        // This would require a test AppImage file
        // manager.install(&test_appimage_path).await.unwrap();
        // manager.remove("test-id").await.unwrap();
    }

    #[test]
    fn test_sandbox_args() {
        let wrapper = SandboxWrapper::new(true);
        let profile = SandboxProfile {
            name: "test".to_string(),
            allowed_dirs: vec![PathBuf::from("~/Downloads")],
            network_access: false,
            x11: false,
            wayland: true,
            sound: true,
            webcam: false,
            custom_args: vec![],
        };

        let args = wrapper.firejail_args(&profile);

        assert!(args.contains(&"--noprofile".to_string()));
        assert!(args.contains(&"--net=none".to_string()));
    }
}
```

## Future Enhancements

1. **AppImageHub Integration**: Browse and install from AppImageHub
2. **Automatic Updates**: Background update checking
3. **AppImageTool Integration**: Create AppImages from apps
4. **Verification**: GPG signature verification
5. **Delta Updates**: Download only changed portions
6. **Automatic Integration**: Auto-integrate on double-click
7. **Version Management**: Keep multiple versions
8. **Transaction Support**: Batch install/remove with rollback
9. **Dependency Resolution**: Check for library dependencies
10. **Cloud Sync**: Sync installed AppImages across devices

## Security Considerations

1. **Verification**: Verify AppImage signatures when available
2. **Sandboxing**: Sandbox by default to limit system access
3. **Permission Warnings**: Warn about AppImages requesting excessive permissions
4. **Checksums**: Verify SHA256 checksums after download
5. **Review**: Show user what files AppImage will access before launch
6. **Isolation**: Isolate AppImages from each other
7. **Network**: Warn about AppImages with network access
8. **X11 Risk**: Warn about X11 forwarding (keylogging risk)

## Integration Points

1. **File Manager**: Double-click to install AppImage
2. **Application Library**: Show AppImages alongside native apps
3. **Software Store**: Browse AppImageHub from software store
4. **Update Manager**: Include AppImages in system updates
5. **Desktop Integration**: Auto-create menu entries
6. **MIME Types**: Handle .AppImage files
7. **Thumbnailer**: Generate thumbnails for AppImages
8. **Context Menu**: Right-click options for AppImages
