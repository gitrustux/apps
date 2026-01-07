# Initial Setup (rustica-initial-setup) Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Initial Setup Wizard
**Phase**: 7.4 - Integration & Polish

## Overview

The Initial Setup Wizard is the **first-run experience** for RUSTUX OS, guiding users through **account creation**, **network configuration**, **privacy settings**, **application preferences**, and **system customization**. It runs automatically after installation or on first boot if no user accounts exist.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Welcome to RUSTUX                                                        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│                    ┌─────────────────────┐                              │
│                    │                     │                              │
│                    │   [RUSTUX Logo]     │                              │
│                    │                     │                              │
│                    └─────────────────────┘                              │
│                                                                           │
│  Let's get started setting up your RUSTUX system.                        │
│                                                                           │
│  This will only take a few minutes.                                    │
│                                                                           │
│                              [Get Started]                               │
│                                                                           │
│  ───────────────────────────────────────────────────────────────────── │
│                                                                           │
│  1. Welcome            ●━━━━━━━━━━━━  ○  ○  ○  ○                      │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│  Create Your Account                                                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                                                                  │   │
│  │   ┌─────────┐                                                    │   │
│  │   │         │  Choose a profile picture (optional)             │   │
│  │   │ [Avatar]│  [Browse...]                                       │   │
│  │   │         │                                                    │   │
│  │   └─────────┘                                                    │   │
│  │                                                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                           │
│  Full Name:                                                              │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │ John Doe                                                          │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│  Username:                                                               │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │ john                                                              │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│  Password:                                                               │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │ •••••••••                    [Show]                               │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│  Confirm Password:                                                       │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │ •••••••••                                                           │ │
│  └───────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│                              [Next]  [Back]                              │
│                                                                           │
│  ───────────────────────────────────────────────────────────────────── │
│                                                                           │
│  2. Create Account      ○━━━━━━━━━━━━  ●━━━━━━━━━━  ○  ○  ○           │
└─────────────────────────────────────────────────────────────────────────┘
```

## Main Application Structure

```rust
pub struct InitialSetup {
    /// Window
    window: Window,

    /// Current page
    current_page: SetupPage,

    /// Navigation history
    history: Vec<SetupPage>,

    /// Setup data collected so far
    data: SetupData,

    /// Pages
    pages: Vec<Box<dyn SetupPage>>,

    /// Can skip current page
    can_skip: bool,

    /// Can go back
    can_go_back: bool,

    /// Is on last page
    is_last_page: bool,
}

pub struct SetupData {
    /// Account creation data
    pub account: AccountData,

    /// Network configuration
    pub network: NetworkData,

    /// Privacy settings
    pub privacy: PrivacyData,

    /// Application preferences
    pub applications: ApplicationData,

    /// System customization
    pub system: SystemData,

    /// Completed flag
    pub completed: bool,
}

pub struct AccountData {
    /// Profile picture
    pub avatar: Option<Image>,

    /// Full name
    pub full_name: String,

    /// Username
    pub username: String,

    /// Password
    pub password: String,

    /// Password hint (optional)
    pub password_hint: Option<String>,
}

pub struct NetworkData {
    /// Connect to network
    pub connect: bool,

    /// Selected network
    pub selected_network: Option<Network>,

    /// Network password
    pub network_password: Option<String>,
}

pub struct PrivacyData {
    /// Location services
    pub location_services: bool,

    /// Automatic crash reports
    pub crash_reports: bool,

    /// Telemetry
    pub telemetry: bool,

    /// Software usage statistics
    pub usage_stats: bool,
}

pub struct ApplicationData {
    /// Install essential apps
    pub install_essentials: bool,

    /// Selected apps to install
    pub selected_apps: Vec<String>,

    /// Flatpak support
    pub flatpak_support: bool,

    /// Snap support
    pub snap_support: bool,
}

pub struct SystemData {
    /// Theme choice
    pub theme: ThemeChoice,

    /// Accent color
    pub accent_color: Color,

    /// Wallpaper choice
    pub wallpaper: WallpaperChoice,

