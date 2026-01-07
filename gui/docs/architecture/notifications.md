# Notification System (rustica-notifications) Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Notification System

## Overview

This specification defines the notification system for Rustica Shell, providing **popup notifications**, **notification center**, **do-not-disturb mode**, and **per-app settings**. It ensures **<500ms notification display** and **accessible alerts**.

## Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                         Popup Notification                          │
├──────────────────────────────────────────────────────────────────────┤
│  🔔 Firefox                                                        │
│  ──────────────────────────────────────────────────────────────────│
│  Download Complete                                                 │
│                                                                   │
│  file_abc123.zip has finished downloading.                         │
│                                                                   │
│  [Open]                    [Dismiss]                               │
└──────────────────────────────────────────────────────────────────────┘
                    ◄────────── 400px width ──────────►

┌──────────────────────────────────────────────────────────────────────┐
│                       Notification Center                           │
├──────────────────────────────────────────────────────────────────────┤
│  Notifications                                    [Clear All] [DND]  │
│  ──────────────────────────────────────────────────────────────────│
│  🔔  Today                                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  🔔 Firefox - 2 min ago                                   │   │
│  │  Download Complete                                      │   │
│  │  file_abc123.zip has finished downloading.               │   │
│  └─────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  📧 Thunderbird - 15 min ago                              │   │
│  │  New email: John Doe                                     │   │
│  │  Subject: Meeting tomorrow...                            │   │
│  └─────────────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  🎵 Spotify - 1 hour ago                                  │   │
│  │  Now Playing: Bohemian Rhapsody                          │   │
│  │  Queen                                                 │   │
│  └─────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────────┘
```

## Notification Types

```rust
pub struct Notification {
    /// Unique ID
    pub id: NotificationId,

    /// Application name
    pub app_name: String,

    /// App icon
    pub app_icon: Icon,

    /// Summary/title
    pub summary: String,

    /// Body text
    pub body: String,

    /// Urgency level
    pub urgency: Urgency,

    /// Actions (buttons)
    pub actions: Vec<NotificationAction>,

    /// Image attachment
    pub image: Option<Image>,

    /// Progress (for progress notifications)
    pub progress: Option<f32>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Expiration
    pub expires: Option<DateTime<Utc>>,

    /// Category
    pub category: NotificationCategory,

    /// Resident (cannot be dismissed)
    pub resident: bool,

    /// Transient (auto-dismiss)
    pub transient: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

pub struct NotificationAction {
    pub id: String,
    pub label: String,
    pub icon: Option<Icon>,
}

pub enum NotificationCategory {
    Email,
    InstantMessage,
    Call,
    System,
    Battery,
    Network,
    Hardware,
    Progress,
    Transfer,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct NotificationId(String);
```

## Notification Daemon

```rust
pub struct NotificationDaemon {
    /// Active notifications
    notifications: Vec<Notification>,

    /// Notification history
    history: Vec<Notification>,

    /// Displayed popup
    popup: Option<Box<dyn PopupWidget>>,

    /// Notification center window
    center_window: Option<Window>,

    /// Do-not-disturb mode
    dnd_enabled: bool,

    /// DND critical bypass
    dnd_critical_bypass: bool,

    /// Per-app settings
    app_settings: HashMap<String, AppNotificationSettings>,
}

pub struct AppNotificationSettings {
    /// Enable notifications
    pub enabled: bool,

    /// Sound enabled
    pub sound: bool,

    /// Sound to play
    pub sound_file: Option<PathBuf>,

    /// Show on lock screen
    pub show_on_lock_screen: bool,

    /// Show in notification center
    pub show_in_center: bool,
}
```

## Notification Display

```rust
impl NotificationDaemon {
    /// Display a notification
    pub fn show(&mut self, notification: Notification) {
        // Check DND
        if self.dnd_enabled && notification.urgency != Urgency::Critical {
            return;
        }

        // Check app settings
        if let Some(settings) = self.app_settings.get(&notification.app_name) {
            if !settings.enabled {
                return;
            }
        }

        // Add to notifications
        self.notifications.push(notification.clone());

        // Play sound
        self.play_sound(&notification);

        // Show popup
        if !notification.resident {
            self.show_popup(notification);
        }

        // Update notification center
        self.update_center();

        // Notify listeners
        self.notify_listeners(NotificationEvent::Added {
            id: notification.id,
        });

        // Schedule expiration
        if let Some(expires) = notification.expires {
            self.schedule_expiration(notification.id, expires);
        }
    }

