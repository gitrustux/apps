# Workspace Manager (rustica-workspaces) Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Workspace Management

## Overview

This specification defines the workspace manager for Rustica Shell, providing **virtual desktops**, **workspace switching**, **overview mode**, and **gesture support**. It ensures **<200ms workspace switch** and **smooth animations**.

## Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                        Workspace Overview (Exposé)                   │
├──────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐      │
│  │  Workspace 1    │  │  Workspace 2    │  │  Workspace 3    │      │
│  │  [Active]       │  │                 │  │                 │      │
│  │  ┌───┐ ┌───┐    │  │  ┌───┐ ┌───┐    │  │  ┌───┐         │      │
│  │  │ A │ │ B │    │  │  │ C │ │ D │    │  │  │ E │         │      │
│  │  └───┘ └───┘    │  │  └───┘ └───┘    │  │  └───┘         │      │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘      │
│  ┌─────────────────┐                                                       │
│  │  Workspace 4    │                                                       │
│  │                 │                                                       │
│  │  (Empty)        │                                                       │
│  └─────────────────┘                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

## Workspace State

```rust
pub struct WorkspaceManager {
    /// All workspaces
    workspaces: Vec<Workspace>,

    /// Current workspace index
    current_index: usize,

    /// Overview mode active
    overview_active: bool,

    /// Animation state
    animation_state: AnimationState,

    /// Gestures
    gesture_state: Option<GestureState>,
}

pub struct Workspace {
    /// Workspace index
    index: usize,

    /// Workspace name
    name: String,

    /// Windows in this workspace
    windows: Vec<WindowId>,

    /// Active window
    active_window: Option<WindowId>,

    /// Workspace layout
    layout: WorkspaceLayout,
}

pub enum WorkspaceLayout {
    Tiling { direction: TilingDirection },
    Stacking,
    Floating,
}

pub enum AnimationState {
    Idle,
    Switching { from: usize, to: usize, progress: f32 },
    OverviewEntering { progress: f32 },
    OverviewActive,
    OverviewLeaving { progress: f32 },
}

pub struct GestureState {
    /// Gesture type
    gesture_type: GestureType,

    /// Start position
    start_position: Point,

    /// Current position
    current_position: Point,

    /// Start time
    start_time: Instant,
}

pub enum GestureType {
    /// Three-finger swipe horizontal (workspace switch)
    HorizontalSwipe,

    /// Three-finger pinch (overview)
    Pinch,

    /// Four-finger swipe (all windows view)
    AllWindows,
}
```

## Workspace Switching

```rust
impl WorkspaceManager {
    /// Switch to next workspace
    pub fn next_workspace(&mut self) {
        if self.current_index < self.workspaces.len() - 1 {
            self.switch_to(self.current_index + 1);
        }
    }

    /// Switch to previous workspace
    pub fn previous_workspace(&mut self) {
        if self.current_index > 0 {
            self.switch_to(self.current_index - 1);
        }
    }

    /// Switch to specific workspace
    pub fn switch_to(&mut self, index: usize) {
        if index >= self.workspaces.len() {
            return;
        }

        let from = self.current_index;
        let to = index;

        // Start animation
        self.animation_state = AnimationState::Switching {
            from,
            to,
            progress: 0.0,
        };

        // Hide current workspace windows
        for window_id in &self.workspaces[from].windows {
            compositor::hide_window(*window_id);
        }

        // Show new workspace windows
        for window_id in &self.workspaces[to].windows {
            compositor::show_window(*window_id);
        }

        // Update current index
        self.current_index = to;

        // Notify listeners
        self.notify_switch(from, to);
    }

    /// Update animation
    pub fn update_animation(&mut self, delta_time: Duration) {
        match &mut self.animation_state {
            AnimationState::Switching { from, to, progress } => {
                *progress += delta_time.as_secs_f32() / 0.2;  // 200ms

                if *progress >= 1.0 {
                    *progress = 1.0;
                    self.animation_state = AnimationState::Idle;

                    // Focus active window in new workspace
                    if let Some(window_id) = self.workspaces[*to].active_window {
                        compositor::focus_window(window_id);
                    }
                }
            }
            _ => {}
        }
    }

    /// Render workspace switch animation
    pub fn render_transition(&self, ctx: &mut RenderContext) {
        if let AnimationState::Switching { from, to, progress } = &self.animation_state {
            // Get workspace snapshots
            let from_snapshot = self.workspace_snapshot(*from);
            let to_snapshot = self.workspace_snapshot(*to);

            // Calculate offset based on direction
            let offset = if to > from {
                // Moving right
                let from_x = -(*progress as f32) * ctx.output_width();
                let to_x = (1.0 - *progress) as f32 * ctx.output_width();

                (from_x, to_x)
            } else {
                // Moving left
                let from_x = (*progress as f32) * ctx.output_width();
                let to_x = -(1.0 - *progress) as f32 * ctx.output_width();

                (from_x, to_x)
            };

            // Render from workspace (sliding out)
            ctx.save();
            ctx.translate(offset.0, 0.0);
            ctx.draw_workspace(from_snapshot);
            ctx.restore();

            // Render to workspace (sliding in)
            ctx.save();
            ctx.translate(offset.1, 0.0);
            ctx.draw_workspace(to_snapshot);
            ctx.restore();
        }
    }
}
```