    /// Desktop effects
    pub enable_effects: bool,

    /// Auto-login
    pub auto_login: bool,
}

pub enum ThemeChoice {
    Auto,
    Light,
    Dark,
}

pub enum WallpaperChoice {
    Default,
    Custom(PathBuf),
}

pub enum SetupPage {
    Welcome,
    CreateAccount,
    Network,
    Privacy,
    Applications,
    Customization,
    Complete,
}
```

## Page System

```rust
pub trait SetupPageImpl {
    /// Render the page
    fn render(&self, ctx: &mut RenderContext, data: &SetupData);

    /// Validate page data
    fn validate(&self, data: &SetupData) -> Result<(), String>;

    /// Apply page data (when moving to next page)
    fn apply(&mut self, data: &mut SetupData) -> Result<(), Error>;

    /// Check if page can be skipped
    fn can_skip(&self, data: &SetupData) -> bool {
        false
    }

    /// Page title
    fn title(&self) -> String;

    /// Page subtitle
    fn subtitle(&self) -> String;
}
```

## Welcome Page

```rust
pub struct WelcomePage;

impl SetupPageImpl for WelcomePage {
    fn render(&self, ctx: &mut RenderContext, data: &SetupData) {
        let center_x = ctx.width() / 2.0;
        let center_y = ctx.height() / 2.0;

        // Logo
        let logo_rect = Rect {
            x: center_x - 64.0,
            y: center_y - 200.0,
            width: 128.0,
            height: 128.0,
        };

        ctx.draw_image(logo_rect, &self.logo);

        // Welcome message
        let title_rect = Rect {
            x: 0.0,
            y: center_y - 40.0,
            width: ctx.width(),
            height: 48.0,
        };

        ctx.draw_text_centered(
            title_rect,
            "Welcome to RUSTUX",
            theme.typography.h2,
            theme.colors.on_background,
        );

        // Subtitle
        let subtitle_rect = Rect {
            x: 0.0,
            y: center_y + 24.0,
            width: ctx.width(),
            height: 24.0,
        };

        ctx.draw_text_centered(
            subtitle_rect,
            "Let's get started setting up your RUSTUX system.",
            theme.typography.body,
            theme.colors.on_surface_variant,
        );

        // Duration hint
        let hint_rect = Rect {
            x: 0.0,
            y: center_y + 56.0,
            width: ctx.width(),
            height: 20.0,
        };

        ctx.draw_text_centered(
            hint_rect,
            "This will only take a few minutes.",
            theme.typography.caption,
            theme.colors.on_surface_variant,
        );

        // Get Started button
        let button_rect = Rect {
            x: center_x - 100.0,
            y: center_y + 100.0,
            width: 200.0,
            height: 48.0,
        };

        ctx.draw_button(button_rect, "Get Started");
    }

    fn validate(&self, data: &SetupData) -> Result<(), String> {
        Ok(())
    }

    fn apply(&mut self, data: &mut SetupData) -> Result<(), Error> {
        Ok(())
    }

    fn title(&self) -> String {
        "Welcome".into()
    }

    fn subtitle(&self) -> String {
        "Setting up your RUSTUX system".into()
    }
}
```

## Create Account Page

```rust
pub struct CreateAccountPage {
    /// Avatar preview
    avatar_preview: Image,

    /// Full name field
    full_name: String,

    /// Username field
    username: String,

    /// Password fields
    password: String,
    confirm_password: String,

    /// Show password
    show_password: bool,

    /// Validation errors
    errors: AccountErrors,
}

pub struct AccountErrors {
    pub username_error: Option<String>,
    pub password_error: Option<String>,
    pub confirm_error: Option<String>,
}

