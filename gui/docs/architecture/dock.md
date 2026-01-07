# Dock (rustica-dock) Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Application Dock

## Overview

This specification defines the application dock for Rustica Shell, providing **quick access to pinned apps**, **running app indicators**, and **drag-and-drop reordering**. It ensures **smooth 60 FPS animations** and **intuitive touch interactions**.

## Architecture

```
┌────────────────────────────────────────────────────────────────────┐
│                            Dock                                   │
│  ┌────┐  ┌────┐  ┌────┐  ┌────┐  ┌────┐  ┌────┐  ┌────┐        │
│  │ 🏠 │  │ 📁 │  │ 🌐 │  │ 📝 │  │ ⚙️ │  │ 🎵 │  │ 📧 │        │
│  │    │  │ •  │  │    │  │ •• │  │    │  │    │  │    │        │
│  └────┘  └────┘  └────┘  └────┘  └────┘  └────┘  └────┘        │
│   Files   Browser  Terminal  Editor  Settings Music    Mail      │
└────────────────────────────────────────────────────────────────────┘
                    ◄───── 64px icon + 8px gap ─────►
```

## Dock Layout

```rust
pub struct Dock {
    /// Dock position
    position: DockPosition,

    /// Dock items (pinned + running)
    items: Vec<DockItem>,

    /// Icon size
    icon_size: f32,

    /// Item spacing
    item_spacing: f32,

    /// Auto-hide
    auto_hide: bool,

    /// Hide state
    hidden: bool,

    /// Animation progress
    animation_progress: f32,

    /// Drag state
    drag_state: Option<DragState>,
}

pub enum DockPosition {
    Left,
    Right,
    Bottom,
}

pub struct DockItem {
    /// App ID
    app_id: String,

    /// Icon
    icon: Icon,

    /// Running windows
    windows: Vec<WindowId>,

    /// Is pinned
    pinned: bool,

    /// Active (focused)
    active: bool,

    /// Hovered
    hovered: bool,

    /// Running indicator dots
    show_dots: bool,
}

pub struct DragState {
    item_index: usize,
    start_position: Point,
    current_position: Point,
}
```

## Dock Configuration

```toml
# ~/.config/rustica/shell/dock.toml

schema_version = "1.0.0"

# Dock position: "left" | "right" | "bottom"
position = "left"

# Icon size (px)
icon_size = 64

# Item spacing
item_spacing = 8

# Auto-hide when not in use
auto_hide = false

# Hide delay (seconds)
hide_delay = 1.0

# Show on all workspaces
show_on_all_workspaces = true

# Pinned apps
[[pinned_apps]]
app_id = "org.gnome.Nautilus"
name = "Files"

[[pinned_apps]]
app_id = "org.mozilla.firefox"
name = "Firefox"

[[pinned_apps]]
app_id = "rustica-terminal"
name = "Terminal"

# Behavior
[behavior]
# Show tooltip on hover
show_tooltips = true

# Show running indicators
show_indicators = true

# Allow drag-and-drop reordering
allow_reorder = true

# Click behavior: "focus" | "focus_or_open" | "launch"
click_behavior = "focus_or_open"
```

## Dock Rendering