## Overview Mode (Exposé)

```rust
impl WorkspaceManager {
    /// Enter overview mode
    pub fn enter_overview(&mut self) {
        self.overview_active = true;
        self.animation_state = AnimationState::OverviewEntering { progress: 0.0 };

        // Arrange all workspaces in grid
        self.arrange_overview();
    }

    /// Exit overview mode
    pub fn exit_overview(&mut self) {
        self.animation_state = AnimationState::OverviewLeaving { progress: 0.0 };
    }

    /// Toggle overview mode
    pub fn toggle_overview(&mut self) {
        if self.overview_active {
            self.exit_overview();
        } else {
            self.enter_overview();
        }
    }

    fn arrange_overview(&mut self) {
        // Calculate grid layout
        let cols = (self.workspaces.len() as f32).sqrt().ceil() as usize;
        let rows = (self.workspaces.len() + cols - 1) / cols;

        let workspace_width = ctx.output_width() / cols as f32;
        let workspace_height = ctx.output_height() / rows as f32;

        for (index, workspace) in self.workspaces.iter_mut().enumerate() {
            let row = index / cols;
            let col = index % cols;

            workspace.overview_rect = Rect {
                x: col as f32 * workspace_width,
                y: row as f32 * workspace_height,
                width: workspace_width,
                height: workspace_height,
            };
        }
    }

    pub fn render_overview(&self, ctx: &mut RenderContext) {
        // Darken background
        ctx.fill_rect(
            ctx.screen_rect(),
            theme.colors.scrim,
        );

        // Render each workspace
        for workspace in &self.workspaces {
            self.render_workspace_overview(ctx, workspace);
        }

        // Render workspace indicators
        self.render_workspace_indicators(ctx);
    }

    fn render_workspace_overview(&self, ctx: &mut RenderContext, workspace: &Workspace) {
        // Workspace background
        ctx.draw_rounded_rect(
            workspace.overview_rect,
            12,
            theme.colors.surface,
        );

        // Workspace border
        if workspace.index == self.current_index {
            ctx.draw_rounded_border(
                workspace.overview_rect,
                12,
                4,
                theme.colors.primary,
            );
        }

        // Scale and position windows
        let scale = 0.3;  // 30% scale for overview

        for (index, window_id) in workspace.windows.iter().enumerate() {
            if let Some(window) = compositor::get_window(*window_id) {
                let window_rect = window.rect();

                // Scale window
                let scaled_rect = Rect {
                    x: workspace.overview_rect.x + window_rect.x * scale,
                    y: workspace.overview_rect.y + window_rect.y * scale,
                    width: window_rect.width * scale,
                    height: window_rect.height * scale,
                };

                // Render window preview
                ctx.draw_window_preview(*window_id, scaled_rect);
            }
        }

        // Workspace name
        let text_rect = Rect {
            x: workspace.overview_rect.x + 16,
            y: workspace.overview_rect.y + workspace.overview_rect.height - 32,
            width: workspace.overview_rect.width - 32,
            height: 24,
        };

        ctx.draw_text(
            text_rect,
            &workspace.name,
            theme.typography.h4,
            theme.colors.on_surface,
            TextAlignment::Left,
        );
    }

    fn render_workspace_indicators(&self, ctx: &mut RenderContext) {
        let indicator_size = 8;
        let indicator_spacing = 12;
        let total_width = self.workspaces.len() as f32 * (indicator_size + indicator_spacing);

        let mut x = (ctx.output_width() - total_width) / 2;
        let y = ctx.output_height() - 48;

        for (index, workspace) in self.workspaces.iter().enumerate() {
            let rect = Rect {
                x,
                y,
                width: indicator_size,
                height: indicator_size,
            };

            if workspace.index == self.current_index {
                ctx.fill_ellipse(rect, theme.colors.primary);
            } else {
                ctx.draw_ellipse(rect, 2, theme.colors.on_surface_variant);
            }

            x += indicator_size + indicator_spacing;
        }
    }
}
```