    fn show_popup(&mut self, notification: Notification) {
        // Determine position
        let position = self.popup_position();

        // Create popup widget
        let popup = PopupWidget::new(notification, position);

        // Show popup
        popup.show();

        // Animate in
        popup.animate_in();

        self.popup = Some(Box::new(popup));
    }

    fn popup_position(&self) -> Position {
        // Top-right corner, below panel
        Position {
            x: screen_width() - 440,  // 400px + 40px padding
            y: 60,  // Panel height + 12px
        }
    }

    /// Dismiss notification
    pub fn dismiss(&mut self, id: NotificationId) {
        // Remove from active
        self.notifications.retain(|n| n.id != id);

        // Add to history
        if let Some(notification) = self.get_notification(id) {
            self.history.push(notification.clone());
        }

        // Hide popup if showing
        if let Some(ref popup) = self.popup {
            if popup.notification_id() == id {
                popup.animate_out();
                self.popup = None;
            }
        }

        // Update center
        self.update_center();

        // Notify listeners
        self.notify_listeners(NotificationEvent::Dismissed { id });
    }
}
```

## Popup Widget

```rust
pub struct PopupWidget {
    notification: Notification,
    position: Position,
    rect: Rect,
    hover_progress: f32,
}

impl PopupWidget {
    pub fn new(notification: Notification, position: Position) -> Self {
        let rect = Rect {
            x: position.x,
            y: position.y,
            width: 400,
            height: Self::calculate_height(&notification),
        };

        Self {
            notification,
            position,
            rect,
            hover_progress: 0.0,
        }
    }

    fn calculate_height(notification: &Notification) -> f32 {
        let mut height = 80.0;  // Header and padding

        // Body text (estimate)
        height += notification.body.lines().count() as f32 * 20.0;

        // Actions
        if !notification.actions.is_empty() {
            height += 48.0;
        }

        // Image
        if notification.image.is_some() {
            height += 200.0;
        }

        // Progress bar
        if notification.progress.is_some() {
            height += 24.0;
        }

        height
    }

