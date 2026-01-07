# Event Model Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Event System

## Overview

The event model defines how input, window lifecycle, and system events flow through Rustica Shell. This specification ensures **<1ms event latency** and **no event drops under normal load**.

## Event Types

### 1. Input Events
```rust
pub enum InputEvent {
    // Pointer (mouse)
    PointerMove { position: (f32, f32), delta: (f32, f32) },
    PointerButton { button: PointerButton, state: ButtonState },
    PointerAxis { axis: Axis, value: f32 },
    PointerFrame,

    // Keyboard
    KeyEvent { key: KeyCode, state: KeyState, modifiers: Modifiers },
    KeyModifiers { modifiers: Modifiers },

    // Touch (multi-touch)
    TouchDown { slot: u32, position: (f32, f32) },
    TouchUp { slot: u32 },
    TouchMotion { slot: u32, position: (f32, f32) },
    TouchCancel { slot: u32 },
    TouchFrame,
}

pub enum PointerButton {
    Left,
    Right,
    Middle,
    // Additional buttons (4, 5, etc.)
}

pub enum ButtonState {
    Pressed,
    Released,
}
```

### 2. Window Lifecycle Events
```rust
pub enum WindowEvent {
    // Creation/destruction
    Create { surface: SurfaceId },
    Destroy { surface: SurfaceId },

    // State changes
    Configure { surface: SurfaceId, size: (u32, u32), state: WindowState },
    Close { surface: SurfaceId },

    // Focus
    FocusIn { surface: SurfaceId },
    FocusOut { surface: SurfaceId },

    // Display
    Enter { surface: SurfaceId },
    Leave { surface: SurfaceId },
}
```

### 3. System Events
```rust
pub enum SystemEvent {
    // Display changes
    DisplayAdd { display: DisplayId },
    DisplayRemove { display: DisplayId },
    DisplayConfigure { display: DisplayId, config: DisplayConfig },

    // Output configuration
    OutputScale { scale: f32 },
    OutputTransform { transform: Transform },

    // Session changes
    Suspend,
    Resume,
    ScreenLock,
    ScreenUnlock,

    // Theme changes
    ThemeChange { theme: Theme },
}
```

### 4. Custom Events
```rust
pub enum CustomEvent {
    // Workspace management
    WorkspaceAdd { id: WorkspaceId },
    WorkspaceRemove { id: WorkspaceId },
    WorkspaceSwitch { from: WorkspaceId, to: WorkspaceId },

    // Notifications
    NotificationAdd { notification: Notification },
    NotificationRemove { id: NotificationId },

    // Application lifecycle
    AppStart { app_id: String },
    AppExit { app_id: String, exit_code: i32 },
}
```

## Event Delivery Mechanism

### Event Flow

```
┌──────────────┐
│   Hardware   │
│   (Input)    │
└──────┬───────┘
       │ libinput
       ▼
┌──────────────────────┐
│  Event Collector     │
│  (Compositor)        │
└──────┬───────────────┘
       │
       ├────► Input Queue (priority-based)
       │
       ▼
┌──────────────────────┐
│  Event Dispatcher    │
│                      │
│  ┌────────────────┐  │
│  │  Priority Queue│  │
│  ├────────────────┤  │
│  │ 1. System     │  │
│  │ 2. Window     │  │
│  │ 3. Input      │  │
│  │ 4. Custom     │  │
│  └────────────────┘  │
└──────┬───────────────┘
       │ dispatch
       ▼
┌──────────────────────┐
│   Target Surface     │
│   (Application)      │
└──────────────────────┘
```

### Priority Levels

| Priority | Event Types | Latency Target |
|----------|-------------|----------------|
| 1 (Critical) | System events (suspend, emergency) | <100μs |
| 2 (High) | Window lifecycle (create, destroy) | <500μs |
| 3 (Normal) | Input events (keyboard, mouse, touch) | <1ms |
| 4 (Low) | Custom events (notifications, theme) | <5ms |

### Event Loop Contract