## Gesture Recognition

```rust
impl WorkspaceManager {
    /// Handle touch down (gesture start)
    pub fn handle_touch_down(&mut self, slot: TouchSlot, position: Point) {
        // Only 3 or 4 finger gestures count
        let touch_count = self.active_touch_slots();

        if touch_count == 3 || touch_count == 4 {
            self.gesture_state = Some(GestureState {
                gesture_type: GestureType::HorizontalSwipe,
                start_position: position,
                current_position: position,
                start_time: Instant::now(),
            });
        }
    }

    /// Handle touch motion
    pub fn handle_touch_motion(&mut self, position: Point) {
        if let Some(ref mut gesture) = self.gesture_state {
            gesture.current_position = position;

            // Calculate gesture
            let delta = Point {
                x: gesture.current_position.x - gesture.start_position.x,
                y: gesture.current_position.y - gesture.start_position.y,
            };

            // Determine gesture type based on movement
            if delta.x.abs() > 100 {
                // Horizontal swipe - workspace switch
                if delta.x > 0 {
                    // Swipe right - previous workspace
                    self.previous_workspace();
                } else {
                    // Swipe left - next workspace
                    self.next_workspace();
                }

                // Reset gesture
                self.gesture_state = None;
            }
        }
    }

    /// Handle pinch gesture
    pub fn handle_pinch(&mut self, scale: f32) {
        // Pinch out → enter overview
        if scale > 1.2 && !self.overview_active {
            self.enter_overview();
        }

        // Pinch in → exit overview
        if scale < 0.8 && self.overview_active {
            self.exit_overview();
        }
    }

    /// Handle touch up (gesture end)
    pub fn handle_touch_up(&mut self) {
        // Clear gesture state
        self.gesture_state = None;
    }
}
```

## Window Assignment

```rust
impl WorkspaceManager {
    /// Assign window to workspace
    pub fn assign_window(&mut self, window_id: WindowId, workspace_index: usize) {
        if workspace_index >= self.workspaces.len() {
            return;
        }

        // Remove from current workspace
        for workspace in &mut self.workspaces {
            workspace.windows.retain(|id| *id != window_id);
        }

        // Add to new workspace
        self.workspaces[workspace_index].windows.push(window_id);

        // Update window workspace
        compositor::set_window_workspace(window_id, workspace_index);
    }

    /// Move window to next workspace
    pub fn move_window_next(&mut self, window_id: WindowId) {
        let next = (self.current_index + 1).min(self.workspaces.len() - 1);
        self.assign_window(window_id, next);
    }

    /// Move window to previous workspace
    pub fn move_window_previous(&mut self, window_id: WindowId) {
        let prev = self.current_index.saturating_sub(1);
        self.assign_window(window_id, prev);
    }

    /// Move window to new workspace
    pub fn move_window_to_new(&mut self, window_id: WindowId) {
        self.add_workspace();
        let new_index = self.workspaces.len() - 1;
        self.assign_window(window_id, new_index);
        self.switch_to(new_index);
    }
}
```

## Workspace Management

