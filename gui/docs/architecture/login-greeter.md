# Login Greeter (rustica-greeter) Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Login Greeter
**Phase**: 7.3 - Integration & Polish

## Overview

The Login Greeter is the **first visual interface** users see when they boot RUSTUX OS. It provides a **beautiful, secure login experience** with **user selection**, **authentication**, **session selection**, **accessibility**, and **auto-login** support. It runs **before** the user session starts.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Rustica Login Greeter                                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│                    ┌─────────────────────┐                              │
│                    │                     │                              │
│                    │   [RUSTUX Logo]     │                              │
│                    │                     │                              │
│                    └─────────────────────┘                              │
│                                                                           │
│                          10:30 AM                                      │
│                      Wednesday, January 7                                │
│                                                                           │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                                                                   │  │
│  │   ┌─────────┐                                                     │  │
│  │   │         │  Password: •••••••••                      [▶]  │  │
│  │   │ [Avatar]│  [←]            [→]                               │  │
│  │   │         │                                                     │  │
│  │   └─────────┘                                                     │  │
│  │                                                                   │  │
│  │      John Doe                                         [⚙️]       │  │
│  │                                                                   │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│                                                                           │
│  [Session: Rustica] [Language: English] [Accessibility]               │
│                                                                           │
│  [Restart]  [Shutdown]                                                  │
│                                                                           │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│  Session Selection Menu                                                  │
├─────────────────────────────────────────────────────────────────────────┤
│  Select Session                                                           │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  ● Rustica (Wayland)                                              │   │
│  │    Default modern desktop experience                               │   │
│  │                                                                  │   │
│  │  ○ GNOME (Wayland)                                                │   │
│  │    GNOME desktop environment                                      │   │
│  │                                                                  │   │
│  │  ○ KDE Plasma (Wayland)                                          │   │
│  │    KDE Plasma desktop environment                                │   │
│  │                                                                  │   │
│  │  ○ XFCE (X11)                                                    │   │
│  │    Lightweight desktop environment                                │   │
│  │                                                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

## Main Application Structure

```rust
pub struct Greeter {
    /// Window (layer shell, fullscreen)
    window: Window,

    /// Current users
    users: Vec<User>,

    /// Selected user index
    selected_user: usize,

    /// Sessions
    sessions: Vec<Session>,

    /// Selected session
    selected_session: String,

    /// Authentication state
    auth_state: AuthState,

    /// Password buffer
    password: String,

    /// Login attempts
    login_attempts: u32,

    /// Locked (too many failed attempts)
    locked: bool,

    /// Lock expiration
    lock_expires: Option<DateTime<Utc>>,

    /// Theme
    theme: Theme,

    /// Background
    background: Background,

    /// Clock timer
    clock_timer: Timer,

    /// Display manager client
    display_manager: DisplayManagerProxy,
}

pub struct User {
    /// Username
    pub name: String,

    /// Real name
    pub real_name: String,

    /// Avatar
    pub avatar: Image,

    /// Home directory
    pub home: PathBuf,

    /// Shell
    pub shell: PathBuf,

    /// Last login time
    pub last_login: Option<DateTime<Utc>>,

    /// Logged in (already has session)
    pub logged_in: bool,

    /// Auto-login user
    pub auto_login: bool,
}

pub struct Session {
    /// Session name
    pub name: String,

    /// Session file (desktop file)
    pub file: PathBuf,

    /// Display type
    pub display_type: DisplayType,

    /// Priority (for sorting)
    pub priority: i32,
}

pub enum DisplayType {
    Wayland,
    X11,
}

pub enum AuthState {
    /// Waiting for user
    Waiting,

    /// Entering password
    EnteringPassword,

    /// Authenticating
    Authenticating,

    /// Authentication succeeded
    Success,

    /// Authentication failed
    Failed(String),

    /// Authentication in progress
   InProgress,
}

pub struct Background {
    /// Type
    pub kind: BackgroundType,

    /// Blur amount
    pub blur: f32,

    /// Dim amount
    pub dim: f32,
}

pub enum BackgroundType {
    /// Solid color
    Color(Color),

    /// Gradient
    Gradient { colors: Vec<Color>, angle: f32 },

    /// Image
    Image { path: PathBuf, fit: BackgroundFit },

    /// Slideshow
    Slideshow { images: Vec<PathBuf>, interval: Duration },
}

pub enum BackgroundFit {
    Cover,
    Contain,
    Fill,
    Scale,
}
```

