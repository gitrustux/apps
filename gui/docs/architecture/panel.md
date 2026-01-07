# Panel (rustica-panel) Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Desktop Panel

## Overview

This specification defines the top panel for Rustica Shell, providing **system status indicators**, **application launcher**, **running applications list**, and **quick settings**. It ensures **<100ms state updates** and **44px minimum touch targets**.

## Architecture

```
┌────────────────────────────────────────────────────────────────────┐
│                        Top Panel (48px height)                     │
├──────┬───────────────────────┬───────────────────────┬───────────┤
│      │                       │                       │           │
│ App  │   Running Apps        │   System Indicators   │  Clock &   │
│ Icon │   (Taskbar)           │                       │  Status    │
│      │                       │                       │           │
└──────┴───────────────────────┴───────────────────────┴───────────┘
│◄────20%────────►│◄────────60%─────────────────────►│◄──20%─────►│
```

## Panel Layout

### Widget Organization

```rust
pub struct PanelLayout {
    /// Left side widgets
    pub left: Vec<Box<dyn Widget>>,

    /// Center widgets
    pub center: Vec<Box<dyn Widget>>,

    /// Right side widgets
    pub right: Vec<Box<dyn Widget>>,
}

impl PanelLayout {
    pub fn new() -> Self {
        let mut left = Vec::new();
        let mut center = Vec::new();
        let mut right = Vec::new();

        // Left: App launcher
        left.push(Box::new(AppLauncherButton::new()));

        // Center: Running applications
        center.push(Box::new(Taskbar::new()));

        // Right: System indicators
        right.push(Box::new(NetworkIndicator::new()));
        right.push(Box::new(AudioIndicator::new()));
        right.push(Box::new(BatteryIndicator::new()));
        right.push(Box::new(NotificationIndicator::new()));
        right.push(Box::new(ClockWidget::new()));

        Self { left, center, right }
    }
}
```

### Panel Configuration

```toml
# ~/.config/rustica/shell/panels/top-panel.toml

schema_version = "1.0.0"

# Panel position: "top" | "bottom" | "left" | "right"
position = "top"

# Panel height
height = 48

# Auto-hide when not in use
auto_hide = false

# Show on all workspaces
show_on_all_workspaces = true

# Widget spacing
widget_spacing = 8

# Widget padding
widget_padding = 4

# Background color (from theme)
background = "surface"

# Border radius
border_radius = 0

# Shadow
shadow = false
```

## Core Widgets

### App Launcher Button

```rust
use librustica::widgets::*;

pub struct AppLauncherButton {
    icon: Icon,
    tooltip: String,
    on_click: Box<dyn Fn()>,
}

impl AppLauncherButton {
    pub fn new() -> Self {
        Self {
            icon: Icon::from_name("rustica-launcher"),
            tooltip: "Applications".into(),
            on_click: Box::new(|| {
                // Open app launcher
                Launcher::show();
            }),
        }
    }
}

impl Widget for AppLauncherButton {
    fn render(&mut self, ctx: &mut RenderContext) {
        // Draw button background (invisible unless hovered)
        if ctx.is_hovered() {
            ctx.fill_rect(ctx.rect(), theme.colors.surface_variant);
        }

        // Draw icon (24px)
        let icon_rect = Rect {
            x: ctx.rect().x + (ctx.rect().width - 24) / 2,
            y: ctx.rect().y + (ctx.rect().height - 24) / 2,
            width: 24,
            height: 24,
        };
        ctx.draw_icon(icon_rect, &self.icon);

        // Draw tooltip on hover
        if ctx.is_hovered() {
            ctx.show_tooltip(self.tooltip.clone());
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::PointerButton { button: PointerButton::Left, state: ButtonState::Pressed } => {
                (self.on_click)();
                EventResult::Handled
            }
            _ => EventResult::NotHandled,
        }
    }

    fn size(&self) -> Size {
        Size {
            width: 48,   // Minimum touch target
            height: 48,
        }
    }
}
```

### Taskbar (Running Apps)

