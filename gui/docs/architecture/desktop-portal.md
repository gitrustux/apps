# Desktop Portal (xdg-desktop-portal-rustica) Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - XDG Desktop Portal
**Phase**: 7.1 - Integration & Polish

## Overview

The **XDG Desktop Portal** is a **D-Bus service** that allows sandboxed applications (Flatpaks, Snaps) to request services from the desktop environment in a secure, controlled manner. It provides **file chooser dialogs**, **screenshot portals**, **account portals**, **printing**, **settings**, and more while maintaining **security boundaries**.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Desktop Portal Backend                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────────┐  │
│  │ File        │  │ Screenshot  │  │ Account     │  │ AppChooser     │  │
│  │ Chooser     │  │ Portal      │  │ Portal      │  │ Portal         │  │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └───────┬───────┘  │
│         │                │                │                  │          │
│         ▼                ▼                ▼                  ▼          │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    Portal Frontend Requests                      │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                    │                                 │
│                                    ▼                                 │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                  Portal Frontend (librustica)                    │  │
│  │  - Shows dialogs                                                 │  │
│  │  - Handles user interaction                                      │  │
│  │  - Returns results to backend                                   │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                           │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                       Security Layer                              │  │
│  │  - Permission checks                                             │  │
│  │  - Sandbox enforcement                                          │  │
│  │  - Access control                                                │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                           │
└─────────────────────────────────────────────────────────────────────────┘
                            ▲
                            │ D-Bus
                            │
        ┌───────────────────┴───────────────────┐
        │                                       │
┌───────┴───────┐                       ┌───────┴───────┐
│  Flatpak App  │                       │  Snap App    │
│  (Sandboxed)  │                       │  (Sandboxed) │
└───────────────┘                       └───────────────┘
```

## Main Portal Structure

```rust
pub struct DesktopPortal {
    /// D-Bus connection
    dbus: Connection,

    /// Frontend (handles dialogs)
    frontend: PortalFrontend,

    /// File chooser portal
    file_chooser: FileChooserPortal,

    /// Screenshot portal
    screenshot: ScreenshotPortal,

    /// Account portal
    account: AccountPortal,

    /// App chooser portal
    app_chooser: AppChooserPortal,

    /// Inhibit portal
    inhibit: InhibitPortal,

    /// Background portal
    background: BackgroundPortal,

    /// Settings portal
    settings: SettingsPortal,

    /// Permissions store
    permissions: PermissionStore,

    /// Session state
    session: PortalSession,
}

pub struct PortalSession {
    /// Session ID
    id: String,

    /// App ID
    app_id: String,

    /// Created at
    created_at: DateTime<Utc>,

    /// Permissions granted
    permissions: HashSet<String>,
}

pub struct PortalFrontend {
    /// Window for dialogs
    window: Window,

    /// Current request
    current_request: Option<FrontendRequest>,
}

pub enum FrontendRequest {
    FileChooser(FileChooserRequest),
    Screenshot(ScreenshotRequest),
    Account(AccountRequest),
    AppChooser(AppChooserRequest),
}
```

## File Chooser Portal

```rust
pub struct FileChooserPortal {
    /// Permissions
    permissions: FilePermissions,

    /// Last used directories
    recent_dirs: Vec<PathBuf>,
}

pub struct FileChooserRequest {
    /// Request handle
    pub handle: String,

    /// App ID
    pub app_id: String,

    /// Parent window ID
    pub parent_window: String,

    /// Title
    pub title: String,

    /// Mode
    pub mode: FileChooserMode,

    /// Accept label
    pub accept_label: Option<String>,

    /// Multiple selection
    pub multiple: bool,

    /// Directory mode
    pub directory: bool,

    /// Filters
    pub filters: Vec<FileFilter>,

    /// Current folder
    pub current_folder: PathBuf,

    /// Current file
    pub current_file: Option<PathBuf>,

    /// Choices (for file types)
    pub choices: Vec<FileChoice>,
}

pub enum FileChooserMode {
    Open,
    Save,
    OpenDirectory,
    SaveDirectory,
}

