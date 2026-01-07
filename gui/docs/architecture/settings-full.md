# Settings Application (rustica-settings) Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - System Settings
**Phase**: 5.1 - System Applications (Full Implementation)

## Overview

This is the **full production Settings application** for Rustica Shell, providing comprehensive system configuration with **real-time preview**, **instant apply**, and **full accessibility**. Unlike the prototype (Phase 4.2), this includes **all setting panels**, **proper D-Bus integration**, and **production-quality code**.

## Architecture

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Settings Window                                                          │
├──────────────────────────────────────────────────────────────────────────┤
│  ◀ Settings                                    [Apply] [Revert]  [Help]  │
├──────────────┬───────────────────────────────────────────────────────────┤
│              │                                                           │
│  ┌────────┐  │  ┌──────────────────────────────────────────────────┐   │
│  │  🎨    │  │  │ Appearance                                        │   │
│  │Appear  │  │  │ ────────────────────────────────────────────────│   │
│  ├────────┤  │  │                                                   │   │
│  │  🖥️    │  │  │ Mode                                             │   │
│  │Display │  │  │ ○ Light  ● Dark  ○ Auto                         │   │
│  ├────────┤  │  │                                                   │   │
│  │  📶    │  │  │ Accent Color                                      │   │
│  │Network │  │  │ [Blue] [Teal] ● Purple [Orange] [Pink]          │   │
│  ├────────┤  │  │                                                   │   │
│  │  📶    │  │  │ Font Size                                       │   │
│  │Bluetooth│  │  │ ◀━━━●━━━▶  12pt                               │   │
│  ├────────┤  │  │                                                   │   │
│  │  🔊    │  │  │                                                   │   │
│  │ Audio  │  │  │                                                   │   │
│  ├────────┤  │  │                                                   │   │
│  │  🔋    │  │  │                                                   │   │
│  │ Power  │  │  │                                                   │   │
│  ├────────┤  │  │                                                   │   │
│  │  ⌨️    │  │  │                                                   │   │
│  │ Input  │  │  │                                                   │   │
│  ├────────┤  │  │                                                   │   │
│  │  👥    │  │  │                                                   │   │
│  │ Users  │  │  │                                                   │   │
│  ├────────┤  │  │                                                   │   │
│  │  📅    │  │  │                                                   │   │
│  │  Time  │  │  │                                                   │   │
│  ├────────┤  │  │                                                   │   │
│  │  ♿    │  │  │                                                   │   │
│  │Access │  │  │                                                   │   │
│  ├────────┤  │  │                                                   │   │
│  │  ℹ️    │  │  │                                                   │   │
│  │ About │  │  │                                                   │   │
│  └────────┘  │  └──────────────────────────────────────────────────┘   │
│              │                                                           │
│  Sidebar 230px│  Content Area 670px                                      │
└──────────────┴───────────────────────────────────────────────────────────┘
                        900px width
```

## Setting Panels

```rust
pub enum SettingsPanel {
    Home,
    Appearance,
    Display,
    Network,
    Bluetooth,
    Audio,
    Power,
    InputDevices,
    Users,
    DateTime,
    Accessibility,
    About,
}

impl SettingsPanel {
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Home => "settings-home",
            Self::Appearance => "appearance",
            Self::Display => "display",
            Self::Network => "network",
            Self::Bluetooth => "bluetooth",
            Self::Audio => "audio",
            Self::Power => "battery",
            Self::InputDevices => "input-keyboard",
            Self::Users => "system-users",
            Self::DateTime => "time",
            Self::Accessibility => "accessibility",
            Self::About => "help-about",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Home => "Settings",
            Self::Appearance => "Appearance",
            Self::Display => "Display",
            Self::Network => "Network",
            Self::Bluetooth => "Bluetooth",
            Self::Audio => "Audio",
            Self::Power => "Power",
            Self::InputDevices => "Input Devices",
            Self::Users => "Users",
            Self::DateTime => "Date & Time",
            Self::Accessibility => "Accessibility",
            Self::About => "About",
        }
    }
}
```

## Main Application Structure

```rust
pub struct SettingsApp {
    /// Current panel
    current_panel: SettingsPanel,

    /// Navigation history
    history: Vec<SettingsPanel>,

    /// Pending changes (not yet applied)
    pending_changes: HashMap<SettingKey, SettingValue>,

    /// Original values (for revert)
    original_values: HashMap<SettingKey, SettingValue>,

    /// Settings daemon client
    daemon_client: SettingsDaemonProxy,

    /// Window
    window: Window,

    /// Theme
    theme: Theme,
}

impl SettingsApp {
    pub fn new() -> Self {
        // Connect to settings daemon
        let daemon_client = SettingsDaemonProxy::new()
            .expect("Failed to connect to settings daemon");

        // Load current settings
        let current_settings = daemon_client.get_all_settings();

        // Store original values
        let original_values = current_settings.clone();

        Self {
            current_panel: SettingsPanel::Home,
            history: Vec::new(),
            pending_changes: HashMap::new(),
            original_values,
            daemon_client,
            window: Window::new(),
            theme: Theme::load(),
        }
    }

    pub fn run(&mut self) {
        // Create window
        self.window.create(900, 600);

        // Render initial panel
        self.render();

        // Run event loop
        self.event_loop();
    }

    fn event_loop(&mut self) {
        loop {
            // Wait for events
            let event = self.window.next_event();

            match event {
                Event::Quit => break,

                Event::Click { position } => {
                    self.handle_click(position);
                }

                Event::Key(key) => {
                    self.handle_key(key);
                }

                Event::SettingsChanged { key, value } => {
                    // External change (from daemon)
                    self.handle_external_change(key, value);
                }
            }

            // Re-render
            self.render();
        }
    }
}
```

## Appearance Panel

```rust
pub struct AppearancePanel {
    /// Theme mode
    theme_mode: ThemeMode,

    /// Accent color
    accent_color: AccentColor,

    /// Font family
    font_family: String,

    /// Font size
    font_size: FontSize,

    /// Icon size
    icon_size: IconSize,

    /// Window decorations
    window_decorations: bool,

    /// Dark mode tweaks
    dark_mode_tweaks: DarkModeTweaks,
}

pub enum ThemeMode {
    Light,
    Dark,
    Auto,
}

pub enum AccentColor {
    Blue,
    Teal,
    Purple,
    Orange,
    Pink,
    Red,
    Green,
}

pub enum FontSize {
    Small,   // 11pt
    Default, // 12pt
    Large,   // 14pt
    XLarge,  // 16pt
}

pub enum IconSize {
    Small,   // 16px
    Default, // 24px
    Large,   // 32px
}

pub struct DarkModeTweaks {
    /// Pure black background
    pure_black: bool,

    /// Reduce transparency
    reduce_transparency: bool,

    /// High contrast mode
    high_contrast: bool,
}

