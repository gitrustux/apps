# Panel Prototype Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Type**: Quick Prototype (UX Validation)
**Timeline**: 3 days (concurrent with shell development)

## Overview

This is the **simplest prototype** - a basic top panel to validate visual design and widget spacing. It is **not production code** and will be replaced with the full implementation in Phase 3.

## Prototype Goals

- Validate 48px panel height
- Test widget spacing and touch targets
- Validate indicator hover states
- Test clock and status display
- Get feedback on visual density
- **DO NOT** implement functionality (just display)

## Minimal Feature Set

### What to Build

```
┌──────────────────────────────────────────────────────────────────────┐
│  [⚙️]  [Firefox]  [Terminal]              [🌐] [🔊] [🔋] [📧]   12:34   │
│   App    Running Apps                Network Audio Battery Email    Time   │
└──────────────────────────────────────────────────────────────────────┘
   48px height
```

**Features:**
- ✓ App launcher button (left)
- ✓ Running apps indicator (center)
- ✓ Network status icon (right)
- ✓ Audio status icon (right)
- ✓ Battery status icon (right)
- ✓ Clock (right)

**What it DOESN'T need:**
- ✗ Working buttons (just visual)
- ✗ Real data (hardcode values)
- ✗ Tooltips (maybe if time permits)
- ✗ Context menus
- ✗ Notifications

## Hardcoded UI

```rust
pub struct PanelPrototype {
    // Panel dimensions
    width: u32,
    height: u32,  // 48px

    // Widgets (just for display)
    app_icon: Icon,
    running_apps: Vec<Icon>,
    network_icon: Icon,
    audio_icon: Icon,
    battery_icon: Icon,
    time: String,
}

impl PanelPrototype {
    pub fn new() -> Self {
        Self {
            width: 1920,
            height: 48,
            app_icon: Icon::from_name("rustica-launcher"),
            running_apps: vec![
                Icon::from_name("firefox"),
                Icon::from_name("terminal"),
            ],
            network_icon: Icon::from_name("network-wireless"),
            audio_icon: Icon::from_name("audio-volume-high"),
            battery_icon: Icon::from_name("battery-good"),
            time: "12:34".into(),
        }
    }

    pub fn render(&self, surface: &WaylandSurface) {
        // Background
        surface.fill_rect(0, 0, self.width, self.height, theme.colors.surface);

        // App launcher (left)
        let app_icon_rect = Rect { x: 24, y: 12, width: 24, height: 24 };
        surface.draw_icon(app_icon_rect, &self.app_icon);

        // Running apps (center)
        let mut x = 80;
        for app_icon in &self.running_apps {
            let app_rect = Rect { x, y: 12, width: 24, height: 24 };
            surface.draw_icon(app_rect, app_icon);
            x += 48;
        }

        // Status icons (right)
        let mut x = self.width - 200;

        let network_rect = Rect { x, y: 12, width: 24, height: 24 };
        surface.draw_icon(network_rect, &self.network_icon);
        x += 40;

        let audio_rect = Rect { x, y: 12, width: 24, height: 24 };
        surface.draw_icon(audio_rect, &self.audio_icon);
        x += 40;

        let battery_rect = Rect { x, y: 12, width: 24, height: 24 };
        surface.draw_icon(battery_rect, &self.battery_icon);
        x += 40;

        // Clock (rightmost)
        let clock_rect = Rect { x, y: 14, width: 80, height: 20 };
        surface.draw_text(clock_rect, &self.time);
    }

    pub fn update_time(&mut self) {
        let now = Local::now();
        self.time = now.format("%H:%M").to_string();
    }
}
```

## Layer Shell Integration