## Authentication

```rust
impl Greeter {
    pub fn authenticate(&mut self, password: String) -> Result<(), Error> {
        // Check lock status
        if self.locked {
            if let Some(expires) = self.lock_expires {
                if Utc::now() < expires {
                    return Err(Error::Locked);
                } else {
                    // Lock expired
                    self.locked = false;
                    self.login_attempts = 0;
                }
            }
        }

        // Update state
        self.auth_state = AuthState::Authenticating;

        // Get selected user
        let user = &self.users[self.selected_user];

        // Authenticate via PAM
        match self.pam_authenticate(&user.name, &password) {
            Ok(_) => {
                // Success!
                self.auth_state = AuthState::Success;
                self.login_attempts = 0;

                // Start session
                self.start_session()?;

                Ok(())
            }

            Err(err) => {
                // Failed
                self.login_attempts += 1;
                self.password.clear();

                // Check if we should lock
                if self.login_attempts >= 3 {
                    self.locked = true;
                    self.lock_expires = Some(Utc::now() + Duration::from_secs(300)); // 5 min
                    self.auth_state = AuthState::Failed("Too many failed attempts. Locked for 5 minutes.".into());
                } else {
                    self.auth_state = AuthState::Failed(format!("Authentication failed. {} attempts remaining.", 3 - self.login_attempts));
                }

                Err(err)
            }
        }
    }

    fn pam_authenticate(&self, username: &str, password: &str) -> Result<(), Error> {
        use pam::constants::{PamFlag, PamResultCode};

        // Create PAM transaction
        let mut pam = pam::Authenticator::with_service("rustica-greeter")?;

        // Set username
        pam.handler().set_username(username)?;

        // Set password
        pam.handler().set_password(password)?;

        // Authenticate
        match pam.authenticate(PamFlag::NONE) {
            PamResultCode::PAM_SUCCESS => Ok(()),

            PamResultCode::PAM_AUTH_ERR | PamResultCode::PAM_USER_UNKNOWN => {
                Err(Error::AuthenticationFailed)
            }

            code => Err(Error::PamError(code)),
        }
    }

    pub fn start_session(&self) -> Result<(), Error> {
        let user = &self.users[self.selected_user];
        let session = self.get_session(&self.selected_session)?;

        // Request session from display manager
        self.display_manager.start_session(
            &user.name,
            &session.file,
            &user.home,
            &user.shell,
        )?;

        Ok(())
    }
}
```

## User Selection