impl AppearancePanel {
    pub fn render(&self, ctx: &mut RenderContext) {
        let mut y = 120.0;

        // Theme Mode
        y = self.draw_section(ctx, y, "Theme Mode");
        y = self.draw_radio_group(ctx, y, &["Light", "Dark", "Auto"], self.theme_mode);
        y += 32.0;

        // Accent Color
        y = self.draw_section(ctx, y, "Accent Color");
        y = self.draw_color_options(ctx, y, &[
            ("Blue", "#1A73E8"),
            ("Teal", "#009688"),
            ("Purple", "#7B1FA2"),
            ("Orange", "#FF6D00"),
            ("Pink", "#E91E63"),
            ("Red", "#D32F2F"),
            ("Green", "#388E3C"),
        ]);
        y += 32.0;

        // Font Family
        y = self.draw_section(ctx, y, "Font Family");
        y = self.draw_dropdown(ctx, y, &[
            "System UI",
            "Sans",
            "Serif",
            "Monospace",
        ], &self.font_family);
        y += 32.0;

        // Font Size
        y = self.draw_section(ctx, y, "Font Size");
        y = self.draw_slider(ctx, y, "A", "", "A", self.font_size);
        y += 32.0;

        // Icon Size
        y = self.draw_section(ctx, y, "Icon Size");
        y = self.draw_radio_group(ctx, y, &["Small", "Default", "Large"], self.icon_size);
        y += 32.0;

        // Window Decorations
        y = self.draw_section(ctx, y, "Window Decorations");
        y = self.draw_toggle(ctx, y, "Show window titlebars", self.window_decorations);
        y += 32.0;

        // Dark Mode Tweaks
        if matches!(self.theme_mode, ThemeMode::Dark | ThemeMode::Auto) {
            y = self.draw_section(ctx, y, "Dark Mode Tweaks");
            y = self.draw_toggle(ctx, y, "Pure black background", self.dark_mode_tweaks.pure_black);
            y += 16.0;
            y = self.draw_toggle(ctx, y, "Reduce transparency", self.dark_mode_tweaks.reduce_transparency);
            y += 16.0;
            y = self.draw_toggle(ctx, y, "High contrast", self.dark_mode_tweaks.high_contrast);
        }
    }

    pub fn apply(&self, daemon: &SettingsDaemonProxy) {
        // Apply theme mode
        daemon.set_theme_mode(self.theme_mode);

        // Apply accent color
        daemon.set_accent_color(self.accent_color);

        // Apply font settings
        daemon.set_font_family(&self.font_family);
        daemon.set_font_size(self.font_size);

        // Apply icon size
        daemon.set_icon_size(self.icon_size);

        // Apply window decorations
        daemon.set_window_decorations(self.window_decorations);

        // Apply dark mode tweaks
        daemon.set_dark_mode_tweaks(self.dark_mode_tweaks);

        // Reload theme
        theme::reload();
    }
}
```

## Display Panel

```rust
pub struct DisplayPanel {
    /// Brightness (0-100)
    brightness: u8,

    /// Scale factor
    scale: ScaleFactor,

    /// Refresh rate
    refresh_rate: u32,

    /// Night light
    night_light: bool,

    /// Night light temperature
    night_light_temperature: u16,

    /// Night light schedule
    night_light_schedule: NightLightSchedule,

    /// Displays
    displays: Vec<DisplayInfo>,

    /// Arrangement mode
    arrangement_mode: bool,
}

pub enum ScaleFactor {
    Scale100,
    Scale125,
    Scale150,
    Scale200,
    Custom(f32),
}

pub enum NightLightSchedule {
    Automatic,
    Custom { start: NaiveTime, end: NaiveTime },
    AlwaysOn,
}

pub struct DisplayInfo {
    id: String,
    name: String,
    resolution: (u32, u32),
    physical_size: (u32, u32),
    refresh_rates: Vec<u32>,
    current_refresh_rate: u32,
    primary: bool,
    enabled: bool,
    position: (i32, i32),
}

impl DisplayPanel {
    pub fn render(&self, ctx: &mut RenderContext) {
        let mut y = 120.0;

        // Brightness
        y = self.draw_section(ctx, y, "Brightness");
        y = self.draw_slider(ctx, y, "🔅", "", "☀️", self.brightness);
        y += 32.0;

        // Scale
        y = self.draw_section(ctx, y, "Display Scale");
        y = self.draw_buttons(ctx, y, &["100%", "125%", "150%", "200%"], self.scale);
        y += 32.0;

        // Refresh Rate
        if !self.displays.is_empty() {
            y = self.draw_section(ctx, y, "Refresh Rate");
            let display = &self.displays[0];
            let rates: Vec<String> = display.refresh_rates.iter()
                .map(|r| format!("{} Hz", r))
                .collect();
            y = self.draw_radio_group(ctx, y, &rates, display.current_refresh_rate);
            y += 32.0;
        }

        // Night Light
        y = self.draw_section(ctx, y, "Night Light");
        y = self.draw_toggle(ctx, y, "Enable Night Light", self.night_light);

        if self.night_light {
            y += 16.0;

            // Temperature
            y = self.draw_slider(ctx, y, "Warm", "", "Cool", self.night_light_temperature);
            y += 24.0;

            // Schedule
            y = self.draw_radio_group(ctx, y, &["Automatic", "Custom", "Always On"], self.night_light_schedule);
        }

        y += 32.0;

        // Displays
        y = self.draw_section(ctx, y, "Displays");

        for display in &self.displays {
            y = self.draw_display_item(ctx, y, display);
            y += 8.0;
        }

        // Arrangement button
        y += 16.0;
        y = self.draw_button(ctx, y, "Arrange Displays");
    }

    pub fn apply(&self, daemon: &SettingsDaemonProxy) {
        // Apply brightness
        daemon.set_brightness(self.brightness);

        // Apply scale
        daemon.set_scale(self.scale);

        // Apply refresh rate
        if let Some(display) = self.displays.first() {
            daemon.set_refresh_rate(&display.id, display.current_refresh_rate);
        }

        // Apply night light
        daemon.set_night_light(
            self.night_light,
            self.night_light_temperature,
            self.night_light_schedule,
        );
    }

    fn draw_display_item(&self, ctx: &mut RenderContext, y: f32, display: &DisplayInfo) -> f32 {
        let item_height = 80.0;
        let x = 240.0;

        // Background
        ctx.fill_rounded_rect(x, y, 660.0, item_height, 8.0, theme.colors.surface);

        // Display icon
        let icon_rect = Rect { x: x + 16.0, y: y + 16.0, width: 48.0, height: 48.0 };
        ctx.draw_icon(icon_rect, "video-display");

        // Display name
        let name_rect = Rect { x: x + 80.0, y: y + 16.0, width: 400.0, height: 20.0 };
        ctx.draw_text(name_rect, &display.name, theme.typography.body, theme.colors.on_surface);

        // Resolution
        let resolution = format!("{} × {}", display.resolution.0, display.resolution.1);
        let res_rect = Rect { x: x + 80.0, y: y + 44.0, width: 200.0, height: 16.0 };
        ctx.draw_text(res_rect, &resolution, theme.typography.caption, theme.colors.on_surface_variant);

        // Primary indicator
        if display.primary {
            let primary_rect = Rect { x: x + 80.0, y: y + 58.0, width: 100.0, height: 16.0 };
            ctx.draw_text(primary_rect, "Primary Display", theme.typography.caption, theme.colors.primary);
        }

        // Enable toggle
        let toggle_rect = Rect { x: x + 560.0, y: y + 24.0, width: 48.0, height: 24.0 };
        self.draw_toggle_at(ctx, toggle_rect, display.enabled);

        y + item_height
    }
}
```

## Network Panel

```rust
pub struct NetworkPanel {
    /// WiFi enabled
    wifi_enabled: bool,