```rust
pub struct Taskbar {
    apps: Vec<TaskbarApp>,
    scroll_offset: usize,
}

pub struct TaskbarApp {
    app_id: String,
    window_ids: Vec<WindowId>,
    icon: Icon,
    title: String,
    focused: bool,
}

impl Taskbar {
    pub fn new() -> Self {
        Self {
            apps: Vec::new(),
            scroll_offset: 0,
        }
    }

    /// Add window to taskbar
    pub fn add_window(&mut self, window_id: WindowId, app_id: String, title: String) {
        // Check if app already exists
        if let Some(app) = self.apps.iter_mut().find(|a| a.app_id == app_id) {
            app.window_ids.push(window_id);
            app.title = title;
        } else {
            // Create new taskbar entry
            let icon = Icon::from_app_id(&app_id);
            self.apps.push(TaskbarApp {
                app_id,
                window_ids: vec![window_id],
                icon,
                title,
                focused: false,
            });
        }
    }

    /// Remove window from taskbar
    pub fn remove_window(&mut self, window_id: WindowId) {
        for app in &mut self.apps {
            app.window_ids.retain(|id| *id != window_id);
        }

        // Remove apps with no windows
        self.apps.retain(|app| !app.window_ids.is_empty());
    }

    /// Set focused app
    pub fn set_focused(&mut self, window_id: WindowId) {
        for app in &mut self.apps {
            app.focused = app.window_ids.contains(&window_id);
        }
    }

    /// Focus window
    pub fn focus_window(&self, window_id: WindowId) {
        compositor::focus_window(window_id);
    }
}

impl Widget for Taskbar {
    fn render(&mut self, ctx: &mut RenderContext) {
        let mut x = ctx.rect().x;

        // Render visible apps
        for app in self.apps.iter().skip(self.scroll_offset) {
            let app_width = 48;  // Fixed width for touch targets

            // Check if still visible
            if x + app_width > ctx.rect().x + ctx.rect().width {
                break;
            }

            let app_rect = Rect {
                x,
                y: ctx.rect().y,
                width: app_width,
                height: ctx.rect().height,
            };

            // Draw background
            if app.focused {
                ctx.fill_rect(app_rect, theme.colors.primary);
            } else if ctx.is_hovered_rect(app_rect) {
                ctx.fill_rect(app_rect, theme.colors.surface_variant);
            }

            // Draw icon
            let icon_rect = Rect {
                x: app_rect.x + (app_width - 24) / 2,
                y: app_rect.y + (app_rect.height - 24) / 2,
                width: 24,
                height: 24,
            };
            ctx.draw_icon(icon_rect, &app.icon);

            // Draw dot indicator for multiple windows
            if app.window_ids.len() > 1 {
                let dot_rect = Rect {
                    x: app_rect.x + app_width - 8,
                    y: app_rect.y + app_rect.height - 8,
                    width: 4,
                    height: 4,
                };
                ctx.fill_rect(dot_rect, theme.colors.primary);
            }

            x += app_width;
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::PointerButton { button: PointerButton::Left, state: ButtonState::Pressed } => {
                // Find app under pointer
                let x = event.pointer_position().x;
                let app_index = (x / 48) as usize + self.scroll_offset;

                if let Some(app) = self.apps.get(app_index) {
                    // Focus first window
                    if let Some(window_id) = app.window_ids.first() {
                        self.focus_window(*window_id);
                        return EventResult::Handled;
                    }
                }

                EventResult::NotHandled
            }
            Event::PointerButton { button: PointerButton::Right, state: ButtonState::Pressed } => {
                // Show context menu
                self.show_context_menu();
                EventResult::Handled
            }
            _ => EventResult::NotHandled,
        }
    }

    fn size(&self) -> Size {
        Size {
            width: self.apps.len() as f32 * 48.0,
            height: 48.0,
        }
    }
}
```

### System Indicators

#### Network Indicator