    pub fn render(&self, ctx: &mut RenderContext) {
        // Background
        ctx.draw_rounded_rect(
            self.rect,
            12,
            theme.colors.surface,
        );

        // Shadow
        ctx.draw_shadow(self.rect, theme.shadows.lg);

        // Border (colored by urgency)
        let border_color = match self.notification.urgency {
            Urgency::Low => theme.colors.outline,
            Urgency::Normal => theme.colors.primary,
            Urgency::Critical => theme.colors.error,
        };

        ctx.draw_rounded_border(
            self.rect,
            12,
            2,
            border_color,
        );

        // App icon and name
        let icon_rect = Rect {
            x: self.rect.x + 16,
            y: self.rect.y + 16,
            width: 32,
            height: 32,
        };

        ctx.draw_icon(icon_rect, &self.notification.app_icon);

        let name_rect = Rect {
            x: icon_rect.x + 40,
            y: self.rect.y + 16,
            width: 300,
            height: 20,
        };

        ctx.draw_text(
            name_rect,
            &self.notification.app_name,
            theme.typography.h4,
            theme.colors.on_surface,
            TextAlignment::Left,
        );

        // Close button
        let close_rect = Rect {
            x: self.rect.x + self.rect.width - 40,
            y: self.rect.y + 16,
            width: 24,
            height: 24,
        };

        ctx.draw_icon(close_rect, &Icon::from_name("close"));

        // Summary
        let summary_rect = Rect {
            x: self.rect.x + 16,
            y: self.rect.y + 60,
            width: self.rect.width - 32,
            height: 24,
        };

        ctx.draw_text(
            summary_rect,
            &self.notification.summary,
            theme.typography.body,
            theme.colors.on_surface,
            TextAlignment::Left,
        );

        // Body
        let body_rect = Rect {
            x: self.rect.x + 16,
            y: summary_rect.y + 28,
            width: self.rect.width - 32,
            height: 100,
        };

        ctx.draw_text_wrapped(
            body_rect,
            &self.notification.body,
            theme.typography.sm,
            theme.colors.on_surface_variant,
        );

        // Actions
        let mut action_x = self.rect.x + 16;
        for action in &self.notification.actions {
            let action_rect = Rect {
                x: action_x,
                y: self.rect.y + self.rect.height - 48,
                width: 120,
                height: 36,
            };

            ctx.draw_button(action_rect, &action.label);

            action_x += 128;
        }

        // Image
        if let Some(ref image) = self.notification.image {
            let image_rect = Rect {
                x: self.rect.x + 16,
                y: body_rect.y + 100,
                width: self.rect.width - 32,
                height: 200,
            };

            ctx.draw_image(image_rect, image);
        }

        // Progress bar
        if let Some(progress) = self.notification.progress {
            let progress_rect = Rect {
                x: self.rect.x + 16,
                y: self.rect.y + self.rect.height - 60,
                width: self.rect.width - 32,
                height: 4,
            };

            // Background
            ctx.fill_rect(
                progress_rect,
                theme.colors.surface_variant,
            );

            // Progress
            let filled_rect = Rect {
                x: progress_rect.x,
                y: progress_rect.y,
                width: progress_rect.width * progress,
                height: progress_rect.height,
            };

            ctx.fill_rect(
                filled_rect,
                theme.colors.primary,
            );
        }
    }
}
```

## Notification Center

```rust
pub struct NotificationCenter {
    /// Notifications
    notifications: Vec<Notification>,

    /// Grouping
    groups: Vec<NotificationGroup>,

    /// Filter
    filter: NotificationFilter,

    /// Scroll position
    scroll_offset: f32,
}

pub struct NotificationGroup {
    /// App name
    app_name: String,

    /// Notifications for this app
    notifications: Vec<Notification>,

    /// Expanded
    expanded: bool,
}

pub enum NotificationFilter {
    All,
    Today,
    Unread,
}

impl NotificationCenter {
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
            groups: Vec::new(),
            filter: NotificationFilter::All,
            scroll_offset: 0.0,
        }
    }

    pub fn render(&mut self, ctx: &mut RenderContext) {
        // Background
        let rect = ctx.screen_rect();
        ctx.fill_rect(rect, theme.colors.scrim.with_alpha(0.95));

        // Header
        let header_rect = Rect {
            x: rect.x + 24,
            y: rect.y + 24,
            width: rect.width - 48,
            height: 48,
        };

        ctx.draw_text(
            header_rect,
            "Notifications",
            theme.typography.h3,
            theme.colors.on_background,
            TextAlignment::Left,
        );

        // Clear All button
        let clear_rect = Rect {
            x: rect.x + rect.width - 150,
            y: rect.y + 32,
            width: 100,
            height: 36,
        };

        ctx.draw_button(clear_rect, "Clear All");

        // DND button
        let dnd_rect = Rect {
            x: clear_rect.x - 120,
            y: rect.y + 32,
            width: 100,
            height: 36,
        };

        let dnd_label = if self.dnd_enabled { "DND On" } else { "DND Off" };
        ctx.draw_button(dnd_rect, dnd_label);

        // Notifications
        let mut y = 100;
        for group in &self.groups {
            y += self.render_group(ctx, group, y);
        }
    }

    fn render_group(&self, ctx: &mut RenderContext, group: &NotificationGroup, y: f32) -> f32 {
        // App header
        let header_rect = Rect {
            x: 24,
            y,
            width: 400,
            height: 32,
        };

        ctx.draw_text(
            header_rect,
            &group.app_name,
            theme.typography.h4,
            theme.colors.on_background,
            TextAlignment::Left,
        );

        let mut height = 40;

        // Notifications (if expanded)
        if group.expanded {
            for notification in &group.notifications {
                let notif_rect = Rect {
                    x: 24,
                    y: y + height,
                    width: 600,
                    height: 100,
                };

                self.render_notification(ctx, notification, notif_rect);
                height += 110;
            }
        }

        height
    }