    /// Available networks
    networks: Vec<Network>,

    /// Connected network
    connected: Option<NetworkConnection>,

    /// Hotspot enabled
    hotspot_enabled: bool,

    /// Hotspot settings
    hotspot_settings: HotspotSettings,

    /// Wired connections
    wired_connections: Vec<WiredConnection>,
}

pub struct Network {
    ssid: String,
    strength: u8,
    secured: bool,
    known: bool,
}

pub struct NetworkConnection {
    ssid: String,
    security: SecurityType,
    ip_address: IpAddr,
    gateway: IpAddr,
}

pub enum SecurityType {
    Wpa2,
    Wpa3,
    Wep,
    Open,
}

pub struct HotspotSettings {
    ssid: String,
    password: String,
    band: WiFiBand,
}

pub enum WiFiBand {
    GHz2_4,
    GHz5,
    Auto,
}

pub struct WiredConnection {
    interface: String,
    carrier: bool,
    speed: u32,
    ip_address: Option<IpAddr>,
}

impl NetworkPanel {
    pub fn render(&self, ctx: &mut RenderContext) {
        let mut y = 120.0;

        // WiFi toggle
        y = self.draw_toggle(ctx, y, "Wi-Fi", self.wifi_enabled);

        if !self.wifi_enabled {
            return;
        }

        y += 32.0;

        // Networks list
        y = self.draw_section(ctx, y, "Networks");

        for network in &self.networks {
            y = self.draw_network_item(ctx, y, network);
        }

        y += 32.0;

        // Hotspot
        y = self.draw_section(ctx, y, "Hotspot");
        y = self.draw_toggle(ctx, y, "Enable Hotspot", self.hotspot_enabled);

        if self.hotspot_enabled {
            y += 16.0;
            y = self.draw_text_field(ctx, y, "SSID", &self.hotspot_settings.ssid);
            y += 16.0;
            y = self.draw_password_field(ctx, y, "Password", &self.hotspot_settings.password);
            y += 16.0;
            y = self.draw_radio_group(ctx, y, &["2.4 GHz", "5 GHz", "Auto"], self.hotspot_settings.band);
        }

        y += 32.0;

        // Wired connections
        if !self.wired_connections.is_empty() {
            y = self.draw_section(ctx, y, "Wired");

            for connection in &self.wired_connections {
                y = self.draw_wired_item(ctx, y, connection);
            }
        }
    }

    fn draw_network_item(&self, ctx: &mut RenderContext, y: f32, network: &Network) -> f32 {
        let item_height = 64.0;
        let x = 240.0;

        // Background (if connected)
        let is_connected = self.connected.as_ref()
            .map(|c| c.ssid == network.ssid)
            .unwrap_or(false);

        if is_connected {
            ctx.fill_rounded_rect(x, y, 660.0, item_height, 8.0,
                theme.colors.primary.with_alpha(0.15));
        } else if self.hover_network == network.ssid {
            ctx.fill_rounded_rect(x, y, 660.0, item_height, 8.0,
                theme.colors.surface_variant);
        }

        // WiFi icon with strength indicator
        let icon_rect = Rect { x: x + 16.0, y: y + 8.0, width: 48.0, height: 48.0 };
        ctx.draw_icon(icon_rect, self.wifi_icon_for_strength(network.strength));

        // SSID
        let text_rect = Rect { x: x + 80.0, y: y + 16.0, width: 400.0, height: 20.0 };
        ctx.draw_text(text_rect, &network.ssid, theme.typography.body, theme.colors.on_surface);

        // Lock icon if secured
        if network.secured {
            let lock_rect = Rect { x: x + 80.0, y: y + 40.0, width: 16.0, height: 16.0 };
            ctx.draw_icon(lock_rect, "lock");
        }

        // Connected indicator
        if is_connected {
            let connected_rect = Rect { x: x + 580.0, y: y + 20.0, width: 80.0, height: 24.0 };
            ctx.draw_button(connected_rect, "Connected");
            ctx.fill_rounded_rect(connected_rect, 12.0, theme.colors.primary.with_alpha(0.2));
            ctx.draw_text(connected_rect, "Connected", theme.typography.caption, theme.colors.primary);
        } else if network.known {
            let known_rect = Rect { x: x + 580.0, y: y + 20.0, width: 60.0, height: 24.0 };
            ctx.draw_text(known_rect, "Saved", theme.typography.caption, theme.colors.on_surface_variant);
        }

        y + item_height + 8.0
    }

    fn wifi_icon_for_strength(&self, strength: u8) -> &'static str {
        if strength < 25 {
            "network-wireless-signal-weak"
        } else if strength < 50 {
            "network-wireless-signal-ok"
        } else if strength < 75 {
            "network-wireless-signal-good"
        } else {
            "network-wireless-signal-excellent"
        }
    }

    pub fn connect_to_network(&mut self, ssid: String, password: Option<String>) {
        // Request connection from network manager
        if let Err(err) = network_manager::connect(&ssid, password) {
            self.show_error(format!("Failed to connect: {}", err));
        }
    }
}
```

## Audio Panel

```rust
pub struct AudioPanel {
    /// Output volume (0-100)
    output_volume: u8,

    /// Output device
    output_device: AudioDevice,

    /// Output devices
    output_devices: Vec<AudioDevice>,

    /// Input volume (0-100)
    input_volume: u8,

    /// Input device
    input_device: AudioDevice,

    /// Input devices
    input_devices: Vec<AudioDevice>,

    /// Balance
    balance: f32,

    /// Fade
    fade: f32,

    /// Sound effects
    sound_effects: bool,

    /// System sounds
    system_sounds: HashMap<String, String>,
}

pub struct AudioDevice {
    id: String,
    name: String,
    icon: String,
    ports: Vec<AudioPort>,
}

pub struct AudioPort {
    id: String,
    name: String,
    available: bool,
}

