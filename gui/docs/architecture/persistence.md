# Persistence & State Model Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - State Management

## Overview

This specification defines how Rustica Shell persists and manages state across sessions. It ensures **<100ms state restore on startup**, **automatic schema migration**, and **synchronous per-user and system-wide configuration**.

## Design Philosophy

1. **Explicit State** - All state is declarative and versioned
2. **Safe Migration** - Schema changes are backward compatible
3. **Fast Restore** - State restore doesn't block compositor startup
4. **Clear Separation** - User config, system config, and runtime state are separate

## File System Layout

### XDG Compliance

```bash
# Per-user configuration
$HOME/.config/rustica/shell/
├── config.toml              # Main shell configuration
├── theme.toml               # Theme preferences
├── workspaces.toml          # Workspace layout
├── panels/
│   ├── top-panel.toml       # Top panel config
│   └── dock.toml            # Dock config
├── apps/
│   └── favorites.toml       # Favorite applications
└── state/
    ├── session.toml         # Current session state
    ├── windows.toml         # Window positions
    └── runtime.toml         # Runtime state (last state)

# Per-user data
$HOME/.local/share/rustica/shell/
├── backgrounds/             # Wallpaper images
├── themes/                  # Custom themes
└── crash_reports/           # Crash dumps

# Per-user cache
$HOME/.cache/rustica/shell/
├── thumbnails/              # Image thumbnails
├── icons/                   # Cached icon data
└── shaders/                 # Compiled shaders

# System-wide configuration
/etc/rustica/shell/
├── system.toml              # System-wide defaults
├── policies/                # Admin policies
│   └── restrictions.toml    # Capability restrictions
└── themes/                  # System themes
    └── default.toml         # Default theme

# Runtime state
/run/rustica/shell/
├── compositor.lock          # Compositor lock file
└── wayland-0               # Wayland display socket
```

## Configuration Schema

### Main Configuration (config.toml)

```toml
# Schema version (required for migration)
schema_version = "1.0.0"

# Desktop environment
[desktop]
# Session type: "desktop" | "mobile" | "server"
session_type = "desktop"

# Display configuration
[display]
# Scale factor for DPI scaling (1.0 = 100%, 2.0 = 200%)
scale = 1.0

# HDR support
hdr_enabled = false

# VSync
vsync = true

# Target framerate (60, 120, 144)
target_framerate = 60

# Theme configuration
[theme]
# Theme mode: "light" | "dark" | "high_contrast"
mode = "dark"

# Custom theme path (optional)
# custom_path = "/home/user/.local/share/rustica/themes/my-theme.toml"

# Follow system theme
follow_system = true

# Window management
[windows]
# Focus model: "click_to_focus" | "sloppy" | "focus_follows_mouse"
focus_model = "click_to_focus"

# Focus stealing prevention
prevent_stealing = true

# Remember window positions
remember_positions = true

# Default window decoration
decorations_enabled = true

# Animation duration (ms)
animation_duration = 200

# Workspaces
[workspaces]
# Number of workspaces
count = 4

# Workspace names (empty = default)
names = ["Main", "Work", "Communication", "Media"]

# Dynamic workspaces (create on demand)
dynamic = false

# Input configuration
[input]
# Keyboard repeat rate (ms)
repeat_delay = 400
repeat_rate = 40

# Touchpad settings
[input.touchpad]
# Tap to click
tap_to_click = true

# Natural scrolling
natural_scroll = true

# Disable while typing
disable_while_typing = true

# Two-finger scroll
two_finger_scroll = true

# Pointer acceleration
[input.pointer]
acceleration = 0.5
acceleration_profile = "adaptive"  # "adaptive" | "flat"

# Accessibility
[accessibility]
# High contrast mode
high_contrast = false

# Reduce motion (disable animations)
reduce_motion = false

# Screen reader enabled
screen_reader = false

# Font scaling (1.0 = 100%)
font_scale = 1.0

# Large text
large_text = false

# Notifications
[notifications]
# Enable notifications
enabled = true

# Notification position: "top_right" | "top_left" | "bottom_right" | "bottom_left"
position = "top_right"

# Do not disturb mode
dnd = false

# Critical notifications bypass DND
critical_bypass_dnd = true

# Privacy
[privacy]
# Screen recording protection
warn_on_screen_record = true

# Input monitoring protection
warn_on_input_monitor = true
```