```rust
pub trait EventLoop {
    // Run the event loop
    fn run(&mut self) -> !;

    // Post events to the queue
    fn post(&mut self, event: Event, priority: Priority);

    // Register event handlers
    fn on_input<F>(&mut self, handler: F) where F: Fn(InputEvent);
    fn on_window<F>(&mut self, handler: F) where F: Fn(WindowEvent);
    fn on_system<F>(&mut self, handler: F) where F: Fn(SystemEvent);
}
```

## Event Filtering

### Per-Surface Filtering

Applications only receive events relevant to them:

```rust
pub struct EventFilter {
    // Only send events for focused surface
    pub focus_only: bool,

    // Drop repeated events (debouncing)
    pub debounce: Duration,

    // Filter by type
    pub allowed: Vec<EventKind>,
}
```

### Global Filters

```rust
// Compositor-level event filters
pub mod filters {
    // Throttle motion events (save CPU)
    pub fn throttle_motion(events: Vec<InputEvent>) -> Vec<InputEvent> {
        events.into_iter()
            .enumerate()
            .filter(|(i, _)| i % 3 == 0)  // 1/3 of motion events
            .map(|(_, e)| e)
            .collect()
    }

    // Combine multiple pointer events
    pub fn coalesce_pointer(events: Vec<InputEvent>) -> Vec<InputEvent> {
        // Combine motion events with same position
        // ...
    }
}
```

## Async/Sync Handling Strategy

### Synchronous Events (Critical Path)

Input events must be handled **synchronously**:

```rust
// Input handling (sync, <1ms)
fn handle_input_sync(event: InputEvent) {
    match event {
        InputEvent::KeyEvent { .. } => {
            // Immediately send to focused window
            focused_surface.send_event(event);
        }
        InputEvent::PointerButton { .. } => {
            // Find surface under pointer
            let surface = find_surface_at(pointer_position);
            surface.send_event(event);
        }
        _ => { /* ... */ }
    }
}
```

### Asynchronous Events (Background)

Non-critical events can be **handled asynchronously**:

```rust
// System events (async, <10ms acceptable)
async fn handle_system_async(event: SystemEvent) {
    match event {
        SystemEvent::ThemeChange { theme } => {
            // Update all windows gradually
            for window in windows {
                window.update_theme(theme.clone()).await.ok();
            }
        }
        _ => { /* ... */ }
    }
}
```

## Event Queue Management

### Queue Structure

```rust
pub struct EventQueue {
    // Separate queues for each priority
    system: VecDeque<SystemEvent>,
    window: VecDeque<WindowEvent>,
    input: VecDeque<InputEvent>,
    custom: VecDeque<CustomEvent>,

    // Statistics
    stats: EventStats,
}

pub struct EventStats {
    pub events_processed: u64,
    pub events_dropped: u64,
    pub avg_latency_ns: u64,
    pub max_latency_ns: u64,
}
```

### Queue Policies

```rust
impl EventQueue {
    // Insert with priority
    pub fn post(&mut self, event: Event, priority: Priority) {
        match priority {
            Priority::Critical => self.system.push_back(event),
            Priority::High => self.window.push_back(event),
            Priority::Normal => self.input.push_back(event),
            Priority::Low => self.custom.push_back(event),
        }
    }

    // Process next batch of events
    pub fn process(&mut self, handler: &mut dyn EventHandler) {
        // Process in priority order
        self.process_system(handler);
        self.process_window(handler);
        self.process_input(handler);
        self.process_custom(handler);
    }

    // Drop events if queue is too full
    pub fn enforce_limits(&mut self) {
        const MAX_QUEUE_SIZE: usize = 1000;

        // Drop oldest events from low-priority queues
        while self.custom.len() > MAX_QUEUE_SIZE {
            self.custom.pop_front();
            self.stats.events_dropped += 1;
        }
    }
}
```

## Touch Event Handling

### Multi-Touch Support