impl AudioPanel {
    pub fn render(&self, ctx: &mut RenderContext) {
        let mut y = 120.0;

        // Output
        y = self.draw_section(ctx, y, "Output");

        // Volume
        y = self.draw_slider(ctx, y, "🔉", "", "🔊", self.output_volume);
        y += 24.0;

        // Device selector
        y = self.draw_device_selector(ctx, y, "Device", &self.output_devices, &self.output_device);
        y += 24.0;

        // Balance
        y = self.draw_balance_slider(ctx, y, self.balance);
        y += 24.0;

        // Output level meter
        y = self.draw_level_meter(ctx, y, "Output Level");
        y += 32.0;

        // Input
        y = self.draw_section(ctx, y, "Input");

        // Volume
        y = self.draw_slider(ctx, y, "🎤", "", "", self.input_volume);
        y += 24.0;

        // Device selector
        y = self.draw_device_selector(ctx, y, "Device", &self.input_devices, &self.input_device);
        y += 24.0;

        // Input level meter
        y = self.draw_level_meter(ctx, y, "Input Level");
        y += 32.0;

        // Sound Effects
        y = self.draw_section(ctx, y, "Sound Effects");
        y = self.draw_toggle(ctx, y, "Enable sound effects", self.sound_effects);

        if self.sound_effects {
            y += 16.0;
            y = self.draw_sound_effects_list(ctx, y);
        }

        y += 32.0;

        // System Sounds
        y = self.draw_section(ctx, y, "System Sounds");
        y = self.draw_system_sounds(ctx, y);
    }

    fn draw_device_selector(&self, ctx: &mut RenderContext, y: f32, label: &str,
        devices: &[AudioDevice], selected: &AudioDevice) -> f32 {
        let x = 240.0;

        // Label
        ctx.draw_text(Rect { x, y, width: 200.0, height: 20.0 }, label,
            theme.typography.body, theme.colors.on_surface);

        // Device dropdown
        let dropdown_rect = Rect { x: x + 200.0, y, width: 440.0, height: 36.0 };
        ctx.fill_rounded_rect(dropdown_rect, 6.0, theme.colors.surface_variant);
        ctx.draw_icon(Rect { x: x + 620.0, y: y + 6.0, width: 24.0, height: 24.0 }, "pan-down");

        // Selected device
        ctx.draw_text(Rect { x: x + 216.0, y: y + 8.0, width: 400.0, height: 20.0 },
            &selected.name, theme.typography.body, theme.colors.on_surface);

        y + 44.0
    }

    fn draw_balance_slider(&self, ctx: &mut RenderContext, y: f32, balance: f32) -> f32 {
        let x = 240.0;
        let slider_width = 200.0;

        // Left label
        ctx.draw_text(Rect { x, y, width: 80.0, height: 20.0 }, "Left",
            theme.typography.caption, theme.colors.on_surface_variant);

        // Right label
        ctx.draw_text(Rect { x: x + slider_width - 80.0, y, width: 80.0, height: 20.0 }, "Right",
            theme.typography.caption, theme.colors.on_surface_variant);

        // Track
        let track_y = y + 24.0;
        ctx.fill_rect(Rect { x: x + 60.0, y: track_y, width: slider_width - 120.0, height: 4.0 },
            theme.colors.surface_variant);

        // Center indicator
        let center_x = x + slider_width / 2.0;
        ctx.fill_rect(Rect { x: center_x - 1.0, y: track_y - 4.0, width: 2.0, height: 12.0 },
            theme.colors.outline);

        // Thumb
        let thumb_x = center_x + (balance * (slider_width / 2.0 - 60.0));
        ctx.fill_circle(thumb_x, track_y + 2.0, 8.0, theme.colors.primary);

        y + 40.0
    }

    fn draw_level_meter(&self, ctx: &mut RenderContext, y: f32, label: &str) -> f32 {
        let x = 240.0;

        // Label
        ctx.draw_text(Rect { x, y, width: 200.0, height: 20.0 }, label,
            theme.typography.body, theme.colors.on_surface);

        // Meter background
        let meter_rect = Rect { x: x + 200.0, y: y + 4.0, width: 440.0, height: 16.0 };
        ctx.fill_rounded_rect(meter_rect, 4.0, theme.colors.surface_variant);

        // Current level (animated)
        let level = self.current_level();
        let filled_width = meter_rect.width * level;
        ctx.fill_rounded_rect(
            Rect { x: meter_rect.x, y: meter_rect.y, width: filled_width, height: meter_rect.height },
            4.0,
            if level > 0.9 { theme.colors.error } else { theme.colors.primary }
        );

        y + 28.0
    }

    pub fn apply(&self, daemon: &SettingsDaemonProxy) {
        // Apply output volume
        daemon.set_output_volume(self.output_volume);

        // Apply output device
        daemon.set_output_device(&self.output_device.id);

        // Apply input volume
        daemon.set_input_volume(self.input_volume);

        // Apply input device
        daemon.set_input_device(&self.input_device.id);

        // Apply balance
        daemon.set_audio_balance(self.balance);

        // Apply sound effects
        daemon.set_sound_effects(self.sound_effects);
    }
}
```

## Power Panel

```rust
pub struct PowerPanel {
    /// Battery percentage
    battery_percentage: u8,

    /// Power state
    power_state: PowerState,

    /// Screen blank timeout
    screen_blank_timeout: u32,

    /// Suspend timeout
    suspend_timeout: u32,

    /// Power button action
    power_button_action: PowerAction,

    /// Lid close action
    lid_close_action: PowerAction,

    /// Critical battery action
    critical_battery_action: PowerAction,

    /// Battery Saver
    battery_saver_enabled: bool,
    battery_saver_threshold: u8,
}

pub enum PowerState {
    Charging,
    Discharging,
    FullyCharged,
    AC,
}

pub enum PowerAction {
    Nothing,
    Suspend,
    Hibernate,
    PowerOff,
}

impl PowerPanel {
    pub fn render(&self, ctx: &mut RenderContext) {
        let mut y = 120.0;

        // Battery status
        y = self.draw_section(ctx, y, "Battery");
        y = self.draw_battery_status(ctx, y, self.battery_percentage, &self.power_state);
        y += 32.0;

        // Power Saving
        y = self.draw_section(ctx, y, "Power Saving");
        y = self.draw_toggle(ctx, y, "Battery Saver", self.battery_saver_enabled);

        if self.battery_saver_enabled {
            y += 16.0;
            y = self.draw_slider(ctx, y, "Turn on at", "", "%", self.battery_saver_threshold);
        }

        y += 32.0;

        // Screen Blank
        y = self.draw_section(ctx, y, "Screen Blank");
        y = self.draw_timeout_selector(ctx, y, "Blank screen after", self.screen_blank_timeout);
        y += 32.0;

        // Suspend
        y = self.draw_section(ctx, y, "Automatic Suspend");
        y = self.draw_timeout_selector(ctx, y, "Suspend after", self.suspend_timeout);
        y += 32.0;

        // Button Actions
        y = self.draw_section(ctx, y, "Button Actions");
        y = self.draw_action_selector(ctx, y, "Power button", self.power_button_action);
        y += 16.0;
        y = self.draw_action_selector(ctx, y, "Lid close", self.lid_close_action);
        y += 16.0;
        y = self.draw_action_selector(ctx, y, "Critical battery", self.critical_battery_action);
    }