```rust
impl Greeter {
    pub fn next_user(&mut self) {
        if self.users.is_empty() {
            return;
        }

        self.selected_user = (self.selected_user + 1) % self.users.len();

        // Clear password when switching users
        self.password.clear();
        self.auth_state = AuthState::Waiting;
    }

    pub fn prev_user(&mut self) {
        if self.users.is_empty() {
            return;
        }

        if self.selected_user == 0 {
            self.selected_user = self.users.len() - 1;
        } else {
            self.selected_user -= 1;
        }

        // Clear password when switching users
        self.password.clear();
        self.auth_state = AuthState::Waiting;
    }

    pub fn select_user(&mut self, username: &str) {
        if let Some(index) = self.users.iter().position(|u| u.name == username) {
            self.selected_user = index;
            self.password.clear();
            self.auth_state = AuthState::Waiting;
        }
    }

    pub fn load_users(&mut self) -> Result<(), Error> {
        let mut users = Vec::new();

        // Read user database
        let passwd_file = std::fs::File::open("/etc/passwd")?;
        let reader = std::io::BufReader::new(passwd_file);

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(':').collect();

            if parts.len() >= 7 {
                let username = parts[0];
                let uid: u32 = parts[2].parse().unwrap_or(0);
                let home = PathBuf::from(parts[5]);
                let shell = PathBuf::from(parts[6]);

                // Skip system users (UID < 1000)
                if uid < 1000 {
                    continue;
                }

                // Check if user has a valid shell
                if !shell.exists() {
                    continue;
                }

                // Load user info
                let real_name = self.get_real_name(&home);
                let avatar = self.load_avatar(&home);
                let last_login = self.get_last_login(&username);

                let user = User {
                    name: username.clone(),
                    real_name,
                    avatar,
                    home,
                    shell,
                    last_login,
                    logged_in: false,
                    auto_login: false,
                };

                users.push(user);
            }
        }

        // Check for existing sessions
        for user in &mut users {
            user.logged_in = self.has_active_session(&user.name);
        }

        // Sort users by real name
        users.sort_by(|a, b| a.real_name.cmp(&b.real_name));

        // Set auto-login user
        if let Some(auto_user) = self.get_autologin_user()? {
            if let Some(index) = users.iter().position(|u| u.name == auto_user) {
                users[index].auto_login = true;
                self.selected_user = index;
            }
        }

        self.users = users;

        Ok(())
    }

    fn get_real_name(&self, home: &Path) -> String {
        // Try to read from ~/.face file
        let face_file = home.join(".face");

        if face_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&face_file) {
                return content.trim().to_string();
            }
        }

        // Fallback to GECOS from /etc/passwd
        // (already parsed above)

        // Final fallback
        "User".into()
    }

    fn load_avatar(&self, home: &Path) -> Image {
        // Try ~/.face.icon
        let face_icon = home.join(".face.icon");

        if face_icon.exists() {
            if Ok(image) = Image::load(&face_icon) {
                return image;
            }
        }

        // Try ~/.face
        let face = home.join(".face");

        if face.exists() {
            if Ok(image) = Image::load(&face) {
                return image;
            }
        }

        // Try system avatar from accountsservice
        let system_avatar = PathBuf::from("/var/lib/AccountsService/icons")
            .join(self.get_username_from_home(home))
            .with_extension("png");

        if system_avatar.exists() {
            if Ok(image) = Image::load(&system_avatar) {
                return image;
            }
        }

        // Fallback to default avatar
        Image::load_from_data(include_bytes!("../resources/default-avatar.png"))
            .unwrap_or_default()
    }

    fn has_active_session(&self, username: &str) -> bool {
        // Check loginctl for active sessions
        let output = std::process::Command::new("loginctl")
            .args(&["list-sessions", "--no-legend"])
            .output();

        if let Ok(output) = {
            let stdout = String::from_utf8_lossy(&output.stdout);

            stdout.lines().any(|line| {
                line.contains(username) && line.contains("active")
            })
        } else {
            false
        }
    }

    fn get_autologin_user(&self) -> Result<Option<String>, Error> {
        // Read from display manager config
        let config_path = "/etc/rustica/display-manager.conf";

        if let Ok(content) = std::fs::read_to_string(config_path) {
            for line in content.lines() {
                if line.starts_with("autologin-user=") {
                    let user = line.trim_start_matches("autologin-user=")
                        .trim_matches('"');

                    return Ok(Some(user.to_string()));
                }
            }
        }

        Ok(None)
    }

    fn get_last_login(&self, username: &str) -> Option<DateTime<Utc>> {
        // Parse lastlog
        let output = std::process::Command::new("lastlog")
            .arg(username)
            .output();

        if let Ok(output) = {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = stdout.lines().collect();

            if lines.len() > 1 {
                // Parse date from second line
                // Format: username   pts/0        ip_address   date   time
                let parts: Vec<&str> = lines[1].split_whitespace().collect();

                if parts.len() >= 5 {
                    let datetime_str = format!("{} {}", parts[3], parts[4]);

                    if let Ok(dt) = DateTime::parse_from_str(&datetime_str, "%a %b %d %H:%M:%S %z %Y") {
                        return Some(dt.with_timezone(&Utc));
                    }
                }
            }

            None
        } else {
            None
        }
    }
}
```