### Theme Configuration (theme.toml)

```toml
schema_version = "1.0.0"

# Color overrides
[colors]
# Custom primary color
# primary = "#1A73E8"

# Custom accent color
# accent = "#009688"

# Font overrides
[fonts]
# Custom font family
# sans = "Inter"
# mono = "JetBrains Mono"

# Font sizes (multipliers)
# h1_scale = 1.0
# body_scale = 1.0
```

### Workspace Configuration (workspaces.toml)

```toml
schema_version = "1.0.0"

[[workspace]]
id = 0
name = "Main"
# Workspace-specific layout rules
layout = "tiling"

[[workspace]]
id = 1
name = "Work"
layout = "stacking"
```

### Panel Configuration (panels/top-panel.toml)

```toml
schema_version = "1.0.0"

# Panel position: "top" | "bottom" | "left" | "right"
position = "top"

# Panel size
size = 48

# Auto-hide
auto_hide = false

# Widgets (left to right)
[[widgets]]
type = "app_launcher"
order = 0

[[widgets]]
type = "workspace_switcher"
order = 1

[[widgets]]
type = "taskbar"
order = 2
show_all_workspaces = false

[[widgets]]
type = "system_tray"
order = 3

[[widgets]]
type = "clock"
order = 4
format = "%H:%M"
show_date = false
```

### Session State (state/session.toml)

```toml
schema_version = "1.0.0"
last_modified = 2025-01-07T10:30:00Z

# Current workspace
current_workspace = 1

# Active applications
[[application]]
app_id = "org.mozilla.firefox"
pid = 1234
workspace = 1

[[application]]
app_id = "com.spotify.Client"
pid = 5678
workspace = 1
```

### Window State (state/windows.toml)

```toml
schema_version = "1.0.0"

[[window]]
app_id = "org.mozilla.firefox"
title = "Mozilla Firefox"
workspace = 1
# Window state: "normal" | "maximized" | "fullscreen" | "minimized"
state = "normal"
position = { x = 100, y = 100 }
size = { width = 1920, height = 1080 }
# Monitor index (for multi-monitor)
monitor = 0

[[window]]
app_id = "com.spotify.Client"
title = "Spotify"
workspace = 1
state = "normal"
position = { x = 100, y = 500 }
size = { width = 800, height = 600 }
monitor = 0
```

## State Management API

### Configuration Manager

```rust
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub struct ConfigManager {
    // Config directories
    config_dir: PathBuf,
    data_dir: PathBuf,
    cache_dir: PathBuf,

    // Loaded configuration
    config: ShellConfig,
    theme: ThemeConfig,
    workspaces: WorkspaceConfig,

    // State cache
    state_cache: Option<StateCache>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShellConfig {
    pub schema_version: String,
    pub desktop: DesktopConfig,
    pub display: DisplayConfig,
    pub theme: ThemePreference,
    pub windows: WindowsConfig,
    pub workspaces: WorkspacesConfig,
    pub input: InputConfig,
    pub accessibility: AccessibilityConfig,
    pub notifications: NotificationConfig,
    pub privacy: PrivacyConfig,
}

impl ConfigManager {
    /// Load configuration from XDG paths
    pub fn load() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or(Error::NoConfigDir)?
            .join("rustica/shell");

        let config = Self::load_config(&config_dir)?;

        Ok(Self {
            config_dir,
            data_dir: dirs::data_local_dir()
                .ok_or(Error::NoDataDir)?
                .join("rustica/shell"),
            cache_dir: dirs::cache_dir()
                .ok_or(Error::NoCacheDir)?
                .join("rustica/shell"),
            config,
            ..Default::default()
        })
    }

    /// Load main configuration with fallbacks
    fn load_config(config_dir: &PathBuf) -> Result<ShellConfig> {
        // 1. Try user config
        let user_config = config_dir.join("config.toml");
        if user_config.exists() {
            let content = fs::read_to_string(&user_config)?;
            let mut config: ShellConfig = toml::from_str(&content)?;

            // Check schema version and migrate if needed
            if config.schema_version != CURRENT_SCHEMA_VERSION {
                config = Self::migrate_config(config)?;
            }

            return Ok(config);
        }

        // 2. Fallback to system config
        let system_config = PathBuf::from("/etc/rustica/shell/system.toml");
        if system_config.exists() {
            let content = fs::read_to_string(&system_config)?;
            return Ok(toml::from_str(&content)?);
        }

        // 3. Fallback to defaults
        Ok(ShellConfig::default())
    }

    /// Save configuration to user directory
    pub fn save(&self) -> Result<()> {
        // Ensure config directory exists
        fs::create_dir_all(&self.config_dir)?;

        // Write config file
        let config_path = self.config_dir.join("config.toml");
        let content = toml::to_string_pretty(&self.config)?;
        fs::write(&config_path, content)?;

        Ok(())
    }

    /// Get a configuration value with override support
    pub fn get<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        // Check user override
        if let Some(value) = self.get_override(key) {
            return Some(value);
        }

        // Check system default
        self.get_system_default(key)
    }

    /// Set a configuration value (user override)
    pub fn set<T>(&mut self, key: &str, value: T) -> Result<()>
    where
        T: Serialize,
    {
        // Store in user overrides
        self.set_override(key, value)?;
        self.save()?;
        Ok(())
    }
}
```