    fn draw_battery_status(&self, ctx: &mut RenderContext, y: f32,
        percentage: u8, state: &PowerState) -> f32 {
        let x = 240.0;

        // Battery icon
        let battery_rect = Rect { x, y, width: 48.0, height: 48.0 };
        ctx.draw_icon(battery_rect, self.battery_icon(percentage, state));

        // Percentage
        let percent_text = format!("{}%", percentage);
        let percent_rect = Rect { x: x + 60.0, y, width: 80.0, height: 20.0 };
        ctx.draw_text(percent_rect, &percent_text, theme.typography.h4, theme.colors.on_surface);

        // Status
        let status_text = match state {
            PowerState::Charging => "Charging",
            PowerState::Discharging => "On battery",
            PowerState::FullyCharged => "Fully charged",
            PowerState::AC => "Plugged in",
        };
        let status_rect = Rect { x: x + 60.0, y: y + 24.0, width: 200.0, height: 16.0 };
        ctx.draw_text(status_rect, status_text, theme.typography.caption, theme.colors.on_surface_variant);

        // Time remaining (if discharging)
        if matches!(state, PowerState::Discharging) {
            if let Some(remaining) = self.calculate_time_remaining(percentage) {
                let time_text = format!("{} remaining", remaining);
                let time_rect = Rect { x: x + 60.0, y: y + 44.0, width: 200.0, height: 16.0 };
                ctx.draw_text(time_rect, &time_text, theme.typography.caption, theme.colors.on_surface_variant);
            }
        }

        y + 72.0
    }

    fn battery_icon(&self, percentage: u8, state: &PowerState) -> &'static str {
        let base = match percentage {
            0..=10 => "battery-empty",
            11..=30 => "battery-low",
            31..=50 => "battery-caution",
            51..=80 => "battery-good",
            _ => "battery-full",
        };

        match state {
            PowerState::Charging => "battery-charging",
            _ => base,
        }
    }

    fn calculate_time_remaining(&self, percentage: u8) -> Option<String> {
        // Estimate based on discharge rate
        let discharge_rate = 1.0; // % per minute (example)
        let minutes = percentage as f32 / discharge_rate;

        if minutes >= 60.0 {
            let hours = (minutes / 60.0).floor();
            let mins = (minutes % 60.0).floor();
            Some(format!("{}h {}m", hours, mins))
        } else {
            Some(format!("{}m", minutes.floor()))
        }
    }

    pub fn apply(&self, daemon: &SettingsDaemonProxy) {
        daemon.set_screen_blank_timeout(self.screen_blank_timeout);
        daemon.set_suspend_timeout(self.suspend_timeout);
        daemon.set_power_button_action(self.power_button_action);
        daemon.set_lid_close_action(self.lid_close_action);
        daemon.set_critical_battery_action(self.critical_battery_action);
        daemon.set_battery_saver(self.battery_saver_enabled, self.battery_saver_threshold);
    }
}
```

## Input Devices Panel

```rust
pub struct InputDevicesPanel {
    /// Keyboards
    keyboards: Vec<Keyboard>,

    /// Pointers (mice, touchpads)
    pointers: Vec<Pointer>,

    /// Displays
    displays: Vec<Touchscreen>,
}

pub struct Keyboard {
    id: String,
    name: String,
    layout: String,
    variant: Option<String>,
    repeat: bool,
    repeat_delay: u32,
    repeat_interval: u32,
}

pub struct Pointer {
    id: String,
    name: String,
    pointer_type: PointerType,
    left_handed: bool,
    acceleration: f32,
    natural_scrolling: bool,
    middle_click_emulation: bool,
    tap_to_click: bool,
    two_finger_scroll: bool,
    disable_while_typing: bool,
}

pub enum PointerType {
    Mouse,
    Touchpad,
    Trackball,
}

pub struct Touchscreen {
    id: String,
    name: String,
    calibration: CalibrationData,
}

impl InputDevicesPanel {
    pub fn render(&self, ctx: &mut RenderContext) {
        let mut y = 120.0;

        // Keyboards
        if !self.keyboards.is_empty() {
            y = self.draw_section(ctx, y, "Keyboards");

            for keyboard in &self.keyboards {
                y = self.draw_keyboard(ctx, y, keyboard);
            }

            y += 32.0;
        }

        // Pointers
        if !self.pointers.is_empty() {
            y = self.draw_section(ctx, y, "Pointers");

            for pointer in &self.pointers {
                y = self.draw_pointer(ctx, y, pointer);
            }
        }
    }

    fn draw_keyboard(&self, ctx: &mut RenderContext, y: f32, keyboard: &Keyboard) -> f32 {
        let x = 240.0;

        // Keyboard name
        ctx.draw_text(Rect { x, y, width: 400.0, height: 20.0 }, &keyboard.name,
            theme.typography.h4, theme.colors.on_surface);

        let mut y = y + 32.0;

        // Layout
        y = self.draw_dropdown(ctx, y, "Layout", &self.available_layouts(), &keyboard.layout);
        y += 16.0;

        // Key repeat
        y = self.draw_toggle(ctx, y, "Key repeat", keyboard.repeat);

        if keyboard.repeat {
            y += 16.0;
            y = self.draw_slider(ctx, y, "Short", "", "Long", keyboard.repeat_delay);
            y += 16.0;
            y = self.draw_slider(ctx, y, "Fast", "", "Slow", keyboard.repeat_interval);
        }

        y + 16.0
    }

    fn draw_pointer(&self, ctx: &mut RenderContext, y: f32, pointer: &Pointer) -> f32 {
        let x = 240.0;

        // Pointer name with icon
        let icon = match pointer.pointer_type {
            PointerType::Mouse => "input-mouse",
            PointerType::Touchpad => "input-touchpad",
            PointerType::Trackball => "input-trackball",
        };
        ctx.draw_icon(Rect { x, y, width: 24.0, height: 24.0 }, icon);
        ctx.draw_text(Rect { x: x + 32.0, y, width: 400.0, height: 20.0 }, &pointer.name,
            theme.typography.h4, theme.colors.on_surface);

        let mut y = y + 32.0;

        // Left-handed mode
        y = self.draw_toggle(ctx, y, "Left-handed mode", pointer.left_handed);
        y += 16.0;

        // Pointer speed
        y = self.draw_slider(ctx, y, "Slow", "", "Fast", pointer.acceleration);
        y += 16.0;

        // Natural scrolling
        y = self.draw_toggle(ctx, y, "Natural scrolling", pointer.natural_scrolling);

        // Touchpad-specific options
        if matches!(pointer.pointer_type, PointerType::Touchpad) {
            y += 16.0;
            y = self.draw_toggle(ctx, y, "Tap to click", pointer.tap_to_click);
            y += 16.0;
            y = self.draw_toggle(ctx, y, "Two-finger scroll", pointer.two_finger_scroll);
            y += 16.0;
            y = self.draw_toggle(ctx, y, "Disable while typing", pointer.disable_while_typing);
        }

        y + 16.0
    }
}
```

## Users Panel

```rust
pub struct UsersPanel {
    /// Current user
    current_user: User,

    /// All users
    users: Vec<User>,

    /// Avatar picker open
    avatar_picker_open: bool,

    /// Adding user
    adding_user: bool,
}

pub struct User {
    username: String,
    real_name: String,
    avatar: Image,
    account_type: AccountType,
    login_shell: String,
    language: String,
    autologin: bool,
}

pub enum AccountType {
    Standard,
    Administrator,
}