pub struct FileFilter {
    /// Filter name
    pub name: String,

    /// Patterns (e.g., "*.txt", "*.png")
    pub patterns: Vec<String>,

    /// MIME types
    pub mime_types: Vec<String>,
}

pub struct FileChoice {
    /// Choice ID
    pub id: String,

    /// Label
    pub label: String,

    /// Options
    pub options: Vec<FileChoiceOption>,
}

pub struct FileChoiceOption {
    /// Option ID
    pub id: String,

    /// Label
    pub label: String,
}

impl FileChooserPortal {
    pub async fn open_file(&mut self, request: FileChooserRequest) -> Result<Vec<url::Url>, Error> {
        // Check permissions
        self.check_file_permission(&request.app_id, &request.current_folder)?;

        // Show file chooser dialog via frontend
        let result = self.frontend.show_file_chooser(request).await?;

        // Remember directory
        if let Some(first) = result.first() {
            if let Ok(path) = first.to_file_path() {
                if let Some(parent) = path.parent() {
                    self.remember_directory(parent);
                }
            }
        }

        Ok(result)
    }

    pub async fn save_file(&mut self, request: FileChooserRequest) -> Result<url::Url, Error> {
        // Check permissions
        self.check_file_permission(&request.app_id, &request.current_folder)?;

        // Show save dialog
        let result = self.frontend.show_save_dialog(request).await?;

        // Create parent directories if needed
        if let Ok(path) = result.to_file_path() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
        }

        Ok(result)
    }

    fn check_file_permission(&self, app_id: &str, path: &Path) -> Result<(), Error> {
        // Check if app has permission to access this path
        // Flatpaks have limited filesystem access

        // XDG directories (always allowed)
        let xdg_dirs = vec![
            dirs::home_dir(),
            dirs::document_dir(),
            dirs::download_dir(),
            dirs::picture_dir(),
            dirs::music_dir(),
            dirs::video_dir(),
        ];

        if let Some(home) = dirs::home_dir() {
            if path.starts_with(&home) {
                return Ok(());
            }
        }

        // Check permission store
        if self.permissions.check_access(app_id, path) {
            return Ok(());
        }

        // Otherwise, need to ask user
        self.prompt_for_permission(app_id, path)?;

        Ok(())
    }

    fn prompt_for_permission(&mut self, app_id: &str, path: &Path) -> Result<(), Error> {
        // Show permission dialog
        let allowed = self.frontend.show_permission_dialog(
            app_id,
            path,
            "This application wants to access files outside your home directory.",
        )?;

        if allowed {
            // Grant permission
            self.permissions.grant_access(app_id, path);
            Ok(())
        } else {
            Err(Error::PermissionDenied)
        }
    }
}
```

## Screenshot Portal

```rust
pub struct ScreenshotPortal {
    /// Permissions
    permissions: HashSet<String>,
}

pub struct ScreenshotRequest {
    /// Request handle
    pub handle: String,

    /// App ID
    pub app_id: String,

    /// Parent window
    pub parent_window: String,

    /// Mode
    pub mode: ScreenshotMode,

    /// Interactive
    pub interactive: bool,

    /// Allow monitors
    pub allow_monitors: bool,

    /// Allow windows
    pub allow_windows: bool,
}

pub enum ScreenshotMode {
    Fullscreen,
    Monitor,
    Window,
    Area,
}