impl SetupPageImpl for CreateAccountPage {
    fn render(&self, ctx: &mut RenderContext, data: &SetupData) {
        let mut y = 200.0;

        // Section header
        ctx.draw_text(
            Rect { x: 100.0, y, width: 400.0, height: 32.0 },
            "Create Your Account",
            theme.typography.h2,
            theme.colors.on_background,
        );

        y += 60.0;

        // Avatar selection
        self.render_avatar_selection(ctx, y);
        y += 160.0;

        // Full name field
        self.render_text_field(
            ctx,
            y,
            "Full Name",
            &data.account.full_name,
            self.errors.username_error.as_deref(),
        );
        y += 80.0;

        // Username field
        self.render_text_field(
            ctx,
            y,
            "Username",
            &data.account.username,
            self.errors.username_error.as_deref(),
        );
        y += 80.0;

        // Password field
        self.render_password_field(
            ctx,
            y,
            "Password",
            &data.account.password,
            self.show_password,
            self.errors.password_error.as_deref(),
        );
        y += 80.0;

        // Confirm password field
        self.render_password_field(
            ctx,
            y,
            "Confirm Password",
            &self.confirm_password,
            self.show_password,
            self.errors.confirm_error.as_deref(),
        );
    }

    fn validate(&self, data: &SetupData) -> Result<(), String> {
        let mut errors = AccountErrors {
            username_error: None,
            password_error: None,
            confirm_error: None,
        };

        // Validate username
        if data.account.username.is_empty() {
            errors.username_error = Some("Username is required".into());
        } else if !is_valid_username(&data.account.username) {
            errors.username_error = Some("Username must be lowercase letters, numbers, and underscores".into());
        } else if username_exists(&data.account.username) {
            errors.username_error = Some("This username already exists".into());
        }

        // Validate password
        if data.account.password.is_empty() {
            errors.password_error = Some("Password is required".into());
        } else if data.account.password.len() < 8 {
            errors.password_error = Some("Password must be at least 8 characters".into());
        }

        // Validate confirm password
        if self.confirm_password != data.account.password {
            errors.confirm_error = Some("Passwords don't match".into());
        }

        if errors.username_error.is_some()
            || errors.password_error.is_some()
            || errors.confirm_error.is_some()
        {
            Err("Please fix the errors below".into())
        } else {
            Ok(())
        }
    }

    fn apply(&mut self, data: &mut SetupData) -> Result<(), Error> {
        // Create user account
        create_user_account(
            &data.account.username,
            &data.account.full_name,
            &data.account.password,
        )?;

        // Set avatar if provided
        if let Some(ref avatar) = data.account.avatar {
            set_user_avatar(&data.account.username, avatar)?;
        }

        Ok(())
    }

    fn title(&self) -> String {
        "Create Your Account".into()
    }

    fn subtitle(&self) -> String {
        "Enter your account information".into()
    }
}

impl CreateAccountPage {
    fn render_avatar_selection(&self, ctx: &mut RenderContext, y: f32) {
        let x = 200.0;

        // Avatar preview
        let avatar_rect = Rect {
            x,
            y,
            width: 128.0,
            height: 128.0,
        };

        ctx.draw_circle_avatar(avatar_rect, &self.avatar_preview);

        // Browse button
        let browse_rect = Rect {
            x: x + 140.0,
            y: y + 40.0,
            width: 120.0,
            height: 36.0,
        };

        ctx.draw_button(browse_rect, "Browse...");

        // Label
        ctx.draw_text(
            Rect { x: x + 140.0, y: y + 88.0, width: 200.0, height: 20.0 },
            "Choose a profile picture (optional)",
            theme.typography.caption,
            theme.colors.on_surface_variant,
        );
    }

    fn render_text_field(&self, ctx: &mut RenderContext, y: f32, label: &str, value: &str, error: Option<&str>) {
        let x = 200.0;

        // Label
        ctx.draw_text(
            Rect { x, y, width: 200.0, height: 20.0 },
            label,
            theme.typography.body,
            theme.colors.on_surface,
        );

        // Field background
        let field_rect = Rect {
            x: y + 24.0,
            y: y + 28.0,
            width: 500.0,
            height: 40.0,
        };

        ctx.fill_rounded_rect(field_rect, 4.0, theme.colors.surface_variant);

        // Field value
        ctx.draw_text(
            Rect { x: field_rect.x + 12.0, y: field_rect.y + 10.0, width: 476.0, height: 20.0 },
            value,
            theme.typography.body,
            theme.colors.on_surface,
        );

        // Error message
        if let Some(error) = error {
            ctx.draw_text(
                Rect { x: field_rect.x, y: field_rect.y + 48.0, width: 500.0, height: 16.0 },
                error,
                theme.typography.caption,
                theme.colors.error,
            );
        }
    }