    fn render_notification(&self, ctx: &mut RenderContext, notification: &Notification, rect: Rect) {
        // Background
        ctx.draw_rounded_rect(
            rect,
            8,
            theme.colors.surface,
        );

        // Time
        let time_ago = notification.timestamp.signed_duration_since(Utc::now());
        let time_str = format_time_ago(time_ago);

        let time_rect = Rect {
            x: rect.x + rect.width - 120,
            y: rect.y + 16,
            width: 100,
            height: 20,
        };

        ctx.draw_text(
            time_rect,
            &time_str,
            theme.typography.caption,
            theme.colors.on_surface_variant,
            TextAlignment::Right,
        );

        // Summary
        let summary_rect = Rect {
            x: rect.x + 16,
            y: rect.y + 16,
            width: rect.width - 140,
            height: 20,
        };

        ctx.draw_text(
            summary_rect,
            &notification.summary,
            theme.typography.body,
            theme.colors.on_surface,
            TextAlignment::Left,
        );

        // Body
        let body_rect = Rect {
            x: rect.x + 16,
            y: rect.y + 40,
            width: rect.width - 32,
            height: 48,
        };

        ctx.draw_text_wrapped(
            body_rect,
            &notification.body,
            theme.typography.sm,
            theme.colors.on_surface_variant,
        );
    }
}
```

## Do-Not-Disturb Mode

```rust
impl NotificationDaemon {
    pub fn set_dnd(&mut self, enabled: bool) {
        self.dnd_enabled = enabled;

        // Update panel indicator
        panel::set_dnd_indicator(enabled);

        // Persist setting
        config::set_dnd(enabled);
    }

    pub fn toggle_dnd(&mut self) {
        self.set_dnd(!self.dnd_enabled);
    }

    pub fn is_dnd_active(&self) -> bool {
        self.dnd_enabled
    }
}
```

## Sound

```rust
impl NotificationDaemon {
    fn play_sound(&self, notification: &Notification) {
        // Check if sound enabled
        if let Some(settings) = self.app_settings.get(&notification.app_name) {
            if !settings.sound {
                return;
            }

            // Use custom sound if specified
            if let Some(ref sound_file) = settings.sound_file {
                audio::play_sound(sound_file);
                return;
            }
        }

        // Play default sound based on urgency
        let sound = match notification.urgency {
            Urgency::Low => "notification-low",
            Urgency::Normal => "notification",
            Urgency::Critical => "notification-critical",
        };

        audio::play_system_sound(sound);
    }
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-notifications/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── daemon.rs
│   │   ├── popup.rs
│   │   ├── center.rs
│   │   ├── dnd.rs
│   │   └── sound.rs
│   └── resources/
│       └── sounds/
└── libs/librustica/
    └── notifications/
        └── src/
            └── lib.rs
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Notification display | <500ms | Request to visible |
| Center open | <300ms | Action to visible |
| Sound play | <100ms | Trigger to audible |
| Memory | <10MB | Daemon usage |

## Success Criteria

- [ ] Notifications display correctly
- [ ] Popup auto-dismiss works
- [ ] Center shows all notifications
- [ ] DND mode works
- [ ] Per-app settings work
- [ ] Sound plays correctly
- [ ] Critical notifications bypass DND
- [ ] Performance targets met
- [ ] Accessibility support

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Notification spam | Rate limiting per app |
| Too many popups | Queue and show sequentially |
| Sound annoyance | Per-app sound control |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [ freedesktop Notifications Specification](https://specifications.freedesktop.org/notification-spec/)
- [GNOME Notifications](https://developer.gnome.org/gnome-shell/stable/shell-shell.html#Shell-notify)
- [KDE Notifications](https://develop.kde.org/docs/plasma/notification/)
- [macOS Notifications](https://developer.apple.com/documentation/usernotifications)