impl ScreenshotPortal {
    pub async fn screenshot(&mut self, request: ScreenshotRequest) -> Result<(url::Url, Image), Error> {
        // Check permission
        if !self.permissions.contains(&request.app_id) {
            let allowed = self.frontend.show_permission_dialog(
                &request.app_id,
                Path::new("<screen>"),
                "This application wants to take a screenshot.",
            )?;

            if !allowed {
                return Err(Error::PermissionDenied);
            }

            self.permissions.insert(request.app_id.clone());
        }

        // Take screenshot
        let (path, image) = match request.mode {
            ScreenshotMode::Fullscreen => {
                self.screenshot_fullscreen().await?
            }

            ScreenshotMode::Monitor if request.allow_monitors => {
                self.screenshot_monitor(request.interactive).await?
            }

            ScreenshotMode::Window if request.allow_windows => {
                self.screenshot_window(request.interactive).await?
            }

            ScreenshotMode::Area => {
                self.screenshot_area(request.interactive).await?
            }

            _ => {
                return Err(Error::UnsupportedMode);
            }
        };

        // Save to temp file
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let filename = format!("screenshot-{}.png", timestamp);
        let temp_dir = std::env::var("TMPDIR").unwrap_or("/tmp".into());
        let path = PathBuf::from(temp_dir).join(&filename);
        image.save(&path)?;

        let url = url::Url::from_file_path(&path)
            .map_err(|_| Error::InvalidPath)?;

        Ok((url, image))
    }

    async fn screenshot_fullscreen(&self) -> Result<(PathBuf, Image), Error> {
        // Capture all outputs
        let capture = ScreenCapture::new()?;
        let region = self.get_fullscreen_region()?;
        let image = capture.capture_frame(&region)?;

        // Generate filename
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let filename = format!("screenshot-{}.png", timestamp);

        let pictures_dir = dirs::picture_dir()
            .unwrap_or_else(|| dirs::home_dir().map(|h| h.join(".local/share")).unwrap());

        let path = pictures_dir.join(&filename);

        Ok((path, image))
    }

    async fn screenshot_area(&self, interactive: bool) -> Result<(PathBuf, Image), Error> {
        if interactive {
            // Show area selector
            let region = self.frontend.show_area_selector().await?;
        } else {
            // Default to primary output
            let region = self.get_primary_output_region()?;
        }

        // Capture region
        let capture = ScreenCapture::new()?;
        let image = capture.capture_frame(&region)?;

        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let filename = format!("screenshot-{}.png", timestamp);

        let path = dirs::picture_dir()
            .unwrap_or_else(|| dirs::home_dir().map(|h| h.join(".local/share")).unwrap())
            .join(&filename);

        Ok((path, image))
    }
}
```

## Account Portal

```rust
pub struct AccountPortal {
    /// User information provider
    user_info: UserInfoProvider,
}

pub struct AccountRequest {
    /// Request handle
    pub handle: String,

    /// App ID
    pub app_id: String,

    /// Reason for requesting user info
    pub reason: Option<String>,
}

pub struct UserInfo {
    /// User ID
    pub id: String,

    /// Real name
    pub name: String,

    /// Username
    pub username: String,

    /// Image URI (avatar)
    pub image: Option<String>,

    /// Language
    pub language: String,

    /// Layout (keyboard)
    pub layout: String,

    /// Layout variant
    pub layout_variant: Option<String>,

    /// Session (desktop session)
    pub session: String,
}

impl AccountPortal {
    pub async fn get_user_info(&self, request: AccountRequest) -> Result<UserInfo, Error> {
        // Show user permission dialog
        let allowed = self.frontend.show_permission_dialog(
            &request.app_id,
            Path::new("<user>"),
            request.reason.as_deref()
                .unwrap_or("This application wants to access your user information."),
        )?;

        if !allowed {
            return Err(Error::PermissionDenied);
        }

        // Get user info
        Ok(self.user_info.get_user_info()?)
    }
}
```

## App Chooser Portal

```rust
pub struct AppChooserPortal {
    /// Application registry
    registry: AppRegistry,
}

pub struct AppChooserRequest {
    /// Request handle
    pub handle: String,

    /// App ID
    pub app_id: String,

    /// Parent window
    pub parent_window: String,

    /// Content type
    pub content_type: String,

    /// Choices (specific apps)
    pub choices: Vec<AppChoice>,

    /// Recent choices
    pub recent_choices: bool,

    /// Current choice
    pub current_choice: Option<String>,
}

pub struct AppChoice {
    /// App ID
    pub id: String,

    /// Name
    pub name: String,

    /// Icon
    pub icon: String,

    /// Priority
    pub priority: u32,
}

