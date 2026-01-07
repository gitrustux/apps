# Settings App Prototype Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Type**: Quick Prototype (UX Validation)
**Timeline**: 1 week (concurrent with shell development)

## Overview

This is a **quick-and-dirty prototype** of the Settings application to validate UX design decisions. It is **not production code** and will be replaced with the full implementation in Phase 5.

## Prototype Goals

- Validate settings navigation UX
- Test category organization
- Validate control layouts (sliders, toggles, dropdowns)
- Test apply/revert behavior
- Get feedback on visual hierarchy
- **DO NOT** implement all settings (just 3-5 core panels)

## Minimal Feature Set

### What to Build

```rust
// Only 3-5 core setting panels

pub struct SettingsPrototype {
    /// Current panel
    current_panel: Panel,

    /// Pending changes (not yet applied)
    pending_changes: HashMap<String, SettingChange>,

    /// Original values (for revert)
    original_values: HashMap<String, SettingValue>,
}

pub enum Panel {
    Home,
    Appearance,    // ✓ Include (test theme switching)
    Display,       // ✓ Include (test DPI scaling)
    Network,       // ✓ Include (test WiFi settings UI)
    Audio,         // ⊙ Maybe (if time permits)
    About,         // ⊙ Maybe (if time permits)
}
```

## UI Layout (Simplified)

```
┌─────────────────────────────────────────────────────────────────┐
│ ⚙️ Settings                                                      │
├─────────────────────────────────────────────────────────────────┤
│  ◀ Settings                                    [Apply] [Revert]  │
├─────────────────────────────────────────────────────────────────┤
│  Sidebar (200px)          │  Content Area (600px)            │
│  ┌──────────────────────┐  │                                   │
│  │ 🎨 Appearance        │  │  Appearance                       │
│  │ 🖥️  Display           │  │  ────────────────────────────│
│  │ 📶 Network           │  │                                   │
│  │ 🔊 Audio             │  │  Theme                           │
│  │ 🔋 Battery           │  │  ○ Light   ● Dark              │
│  │ ℹ️  About             │  │                                   │
│  │                      │  │  Accent Color                    │
│  │                      │  │  ○ Blue   ○ Teal   ● Purple    │
│  │                      │  │                                   │
│  └──────────────────────┘  │                                   │
│                           │  Font Size                       │
│                           │  ◀━━━●━━━▶  [A]              │
│                           │                                   │
│                           │  Icon Size                       │
│                           │  ○ Small  ● Large  ○ Large      │
└─────────────────────────────────────────────────────────────────┘
```

## Panel Implementations

### 1. Appearance Panel

```rust
pub struct AppearancePanel {
    // Settings
    theme_mode: ThemeMode,      // Light, Dark, Auto
    accent_color: AccentColor,   // Blue, Teal, Purple, etc.
    font_size: FontSize,        // Small, Default, Large
    icon_size: IconSize,        // Small, Default, Large

    // Original values (for revert)
    original_theme_mode: ThemeMode,
    original_accent_color: AccentColor,
}

pub enum ThemeMode {
    Light,
    Dark,
    Auto,
}

impl AppearancePanel {
    pub fn render(&self, surface: &WaylandSurface) {
        let mut y = 120;

        // Theme Mode
        y = self.draw_section(surface, y, "Theme Mode");
        y = self.draw_radio_group(surface, y, &["Light", "Dark", "Auto"], self.theme_mode);
        y += 24;

        // Accent Color
        y = self.draw_section(surface, y, "Accent Color");
        y = self.draw_color_options(surface, y, &[
            ("Blue", "#1A73E8"),
            ("Teal", "#009688"),
            ("Purple", "#7B1FA2"),
            ("Orange", "#FF6D00"),
        ]);
        y += 24;

        // Font Size
        y = self.draw_section(surface, y, "Font Size");
        y = self.draw_slider(surface, y, "Small", "Default", "Large", self.font_size);
        y += 24;

        // Icon Size
        y = self.draw_section(surface, y, "Icon Size");
        y = self.draw_radio_group(surface, y, &["Small", "Default", "Large"], self.icon_size);
    }

    pub fn apply(&self) {
        // Apply theme changes
        theme::set_mode(self.theme_mode);
        theme::set_accent_color(self.accent_color);
        theme::set_font_size(self.font_size);
        theme::set_icon_size(self.icon_size);
    }
}
```

### 2. Display Panel