```rust
impl Dock {
    pub fn render(&mut self, ctx: &mut RenderContext) {
        // Skip if hidden
        if self.hidden && self.animation_progress == 0.0 {
            return;
        }

        // Apply animation
        let offset = self.calculate_animation_offset();

        // Render dock background
        self.render_background(ctx, offset);

        // Render dock items
        self.render_items(ctx, offset);

        // Render drag preview
        if let Some(ref drag) = self.drag_state {
            self.render_drag_preview(ctx, drag);
        }
    }

    fn render_background(&self, ctx: &mut RenderContext, offset: f32) {
        let bg_rect = match self.position {
            DockPosition::Left => Rect {
                x: 0,
                y: offset,
                width: 80,  // 64px icon + 16px padding
                height: self.calculate_height(),
            },
            DockPosition::Right => Rect {
                x: ctx.output_width() - 80,
                y: offset,
                width: 80,
                height: self.calculate_height(),
            },
            DockPosition::Bottom => Rect {
                x: 0,
                y: ctx.output_height() - 80,
                width: ctx.output_width(),
                height: 80,
            },
        };

        // Rounded corners
        let radius = match self.position {
            DockPosition::Left => 8.0,
            DockPosition::Right => 8.0,
            DockPosition::Bottom => 12.0,
        };

        // Background with blur
        ctx.draw_rounded_rect(
            bg_rect,
            radius,
            theme.colors.surface.with_alpha(0.9),
        );
        ctx.apply_blur(bg_rect, 20);

        // Border
        ctx.draw_rounded_border(
            bg_rect,
            radius,
            1,
            theme.colors.outline,
        );
    }

    fn render_items(&self, ctx: &mut RenderContext, offset: f32) {
        let mut y = offset + 16;  // Top padding

        for (index, item) in self.items.iter().enumerate() {
            // Skip dragged item
            if let Some(ref drag) = self.drag_state {
                if drag.item_index == index {
                    continue;
                }
            }

            let item_rect = Rect {
                x: match self.position {
                    DockPosition::Left => 8,
                    DockPosition::Right => 8,
                    DockPosition::Bottom => y,
                },
                y: match self.position {
                    DockPosition::Left => y,
                    DockPosition::Right => y,
                    DockPosition::Bottom => 8,
                },
                width: self.icon_size,
                height: self.icon_size,
            };

            // Render item
            self.render_item(ctx, item, item_rect);

            y += self.icon_size + self.item_spacing;
        }
    }

    fn render_item(&self, ctx: &mut RenderContext, item: &DockItem, rect: Rect) {
        // Draw hover/active background
        if item.active {
            let bg_rect = rect.inset(-4);
            ctx.draw_rounded_rect(
                bg_rect,
                8,
                theme.colors.primary.with_alpha(0.3),
            );
        } else if item.hovered {
            let bg_rect = rect.inset(-4);
            ctx.draw_rounded_rect(
                bg_rect,
                8,
                theme.colors.surface_variant,
            );
        }

        // Draw icon
        ctx.draw_icon(rect, &item.icon);

        // Draw running indicators
        if item.windows.len() > 0 {
            self.render_indicators(ctx, item, rect);
        }

        // Draw tooltip
        if item.hovered {
            let tooltip = if item.windows.len() > 0 {
                format!("{} - {} windows", item.app_id, item.windows.len())
            } else {
                item.app_id.clone()
            };
            ctx.show_tooltip(tooltip);
        }
    }

    fn render_indicators(&self, ctx: &mut RenderContext, item: &DockItem, icon_rect: Rect) {
        let dot_size = 4;
        let dot_spacing = 6;

        // Calculate width of all dots
        let total_width = (item.windows.len() - 1) as f32 * dot_spacing + item.windows.len() as f32 * dot_size;
        let start_x = icon_rect.x + (icon_rect.width - total_width) / 2;
        let y = icon_rect.y + icon_rect.height + 8;

        for (i, _window) in item.windows.iter().enumerate() {
            let dot_rect = Rect {
                x: start_x + i as f32 * (dot_size + dot_spacing),
                y,
                width: dot_size,
                height: dot_size,
            };

            ctx.fill_ellipse(dot_rect, theme.colors.primary);
        }
    }

    fn render_drag_preview(&self, ctx: &mut RenderContext, drag: &DragState) {
        let item = &self.items[drag.item_index];

        let rect = Rect {
            x: drag.current_position.x - self.icon_size / 2,
            y: drag.current_position.y - self.icon_size / 2,
            width: self.icon_size,
            height: self.icon_size,
        };

        // Semi-transparent background
        ctx.draw_rounded_rect(
            rect,
            12,
            theme.colors.surface.with_alpha(0.8),
        );

        // Icon
        ctx.draw_icon(rect, &item.icon);

        // Shadow
        ctx.draw_shadow(rect, theme.shadows.lg);
    }
}
```

## Dock Interactions

### Click Handling