impl AppChooserPortal {
    pub async fn choose_application(&mut self, request: AppChooserRequest) -> Result<String, Error> {
        // Get applications that can handle this content type
        let apps = if request.choices.is_empty() {
            self.registry.get_apps_for_content_type(&request.content_type)?
        } else {
            // Use provided choices
            request.choices
        };

        // Show app chooser dialog
        let chosen_app_id = self.frontend.show_app_chooser(
            &request.app_id,
            &request.content_type,
            apps,
            request.recent_choices,
        ).await?;

        // Remember choice
        if let Some(ref choice) = chosen_app_id {
            self.remember_choice(&request.content_type, choice);
        }

        chosen_app_id.ok_or(Error::NoChoice)
    }

    fn remember_choice(&mut self, content_type: &str, app_id: &str) {
        // Store in preference file
        let pref_path = dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().map(|h| h.join(".config")).unwrap())
            .join("rustica/portal/app-choices.toml");

        // Load existing preferences
        let mut preferences = if pref_path.exists() {
            let content = std::fs::read_to_string(&pref_path)?;
            toml::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        // Update preference
        preferences.insert(content_type.to_string(), app_id.to_string());

        // Save
        std::fs::write(&pref_path, toml::to_string_pretty(&preferences)?)?;

        Ok(())
    }
}
```

## Inhibit Portal

```rust
pub struct InhibitPortal {
    /// Active inhibitions
    inhibitions: HashMap<String, Inhibition>,

    /// Session manager client
    session_manager: SessionManagerProxy,
}

pub struct Inhibition {
    /// Inhibition ID
    pub id: String,

    /// App ID
    pub app_id: String,

    /// Reason
    pub reason: String,

    /// Flags
    pub flags: InhibitFlags,
}

pub struct InhibitFlags {
    /// Logout
    pub logout: bool,

    /// User switch
    pub switch: bool,

    /// Suspend
    pub suspend: bool,

    /// Idle
    pub idle: bool,
}

impl InhibitPortal {
    pub async fn inhibit(&mut self, request: InhibitRequest) -> Result<String, Error> {
        // Create inhibition
        let inhibition_id = uuid::Uuid::new_v4().to_string();

        let inhibition = Inhibition {
            id: inhibition_id.clone(),
            app_id: request.app_id,
            reason: request.reason,
            flags: request.flags,
        };

        // Register with session manager
        self.session_manager.inhibit_logout(
            inhibition.app_id.clone(),
            inhibition.reason.clone(),
            inhibition.app_id.clone(),
            (
                inhibition.flags.logout,
                inhibition.flags.switch,
                inhibition.flags.suspend,
                inhibition.flags.idle,
            ),
        ).await?;

        // Store inhibition
        self.inhibitions.insert(inhibition_id.clone(), inhibition);

        Ok(inhibition_id)
    }

    pub async fn uninhibit(&mut self, inhibition_id: String) -> Result<(), Error> {
        if let Some(inhibition) = self.inhibitions.remove(&inhibition_id) {
            // Unregister with session manager
            self.session_manager.uninhibit_logout(inhibition_id).await?;
        }

        Ok(())
    }
}

pub struct InhibitRequest {
    /// App ID
    pub app_id: String,

    /// Toplevel window ID
    pub window: String,

    /// Reason
    pub reason: String,

    /// Flags
    pub flags: InhibitFlags,
}
```

## Background Portal

```rust
pub struct BackgroundPortal {
    /// Permissions
    permissions: PermissionStore,

    /// Running background apps
    background_apps: HashMap<String, BackgroundApp>,
}

pub struct BackgroundApp {
    /// App ID
    pub id: String,

    /// Command
    pub command: Vec<String>,

    /// Started at
    pub started_at: DateTime<Utc>,
}

pub struct BackgroundRequest {
    /// App ID
    pub app_id: String,

    /// Command to run
    pub command: Vec<String>,

    /// Reason
    pub reason: String,
}