```rust
pub struct NetworkIndicator {
    icon: Icon,
    status: NetworkStatus,
}

pub enum NetworkStatus {
    Disconnected,
    Ethernet,
    Wifi { ssid: String, strength: u8 },
    Vpn { ssid: String },
}

impl NetworkIndicator {
    pub fn new() -> Self {
        Self {
            icon: Icon::from_name("network-offline"),
            status: NetworkStatus::Disconnected,
        }
    }

    pub fn update_status(&mut self, status: NetworkStatus) {
        self.status = status;
        self.icon = match &self.status {
            NetworkStatus::Disconnected => Icon::from_name("network-offline"),
            NetworkStatus::Ethernet => Icon::from_name("network-wired"),
            NetworkStatus::Wifi { strength, .. } => {
                match strength {
                    0..=25 => Icon::from_name("network-wireless-signal-weak"),
                    26..=50 => Icon::from_name("network-wireless-signal-ok"),
                    51..=75 => Icon::from_name("network-wireless-signal-good"),
                    76..=100 => Icon::from_name("network-wireless-signal-excellent"),
                    _ => Icon::from_name("network-wireless"),
                }
            }
            NetworkStatus::Vpn { .. } => Icon::from_name("network-vpn"),
        };
    }
}

impl Widget for NetworkIndicator {
    fn render(&mut self, ctx: &mut RenderContext) {
        let icon_rect = Rect {
            x: ctx.rect().x + (ctx.rect().width - 24) / 2,
            y: ctx.rect().y + (ctx.rect().height - 24) / 2,
            width: 24,
            height: 24,
        };

        ctx.draw_icon(icon_rect, &self.icon);

        if ctx.is_hovered() {
            let tooltip = match &self.status {
                NetworkStatus::Disconnected => "Not connected".into(),
                NetworkStatus::Ethernet => "Wired connection".into(),
                NetworkStatus::Wifi { ssid, strength } => {
                    format!("{} ({}%)", ssid, strength)
                }
                NetworkStatus::Vpn { ssid } => {
                    format!("VPN: {}", ssid)
                }
            };
            ctx.show_tooltip(tooltip);
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::PointerButton { button: PointerButton::Left, state: ButtonState::Pressed } => {
                // Open network settings
                Settings::open_network();
                EventResult::Handled
            }
            _ => EventResult::NotHandled,
        }
    }

    fn size(&self) -> Size {
        Size { width: 44, height: 48 }
    }
}
```

#### Audio Indicator

```rust
pub struct AudioIndicator {
    icon: Icon,
    volume: u8,  // 0-100
    muted: bool,
}

impl AudioIndicator {
    pub fn new() -> Self {
        Self {
            icon: Icon::from_name("audio-volume-high"),
            volume: 50,
            muted: false,
        }
    }

    pub fn set_volume(&mut self, volume: u8) {
        self.volume = volume.min(100);
        self.update_icon();
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        self.update_icon();
    }

    fn update_icon(&mut self) {
        self.icon = if self.muted {
            Icon::from_name("audio-volume-muted")
        } else if self.volume == 0 {
            Icon::from_name("audio-volume-low")
        } else if self.volume < 33 {
            Icon::from_name("audio-volume-low")
        } else if self.volume < 66 {
            Icon::from_name("audio-volume-medium")
        } else {
            Icon::from_name("audio-volume-high")
        };
    }
}

impl Widget for AudioIndicator {
    fn render(&mut self, ctx: &mut RenderContext) {
        let icon_rect = Rect {
            x: ctx.rect().x + (ctx.rect().width - 24) / 2,
            y: ctx.rect().y + (ctx.rect().height - 24) / 2,
            width: 24,
            height: 24,
        };

        ctx.draw_icon(icon_rect, &self.icon);

        if ctx.is_hovered() {
            let tooltip = if self.muted {
                "Muted".into()
            } else {
                format!("Volume: {}%", self.volume)
            };
            ctx.show_tooltip(tooltip);
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::PointerButton { button: PointerButton::Left, state: ButtonState::Pressed } => {
                // Toggle mute
                self.muted = !self.muted;
                self.update_icon();
                audio::set_muted(self.muted);
                EventResult::Handled
            }
            Event::PointerAxis { axis: Axis::Vertical, value } => {
                // Adjust volume with scroll
                let delta = (*value * 5) as i8;
                let new_volume = (self.volume as i8 + delta).clamp(0, 100) as u8;
                self.set_volume(new_volume);
                audio::set_volume(new_volume);
                EventResult::Handled
            }
            _ => EventResult::NotHandled,
        }
    }

    fn size(&self) -> Size {
        Size { width: 44, height: 48 }
    }
}
```

#### Battery Indicator