```rust
pub struct TouchState {
    // Active touch points
    slots: [Option<TouchPoint>; 10],  // Max 10 touch points

    // Gesture recognition
    recognizer: GestureRecognizer,
}

pub struct TouchPoint {
    id: u32,
    position: (f32, f32),
    start_position: (f32, f32),
    start_time: Instant,
}

pub enum Gesture {
    // Single touch
    Tap { position: (f32, f32), time: Instant },
    LongPress { position: (f32, f32) },

    // Multi-touch
    Pinch { center: (f32, f32), scale: f32 },
    Rotate { center: (f32, f32), angle: f32 },
    Swipe { start: (f32, f32), end: (f32, f32), velocity: f32 },
}
```

### Gesture Recognition

```rust
impl TouchState {
    pub fn handle(&mut self, event: InputEvent) -> Option<Gesture> {
        match event {
            InputEvent::TouchDown { slot, position } => {
                self.slots[slot as usize] = Some(TouchPoint {
                    id: slot,
                    position,
                    start_position: position,
                    start_time: Instant::now(),
                });
            }
            InputEvent::TouchMotion { slot, position } => {
                if let Some(point) = &mut self.slots[slot as usize] {
                    point.position = position;
                }
            }
            InputEvent::TouchUp { slot } => {
                self.slots[slot as usize] = None;
            }
            _ => {}
        }

        // Recognize gestures
        self.recognizer.detect(&self.slots)
    }
}
```

## Performance Requirements

| Metric | Target | Measurement Point |
|--------|--------|-------------------|
| Input latency | <1ms | Hardware to application |
| Event dispatch | <100μs | Queue to handler |
| Queue depth | <100 events | Under normal load |
| Event drops | 0% | Under normal load |
| CPU usage | <5% | Event loop only |

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── libs/librustica/src/
│   ├── events/
│   │   ├── mod.rs              # Event module
│   │   ├── event.rs            # Event enums
│   │   ├── queue.rs            # Event queue
│   │   ├── dispatcher.rs       # Event dispatcher
│   │   ├── filter.rs           # Event filtering
│   │   └── touch.rs            # Touch handling
│   └── ...
│
└── rustica-comp/src/
    ├── input/
    │   ├── mod.rs
    │   ├── keyboard.rs         # Keyboard input
    │   ├── pointer.rs          # Mouse/pointer
    │   ├── touch.rs            # Touch input
    │   └── tablet.rs           # Tablet/pen input
    └── events/
        ├── mod.rs
        ├── loop.rs             # Event loop
        └── handler.rs          # Event handlers
```

## Example Usage

### Application Event Handling

```rust
use librustica::events::*;

struct MyApp {
    state: AppState,
}

impl EventHandler for MyApp {
    fn on_input(&mut self, event: InputEvent) {
        match event {
            InputEvent::KeyEvent { key, state: ButtonState::Pressed, .. } => {
                self.handle_keypress(key);
            }
            InputEvent::PointerButton { button: PointerButton::Left, state: ButtonState::Pressed } => {
                self.handle_click();
            }
            _ => {}
        }
    }

    fn on_window(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Close { .. } => {
                self.save_state();
                std::process::exit(0);
            }
            _ => {}
        }
    }
}

fn main() {
    let mut app = MyApp { state: AppState::new() };
    let mut event_loop = EventLoop::new();

    event_loop.run(|event| {
        app.handle_event(event);
    });
}
```

## Success Criteria

- [ ] All event types defined and documented
- [ ] Event queue processes events in priority order
- [ ] Input events delivered in <1ms
- [ ] No event drops under normal load
- [ ] Touch gestures recognized correctly
- [ ] Async/sync event handling works
- [ ] Performance targets met

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Event queue overflow | Drop low-priority events, monitor queue depth |
| High CPU usage | Throttle motion events, coalesce similar events |
| Input lag | Profile hot paths, optimize critical path |
| Touch gesture conflicts | Make gesture recognition configurable |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [Wayland Event Handling](https://wayland.freedesktop.org/docs/html/ch04.html)
- [Smithay Event Loop](https://docs.rs/smithay/*/smithay/wayland/struct.EventQueue.html)
- [Libinput Touch Handling](https://wayland.freedesktop.org/libinput/doc/latest/)
- [GTK Event Controllers](https://docs.gtk.org/gtk4/gtk4/enum.EventControllerPropagationPhase.html)