impl UsersPanel {
    pub fn render(&self, ctx: &mut RenderContext) {
        let mut y = 120.0;

        // Current user
        y = self.draw_section(ctx, y, "Your Account");
        y = self.draw_user_card(ctx, y, &self.current_user, true);

        // Edit profile button
        y = self.draw_button(ctx, y, "Edit Profile");

        y += 32.0;

        // Other users
        y = self.draw_section(ctx, y, "Other Users");

        for user in &self.users {
            if user.username != self.current_user.username {
                y = self.draw_user_card(ctx, y, user, false);
            }
        }

        // Add user button
        y += 16.0;
        y = self.draw_button(ctx, y, "Add User...");
    }

    fn draw_user_card(&self, ctx: &mut RenderContext, y: f32, user: &User, is_current: bool) -> f32 {
        let x = 240.0;
        let card_height = 80.0;

        // Background
        ctx.fill_rounded_rect(x, y, 660.0, card_height, 8.0, theme.colors.surface);

        // Avatar
        let avatar_rect = Rect { x: x + 16.0, y: y + 8.0, width: 64.0, height: 64.0 };
        ctx.draw_circle_avatar(avatar_rect, &user.avatar);

        // Name
        let name_rect = Rect { x: x + 96.0, y: y + 16.0, width: 400.0, height: 24.0 };
        ctx.draw_text(name_rect, &user.real_name, theme.typography.h4, theme.colors.on_surface);

        // Username
        let username_rect = Rect { x: x + 96.0, y: y + 44.0, width: 200.0, height: 16.0 };
        ctx.draw_text(username_rect, &format!("@{}", user.username),
            theme.typography.caption, theme.colors.on_surface_variant);

        // Account type badge
        let badge_rect = Rect { x: x + 96.0, y: y + 60.0, width: 100.0, height: 16.0 };
        let badge_text = match user.account_type {
            AccountType::Standard => "Standard",
            AccountType::Administrator => "Administrator",
        };
        ctx.draw_text(badge_rect, badge_text, theme.typography.caption, theme.colors.primary);

        // Current user indicator
        if is_current {
            let indicator_rect = Rect { x: x + 580.0, y: y + 28.0, width: 80.0, height: 24.0 };
            ctx.fill_rounded_rect(indicator_rect, 12.0, theme.colors.primary.with_alpha(0.2));
            ctx.draw_text(indicator_rect, "You", theme.typography.caption, theme.colors.primary);
        }

        y + card_height + 8.0
    }

    pub fn add_user(&mut self, username: String, real_name: String, account_type: AccountType) {
        // Call user management service
        match user_manager::create_user(&username, &real_name, account_type) {
            Ok(_) => {
                self.users.push(User {
                    username,
                    real_name,
                    avatar: Image::default_avatar(),
                    account_type,
                    login_shell: "/bin/bash".into(),
                    language: "en_US.UTF-8".into(),
                    autologin: false,
                });
            }
            Err(err) => {
                self.show_error(format!("Failed to create user: {}", err));
            }
        }
    }
}
```

## Date & Time Panel

```rust
pub struct DateTimePanel {
    /// Automatic time
    automatic: bool,

    /// NTP servers
    ntp_servers: Vec<String>,

    /// Manual date
    manual_date: NaiveDate,

    /// Manual time
    manual_time: NaiveTime,

    /// Timezone
    timezone: String,

    /// Timezone filter (search)
    timezone_filter: String,

    /// Timezone search results
    timezone_results: Vec<TimezoneInfo>,
    /// 24-hour time
    hour_24: bool,

    /// Show seconds
    show_seconds: bool,

    /// Date format
    date_format: DateFormat,

    /// First day of week
    first_day_of_week: Weekday,
}

pub enum DateFormat {
    Iso,        // 2025-01-07
    Us,         // 01/07/2025
    Europe,     // 07/01/2025
}

pub struct TimezoneInfo {
    name: String,
    region: String,
    offset: i32,
}

impl DateTimePanel {
    pub fn render(&self, ctx: &mut RenderContext) {
        let mut y = 120.0;

        // Automatic time
        y = self.draw_section(ctx, y, "Automatic Time");
        y = self.draw_toggle(ctx, y, "Set time automatically", self.automatic);

        if self.automatic {
            y += 16.0;
            y = self.draw_text_field(ctx, y, "NTP Servers",
                &self.ntp_servers.join(", "));
        } else {
            y += 16.0;
            y = self.draw_date_selector(ctx, y, "Date", self.manual_date);
            y += 16.0;
            y = self.draw_time_selector(ctx, y, "Time", self.manual_time);
        }

        y += 32.0;

        // Timezone
        y = self.draw_section(ctx, y, "Timezone");
        y = self.draw_timezone_selector(ctx, y, &self.timezone);

        y += 32.0;

        // Format
        y = self.draw_section(ctx, y, "Format");

        y = self.draw_toggle(ctx, y, "24-hour time", self.hour_24);
        y += 16.0;
        y = self.draw_toggle(ctx, y, "Show seconds", self.show_seconds);
        y += 16.0;
        y = self.draw_radio_group(ctx, y, &["ISO", "US", "Europe"], self.date_format);
        y += 16.0;
        y = self.draw_radio_group(ctx, y, &["Monday", "Sunday", "Saturday"],
            self.first_day_of_week);
    }

    fn draw_timezone_selector(&self, ctx: &mut RenderContext, y: f32, current: &str) -> f32 {
        let x = 240.0;

        // Search box
        let search_rect = Rect { x, y, width: 660.0, height: 36.0 };
        ctx.fill_rounded_rect(search_rect, 6.0, theme.colors.surface_variant);
        ctx.draw_text(Rect { x: x + 12.0, y: y + 8.0, width: 400.0, height: 20.0 },
            "Search timezones...", theme.typography.body, theme.colors.on_surface_variant);

        let mut y = y + 48.0;

        // Current timezone
        ctx.draw_text(Rect { x, y, width: 200.0, height: 20.0 }, "Current Timezone",
            theme.typography.caption, theme.colors.on_surface_variant);
        ctx.draw_text(Rect { x: x + 200.0, y, width: 460.0, height: 20.0 },
            current, theme.typography.body, theme.colors.on_surface);

        y + 28.0
    }

    pub fn apply(&self, daemon: &SettingsDaemonProxy) {
        if self.automatic {
            daemon.enable_ntp(&self.ntp_servers);
        } else {
            daemon.set_datetime(self.manual_date, self.manual_time);
        }

        daemon.set_timezone(&self.timezone);
        daemon.set_time_format(self.hour_24, self.show_seconds, self.date_format);
    }
}
```

## Accessibility Panel

```rust
pub struct AccessibilityPanel {
    /// High contrast
    high_contrast: bool,

    /// Reduce animation
    reduce_animation: bool,

    /// Screen reader
    screen_reader: bool,

    /// Screen reader settings
    screen_reader_settings: ScreenReaderSettings,

    /// Text size
    text_size: TextSize,

    /// Screen magnifier
    magnifier_enabled: bool,

    /// Magnifier settings
    magnifier_settings: MagnifierSettings,