## Session Management

```rust
impl Greeter {
    pub fn load_sessions(&mut self) -> Result<(), Error> {
        let mut sessions = Vec::new();

        // Scan wayland-sessions directory
        let wayland_dir = PathBuf::from("/usr/share/wayland-sessions");

        if wayland_dir.exists() {
            for entry in std::fs::read_dir(wayland_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                    if let Ok(session) = self.parse_session_file(&path) {
                        sessions.push(session);
                    }
                }
            }
        }

        // Scan xsessions directory
        let x11_dir = PathBuf::from("/usr/share/xsessions");

        if x11_dir.exists() {
            for entry in std::fs::read_dir(x11_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                    if let Ok(session) = self.parse_session_file(&path) {
                        sessions.push(session);
                    }
                }
            }
        }

        // Sort by priority (default first)
        sessions.sort_by(|a, b| b.priority.cmp(&a.priority));

        self.sessions = sessions;

        // Set default session
        if !self.sessions.is_empty() {
            self.selected_session = self.sessions[0].name.clone();
        }

        Ok(())
    }

    fn parse_session_file(&self, path: &Path) -> Result<Session, Error> {
        let content = std::fs::read_to_string(path)?;

        let mut name = String::new();
        let mut priority = 0;
        let mut exec = String::new();
        let mut tries_desktop = String::new();

        // Parse desktop file
        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("Name=") {
                name = line.trim_start_matches("Name=").to_string();
            } else if line.starts_with("Exec=") {
                exec = line.trim_start_matches("Exec=").to_string();
            } else if line.starts_with("X-Rustica-Priority=") {
                if let Ok(p) = line.trim_start_matches("X-Rustica-Priority=").parse::<i32>() {
                    priority = p;
                }
            } else if line.starts_with("DesktopNames=") {
                tries_desktop = line.trim_start_matches("DesktopNames=").to_string();
            }
        }

        // Determine display type from path
        let display_type = if path.to_string_lossy().contains("wayland-sessions") {
            DisplayType::Wayland
        } else {
            DisplayType::X11
        };

        // Set default priority
        if priority == 0 {
            // Default priorities
            priority = match name.as_str() {
                "Rustica" => 100,
                "GNOME" => 90,
                "plasma" => 80,
                "XFCE" => 70,
                _ => 50,
            };
        }

        Ok(Session {
            name,
            file: path.to_path_buf(),
            display_type,
            priority,
        })
    }

    pub fn get_session(&self, name: &str) -> Result<Session, Error> {
        self.sessions.iter()
            .find(|s| s.name == name)
            .cloned()
            .ok_or(Error::SessionNotFound)
    }
}
```

## Rendering