```rust
impl WorkspaceManager {
    /// Add new workspace
    pub fn add_workspace(&mut self) {
        let index = self.workspaces.len();

        let workspace = Workspace {
            index,
            name: format!("Workspace {}", index + 1),
            windows: Vec::new(),
            active_window: None,
            layout: WorkspaceLayout::Tiling {
                direction: TilingDirection::Horizontal,
            },
        };

        self.workspaces.push(workspace);
    }

    /// Remove workspace
    pub fn remove_workspace(&mut self, index: usize) {
        if self.workspaces.len() <= 1 {
            return;  // Keep at least one workspace
        }

        // Move windows to previous workspace
        let target = index.saturating_sub(1);

        for window_id in self.workspaces[index].windows.clone() {
            self.assign_window(window_id, target);
        }

        // Remove workspace
        self.workspaces.remove(index);

        // Renumber workspaces
        for (i, workspace) in self.workspaces.iter_mut().enumerate() {
            workspace.index = i;
        }

        // Update current index if needed
        if self.current_index >= self.workspaces.len() {
            self.current_index = self.workspaces.len() - 1;
        }
    }

    /// Rename workspace
    pub fn rename_workspace(&mut self, index: usize, name: String) {
        if index < self.workspaces.len() {
            self.workspaces[index].name = name;
        }
    }
}
```

## Keyboard Shortcuts

```rust
impl WorkspaceManager {
    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key {
            // Super + 1-9: Switch to workspace
            KeyEvent {
                key: KeyCode::Key1 ..= KeyCode::Key9,
                modifiers: Modifiers::SUPER,
                state: ButtonState::Pressed,
            } => {
                let index = (key.key as usize) - (KeyCode::Key1 as usize);
                self.switch_to(index);
                true
            }

            // Super + Ctrl + 1-9: Move window to workspace
            KeyEvent {
                key: KeyCode::Key1 ..= KeyCode::Key9,
                modifiers: Modifiers::SUPER | Modifiers::CONTROL,
                state: ButtonState::Pressed,
            } => {
                if let Some(window_id) = compositor::focused_window() {
                    let index = (key.key as usize) - (KeyCode::Key1 as usize);
                    self.assign_window(window_id, index);
                }
                true
            }

            // Super + PageUp/Down: Next/Previous workspace
            KeyEvent {
                key: KeyCode::PageUp,
                modifiers: Modifiers::SUPER,
                state: ButtonState::Pressed,
            } => {
                self.previous_workspace();
                true
            }

            KeyEvent {
                key: KeyCode::PageDown,
                modifiers: Modifiers::SUPER,
                state: ButtonState::Pressed,
            } => {
                self.next_workspace();
                true
            }

            // Super + Shift + PageUp/Down: Move window
            KeyEvent {
                key: KeyCode::PageUp,
                modifiers: Modifiers::SUPER | Modifiers::SHIFT,
                state: ButtonState::Pressed,
            } => {
                if let Some(window_id) = compositor::focused_window() {
                    self.move_window_previous(window_id);
                }
                true
            }

            KeyEvent {
                key: KeyCode::PageDown,
                modifiers: Modifiers::SUPER | Modifiers::SHIFT,
                state: ButtonState::Pressed,
            } => {
                if let Some(window_id) = compositor::focused_window() {
                    self.move_window_next(window_id);
                }
                true
            }

            // Overview mode
            KeyEvent {
                key: KeyCode::Escape,
                modifiers: Modifiers::SUPER,
                state: ButtonState::Pressed,
            } => {
                self.toggle_overview();
                true
            }

            _ => false,
        }
    }
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-workspaces/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── manager.rs
│   │   ├── workspace.rs
│   │   ├── overview.rs
│   │   ├── gestures.rs
│   │   └── animation.rs
│   └── resources/
└── libs/librustica/
    └── workspace/
        └── src/
            └── lib.rs
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Workspace switch | <200ms | Start to complete |
| Overview enter | <150ms | Start to visible |
| Overview exit | <150ms | Start to hidden |
| Animation FPS | 60 | Smooth motion |
| Memory | <15MB | Total usage |

## Success Criteria

- [ ] Workspace switching works
- [ ] Overview mode works
- [ ] Gestures recognized correctly
- [ ] Keyboard shortcuts work
- [ ] Window assignment works
- [ ] Add/remove workspaces works
- [ ] Performance targets met
- [ ] Accessibility support

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Gesture conflicts | Configurable gestures, timeout |
| Animation jank | Hardware acceleration |
| Too many workspaces | Dynamic workspace creation |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [GNOME Workspaces](https://help.gnome.org/users/gnome-shell/stable/workspace-switching)
- [KDE Virtual Desktops](https://docs.kde.org/stable5/en/kde-workspace/kcontrol/kcm_virtualdesktops/)
- [macOS Spaces](https://support.apple.com/en-us/HT204100)
- [Cosmic Workspaces](https://github.com/pop-os/cosmic-workspaces-epoch)