    /// Visual alerts
    visual_alerts: bool,

    /// Audio alerts
    audio_alerts: bool,

    /// Sticky keys
    sticky_keys: bool,

    /// Slow keys
    slow_keys: bool,

    /// Bounce keys
    bounce_keys: bool,

    /// Mouse keys
    mouse_keys: bool,
}

pub enum TextSize {
    Normal,
    Large,
    Larger,
}

pub struct ScreenReaderSettings {
    speech_rate: f32,
    pitch: f32,
    volume: u8,
}

pub struct MagnifierSettings {
    magnification: f32,
    follow_focus: bool,
    follow_mouse: bool,
    lens_shape: LensShape,
}

pub enum LensShape {
    Rectangle,
    Circle,
}

impl AccessibilityPanel {
    pub fn render(&self, ctx: &mut RenderContext) {
        let mut y = 120.0;

        // Visual
        y = self.draw_section(ctx, y, "Visual");
        y = self.draw_toggle(ctx, y, "High contrast", self.high_contrast);
        y += 16.0;
        y = self.draw_toggle(ctx, y, "Reduce animation", self.reduce_animation);
        y += 16.0;
        y = self.draw_radio_group(ctx, y, &["Normal", "Large", "Larger"], self.text_size);
        y += 16.0;
        y = self.draw_toggle(ctx, y, "Screen magnifier", self.magnifier_enabled);

        if self.magnifier_enabled {
            y += 16.0;
            y = self.draw_slider(ctx, y, "1.0×", "", "5.0×", self.magnifier_settings.magnification);
            y += 16.0;
            y = self.draw_toggle(ctx, y, "Follow keyboard focus", self.magnifier_settings.follow_focus);
            y += 16.0;
            y = self.draw_toggle(ctx, y, "Follow mouse cursor", self.magnifier_settings.follow_mouse);
        }

        y += 16.0;
        y = self.draw_toggle(ctx, y, "Visual alerts", self.visual_alerts);

        y += 32.0;

        // Audio
        y = self.draw_section(ctx, y, "Audio");
        y = self.draw_toggle(ctx, y, "Screen reader", self.screen_reader);

        if self.screen_reader {
            y += 16.0;
            y = self.draw_slider(ctx, y, "Slow", "", "Fast", self.screen_reader_settings.speech_rate);
            y += 16.0;
            y = self.draw_slider(ctx, y, "Low", "", "High", self.screen_reader_settings.pitch);
            y += 16.0;
            y = self.draw_slider(ctx, y, "Quiet", "", "Loud", self.screen_reader_settings.volume);
        }

        y += 16.0;
        y = self.draw_toggle(ctx, y, "Audio alerts", self.audio_alerts);

        y += 32.0;

        // Keyboard
        y = self.draw_section(ctx, y, "Keyboard");
        y = self.draw_toggle(ctx, y, "Sticky keys", self.sticky_keys);
        y += 16.0;
        y = self.draw_toggle(ctx, y, "Slow keys", self.slow_keys);
        y += 16.0;
        y = self.draw_toggle(ctx, y, "Bounce keys", self.bounce_keys);

        y += 32.0;

        // Mouse/Pointing
        y = self.draw_section(ctx, y, "Mouse & Pointing");
        y = self.draw_toggle(ctx, y, "Mouse keys", self.mouse_keys);
    }

    pub fn apply(&self, daemon: &SettingsDaemonProxy) {
        daemon.set_accessibility_high_contrast(self.high_contrast);
        daemon.set_accessibility_reduce_animation(self.reduce_animation);
        daemon.set_accessibility_screen_reader(self.screen_reader, &self.screen_reader_settings);
        daemon.set_accessibility_text_size(self.text_size);
        daemon.set_accessibility_magnifier(self.magnifier_enabled, &self.magnifier_settings);
        daemon.set_accessibility_visual_alerts(self.visual_alerts);
        daemon.set_accessibility_audio_alerts(self.audio_alerts);
        daemon.set_accessibility_sticky_keys(self.sticky_keys);
        daemon.set_accessibility_slow_keys(self.slow_keys);
        daemon.set_accessibility_bounce_keys(self.bounce_keys);
        daemon.set_accessibility_mouse_keys(self.mouse_keys);

        // Notify AT-SPI
        atspi::update_accessibility_settings();
    }
}
```

## About Panel

```rust
pub struct AboutPanel {
    /// System information
    system_info: SystemInfo,

    /// Hardware information
    hardware_info: HardwareInfo,

    /// Software updates
    update_info: Option<UpdateInfo>,
}

pub struct SystemInfo {
    os_name: String,
    os_version: String,
    build_id: String,
    desktop_environment: String,
    windowing_system: String,
}

pub struct HardwareInfo {
    cpu: String,
    memory: usize,
    disk: Vec<DiskInfo>,
    gpu: Vec<GpuInfo>,
}

pub struct DiskInfo {
    device: String,
    size: u64,
    used: u64,
}

pub struct GpuInfo {
    name: String,
    driver: String,
}

pub struct UpdateInfo {
    available: bool,
    version: String,
    size: u64,
}

impl AboutPanel {
    pub fn render(&self, ctx: &mut RenderContext) {
        let mut y = 120.0;

        // Logo and OS name
        let logo_rect = Rect { x: 420.0, y: y, width: 64.0, height: 64.0 };
        ctx.draw_icon(logo_rect, "rustica-logo");

        let name_rect = Rect { x: 240.0, y: y + 80.0, width: 420.0, height: 32.0 };
        ctx.draw_text(name_rect, &self.system_info.os_name,
            theme.typography.h2, theme.colors.on_surface, TextAlignment::Center);

        let version_rect = Rect { x: 240.0, y: y + 116.0, width: 420.0, height: 20.0 };
        ctx.draw_text(version_rect, &self.system_info.os_version,
            theme.typography.body, theme.colors.on_surface_variant, TextAlignment::Center);

        y += 160.0;

        // System details
        y = self.draw_section(ctx, y, "System");
        y = self.draw_detail_row(ctx, y, "OS Name", &self.system_info.os_name);
        y = self.draw_detail_row(ctx, y, "OS Version", &self.system_info.os_version);
        y = self.draw_detail_row(ctx, y, "Build ID", &self.system_info.build_id);
        y = self.draw_detail_row(ctx, y, "Desktop Environment", &self.system_info.desktop_environment);
        y = self.draw_detail_row(ctx, y, "Windowing System", &self.system_info.windowing_system);

        y += 32.0;

        // Hardware
        y = self.draw_section(ctx, y, "Hardware");
        y = self.draw_detail_row(ctx, y, "Processor", &self.hardware_info.cpu);

        let memory_gb = self.hardware_info.memory / (1024 * 1024 * 1024);
        y = self.draw_detail_row(ctx, y, "Memory", &format!("{} GB", memory_gb));

        for disk in &self.hardware_info.disk {
            let used_gb = disk.used / (1024 * 1024 * 1024);
            let total_gb = disk.size / (1024 * 1024 * 1024);
            y = self.draw_detail_row(ctx, y, &disk.device,
                &format!("{} GB / {} GB", used_gb, total_gb));
        }

        for gpu in &self.hardware_info.gpu {
            y = self.draw_detail_row(ctx, y, "Graphics", &gpu.name);
        }

        y += 32.0;

        // Updates
        y = self.draw_section(ctx, y, "Software Updates");

        if let Some(ref update) = self.update_info {
            if update.available {
                y = self.draw_update_banner(ctx, y, update);
                y += 16.0;
                y = self.draw_button(ctx, y, "Install Update");
            } else {
                y = self.draw_detail_row(ctx, y, "Status", "Up to date");
            }
        }

        // Additional buttons
        y += 32.0;
        y = self.draw_button(ctx, y, "System Report...");
        y += 8.0;
        y = self.draw_button(ctx, y, "Check for Updates");
    }