    fn render_password_field(&self, ctx: &mut RenderContext, y: f32, label: &str, value: &str, show: bool, error: Option<&str>) {
        let x = 200.0;

        // Label
        ctx.draw_text(
            Rect { x, y, width: 200.0, height: 20.0 },
            label,
            theme.typography.body,
            theme.colors.on_surface,
        );

        // Field background
        let field_rect = Rect {
            x: x + 200.0,
            y: y + 28.0,
            width: 500.0,
            height: 40.0,
        };

        ctx.fill_rounded_rect(field_rect, 4.0, theme.colors.surface_variant);

        // Password dots
        let dots = if show { value } else { &"*".repeat(value.len()) };
        ctx.draw_text(
            Rect { x: field_rect.x + 12.0, y: field_rect.y + 10.0, width: 476.0, height: 20.0 },
            dots,
            theme.typography.body,
            theme.colors.on_surface,
        );

        // Show/Hide toggle
        let toggle_rect = Rect {
            x: field_rect.x + field_rect.width - 48.0,
            y: field_rect.y + 8.0,
            width: 24.0,
            height: 24.0,
        };

        let toggle_icon = if show { "eye-open" } else { "eye-closed" };
        ctx.draw_icon(toggle_rect, toggle_icon);

        // Error message
        if let Some(error) = error {
            ctx.draw_text(
                Rect { x: field_rect.x, y: field_rect.y + 48.0, width: 500.0, height: 16.0 },
                error,
                theme.typography.caption,
                theme.colors.error,
            );
        }
    }
}

fn is_valid_username(username: &str) -> bool {
    username.chars().all(|c| c.is_alphanumeric() || c == '_')
        && username.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false)
}