```rust
pub struct BatteryIndicator {
    icon: Icon,
    percentage: u8,
    charging: bool,
}

impl BatteryIndicator {
    pub fn new() -> Self {
        Self {
            icon: Icon::from_name("battery-full"),
            percentage: 100,
            charging: false,
        }
    }

    pub fn update(&mut self, percentage: u8, charging: bool) {
        self.percentage = percentage;
        self.charging = charging;
        self.update_icon();
    }

    fn update_icon(&mut self) {
        self.icon = if self.charging {
            Icon::from_name("battery-charging")
        } else if self.percentage > 90 {
            Icon::from_name("battery-full")
        } else if self.percentage > 70 {
            Icon::from_name("battery-good")
        } else if self.percentage > 40 {
            Icon::from_name("battery-medium")
        } else if self.percentage > 20 {
            Icon::from_name("battery-low")
        } else {
            Icon::from_name("battery-critical")
        };
    }
}

impl Widget for BatteryIndicator {
    fn render(&mut self, ctx: &mut RenderContext) {
        let icon_rect = Rect {
            x: ctx.rect().x + (ctx.rect().width - 24) / 2,
            y: ctx.rect().y + (ctx.rect().height - 24) / 2,
            width: 24,
            height: 24,
        };

        ctx.draw_icon(icon_rect, &self.icon);

        if ctx.is_hovered() {
            let tooltip = if self.charging {
                format!("Charging: {}%", self.percentage)
            } else {
                format!("Battery: {}%", self.percentage)
            };
            ctx.show_tooltip(tooltip);
        }

        // Warn on low battery
        if self.percentage < 20 && !self.charging {
            ctx.show_notification(Notification {
                title: "Low Battery".into(),
                body: format!("Battery at {}%", self.percentage),
                urgency: Urgency::Warning,
            });
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::PointerButton { button: PointerButton::Left, state: ButtonState::Pressed } => {
                // Open power settings
                Settings::open_power();
                EventResult::Handled
            }
            _ => EventResult::NotHandled,
        }
    }

    fn size(&self) -> Size {
        Size { width: 44, height: 48 }
    }
}
```

#### Clock Widget

```rust
pub struct ClockWidget {
    format: ClockFormat,
    show_date: bool,
}

pub enum ClockFormat {
    Hour12,
    Hour24,
}

impl ClockWidget {
    pub fn new() -> Self {
        Self {
            format: ClockFormat::Hour24,
            show_date: false,
        }
    }

    fn get_time_text(&self) -> String {
        let now = Local::now();

        match self.format {
            ClockFormat::Hour12 => {
                if self.show_date {
                    now.format("%a %b %d %I:%M %P").to_string()
                } else {
                    now.format("%I:%M %P").to_string()
                }
            }
            ClockFormat::Hour24 => {
                if self.show_date {
                    now.format("%a %b %d %H:%M").to_string()
                } else {
                    now.format("%H:%M").to_string()
                }
            }
        }
    }
}

impl Widget for ClockWidget {
    fn render(&mut self, ctx: &mut RenderContext) {
        let text = self.get_time_text();

        ctx.draw_text(
            ctx.rect(),
            &text,
            theme.typography.body,
            theme.colors.on_primary,
            TextAlignment::Center,
        );

        if ctx.is_hovered() {
            let tooltip = now().format("%A, %B %d, %Y").to_string();
            ctx.show_tooltip(tooltip);
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::PointerButton { button: PointerButton::Left, state: ButtonState::Pressed } => {
                // Toggle date display
                self.show_date = !self.show_date;
                EventResult::Handled
            }
            Event::PointerButton { button: PointerButton::Right, state: ButtonState::Pressed } => {
                // Open calendar
                Calendar::show();
                EventResult::Handled
            }
            _ => EventResult::NotHandled,
        }
    }

    fn size(&self) -> Size {
        // Calculate based on text
        let text = self.get_time_text();
        let metrics = FontMetrics::new(theme.typography.body);
        Size {
            width: metrics.measure_text(&text) + 16,  // padding
            height: 48,
        }
    }
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-panel/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── panel.rs
│   │   ├── widgets/
│   │   │   ├── mod.rs
│   │   │   ├── launcher.rs
│   │   │   ├── taskbar.rs
│   │   │   ├── indicators/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── network.rs
│   │   │   │   ├── audio.rs
│   │   │   │   ├── battery.rs
│   │   │   │   └── clock.rs
│   │   └── config.rs
│   └── resources/
│       └── icons/
└── libs/librustica/
    └── widgets/
        └── panel/
            └── src/
                └── lib.rs
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| State update | <100ms | Change to visible |
| Render | <16ms | Per frame |
| Memory | <20MB | Total panel usage |
| Startup | <500ms | Launch to visible |

## Success Criteria

- [ ] All widgets render correctly
- [ ] Touch targets ≥44×44px
- [ ] State updates in <100ms
- [ ] Keyboard navigation works
- [ ] Tooltips show correctly
- [ ] Context menus work
- [ ] Accessibility support
- [ ] Performance targets met

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Too many indicators | Show only critical, hide others in menu |
| Performance with many apps | Limit visible apps, scrollable taskbar |
| Touch targets too small | Enforce 44px minimum in layout engine |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [GNOME Top Bar](https://developer.gnome.org/hig/patterns/controls/top-bar.html)
- [KDE Panel](https://docs.kde.org/stable5/en/kde-workspace/plasma-desktop/)
- [Cosmic Panel](https://github.com/pop-os/cosmic-panel)