### State Persistence

```rust
pub struct StateManager {
    config: ConfigManager,
    session_state: SessionState,
    window_state: WindowState,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionState {
    pub schema_version: String,
    pub last_modified: DateTime<Utc>,
    pub current_workspace: usize,
    pub applications: Vec<AppState>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowState {
    pub schema_version: String,
    pub windows: Vec<WindowLayout>,
}

impl StateManager {
    /// Save current session state
    pub fn save_session(&mut self) -> Result<()> {
        let state_dir = self.config.config_dir.join("state");
        fs::create_dir_all(&state_dir)?;

        self.session_state.last_modified = Utc::now();

        let content = toml::to_string_pretty(&self.session_state)?;
        fs::write(state_dir.join("session.toml"), content)?;

        Ok(())
    }

    /// Save window positions
    pub fn save_windows(&mut self, windows: &[Window]) -> Result<()> {
        let state_dir = self.config.config_dir.join("state");

        self.window_state.windows = windows
            .iter()
            .map(|w| WindowLayout {
                app_id: w.app_id().clone(),
                title: w.title().clone(),
                workspace: w.workspace(),
                state: w.state(),
                position: w.position(),
                size: w.size(),
                monitor: w.monitor(),
            })
            .collect();

        let content = toml::to_string_pretty(&self.window_state)?;
        fs::write(state_dir.join("windows.toml"), content)?;

        Ok(())
    }

    /// Restore session on startup
    pub fn restore_session(&mut self) -> Result<RestorePlan> {
        let state_dir = self.config.config_dir.join("state");

        // Load session state
        let session_path = state_dir.join("session.toml");
        if session_path.exists() {
            let content = fs::read_to_string(&session_path)?;
            self.session_state = toml::from_str(&content)?;
        }

        // Load window state
        let windows_path = state_dir.join("windows.toml");
        if windows_path.exists() {
            let content = fs::read_to_string(&windows_path)?;
            self.window_state = toml::from_str(&content)?;
        }

        Ok(RestorePlan {
            workspace: self.session_state.current_workspace,
            windows: self.window_state.windows.clone(),
        })
    }
}

pub struct RestorePlan {
    pub workspace: usize,
    pub windows: Vec<WindowLayout>,
}
```

## Schema Migration

### Version Migration System

```rust
pub struct SchemaMigrator;

impl SchemaMigrator {
    /// Migrate configuration to latest schema version
    pub fn migrate_config(config: ShellConfig) -> Result<ShellConfig> {
        match config.schema_version.as_str() {
            "1.0.0" => {
                // Already latest
                Ok(config)
            }
            "0.9.0" => {
                // Migrate from 0.9.0 to 1.0.0
                let config = Self::migrate_090_to_100(config)?;
                Self::migrate_config(config)  // Recursively check
            }
            version => {
                // Unknown version - try to load with defaults
                log::warn!("Unknown config schema version: {}", version);
                Ok(ShellConfig::default())
            }
        }
    }

    /// Migrate from schema 0.9.0 to 1.0.0
    fn migrate_090_to_100(mut config: ShellConfig) -> Result<ShellConfig> {
        // Example: Add new field with default
        if config.desktop.session_type.is_empty() {
            config.desktop.session_type = "desktop".into();
        }

        // Update schema version
        config.schema_version = "1.0.0".into();

        Ok(config)
    }
}
```