```rust
pub struct DisplayPanel {
    // Settings
    brightness: u8,           // 0-100
    scale: f32,               // 1.0, 1.25, 1.5, 2.0
    refresh_rate: u32,        // 60, 120, 144
    night_light: bool,
    night_light_temperature: u16,  // 3000K - 6500K

    // Displays
    displays: Vec<DisplayInfo>,
}

impl DisplayPanel {
    pub fn render(&self, surface: &WaylandSurface) {
        let mut y = 120;

        // Brightness
        y = self.draw_section(surface, y, "Brightness");
        y = self.draw_slider(surface, y, "🔅", "", "☀️", self.brightness);
        y += 24;

        // Scale
        y = self.draw_section(surface, y, "Display Scale");
        y = self.draw_buttons(surface, y, &["100%", "125%", "150%", "200%"], self.scale);
        y += 24;

        // Refresh Rate
        y = self.draw_section(surface, y, "Refresh Rate");
        y = self.draw_radio_group(surface, y, &["60 Hz", "120 Hz", "144 Hz"], self.refresh_rate);
        y += 24;

        // Night Light
        y = self.draw_section(surface, y, "Night Light");
        y = self.draw_toggle(surface, y, "Enable", self.night_light);

        if self.night_light {
            y += 16;
            y = self.draw_slider(surface, y, "Warm", "", "Cool", self.night_light_temperature);
        }
    }

    pub fn apply(&self) {
        // Apply display settings
        display::set_brightness(self.brightness);
        display::set_scale(self.scale);
        display::set_refresh_rate(self.refresh_rate);
        display::set_night_light(self.night_light, self.night_light_temperature);
    }
}
```

### 3. Network Panel

```rust
pub struct NetworkPanel {
    // Available networks
    networks: Vec<Network>,

    // Connected network
    connected: Option<Network>,

    // WiFi enabled
    wifi_enabled: bool,

    // Hotspot
    hotspot_enabled: bool,
    hotspot_password: String,
}

pub struct Network {
    ssid: String,
    strength: u8,           // 0-100
    secured: bool,
}

impl NetworkPanel {
    pub fn render(&self, surface: &WaylandSurface) {
        let mut y = 120;

        // WiFi toggle
        y = self.draw_toggle(surface, y, "Wi-Fi", self.wifi_enabled);

        if !self.wifi_enabled {
            return;
        }

        y += 24;

        // Networks list
        y = self.draw_section(surface, y, "Networks");

        for network in &self.networks {
            y = self.draw_network_item(surface, y, network);
        }

        y += 24;

        // Hotspot
        y = self.draw_section(surface, y, "Hotspot");
        y = self.draw_toggle(surface, y, "Enable Hotspot", self.hotspot_enabled);
    }

    fn draw_network_item(&self, surface: &WaylandSurface, y: f32, network: &Network) -> f32 {
        let item_height = 64;

        // Background (if connected)
        if self.connected.as_ref().map(|n| &n.ssid == &network.ssid) {
            surface.fill_rect(220, y, 560, item_height, theme.colors.primary.with_alpha(0.2));
        }

        // WiFi icon
        let icon_rect = Rect { x: 240, y: y + 8, width: 48, height: 48 };
        surface.draw_icon(icon_rect, self.wifi_icon_for_strength(network.strength));

        // SSID
        let text_rect = Rect { x: 300, y: y + 16, width: 400, height: 20 };
        surface.draw_text(text_rect, &network.ssid);

        // Lock icon if secured
        if network.secured {
            let lock_rect = Rect { x: 500, y: y + 16, width: 24, height: 24 };
            surface.draw_icon(lock_rect, "lock");
        }

        // Connect button (if not connected)
        if self.connected.as_ref().map(|n| &n.ssid != &network.ssid) {
            let btn_rect = Rect { x: 580, y: y + 16, width: 80, height: 36 };
            surface.draw_button(btn_rect, "Connect");
        }

        y + item_height + 8
    }
}
```

## Simplified Code Structure

```
/var/www/rustux.com/prod/apps/gui/prototypes/rustica-settings/
├── Cargo.toml
└── src/
    ├── main.rs               # Entry point
    ├── app.rs                # Main app window
    ├── panels/
    │   ├── mod.rs
    │   ├── appearance.rs      # Theme settings
    │   ├── display.rs         # Display settings
    │   ├── network.rs         # Network settings
    │   └── home.rs            # Panel selection
    └── ui.rs                 # Simple UI rendering
```

## Stub Components

### Radio Group

```rust
fn draw_radio_group(surface: &WaylandSurface, y: f32, options: &[&str], selected: usize) -> f32 {
    let mut x = 240;
    let spacing = 150;

    for (i, option) in options.iter().enumerate() {
        // Radio circle
        surface.draw_circle(x, y + 10, 8, theme.colors.outline);

        // Selected indicator
        if i == selected {
            surface.fill_circle(x, y + 10, 4, theme.colors.primary);
        }

        // Label
        surface.draw_text(x + 20, y + 2, option);

        x += spacing;
    }

    y + 24
}
```