```rust
impl Greeter {
    pub fn render(&self, ctx: &mut RenderContext) {
        // Clear screen
        ctx.fill_rect(ctx.screen_rect(), theme.colors.background);

        // Draw background
        self.draw_background(ctx);

        // Draw logo
        self.draw_logo(ctx);

        // Draw clock
        self.draw_clock(ctx);

        // Draw user selection
        self.draw_user_selection(ctx);

        // Draw password field
        self.draw_password_field(ctx);

        // Draw session selector
        self.draw_session_selector(ctx);

        // Draw system options
        self.draw_system_options(ctx);

        // Draw power options
        self.draw_power_options(ctx);
    }

    fn draw_background(&self, ctx: &mut RenderContext) {
        let rect = ctx.screen_rect();

        match &self.background.kind {
            BackgroundType::Color(color) => {
                ctx.fill_rect(rect, *color);
            }

            BackgroundType::Gradient { colors, angle } => {
                ctx.draw_linear_gradient(rect, colors, *angle);
            }

            BackgroundType::Image { path, fit } => {
                if let Ok(image) = Image::load(path) {
                    ctx.draw_image_fit(rect, &image, fit);
                }
            }

            BackgroundType::Slideshow { .. } => {
                // Handle slideshow
            }
        }

        // Apply blur if configured
        if self.background.blur > 0.0 {
            ctx.apply_blur(self.background.blur);
        }

        // Apply dim if configured
        if self.background.dim > 0.0 {
            ctx.fill_rect(rect, Color::rgba(0, 0, 0, (self.background.dim * 255.0) as u8));
        }
    }

    fn draw_logo(&self, ctx: &mut RenderContext) {
        let logo_size = 128.0;
        let logo_rect = Rect {
            x: (ctx.width() - logo_size) / 2.0,
            y: 100.0,
            width: logo_size,
            height: logo_size,
        };

        // Draw logo
        ctx.draw_image(logo_rect, &self.logo);

        // Draw distribution name below
        let name_rect = Rect {
            x: 0.0,
            y: logo_rect.y + logo_size + 16.0,
            width: ctx.width(),
            height: 32.0,
        };

        ctx.draw_text_centered(
            name_rect,
            "RUSTUX",
            theme.typography.h2,
            theme.colors.on_background,
        );
    }

    fn draw_clock(&self, ctx: &mut RenderContext) {
        let now = Local::now();

        // Time
        let time_str = now.format("%_I:%M %p").to_string();
        let time_rect = Rect {
            x: 0.0,
            y: 300.0,
            width: ctx.width(),
            height: 48.0,
        };

        ctx.draw_text_centered(
            time_rect,
            &time_str,
            theme.typography.h3,
            theme.colors.on_background,
        );

        // Date
        let date_str = now.format("%A, %B %-d").to_string();
        let date_rect = Rect {
            x: 0.0,
            y: 356.0,
            width: ctx.width(),
            height: 24.0,
        };

        ctx.draw_text_centered(
            date_rect,
            &date_str,
            theme.typography.body,
            theme.colors.on_surface_variant,
        );
    }

    fn draw_user_selection(&self, ctx: &mut RenderContext) {
        if self.users.is_empty() {
            return;
        }

        let user = &self.users[self.selected_user];

        let user_center_x = ctx.width() / 2.0;
        let user_center_y = 500.0;
        let avatar_size = 128.0;

        // Avatar background
        let avatar_rect = Rect {
            x: user_center_x - avatar_size / 2.0,
            y: user_center_y - avatar_size / 2.0,
            width: avatar_size,
            height: avatar_size,
        };

        ctx.draw_circle_avatar(avatar_rect, &user.avatar);

        // User name (below avatar)
        let name_rect = Rect {
            x: 0.0,
            y: user_center_y + avatar_size / 2.0 + 16.0,
            width: ctx.width(),
            height: 24.0,
        };

        ctx.draw_text_centered(
            name_rect,
            &user.real_name,
            theme.typography.h4,
            theme.colors.on_background,
        );

        // Navigation arrows (if multiple users)
        if self.users.len() > 1 {
            // Left arrow
            let left_rect = Rect {
                x: user_center_x - 200.0,
                y: user_center_y - 20.0,
                width: 40.0,
                height: 40.0,
            };

            ctx.draw_button(left_rect, "◀");

            // Right arrow
            let right_rect = Rect {
                x: user_center_x + 160.0,
                y: user_center_y - 20.0,
                width: 40.0,
                height: 40.0,
            };

            ctx.draw_button(right_rect, "▶");
        }
    }

    fn draw_password_field(&self, ctx: &mut RenderContext) {
        let field_width = 400.0;
        let field_y = 680.0;

        // Password field background
        let field_rect = Rect {
            x: (ctx.width() - field_width) / 2.0,
            y: field_y,
            width: field_width,
            height: 48.0,
        };

        ctx.fill_rounded_rect(field_rect, 24.0, theme.colors.surface_variant);

        // Lock icon
        ctx.draw_icon(
            Rect { x: field_rect.x + 16.0, y: field_rect.y + 12.0, width: 24.0, height: 24.0 },
            "lock",
        );

        // Password dots
        let password_dots = "•".repeat(self.password.chars().count());

        ctx.draw_text(
            Rect {
                x: field_rect.x + 56.0,
                y: field_rect.y + 14.0,
                width: field_width - 120.0,
                height: 20.0,
            },
            &password_dots,
            theme.typography.body,
            theme.colors.on_surface,
        );

        // Submit button
        let submit_rect = Rect {
            x: field_rect.x + field_width - 48.0,
            y: field_rect.y + 12.0,
            width: 24.0,
            height: 24.0,
        };

        ctx.draw_icon(submit_rect, "go-next");

        // Focus ring (if entering password)
        if matches!(self.auth_state, AuthState::EnteringPassword) {
            ctx.stroke_rounded_rect(field_rect, 24.0, 3.0, theme.colors.primary);
        }

        // Error message (if authentication failed)
        if let AuthState::Failed(message) = &self.auth_state {
            let error_rect = Rect {
                x: 0.0,
                y: field_rect.y + 56.0,
                width: ctx.width(),
                height: 24.0,
            };

            ctx.draw_text_centered(
                error_rect,
                message,
                theme.typography.body,
                theme.colors.error,
            );
        }
    }

    fn draw_session_selector(&self, ctx: &mut RenderContext) {
        let button_width = 200.0;
        let button_y = 760.0;

        // Session selector button
        let button_rect = Rect {
            x: (ctx.width() - button_width) / 2.0,
            y: button_y,
            width: button_width,
            height: 36.0,
        };

        let button_text = format!("Session: {}", self.selected_session);
        ctx.draw_button(button_rect, &button_text);
    }

    fn draw_system_options(&self, ctx: &mut RenderContext) {
        let options = &["Session", "Language", "Accessibility"];
        let mut x = ctx.width() - 600.0;
        let y = ctx.height() - 60.0;

        for option in options {
            let rect = Rect { x, y, width: 140.0, height: 36.0 };
            ctx.draw_button(rect, option);
            x += 156.0;
        }
    }

    fn draw_power_options(&self, ctx: &mut RenderContext) {
        let restart_rect = Rect {
            x: 24.0,
            y: ctx.height() - 60.0,
            width: 120.0,
            height: 36.0,
        };

        let shutdown_rect = Rect {
            x: 156.0,
            y: ctx.height() - 60.0,
            width: 120.0,
            height: 36.0,
        };

        ctx.draw_button(restart_rect, "Restart");
        ctx.draw_button(shutdown_rect, "Shutdown");
    }
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-greeter/
│   ├── Cargo.toml
│   ├── resources/
│   │   ├── logo.png
│   │   ├── default-avatar.png
│   │   └── backgrounds/
│   └── src/
│       ├── main.rs
│       ├── greeter.rs
│       ├── auth.rs
│       ├── users.rs
│       ├── sessions.rs
│       ├── background.rs
│       └── clock.rs
└── systemd/
    └── rustica-greeter.service
```