### Migration Policy

| Schema Version | Policy | Action |
|----------------|--------|--------|
| **Latest** | Keep as-is | No migration needed |
| **Previous minor** | Auto-migrate | Update schema_version |
| **Older minor** | Auto-migrate + backup | Backup old config, migrate |
| **Different major** | Manual review | Show warning, use defaults |

## Per-User vs System-Wide Configuration

### Configuration Merge Strategy

```rust
pub fn load_merged_config() -> ShellConfig {
    // 1. Load system defaults (read-only, admin-managed)
    let system_config = load_system_config()
        .unwrap_or_else(|_| ShellConfig::system_defaults());

    // 2. Load user config (read-write, user-managed)
    let user_config = load_user_config()
        .unwrap_or_else(|_| ShellConfig::default());

    // 3. Merge with user taking precedence
    system_config.merge(user_config)
}

impl ShellConfig {
    pub fn merge(self, user: ShellConfig) -> ShellConfig {
        Self {
            schema_version: user.schema_version,
            desktop: user.desktop,  // User always wins
            display: self.display.merge(user.display),
            theme: user.theme,  // User always wins
            windows: self.windows.merge(user.windows),
            workspaces: user.workspaces,  // User always wins
            input: self.input.merge(user.input),
            accessibility: user.accessibility,  // User always wins
            notifications: self.notifications.merge(user.notifications),
            privacy: user.privacy,  // User always wins
        }
    }
}
```