fn username_exists(username: &str) -> bool {
    // Check if user exists in system
    std::process::Command::new("id")
        .arg(username)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn create_user_account(username: &str, full_name: &str, password: &str) -> Result<(), Error> {
    // Create user using useradd
    let output = std::process::Command::new("useradd")
        .arg("-m")
        .arg("-c")
        .arg(full_name)
        .arg(username)
        .output()?;

    if !output.status.success() {
        return Err(Error::UserCreationFailed);
    }

    // Set password
    echo_password(username, password)?;

    // Add user to required groups
    for group in &["wheel", "audio", "video", "network", "plugdev"] {
        std::process::Command::new("usermod")
            .arg("-aG")
            .arg(group)
            .arg(username)
            .output()?;
    }

    Ok(())
}

fn echo_password(username: &str, password: &str) -> Result<(), Error> {
    use std::io::Write;

    let password_process = std::process::Command::new("passwd")
        .arg(username)
        .stdin(Stdio::piped())
        .spawn()?;

    let mut stdin = password_process.stdin.ok_or(Error::NoStdin)?;
    writeln!(stdin, "{}", password)?;
    writeln!(stdin, "{}", password)?;
    drop(stdin);

    let output = password_process.wait_with_output()?;

    if !output.status.success() {
        return Err(Error::PasswordSetFailed);
    }

    Ok(())
}

fn set_user_avatar(username: &str, avatar: &Image) -> Result<(), Error> {
    let home = dirs::home_dir()
        .ok_or(Error::NoHomeDir)?
        .join(username);

    // Create .faces directory
    let faces_dir = home.join(".faces");
    std::fs::create_dir_all(&faces_dir)?;

    // Save avatar
    let avatar_path = faces_dir.join(username);
    avatar.save(&avatar_path)?;

    // Create symlink to default avatar
    let default_avatar = home.join(".face");
    if default_avatar.exists() {
        std::fs::remove_file(&default_avatar)?;
    }

    std::os::unix::fs::symlink(&avatar_path, &default_avatar)?;

    Ok(())
}
```

## Network Page

```rust
pub struct NetworkPage {
    /// Available networks
    networks: Vec<Network>,

    /// Selected network
    selected_network: Option<Network>,

    /// Password
    password: String,

    /// Connecting
    connecting: bool,

    /// Connection result
    connection_result: Option<Result<(), String>>,
}

pub struct Network {
    pub ssid: String,
    pub strength: u8,
    pub secured: bool,
}

impl SetupPageImpl for NetworkPage {
    fn render(&self, ctx: &mut RenderContext, data: &SetupData) {
        let mut y = 200.0;

        // Section header
        ctx.draw_text(
            Rect { x: 100.0, y, width: 600.0, height: 32.0 },
            "Connect to a Network",
            theme.typography.h2,
            theme.colors.on_background,
        );

        y += 60.0;

        // Skip option
        let skip_rect = Rect {
            x: 100.0,
            y,
            width: 400.0,
            height: 24.0,
        };

        let skip_text = "Skip for now (you can set up network later)";
        ctx.draw_text(
            skip_rect,
            skip_text,
            theme.typography.body,
            theme.colors.primary,
        );

        // Draw connect option
        if data.network.connect {
            y += 60.0;

            // Network list
            for network in &self.networks {
                y = self.draw_network(ctx, y, network);
                y += 56.0;
            }
        }

        // Password field (if secured network selected)
        if let Some(ref network) = self.selected_network {
            if network.secured && data.network.connect {
                self.render_password_field(ctx, y, "Network Password", &self.password, false, None);
            }
        }
    }

    fn draw_network(&self, ctx: &mut RenderContext, y: f32, network: &Network) -> f32 {
        let x = 100.0;
        let card_height = 48.0;

        // Background
        let bg_rect = Rect {
            x,
            y,
            width: 700.0,
            height: card_height,
        };

        if self.selected_network.as_ref() == Some(network) {
            ctx.fill_rounded_rect(bg_rect, 8.0, theme.colors.primary.with_alpha(0.15));
        }

        // WiFi icon
        let icon_rect = Rect { x: x + 16.0, y: y + 12.0, width: 24.0, height: 24.0 };
        ctx.draw_icon(icon_rect, self.wifi_icon_for_strength(network.strength));

        // SSID
        ctx.draw_text(
            Rect { x: x + 56.0, y: y + 14.0, width: 400.0, height: 20.0 },
            &network.ssid,
            theme.typography.body,
            theme.colors.on_surface,
        );

        // Lock icon if secured
        if network.secured {
            let lock_rect = Rect { x: x + 660.0, y: y + 12.0, width: 24.0, height: 24.0 };
            ctx.draw_icon(lock_rect, "lock");
        }

        y + card_height + 8.0
    }

    fn validate(&self, data: &SetupData) -> Result<(), String> {
        if data.network.connect && data.network.selected_network.is_none() {
            Err("Please select a network to connect to".into())
        } else {
            Ok(())
        }
    }

    fn apply(&mut self, data: &mut SetupData) -> Result<(), Error> {
        if !data.network.connect {
            return Ok(());
        }

        // Connect to network
        if let Some(ref network) = data.network.selected_network {
            self.connecting = true;

            let result = if network.secured {
                connect_to_wifi(&network.ssid, &data.network.network_password)
            } else {
                connect_to_wifi(&network.ssid, "")
            };

            self.connection_result = Some(result.clone());

            result.map_err(|e| e.to_string())
        } else {
            Ok(())
        }
    }

    fn title(&self) -> String {
        "Network".into()
    }

    fn subtitle(&self) -> String {
        "Connect to a network (optional)".into()
    }

    fn can_skip(&self, data: &SetupData) -> bool {
        true
    }
}
```

## Complete Page

```rust
pub struct CompletePage;

impl SetupPageImpl for CompletePage {
    fn render(&self, ctx: &mut RenderContext, data: &SetupData) {
        let center_x = ctx.width() / 2.0;
        let center_y = ctx.height() / 2.0;

        // Success icon
        let icon_rect = Rect {
            x: center_x - 32.0,
            y: center_y - 100.0,
            width: 64.0,
            height: 64.0,
        };

        ctx.draw_icon(icon_rect, "check-circle");

        // Success message
        ctx.draw_text_centered(
            Rect { x: 0.0, y: center_y - 20.0, width: ctx.width(), height: 32.0 },
            "You're all set!",
            theme.typography.h2,
            theme.colors.on_background,
        );

        // Subtitle
        ctx.draw_text_centered(
            Rect { x: 0.0, y: center_y + 24.0, width: ctx.width(), height: 24.0 },
            "Your RUSTUX system is ready to use.",
            theme.typography.body,
            theme.colors.on_surface_variant,
        );

        // Restart button
        let button_rect = Rect {
            x: center_x - 100.0,
            y: center_y + 80.0,
            width: 200.0,
            height: 48.0,
        };

        ctx.draw_button(button_rect, "Restart Now");
    }

    fn validate(&self, data: &SetupData) -> Result<(), String> {
        Ok(())
    }

    fn apply(&mut self, data: &mut SetupData) -> Result<(), Error> {
        // Mark setup as complete
        data.completed = true;

        // Save setup data
        self.save_setup_data(data)?;

        // Configure system based on choices
        self.apply_setup_settings(data)?;

        Ok(())
    }

    fn title(&self) -> String {
        "Complete!".into()
    }

    fn subtitle(&self) -> String {
        "Your system is ready".into()
    }
}

impl CompletePage {
    fn save_setup_data(&self, data: &SetupData) -> Result<(), Error> {
        let config_path = PathBuf::from("/etc/rustica/setup-complete");

        let config = format!(
            "# RUSTUX Initial Setup Complete\n\
            completed=true\n\
            completed_at={}\n\
            user={}\n",
            Utc::now().to_rfc3339(),
            data.account.username
        );

        std::fs::write(&config_path, config)?;

        Ok(())
    }

    fn apply_setup_settings(&self, data: &SetupData) -> Result<(), Error> {
        // Apply privacy settings
        apply_privacy_settings(&data.privacy)?;

        // Install selected applications
        if data.applications.install_essentials {
            install_essentials()?;
        }

        for app in &data.applications.selected_apps {
            install_application(app)?;
        }

        // Set theme
        apply_theme(&data.system.theme, data.system.accent_color)?;

        // Set wallpaper
        apply_wallpaper(&data.system.wallpaper)?;

        Ok(())
    }
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-initial-setup/
│   ├── Cargo.toml
│   ├── resources/
│   │   └── icons/
│   └── src/
│       ├── main.rs
│       ├── app.rs
│       ├── pages/
│       │   ├── mod.rs
│       │   ├── welcome.rs
│       │   ├── account.rs
│       │   ├── network.rs
│       │   ├── privacy.rs
│       │   ├── applications.rs
│       │   ├── customization.rs
│       │   └── complete.rs
│       ├── data.rs
│       └── system.rs
└── systemd/
    └── rustica-initial-setup.service
```

## Dependencies

```toml
[package]
name = "rustica-initial-setup"
version = "1.0.0"
edition = "2021"

[dependencies]
# GUI framework
librustica = { path = "../../../libs/librustica" }

# Image handling
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
| Startup time | <500ms | To visible |
| Page transitions | <100ms | Between pages |
| Account creation | <1s | Click to created |
| Memory | <100MB | Setup usage |

## Success Criteria

- [ ] All pages display correctly
- [ ] Account creation works
- [ ] Network connection works
- [ ] Privacy settings apply
- [ ] App installation works
- [ ] Theme applies correctly
- [ ] Can skip optional pages
- [ ] Full accessibility
- [ ] Data persists correctly
- [ ] System restarts successfully

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## Timeline

- Week 1: Page framework + welcome page
- Week 2: Account creation page
- Week 3: Network connection page
- Week 4: Privacy + applications pages
- Week 5: Customization page
- Week 6: Complete page + system integration

**Total**: 6 weeks