impl BackgroundPortal {
    pub async fn request_background(&mut self, request: BackgroundRequest) -> Result<String, Error> {
        // Check permission
        if !self.permissions.has_background_permission(&request.app_id) {
            let allowed = self.frontend.show_permission_dialog(
                &request.app_id,
                Path::new("<background>"),
                &format!(
                    "This application wants to run in the background. Reason: {}",
                    request.reason
                ),
            )?;

            if !allowed {
                return Err(Error::PermissionDenied);
            }

            self.permissions.grant_background_permission(&request.app_id);
        }

        // Start background process
        let app_id = self.spawn_background(request)?;

        Ok(app_id)
    }

    fn spawn_background(&mut self, request: BackgroundRequest) -> Result<String, Error> {
        let mut command = std::process::Command::new(&request.command[0]);
        command.args(&request.command[1..]);

        // Detach process
        #[cfg(unix)]
        unsafe {
            command.pre_exec(|| {
                // Fork to background
                libc::setsid();
                Ok(())
            });
        }

        let child = command.spawn()?;

        let app = BackgroundApp {
            id: request.app_id.clone(),
            command: request.command,
            started_at: Utc::now(),
        };

        self.background_apps.insert(app.id.clone(), app);

        Ok(app.id)
    }
}
```

## Settings Portal

```rust
pub struct SettingsPortal {
    /// Settings daemon client
    settings_daemon: SettingsDaemonProxy,

    /// Allowed read-only settings
    read_only: Vec<String>,

    /// Allowed read-write settings
    read_write: Vec<String>,
}

impl SettingsPortal {
    pub async fn read_setting(&self, app_id: &str, namespace: &str, key: &str) -> Result<SettingValue, Error> {
        // Check read permission
        let setting_key = format!("{}.{}", namespace, key);

        if !self.can_read(app_id, &setting_key) {
            return Err(Error::PermissionDenied);
        }

        // Read from settings daemon
        Ok(self.settings_daemon.get_setting(setting_key).await?)
    }

    pub async fn write_setting(&mut self, app_id: &str, namespace: &str, key: &str, value: SettingValue) -> Result<(), Error> {
        // Check write permission
        let setting_key = format!("{}.{}", namespace, key);

        if !self.can_write(app_id, &setting_key) {
            return Err(Error::PermissionDenied);
        }

        // Write to settings daemon
        self.settings_daemon.set_setting(setting_key, value).await?;

        Ok(())
    }

    fn can_read(&self, app_id: &str, key: &str) -> bool {
        // Check if key is in read-only list
        if self.read_only.contains(&key.to_string()) {
            return true;
        }

        // Check read-write list
        self.read_write.contains(&key.to_string())
    }

    fn can_write(&self, app_id: &str, key: &str) -> bool {
        self.read_write.contains(&key.to_string())
    }
}
```

## Permission Store

```rust
pub struct PermissionStore {
    /// Filesystem access permissions
    filesystem: HashMap<String, Vec<PathPermission>>,

    /// Background permissions
    background: HashSet<String>,

    /// Screenshot permissions
    screenshots: HashSet<String>,

    /// Inhibit permissions
    inhibit: HashSet<String>,

    /// Settings permissions
    settings: HashMap<String, SettingsPermission>,
}

pub struct PathPermission {
    /// Path
    pub path: PathBuf,

    /// Permission
    pub permission: PathPermissionType,
}

pub enum PathPermissionType {
    Read,
    Write,
    Execute,
}

pub enum SettingsPermission {
    ReadOnly,
    ReadWrite,
}

impl PermissionStore {
    pub fn load() -> Result<Self, Error> {
        let config_path = dirs::config_dir()
            .ok_or(Error::NoConfigDir)?
            .join("rustica/portal/permissions.toml");

        let permissions = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            Self::default()
        };