### Policy Enforcement

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PolicyConfig {
    /// Restrictions on what users can configure
    pub restrictions: Vec<Restriction>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Restriction {
    /// Path to restrict (e.g., "desktop.session_type")
    pub path: String,

    /// Allowed values (empty = no restriction)
    pub allowed: Vec<String>,

    /// Locked (user cannot change)
    pub locked: bool,
}

impl ConfigManager {
    /// Apply policy restrictions to config
    pub fn apply_policies(&mut self, policies: &PolicyConfig) -> Result<()> {
        for restriction in &policies.restrictions {
            if restriction.locked {
                // Reset to system default
                self.reset_to_system_default(&restriction.path)?;
            }

            if !restriction.allowed.is_empty() {
                // Validate against allowed values
                self.validate_allowed(&restriction.path, &restriction.allowed)?;
            }
        }

        Ok(())
    }
}
```

## State Synchronization

### Hot Reload

```rust
pub struct ConfigWatcher {
    manager: ConfigManager,
    watcher: RecommendedWatcher,
    listeners: Vec<Box<dyn ConfigChangeListener>>,
}

pub trait ConfigChangeListener: Send {
    fn on_config_changed(&self, key: &str, value: &ConfigValue);
}

impl ConfigWatcher {
    /// Watch configuration files for changes
    pub fn watch(mut self) -> Result<()> {
        let watch_path = self.manager.config_dir.clone();

        // Watch for file changes
        let mut watcher = notify::recommended_watcher(move |res| {
            match res {
                Ok(Event {
                    kind: EventKind::Modify(_),
                    ..
                }) => {
                    // Reload configuration
                    if let Ok(new_config) = ConfigManager::load() {
                        self.notify_listeners(new_config);
                    }
                }
                _ => {}
            }
        })?;

        watcher.watch(&watch_path, RecursiveMode::Recursive)?;
        self.watcher = watcher;

        Ok(())
    }

    fn notify_listeners(&self, new_config: ConfigManager) {
        for listener in &self.listeners {
            listener.on_config_changed(&new_config);
        }
    }

    pub fn register_listener(&mut self, listener: Box<dyn ConfigChangeListener>) {
        self.listeners.push(listener);
    }
}
```

### State Sync Between Components

```rust
// State change notification
pub enum StateChange {
    ThemeChanged { theme: Theme },
    ScaleChanged { scale: f32 },
    WorkspaceChanged { workspace: usize },
    WindowMoved { window: WindowId, position: (i32, i32) },
}

// Broadcast state changes to all components
pub struct StateBroker {
    subscribers: Vec<Box<dyn StateSubscriber>>,
}

pub trait StateSubscriber: Send {
    fn on_state_change(&self, change: &StateChange);
}

impl StateBroker {
    pub fn publish(&self, change: StateChange) {
        for subscriber in &self.subscribers {
            subscriber.on_state_change(&change);
        }
    }

    pub fn subscribe(&mut self, subscriber: Box<dyn StateSubscriber>) {
        self.subscribers.push(subscriber);
    }
}
```

## Error Handling

### Configuration Errors

```rust
pub enum ConfigError {
    /// Configuration file not found
    NotFound(PathBuf),

    /// Invalid TOML syntax
    InvalidToml(String),

    /// Schema version mismatch
    SchemaVersion {
        found: String,
        expected: String,
    },

    /// Invalid value
    InvalidValue {
        key: String,
        value: String,
        reason: String,
    },

    /// Permission denied
    PermissionDenied(PathBuf),

    /// Migration failed
    MigrationFailed {
        from: String,
        to: String,
        error: String,
    },
}

impl ConfigManager {
    /// Handle configuration errors gracefully
    pub fn load_with_fallback() -> Self {
        match Self::load() {
            Ok(manager) => manager,
            Err(e) => {
                log::error!("Failed to load config: {}", e);

                // Show notification to user
                show_notification(Notification {
                    title: "Configuration Error".into(),
                    body: format!("Using default configuration: {}", e),
                    urgency: Urgency::Warning,
                });

                // Return defaults
                Self::with_defaults()
            }
        }
    }
}
```

## Performance Requirements

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Config load | <50ms | Cold start |
| Config save | <20ms | Write to disk |
| Session restore | <100ms | Startup to visible |
| Window state save | <10ms | After window move |
| Hot reload | <50ms | File change to applied |

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── libs/librustica/src/
│   ├── config/
│   │   ├── mod.rs              # Config module
│   │   ├── manager.rs          # Config manager
│   │   ├── schema.rs           # Config schemas
│   │   ├── migration.rs        # Schema migration
│   │   ├── policy.rs           # Policy enforcement
│   │   └── state.rs            # State persistence
│   └── ...
│
├── rustica-comp/src/
│   └── state/
│       ├── mod.rs
│       ├── session.rs          # Session management
│       └── watcher.rs          # File watcher
│
└── system/rustica-config/
    └── src/
        └── cli.rs              # Config CLI tool
```

## Example Usage

### Application Configuration

```rust
use librustica::config::*;

// Load configuration
let config = ConfigManager::load_with_fallback();

// Get theme
let theme = config.get_theme();
compositor.set_theme(theme);

// Get scale factor
let scale = config.get_display_config().scale;
compositor.set_scale(scale);

// Watch for changes
let watcher = ConfigWatcher::new(config)?;
watcher.register_listener(Box::new(compositor));
watcher.watch()?;

// Save state
compositor.on_window_close(|window| {
    state_manager.save_window_state(window);
});
```

### CLI Config Tool

```bash
# Get configuration value
$ rustica-config get theme.mode
dark

# Set configuration value
$ rustica-config set theme.mode light

# Reset to default
$ rustica-config reset theme.mode

# Show current config
$ rustica-config show

# Edit config file
$ rustica-config edit

# Migrate config
$ rustica-config migrate
```

## Success Criteria

- [ ] All state persists across sessions
- [ ] Schema migration works for all versions
- [ ] Configuration loads in <50ms
- [ ] Hot reload applies changes in <50ms
- [ ] User and system config merge correctly
- [ ] Policies enforce restrictions
- [ ] State restore completes in <100ms
- [ ] Configuration errors handled gracefully

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Corrupt config files | Backup before write, validate schema |
| Migration failures | Keep old versions, provide rollback |
| Permission issues | Check permissions early, show clear errors |
| Concurrent writes | Use file locking, atomic writes |
| Performance impact | Lazy loading, caching, debounced writes |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html)
- [TOML Specification](https://toml.io/en/)
- [Systemd Configuration Files](https://www.freedesktop.org/software/systemd/man/systemd.syntax.html)
- [GNOME Settings Architecture](https://developer.gnome.org/gio/stable/GSettings.html)