```rust
impl Dock {
    pub fn handle_click(&mut self, position: Point) -> bool {
        let item_index = self.find_item_at(position);

        if let Some(index) = item_index {
            let item = &self.items[index];

            if item.windows.is_empty() {
                // Launch app
                self.launch_app(&item.app_id);
            } else {
                // Focus or minimize
                if item.active {
                    // Minimize all windows
                    for window_id in &item.windows {
                        compositor::minimize_window(*window_id);
                    }
                } else {
                    // Focus first window
                    if let Some(window_id) = item.windows.first() {
                        compositor::focus_window(*window_id);
                    }
                }
            }

            return true;
        }

        false
    }

    pub fn handle_right_click(&mut self, position: Point) -> bool {
        let item_index = self.find_item_at(position);

        if let Some(index) = item_index {
            let item = &self.items[index];
            self.show_context_menu(item, position);
            return true;
        }

        false
    }

    fn find_item_at(&self, position: Point) -> Option<usize> {
        let mut y = 16.0;

        for (index, _) in self.items.iter().enumerate() {
            let item_rect = Rect {
                x: 8.0,
                y,
                width: self.icon_size,
                height: self.icon_size,
            };

            if item_rect.contains(position) {
                return Some(index);
            }

            y += self.icon_size + self.item_spacing;
        }

        None
    }
}
```

### Drag-and-Drop

```rust
impl Dock {
    pub fn handle_drag_start(&mut self, position: Point) -> bool {
        let item_index = self.find_item_at(position);

        if let Some(index) = item_index {
            self.drag_state = Some(DragState {
                item_index: index,
                start_position: position,
                current_position: position,
            });
            return true;
        }

        false
    }

    pub fn handle_drag_move(&mut self, position: Point) {
        if let Some(ref mut drag) = self.drag_state {
            drag.current_position = position;

            // Check if should swap with nearby item
            let target_index = self.find_item_at(position);

            if let Some(target) = target_index {
                if target != drag.item_index {
                    self.swap_items(drag.item_index, target);
                    drag.item_index = target;
                }
            }
        }
    }

    pub fn handle_drag_end(&mut self) {
        if self.drag_state.is_some() {
            // Save new order to config
            self.save_config();
            self.drag_state = None;
        }
    }

    fn swap_items(&mut self, from: usize, to: usize) {
        self.items.swap(from, to);
    }
}
```

### Hover Effects

```rust
impl Dock {
    pub fn handle_hover(&mut self, position: Point) {
        // Clear all hovers
        for item in &mut self.items {
            item.hovered = false;
        }

        // Set hover for item under cursor
        let item_index = self.find_item_at(position);

        if let Some(index) = item_index {
            self.items[index].hovered = true;

            // Start animation
            if !self.items[index].hovered {
                self.start_hover_animation(index);
            }
        }
    }

    fn start_hover_animation(&mut self, item_index: usize) {
        // Scale icon on hover
        let item = &mut self.items[item_index];
        // Animation: 1.0 → 1.2 over 150ms
    }
}
```

## Auto-Hide

```rust
impl Dock {
    pub fn update(&mut self) {
        // Check if should hide
        if self.auto_hide && !self.is_hovered() {
            self.start_hide_animation();
        } else {
            self.start_show_animation();
        }
    }

    fn is_hovered(&self) -> bool {
        self.items.iter().any(|i| i.hovered)
    }

    fn start_hide_animation(&mut self) {
        // Animate from 1.0 to 0.0 over 300ms
        if !self.hidden {
            self.animation_progress -= 0.05;  // 60 FPS → 300ms

            if self.animation_progress <= 0.0 {
                self.animation_progress = 0.0;
                self.hidden = true;
            }
        }
    }

    fn start_show_animation(&mut self) {
        // Animate from 0.0 to 1.0 over 200ms
        if self.hidden {
            self.hidden = false;
        }

        self.animation_progress += 0.083;  // 60 FPS → 200ms

        if self.animation_progress >= 1.0 {
            self.animation_progress = 1.0;
        }
    }

    fn calculate_animation_offset(&self) -> f32 {
        match self.position {
            DockPosition::Left | DockPosition::Right => {
                // Slide in from side
                let dock_width = 80.0;
                dock_width * (1.0 - self.animation_progress)
            }
            DockPosition::Bottom => {
                // Slide up from bottom
                let dock_height = 80.0;
                dock_height * (1.0 - self.animation_progress)
            }
        }
    }
}
```