### Toggle Switch

```rust
fn draw_toggle(surface: &WaylandSurface, y: f32, label: &str, value: bool) -> f32 {
    // Label
    surface.draw_text(240, y, label);

    // Switch track
    let track_x = 400;
    let track_y = y + 6;
    let track_width = 48;
    let track_height = 24;

    surface.fill_rounded_rect(
        track_x, track_y,
        track_width, track_height,
        12,
        if value { theme.colors.primary } else { theme.colors.surface_variant }
    );

    // Switch thumb
    let thumb_x = if value { track_x + track_width - 20 } else { track_x + 4 };

    surface.fill_rounded_rect(
        thumb_x, track_y + 4,
        16, 16,
        8,
        theme.colors.on_primary
    );

    y + 36
}
```

### Slider

```rust
fn draw_slider(surface: &WaylandSurface, y: f32, min_label: &str, max_label: &str, value: f32) -> f32 {
    let slider_x = 240;
    let slider_y = y + 10;
    let slider_width = 320;

    // Min label
    surface.draw_text(slider_x, y, min_label);

    // Max label
    surface.draw_text(slider_x + slider_width, y, max_label);

    // Track
    surface.fill_rect(
        slider_x + 60,
        slider_y + 10,
        slider_width - 120,
        4,
        theme.colors.surface_variant
    );

    // Filled track
    let filled_width = (slider_width - 120) * value;
    surface.fill_rect(
        slider_x + 60,
        slider_y + 10,
        filled_width,
        4,
        theme.colors.primary
    );

    // Thumb
    surface.fill_circle(
        slider_x + 60 + filled_width,
        slider_y + 12,
        8,
        theme.colors.primary
    );

    y + 32
}
```

## Apply/Revert Behavior

```rust
impl SettingsPrototype {
    pub fn apply_changes(&mut self) {
        // Apply all pending changes
        for (key, change) in &self.pending_changes {
            match change {
                SettingChange::ThemeMode(mode) => {
                    theme::set_mode(*mode);
                }
                SettingChange::AccentColor(color) => {
                    theme::set_accent_color(*color);
                }
                // ... other changes
            }
        }

        // Save to config
        self.save_config();

        // Clear pending
        self.pending_changes.clear();

        // Update originals
        self.refresh_originals();
    }

    pub fn revert_changes(&mut self) {
        // Revert all changes
        for (key, original) in &self.original_values {
            match original {
                SettingValue::ThemeMode(mode) => {
                    theme::set_mode(*mode);
                }
                // ... other values
            }
        }

        // Clear pending
        self.pending_changes.clear();
    }

    pub fn has_pending_changes(&self) -> bool {
        !self.pending_changes.is_empty()
    }
}
```

## Navigation

```rust
impl SettingsPrototype {
    pub fn navigate_to_panel(&mut self, panel: Panel) {
        // Save current panel state
        // Navigate to new panel
        self.current_panel = panel;
    }

    pub fn go_back(&mut self) {
        // Navigate back to home
        self.current_panel = Panel::Home;
    }
}
```

## Testing Checklist

- [ ] Can navigate between panels
- [ ] Can apply settings changes
- [ ] Can revert settings changes
- [ ] Theme switching works instantly
- [ ] Accent color changes apply
- [ ] Display scale changes work
- [ ] WiFi network list shows
- [ ] Toggle switches work
- [ ] Sliders work

## Validation Questions

1. **Sidebar**: Is 200px sidebar appropriate? Too narrow/wide?
2. **Apply/Revert**: Should changes apply immediately or require confirmation?
3. **Theme**: Is theme switch instant or need restart?
4. **Organization**: Are panel categories clear and discoverable?
5. **Controls**: Are sliders/toggles/dropdowns intuitive?

## Deliverable

- Working prototype binary
- Screenshot of each panel
- List of UX issues discovered
- List of requested settings to add in full version

## Success Criteria

- [ ] 3-5 core panels functional
- [ ] Apply/revert works
- [ ] Theme changes visible immediately
- [ ] No crashes on normal operations
- [ ] Feedback gathered

## Sign-Off

**Prototype Developer**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅ (for prototype only)

**Note**: This prototype will be discarded. Full implementation in Phase 5.

---

## Timeline

- Day 1: Project structure + home panel
- Day 2: Appearance panel + theme switching
- Day 3: Display panel + DPI testing
- Day 4: Network panel + WiFi UI
- Day 5: Testing + feedback gathering

**Total**: 1 week