        Ok(permissions)
    }

    pub fn check_access(&self, app_id: &str, path: &Path) -> bool {
        if let Some(perms) = self.filesystem.get(app_id) {
            perms.iter().any(|p| path.starts_with(&p.path))
        } else {
            false
        }
    }

    pub fn grant_access(&mut self, app_id: &str, path: &Path) {
        self.filesystem
            .entry(app_id.to_string())
            .or_insert_with(Vec::new)
            .push(PathPermission {
                path: path.to_path_buf(),
                permission: PathPermissionType::Read,
            });

        self.save();
    }

    pub fn has_background_permission(&self, app_id: &str) -> bool {
        self.background.contains(app_id)
    }

    pub fn grant_background_permission(&mut self, app_id: &str) {
        self.background.insert(app_id.to_string());
        self.save();
    }

    fn save(&self) {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().map(|h| h.join(".config")).unwrap())
            .join("rustica/portal/permissions.toml");

        std::fs::create_dir_all(config_path.parent().unwrap()).ok();

        std::fs::write(&config_path, toml::to_string_pretty(self).ok()).ok();
    }
}
```

## D-Bus Interface

```rust
// Main portal service name: org.freedesktop.portal.Desktop
// Object path: /org/freedesktop/portal/desktop

#[dbus_interface(name = "org.freedesktop.portal.FileChooser")]
impl FileChooserPortal {
    /// Open file
    fn open_file(
        &self,
        parent_window: String,
        title: String,
        options: FileChooserOptions,
    ) -> Result<String, Error> {
        // Returns request handle
    }

    /// Save file
    fn save_file(
        &self,
        parent_window: String,
        title: String,
        options: FileChooserOptions,
    ) -> Result<String, Error> {
        // Returns request handle
    }
}

#[dbus_interface(name = "org.freedesktop.portal.Screenshot")]
impl ScreenshotPortal {
    /// Screenshot
    fn screenshot(
        &self,
        parent_window: String,
        options: ScreenshotOptions,
    ) -> Result<String, Error> {
        // Returns request handle
    }

    /// Pick color
    fn pick_color(
        &self,
        parent_window: String,
        options: PickColorOptions,
    ) -> Result<String, Error> {
        // Returns request handle
    }
}

#[dbus_interface(name = "org.freedesktop.portal.Inhibit")]
impl InhibitPortal {
    /// Inhibit session
    fn inhibit(
        &self,
        parent_window: String,
        reasons: u32,
        flags: u32,
    ) -> Result<String, Error> {
        // Returns inhibition ID
    }
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── xdg-desktop-portal-rustica/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── portal.rs
│       ├── file_chooser.rs
│       ├── screenshot.rs
│       ├── account.rs
│       ├── app_chooser.rs
│       ├── inhibit.rs
│       ├── background.rs
│       ├── settings.rs
│       ├── permissions.rs
│       └── frontend.rs
└── systemd/
    └── xdg-desktop-portal-rustica.service
```

## Systemd Service

```ini
[Unit]
Description=Rustica XDG Desktop Portal
Documentation=man:xdg-desktop-portal(1)
After=dbus.service

[Service]
Type=dbus
BusName=org.freedesktop.portal.Desktop
ExecStart=/usr/libexec/xdg-desktop-portal-rustica
Restart=on-failure

[Install]
WantedBy=graphical-session.target
```

## Dependencies

```toml
[package]
name = "xdg-desktop-portal-rustica"
version = "1.0.0"
edition = "2021"

[dependencies]
# D-Bus
zbus = "3.0"
zvariant = "3.0"

# Portal frontend (librustica)
librustica = { path = "../../../libs/librustica" }

# Screen capture
pipewire = "0.1"

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# XDG
dirs = "5.0"

# UUID
uuid = { version = "1.0", features = ["v4"] }
```

## Success Criteria

- [ ] All portals functional
- [ ] File chooser works
- [ ] Screenshot works
- [ ] Account portal works
- [ ] App chooser works
- [ ] Inhibit works
- [ ] Background works
- [ ] Settings works
- [ ] Permission store works
- [ ] Security enforced
- [ ] Flatpak compatible

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## Timeline

- Week 1: Core portal + D-Bus interface
- Week 2: File chooser portal
- Week 3: Screenshot portal
- Week 4: Account + app chooser portals
- Week 5: Inhibit + background portals
- Week 6: Settings portal + permission store

**Total**: 6 weeks