```rust
// Use Wayland layer-shell protocol to position panel at top of screen

fn main() {
    // Connect to Wayland compositor
    let display = wayland_client::Display::connect().unwrap();

    // Get layer shell interface
    let compositor = display.get_registry()
        .bind::<LayerShell>(&display)
        .unwrap();

    // Create layer surface
    let surface = compositor.create_layer_surface(
        &display,
        Layer::Top,
        Some("rustica-panel"),
    );

    // Set panel size
    surface.set_size(1920, 48);
    surface.set_anchor(Anchor::Top | Anchor::Left | Anchor::Right);

    // Commit surface
    surface.commit();

    // Render loop
    let mut panel = PanelPrototype::new();

    loop {
        // Update time
        panel.update_time();

        // Render
        panel.render(&surface);

        // Commit
        surface.commit();

        // Wait for event
        display.dispatch().unwrap();
    }
}
```

## Simulated Interactions

```rust
impl PanelPrototype {
    // Simulate hover states (just for visual testing)

    pub fn set_hover_app_launcher(&mut self, hover: bool) {
        if hover {
            // Draw light background
            self.hover_app_launcher = true;
        }
    }

    pub fn set_hover_network(&mut self, hover: bool) {
        self.hover_network = hover;

        if hover {
            // Show tooltip
            self.tooltip = Some("Connected: WiFi".into());
        }
    }

    pub fn render_with_hover(&self, surface: &WaylandSurface) {
        // Render normal panel
        self.render(surface);

        // Draw hover effects
        if self.hover_app_launcher {
            surface.fill_rounded_rect(16, 4, 48, 40, 8, theme.colors.surface_variant);
        }

        // Draw tooltips
        if let Some(ref tooltip) = self.tooltip {
            surface.draw_tooltip(100, 56, tooltip);
        }
    }
}
```

## Minimal Dependencies

```toml
[package]
name = "rustica-panel-prototype"
version = "0.1.0"
edition = "2021"

[dependencies]
# Wayland client
wayland-client = "0.31"
wayland-protocols = { version = "0.31", features = ["unstable_protocols"] }

# Simple rendering
softbuffer = "0.3"

# Icons (just load from filesystem)

# Time
chrono = "0.4"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

## Code Structure

```
/var/www/rustux.com/prod/apps/gui/prototypes/rustica-panel/
├── Cargo.toml
└── src/
    ├── main.rs           # Entry point + layer shell setup
    ├── panel.rs          # Panel rendering
    └── widgets.rs        # Widget drawing (icons, text)
```

## Touch Target Validation

The main validation goal is testing if **48px height** and **widget spacing** is adequate for touch:

```rust
// Validate touch targets

fn validate_touch_targets(panel: &PanelPrototype) {
    // All interactive elements must be ≥44×44px

    assert!(panel.app_launcher_target().width >= 44);
    assert!(panel.app_launcher_target().height >= 44);

    for widget in panel.widgets() {
        assert!(widget.touch_target().width >= 44);
        assert!(widget.touch_target().height >= 44);
    }
}
```

## Visual Validation Checklist

- [ ] Panel height (48px) looks appropriate
- [ ] App launcher icon is visible
- [ ] Running apps are clearly distinguishable
- [ ] Status icons are recognizable
- [ ] Clock is readable
- [ ] Spacing between widgets is balanced
- [ ] Background doesn't clash with wallpapers
- [ ] Panel doesn't feel too crowded
- [ ] Panel doesn't feel too sparse

## Testing Questions

1. **Height**: Is 48px sufficient for touch targets?
2. **Density**: Does panel feel crowded or sparse?
3. **Icon size**: Are 24px icons at 48px scale appropriate?
4. **Spacing**: Is 8px between widgets enough?
5. **Visual hierarchy**: Can users distinguish app launcher from running apps?

## Deliverable

- Working panel prototype (runs on Wayland)
- Screenshot of panel with different states
- List of visual issues discovered
- Validation results for touch targets

## Success Criteria

- [ ] Panel appears at top of screen
- [ ] All icons render correctly
- [ ] Clock updates every minute
- [ ] Hover states work (if implemented)
- [ ] Visual feedback gathered

## Sign-Off

**Prototype Developer**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅ (for prototype only)

**Note**: This prototype will be discarded. Full implementation in Phase 3.

---

## Timeline

- Day 1: Project structure + layer shell setup
- Day 2: Widget rendering + icons
- Day 3: Clock update + hover states + screenshots

**Total**: 3 days