## Pin Management

```rust
impl Dock {
    pub fn pin_app(&mut self, app_id: String) {
        // Check if already pinned
        if self.items.iter().any(|i| i.app_id == app_id && i.pinned) {
            return;
        }

        // Add to dock
        let icon = Icon::from_app_id(&app_id);

        let item = DockItem {
            app_id,
            icon,
            windows: Vec::new(),
            pinned: true,
            active: false,
            hovered: false,
            show_dots: false,
        };

        self.items.push(item);
        self.save_config();
    }

    pub fn unpin_app(&mut self, app_id: &str) {
        // Remove if not running
        self.items.retain(|i| {
            !(i.app_id == app_id && i.pinned && i.windows.is_empty())
        });

        // Mark as unpinned if running
        for item in &mut self.items {
            if item.app_id == app_id {
                item.pinned = false;
            }
        }

        self.save_config();
    }

    pub fn toggle_pin(&mut self, app_id: &str) {
        let item = self.items.iter_mut()
            .find(|i| i.app_id == app_id);

        if let Some(item) = item {
            item.pinned = !item.pinned;

            // Remove if unpinned and not running
            if !item.pinned && item.windows.is_empty() {
                self.items.retain(|i| i.app_id != app_id);
            }

            self.save_config();
        }
    }
}
```

## Context Menu

```rust
impl Dock {
    fn show_context_menu(&self, item: &DockItem, position: Point) {
        let mut menu = ContextMenu::new(position);

        menu.add_item(ContextMenuItem {
            label: if item.windows.is_empty() {
                format!("Open {}", item.app_id)
            } else {
                "Close All Windows".into()
            },
            action: Box::new(|| {
                if item.windows.is_empty() {
                    self.launch_app(&item.app_id);
                } else {
                    for window_id in &item.windows {
                        compositor::close_window(*window_id);
                    }
                }
            }),
        });

        menu.add_separator();

        menu.add_item(ContextMenuItem {
            label: if item.pinned {
                "Unpin from Dock".into()
            } else {
                "Pin to Dock".into()
            },
            action: Box::new(|| {
                self.toggle_pin(&item.app_id);
            }),
        });

        menu.add_separator();

        menu.add_item(ContextMenuItem {
            label: "Quit".into(),
            action: Box::new(|| {
                self.quit_app(&item.app_id);
            }),
        });

        menu.show();
    }
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-dock/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── dock.rs
│   │   ├── item.rs
│   │   ├── drag.rs
│   │   └── config.rs
│   └── resources/
│       └── icons/
└── libs/librustica/
    └── widgets/
        └── dock/
            └── src/
                └── lib.rs
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Animation FPS | 60 | Smooth movement |
| Hover response | <16ms | Mouse move to visual |
| Click response | <50ms | Click to action |
| Memory | <10MB | Total dock usage |
| Startup | <300ms | Launch to visible |

## Success Criteria

- [ ] Dock renders correctly
- [ ] Drag-and-drop reordering works
- [ ] Auto-hide animations smooth
- [ ] Running indicators show
- [ ] Context menus work
- [ ] Pin/unpin works
- [ ] Touch targets adequate
- [ ] Performance targets met
- [ ] Accessibility support

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Drag performance | Use hardware acceleration |
| Too many items | Scrollable dock, item limit |
| Animation jank | Double-buffer rendering, limit redraws |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [macOS Dock](https://developer.apple.com/design/human-interface-guidelines/macos/menus-and-bars/dock/)
- [KDE Dock](https://docs.kde.org/stable5/en/kde-workspace/plasma-desktop/)
- [Cosmic Dock](https://github.com/pop-os/cosmic-dock)
- [Unity Dock](https://askubuntu.com/questions/596154/how-does-the-dock-work)
