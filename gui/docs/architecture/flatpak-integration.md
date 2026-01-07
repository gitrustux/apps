# Phase 9.2: Flatpak Integration (rustica-flatpak)

## Overview

**Component**: rustica-flatpak
**Purpose**: Deep Flatpak integration for sandboxed applications
**Language**: Rust
**Dependencies:** flatpak-rs, dbus (zbus), xdg-desktop-portal

## Goals

1. **First-Class Support**: Flatpak apps work seamlessly alongside native apps
2. **Sandbox Transparency**: Show users what permissions apps are requesting
3. **Portal Integration**: Full xdg-desktop-portal support
4. **Auto-Updates**: Background update checking for Flatpak apps
5. **Remotes Management**: Easy management of Flatpak remotes

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    User Applications                         │
│              (Flatpak apps, native apps)                     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  rustica-flatpak                             │
│                  (Flatpak Integration)                       │
├─────────────────────────────────────────────────────────────┤
│  FlatpakManager     │  PermissionUI    │  RemoteManager    │
│  - Install/Remove   │  - Permission    │  - Add/remove     │
│  - Update           │    dialogs       │  - List remotes   │
│  - List apps        │  - Permission    │  - Enable/disable │
│  - Override perms   │    overrides     │  - Priority       │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              flatpak system library                          │
│              (Flatpak backend)                               │
└─────────────────────────────────────────────────────────────┘
```

## Core Data Structures

```rust
/// Flatpak application reference
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlatpakRef {
    pub kind: RefKind,
    pub id: String,
    pub branch: String,
    pub arch: String,
    pub origin: String,  // Remote name
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RefKind {
    App,
    Runtime,
    Sdk,
}

impl FromStr for FlatpakRef {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        // Parse "app/org.gnome.Builder/x86_64/stable" format
        let parts: Vec<&str> = s.split('/').collect();

        if parts.len() != 4 {
            return Err(Error::InvalidRef);
        }

        let kind = match parts[0] {
            "app" => RefKind::App,
            "runtime" => RefKind::Runtime,
            "sdk" => RefKind::Sdk,
            _ => return Err(Error::InvalidRef),
        };

        Ok(Self {
            kind,
            id: parts[1].to_string(),
            arch: parts[2].to_string(),
            branch: parts[3].to_string(),
            origin: String::new(),
        })
    }
}

impl fmt::Display for FlatpakRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}",
            match self.kind {
                RefKind::App => "app",
                RefKind::Runtime => "runtime",
                RefKind::Sdk => "sdk",
            },
            self.id,
            self.arch,
            self.branch
        )
    }
}

/// Flatpak remote repository
#[derive(Debug, Clone)]
pub struct FlatpakRemote {
    pub name: String,
    pub url: String,
    pub title: Option<String>,
    pub default_branch: String,
    pub enabled: bool,
    pub gpg_verify: bool,
    pub priority: i32,
    pub filter: Vec<String>,  // Package filters
}