    fn draw_detail_row(&self, ctx: &mut RenderContext, y: f32, label: &str, value: &str) -> f32 {
        let x = 240.0;

        ctx.draw_text(Rect { x, y, width: 200.0, height: 20.0 }, label,
            theme.typography.caption, theme.colors.on_surface_variant);

        ctx.draw_text(Rect { x: x + 220.0, y, width: 440.0, height: 20.0 }, value,
            theme.typography.body, theme.colors.on_surface);

        y + 28.0
    }

    fn draw_update_banner(&self, ctx: &mut RenderContext, y: f32, update: &UpdateInfo) -> f32 {
        let x = 240.0;
        let banner_height = 80.0;

        // Background
        ctx.fill_rounded_rect(x, y, 660.0, banner_height, 8.0,
            theme.colors.primary.with_alpha(0.15));

        // Icon
        ctx.draw_icon(Rect { x: x + 16.0, y: y + 16.0, width: 48.0, height: 48.0 },
            "software-update-available");

        // Text
        ctx.draw_text(Rect { x: x + 80.0, y: y + 16.0, width: 400.0, height: 24.0 },
            "Update Available", theme.typography.h4, theme.colors.on_surface);

        let size_mb = update.size / (1024 * 1024);
        ctx.draw_text(Rect { x: x + 80.0, y: y + 44.0, width: 400.0, height: 16.0 },
            &format!("Version {} • {} MB", update.version, size_mb),
            theme.typography.caption, theme.colors.on_surface_variant);

        y + banner_height
    }
}
```

## Navigation & State Management

```rust
impl SettingsApp {
    fn navigate_to_panel(&mut self, panel: SettingsPanel) {
        // Add to history
        self.history.push(self.current_panel.clone());

        // Navigate
        self.current_panel = panel;

        // Clear pending changes
        if !self.pending_changes.is_empty() {
            self.prompt_unsaved_changes();
        }

        // Render
        self.render();
    }

    fn go_back(&mut self) {
        if let Some(previous) = self.history.pop() {
            self.current_panel = previous;
            self.render();
        }
    }

    fn apply_changes(&mut self) {
        // Apply all pending changes
        for (key, value) in &self.pending_changes {
            self.daemon_client.set_setting(key, value);
        }

        // Clear pending
        self.pending_changes.clear();

        // Update originals
        self.refresh_originals();

        // Show success notification
        self.show_notification("Settings applied");
    }

    fn revert_changes(&mut self) {
        // Revert all pending changes
        for (key, original) in &self.original_values {
            self.daemon_client.set_setting(key, original);
        }

        // Clear pending
        self.pending_changes.clear();

        // Reload settings
        self.refresh_originals();

        // Render
        self.render();

        // Show notification
        self.show_notification("Settings reverted");
    }

    fn has_pending_changes(&self) -> bool {
        !self.pending_changes.is_empty()
    }

    fn handle_external_change(&mut self, key: SettingKey, value: SettingValue) {
        // Update original value
        self.original_values.insert(key, value);

        // Remove from pending if we have a local change
        self.pending_changes.remove(&key);

        // Re-render to show updated value
        self.render();
    }
}
```

## D-Bus Integration

```rust
// D-Bus interface for settings daemon

#[dbus_proxy(interface = "org.rustica.SettingsDaemon")]
trait SettingsDaemonProxy {
    /// Get all settings
    fn get_all_settings(&self) -> HashMap<SettingKey, SettingValue>;

    /// Get a single setting
    fn get_setting(&self, key: SettingKey) -> SettingValue;

    /// Set a setting
    fn set_setting(&self, key: SettingKey, value: SettingValue);

    /// Reset a setting to default
    fn reset_setting(&self, key: SettingKey);

    /// Apply all pending changes
    fn apply_changes(&self);

    /// Revert all pending changes
    fn revert_changes(&self);

    /// Signal: Setting changed
    #[dbus_proxy(signal)]
    fn setting_changed(&self, key: SettingKey, value: SettingValue);

    /// Signal: Settings applied
    #[dbus_proxy(signal)]
    fn settings_applied(&self);
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-settings/
│   ├── Cargo.toml
│   ├── resources/
│   │   └── icons/
│   └── src/
│       ├── main.rs
│       ├── app.rs
│       ├── panels/
│       │   ├── mod.rs
│       │   ├── home.rs
│       │   ├── appearance.rs
│       │   ├── display.rs
│       │   ├── network.rs
│       │   ├── bluetooth.rs
│       │   ├── audio.rs
│       │   ├── power.rs
│       │   ├── input.rs
│       │   ├── users.rs
│       │   ├── datetime.rs
│       │   ├── accessibility.rs
│       │   └── about.rs
│       ├── widgets/
│       │   ├── mod.rs
│       │   ├── sidebar.rs
│       │   ├── toggle.rs
│       │   ├── slider.rs
│       │   ├── dropdown.rs
│       │   └── button.rs
│       └── dbus/
│           └── daemon_proxy.rs
└── libs/
    └── librustica/
        └── settings/
            └── src/
                └── types.rs
```

## Dependencies

```toml
[package]
name = "rustica-settings"
version = "1.0.0"
edition = "2021"

[dependencies]
# GUI framework
librustica = { path = "../../../libs/librustica" }

# D-Bus
zbus = "3.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Time
chrono = "0.4"

# Network
network-manager = "0.1"

# Audio
pulseaudio = "0.1"

# User management
users = "0.11"

# System info
sysinfo = "0.29"

# Hardware
pci-ids = "0.2"

# Updates
rustica-update = { path = "../../package-manager/rustica-update" }
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Panel load | <200ms | Click to visible |
| Settings apply | <100ms | Apply to confirmed |
| Search filter | <50ms | Keystroke to filtered |
| Memory | <50MB | App usage |

## Success Criteria

- [ ] All 11 panels functional
- [ ] Real-time preview works
- [ ] Apply/revert works correctly
- [ ] D-Bus integration complete
- [ ] Full accessibility support
- [ ] Performance targets met
- [ ] All settings persist correctly

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## Timeline

- Week 1: Core app structure + sidebar + 3 basic panels
- Week 2: 4 more panels + widget library
- Week 3: Remaining panels + D-Bus integration
- Week 4: Testing + polish + accessibility

**Total**: 4 weeks