## Dependencies

```toml
[package]
name = "rustica-greeter"
version = "1.0.0"
edition = "2021"

[dependencies]
# GUI framework
librustica = { path = "../../../libs/librustica" }

# Authentication
pam = "0.7"

# Wayland
wayland-client = "0.31"
wayland-protocols = { version = "0.31", features = ["unstable_protocols"] }

# Image loading
image = "0.24"

# Serialization
serde = { version = "1.0", features = ["derive"] }

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Date/time
chrono = "0.4"

# XDG
dirs = "5.0"
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Startup time | <1s | To visible |
| User load | <200ms | All users |
| Authentication | <500ms | Password to success |
| Memory | <50MB | Greeter usage |

## Success Criteria

- [ ] Greeter displays on boot
- [ ] User selection works
- [ ] Authentication works
- [ ] Session selection works
- [ ] Auto-login works
- [ ] Background displays
- [ ] Clock updates
- [ ] Restart/shutdown work
- [ ] Multiple monitors handled
- [ ] Full accessibility

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## Timeline

- Week 1: Basic greeter + authentication
- Week 2: User selection + session management
- Week 3: Background + styling
- Week 4: Accessibility + internationalization
- Week 5: Auto-login + lockout
- Week 6: Multi-monitor + polish

**Total**: 6 weeks