/// Flatpak application metadata
#[derive(Debug, Clone)]
pub struct FlatpakApp {
    pub ref_: FlatpakRef,
    pub metadata: AppMetadata,
    pub install_state: InstallState,
    pub permissions: PermissionSet,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallState {
    NotInstalled,
    Installed(InstalledInfo),
    UpdateAvailable(UpdateInfo),
}

#[derive(Debug, Clone)]
pub struct InstalledInfo {
    pub version: String,
    pub installed_size: u64,
    pub install_date: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub size_bytes: u64,
    pub commit: String,
}

/// Flatpak permission set
#[derive(Debug, Clone)]
pub struct PermissionSet {
    pub shared: Vec<SharedNamespace>,
    pub sockets: Vec<Socket>,
    pub devices: Vec<Device>,
    pub filesystems: Vec<FilesystemPermission>,
    pub persistent: Vec<String>,  // Persistent directories
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedNamespace {
    Network,
    Ipc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Socket {
    X11,
    Wayland,
    PulseAudio,
    SessionBus,
    SystemBus,
    FallbackX11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Device {
    Dri,
    All,
    Kvm,
    Shm,
}

#[derive(Debug, Clone)]
pub struct FilesystemPermission {
    pub path: String,
    pub mode: FilesystemMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilesystemMode {
    ReadOnly,
    WriteOnly,
    ReadWrite,
    Create,
}
```

## Flatpak Manager

```rust
pub struct FlatpakManager {
    system_install: Installation,
    user_install: Installation,
    config: FlatpakConfig,
}

impl FlatpakManager {
    pub fn new() -> Result<Self, Error> {
        let system_install = Installation::default_system()?;
        let user_install = Installation::default_user()?;
        let config = FlatpakConfig::load()?;

        Ok(Self {
            system_install,
            user_install,
            config,
        })
    }

    /// List all installed Flatpak apps
    pub async fn list_installed(&self) -> Result<Vec<FlatpakApp>, Error> {
        let mut apps = Vec::new();

        // List system apps
        let system_refs = self.system_install.list_installed_refs()?;
        for ref_ in system_refs {
            if ref_.kind() == RefKind::App {
                apps.push(self.ref_to_app(ref_, &self.system_install)?);
            }
        }

        // List user apps
        let user_refs = self.user_install.list_installed_refs()?;
        for ref_ in user_refs {
            if ref_.kind() == RefKind::App {
                apps.push(self.ref_to_app(ref_, &self.user_install)?);
            }
        }

        Ok(apps)
    }

    /// Search for available apps
    pub async fn search(&self, query: &str) -> Result<Vec<FlatpakApp>, Error> {
        let mut results = Vec::new();

        // Search all enabled remotes
        for remote in self.list_remotes()? {
            if !remote.enabled {
                continue;
            }

            let remote_refs = self.search_remote(&remote, query)?;
            results.extend(remote_refs);
        }

        Ok(results)
    }

    /// Search specific remote
    fn search_remote(&self, remote: &FlatpakRemote, query: &str) -> Result<Vec<FlatpakApp>, Error> {
        let installation = if remote.name.contains("user") {
            &self.user_install
        } else {
            &self.system_install
        };

        let refs = installation.search_remote(&remote.name, query)?;

        let mut apps = Vec::new();
        for ref_ in refs {
            if ref_.kind() == RefKind::App {
                apps.push(self.ref_to_app(ref_, installation)?);
            }
        }

        Ok(apps)
    }

    /// Get app metadata
    pub async fn get_metadata(&self, ref_: &FlatpakRef) -> Result<FlatpakApp, Error> {
        let installation = if ref_.origin.contains("user") {
            &self.user_install
        } else {
            &self.system_install
        };

        self.ref_to_app(ref_, installation)
    }

    /// Install Flatpak app
    pub async fn install(&self, ref_: &FlatpakRef) -> Result<InstallHandle, Error> {
        let installation = if ref_.origin.contains("user") {
            &self.user_install
        } else {
            &self.system_install
        };

        // Check permissions before installing
        let metadata = self.get_metadata(ref_).await?;
        self.request_permission_approval(&metadata)?;

        // Create installation handle
        let (sender, receiver) = channel();

        let ref_clone = ref_.clone();
        let installation_clone = installation.clone();

        tokio::spawn(async move {
            match installation_clone.install(&ref_clone, |progress| {
                let _ = sender.send(InstallProgress::Downloading {
                    progress: progress as f32 / 100.0,
                    bytes_downloaded: 0,
                    total_bytes: 0,
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
            id: ref_.to_string(),
            sender,
        })
    }

    /// Remove Flatpak app
    pub async fn remove(&self, ref_: &FlatpakRef) -> Result<(), Error> {
        let installation = if ref_.origin.contains("user") {
            &self.user_install
        } else {
            &self.system_install
        };

        installation.uninstall(ref_).await?;

        Ok(())
    }

    /// Update Flatpak app
    pub async fn update(&self, ref_: &FlatpakRef) -> Result<(), Error> {
        let installation = if ref_.origin.contains("user") {
            &self.user_install
        } else {
            &self.system_install
        };

        installation.update(ref_).await?;

        Ok(())
    }

    /// Get available updates
    pub async fn get_updates(&self) -> Result<Vec<UpdateInfo>, Error> {
        let mut updates = Vec::new();

        // Check system updates
        let system_updates = self.system_install.list_updates()?;
        for update in system_updates {
            updates.push(UpdateInfo {
                ref_: update.ref_.clone(),
                version: update.version,
                size_bytes: update.size_bytes,
                commit: update.commit,
            });
        }

        // Check user updates
        let user_updates = self.user_install.list_updates()?;
        for update in user_updates {
            updates.push(UpdateInfo {
                ref_: update.ref_.clone(),
                version: update.version,
                size_bytes: update.size_bytes,
                commit: update.commit,
            });
        }

        Ok(updates)
    }

    fn ref_to_app(&self, ref_: FlatpakRef, installation: &Installation) -> Result<FlatpakApp>, Error> {
        let metadata = installation.get_app_metadata(&ref_)?;

        let install_state = if installation.is_installed(&ref_)? {
            let installed_info = installation.get_installed_info(&ref_)?;

            if installation.has_update(&ref_)? {
                let update_info = installation.get_update_info(&ref_)?;
                InstallState::UpdateAvailable(update_info)
            } else {
                InstallState::Installed(installed_info)
            }
        } else {
            InstallState::NotInstalled
        };

        let permissions = self.get_permissions(&ref_, installation)?;

        Ok(FlatpakApp {
            ref_,
            metadata,
            install_state,
            permissions,
            size_bytes: 0,
        })
    }

    fn get_permissions(&self, ref_: &FlatpakRef, installation: &Installation) -> Result<PermissionSet, Error> {
        // Read metadata file
        let metadata_file = installation.get_metadata_file(ref_)?;

        let metadata = FlatpakMetadata::from_file(&metadata_file)?;

        Ok(metadata.permission_set)
    }

    fn request_permission_approval(&self, app: &FlatpakApp) -> Result<(), Error> {
        // Check if auto-approve is enabled
        if self.config.auto_approve_safe_permissions {
            // Only approve safe permissions
            if app.permissions.is_safe() {
                return Ok(());
            }
        }

        // Show permission dialog
        let dialog = FlatpakPermissionDialog {
            app_name: app.metadata.name.clone(),
            permissions: app.permissions.clone(),
        };

        let approved = dialog.show()?;

        if !approved {
            return Err(Error::PermissionsDenied);
        }

        Ok(())
    }
}
```

## Permission Override Manager

```rust
pub struct PermissionOverrideManager {
    overrides_path: PathBuf,
}

impl PermissionOverrideManager {
    pub fn new() -> Result<Self, Error> {
        let overrides_path = PathBuf::from("/var/lib/flatpak/overrides");

        Ok(Self { overrides_path })
    }

    /// Create permission override for app
    pub fn create_override(&self, ref_: &FlatpakRef, permissions: PermissionSet) -> Result<(), Error> {
        let override_file = self.overrides_path.join(ref_.id.replace(".", "_"));

        let content = self.format_override(permissions)?;

        std::fs::write(&override_file, content)?;

        Ok(())
    }

    /// Remove permission override
    pub fn remove_override(&self, ref_: &FlatpakRef) -> Result<(), Error> {
        let override_file = self.overrides_path.join(ref_.id.replace(".", "_"));

        std::fs::remove_file(override_file)?;

        Ok(())
    }

    /// Get current override for app
    pub fn get_override(&self, ref_: &FlatpakRef) -> Result<Option<PermissionSet>, Error> {
        let override_file = self.overrides_path.join(ref_.id.replace(".", "_"));

        if !override_file.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&override_file)?;
        let permissions = self.parse_override(&content)?;

        Ok(Some(permissions))
    }

    fn format_override(&self, permissions: PermissionSet) -> Result<String, Error> {
        let mut output = String::new();

        // Context section
        output.push_str("[Context]\n");

        // Shared namespaces
        for shared in &permissions.shared {
            match shared {
                SharedNamespace::Network => output.push_str("shared=network\n"),
                SharedNamespace::Ipc => output.push_str("shared=ipc\n"),
            }
        }

        // Sockets
        for socket in &permissions.sockets {
            match socket {
                Socket::X11 => output.push_str("sockets=x11\n"),
                Socket::Wayland => output.push_str("sockets=wayland\n"),
                Socket::PulseAudio => output.push_str("sockets=pulseaudio\n"),
                Socket::SessionBus => output.push_str("sockets=session-bus\n"),
                Socket::SystemBus => output.push_str("sockets=system-bus\n"),
                Socket::FallbackX11 => output.push_str("sockets=fallback-x11\n"),
            }
        }

        // Devices
        for device in &permissions.devices {
            match device {
                Device::Dri => output.push_str("devices=dri\n"),
                Device::All => output.push_str("devices=all\n"),
                Device::Kvm => output.push_str("devices=kvm\n"),
                Device::Shm => output.push_str("devices=shm\n"),
            }
        }

        // Filesystems
        for fs in &permissions.filesystems {
            let suffix = match fs.mode {
                FilesystemMode::ReadOnly => ":ro",
                FilesystemMode::WriteOnly => ":wo",
                FilesystemMode::ReadWrite => ":rw",
                FilesystemMode::Create => ":create",
            };

            output.push_str(&format!("filesystem={}{}\n", fs.path, suffix));
        }

        // Persistent directories
        for persistent in &permissions.persistent {
            output.push_str(&format!("persistent={}\n", persistent));
        }

        // Environment variables
        if !permissions.environment.is_empty() {
            output.push_str("\n[Environment]\n");
            for (key, value) in &permissions.environment {
                output.push_str(&format!("{}={}\n", key, value));
            }
        }

        Ok(output)
    }

    fn parse_override(&self, content: &str) -> Result<PermissionSet, Error> {
        // Parse flatpak override file
        // This is simplified
        Ok(PermissionSet::default())
    }
}
```

## Remote Manager

```rust
pub struct RemoteManager {
    system_install: Installation,
    user_install: Installation,
}

impl RemoteManager {
    pub fn new() -> Result<Self, Error> {
        let system_install = Installation::default_system()?;
        let user_install = Installation::default_user()?;

        Ok(Self {
            system_install,
            user_install,
        })
    }

    /// List all remotes
    pub fn list_remotes(&self) -> Result<Vec<FlatpakRemote>, Error> {
        let mut remotes = Vec::new();

        // List system remotes
        let system_remotes = self.system_install.list_remotes()?;
        remotes.extend(system_remotes);

        // List user remotes
        let user_remotes = self.user_install.list_remotes()?;
        remotes.extend(user_remotes);

        Ok(remotes)
    }

    /// Add new remote
    pub async fn add_remote(
        &self,
        name: String,
        url: String,
        user: bool,
        gpg_verify: bool,
    ) -> Result<(), Error> {
        let installation = if user {
            &self.user_install
        } else {
            &self.system_install
        };

        installation.add_remote(
            &name,
            &url,
            gpg_verify,
        ).await?;

        Ok(())
    }

    /// Remove remote
    pub async fn remove_remote(&self, name: String, user: bool) -> Result<(), Error> {
        let installation = if user {
            &self.user_install
        } else {
            &self.system_install
        };

        installation.remove_remote(&name).await?;

        Ok(())
    }

    /// Enable/disable remote
    pub async fn set_remote_enabled(&self, name: String, enabled: bool, user: bool) -> Result<(), Error> {
        let installation = if user {
            &self.user_install
        } else {
            &self.system_install
        };

        installation.set_remote_enabled(&name, enabled).await?;

        Ok(())
    }

    /// Update remote metadata
    pub async fn update_remote(&self, name: String, user: bool) -> Result<(), Error> {
        let installation = if user {
            &self.user_install
        } else {
            &self.system_install
        };

        installation.update_remote(&name).await?;

        Ok(())
    }
}
```

## Portal Integration

```rust
/// xdg-desktop-portal integration for Flatpak apps
pub struct PortalBackend {
    impl_: PortalImpl,
}

impl PortalBackend {
    pub fn new() -> Result<Self, Error> {
        let impl_ = PortalImpl::new()?;

        Ok(Self { impl_ })
    }

    /// Handle file chooser request
    pub async fn file_chooser(&self, app_id: &str, params: FileChooserParams) -> Result<Vec<String>, Error> {
        self.impl_.file_chooser(app_id, params).await
    }

    /// Handle open URI request
    pub async fn open_uri(&self, app_id: &str, uri: String) -> Result<(), Error> {
        self.impl_.open_uri(app_id, uri).await
    }

    /// Handle notification request
    pub async fn notification(&self, app_id: &str, notification: Notification) -> Result<(), Error> {
        self.impl_.notification(app_id, notification).await
    }

    /// Handle screenshot request
    pub async fn screenshot(&self, app_id: &str, params: ScreenshotParams) -> Result<String, Error> {
        self.impl_.screenshot(app_id, params).await
    }

    /// Handle clipboard request
    pub async fn clipboard(&self, app_id: &str, action: ClipboardAction) -> Result<(), Error> {
        self.impl_.clipboard(app_id, action).await
    }
}

#[derive(Debug, Clone)]
pub struct FileChooserParams {
    pub title: String,
    pub mode: FileChooserMode,
    pub multiple: bool,
    pub filters: Vec<FileFilter>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChooserMode {
    OpenFile,
    OpenDirectory,
    SaveFile,
}

#[derive(Debug, Clone)]
pub struct FileFilter {
    pub name: String,
    pub patterns: Vec<String>,
    pub mime_types: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ScreenshotParams {
    pub interactive: bool,
    pub capture_cursor: bool,
}
```

## Configuration

```toml
# /etc/rustica/flatpak.conf
[general]
# Enable Flatpak support
enabled = true

# Auto-add Flathub on first use
auto_add_flathub = true

[updates]
# Check for updates interval (seconds)
check_interval = 86400

# Auto-install Flatpak updates
auto_install = false

# Background download updates
background_download = true

[remotes]
# Default remotes to add
default_remotes = [
    { name = "flathub", url = "https://dl.flathub.org/repo/flathub.flatpakrepo" }
]

# GPG verification
gpg_verify = true

[permissions]
# Auto-approve safe permissions
auto_approve_safe = true

# Show permission warnings
show_warnings = true

# Permission override directory
overrides_dir = "/var/lib/flatpak/overrides"

[interface]
# Show Flatpak badge in app library
show_badge = true

# Group Flatpak apps separately
group_separately = false

# Show runtime information
show_runtime = true
```

## D-Bus Interface

```rust
#[dbus_interface(name = "org.rustica.Flatpak")]
impl FlatpakManager {
    /// List installed Flatpak apps
    async fn list_installed(&self) -> Result<Vec<FlatpakApp>, Error> {
        self.list_installed().await
    }

    /// Search for apps
    async fn search(&self, query: String) -> Result<Vec<FlatpakApp>, Error> {
        self.search(&query).await
    }

    /// Get app metadata
    async fn get_metadata(&self, ref_: String) -> Result<FlatpakApp>, Error> {
        let ref_ = FlatpakRef::from_str(&ref_)?;
        self.get_metadata(&ref_).await
    }

    /// Install app
    async fn install(&self, ref_: String) -> Result<String, Error> {
        let ref_ = FlatpakRef::from_str(&ref_)?;
        let handle = self.install(&ref_).await?;
        Ok(handle.id)
    }

    /// Remove app
    async fn remove(&self, ref_: String) -> Result<(), Error> {
        let ref_ = FlatpakRef::from_str(&ref_)?;
        self.remove(&ref_).await
    }

    /// Update app
    async fn update(&self, ref_: String) -> Result<(), Error> {
        let ref_ = FlatpakRef::from_str(&ref_)?;
        self.update(&ref_).await
    }

    /// List remotes
    fn list_remotes(&self) -> Result<Vec<FlatpakRemote>, Error> {
        RemoteManager::new()?.list_remotes()
    }

    /// Add remote
    async fn add_remote(&self, name: String, url: String, user: bool) -> Result<(), Error> {
        RemoteManager::new()?.add_remote(name, url, user, true).await
    }

    /// Remove remote
    async fn remove_remote(&self, name: String, user: bool) -> Result<(), Error> {
        RemoteManager::new()?.remove_remote(name, user).await
    }

    /// Create permission override
    fn create_permission_override(&self, ref_: String, permissions: PermissionSet) -> Result<(), Error> {
        let ref_ = FlatpakRef::from_str(&ref_)?;
        PermissionOverrideManager::new()?.create_override(&ref_, permissions)
    }

    /// Remove permission override
    fn remove_permission_override(&self, ref_: String) -> Result<(), Error> {
        let ref_ = FlatpakRef::from_str(&ref_)?;
        PermissionOverrideManager::new()?.remove_override(&ref_)
    }
}
```

## Security Considerations

1. **Sandbox Verification**: Verify all Flatpak apps are properly sandboxed
2. **Permission Auditing**: Log all permission requests and grants
3. **Override Safety**: Warn about permission overrides
4. **Remote GPG**: Enforce GPG verification for remotes
5. **Portal Validation**: Validate all portal requests

## Dependencies

```toml
[dependencies]
flatpak-rs = "0.5"
zbus = "4"
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
    async fn test_list_installed() {
        let manager = FlatpakManager::new().unwrap();
        let apps = manager.list_installed().await.unwrap();

        // Should return a list (may be empty)
        assert!(apps.len() >= 0);
    }

    #[tokio::test]
    async fn test_search() {
        let manager = FlatpakManager::new().unwrap();
        let results = manager.search("firefox").await.unwrap();

        // Should find Firefox in Flathub
        assert!(results.iter().any(|app| app.ref_.id.contains("firefox")));
    }

    #[test]
    fn test_flatpak_ref_parsing() {
        let ref_str = "app/org.gnome.Builder/x86_64/stable";
        let ref_ = FlatpakRef::from_str(ref_str).unwrap();

        assert_eq!(ref_.kind, RefKind::App);
        assert_eq!(ref_.id, "org.gnome.Builder");
        assert_eq!(ref_.arch, "x86_64");
        assert_eq!(ref_.branch, "stable");
    }

    #[test]
    fn test_permission_override() {
        let manager = PermissionOverrideManager::new().unwrap();

        let ref_ = FlatpakRef {
            kind: RefKind::App,
            id: "com.example.Test".to_string(),
            arch: "x86_64".to_string(),
            branch: "stable".to_string(),
            origin: "flathub".to_string(),
        };

        let permissions = PermissionSet {
            shared: vec![SharedNamespace::Network],
            ..Default::default()
        };

        manager.create_override(&ref_, permissions).unwrap();
        manager.remove_override(&ref_).unwrap();
    }
}
```

## Future Enhancements

1. **Flatpak Bundles**: Support for .flatpak bundle files
2. **Runtime Management**: Manage multiple runtime versions
3. **Automatic Overrides**: Suggest permission overrides based on usage
4. **Delta Updates**: Implement delta update support
5. **OCI Remotes**: Support for OCI-based Flatpak remotes
6. **App Streams**: Enhanced appstream metadata display
7. **Error Recovery**: Better error handling and recovery
8. **Installation Verification**: Verify installation integrity
9. **Background Installation**: Queue installations for later
10. **Dependency Visualization**: Show runtime dependencies
