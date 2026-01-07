# Phase 8.1: Touch Gesture System (rustica-gestures)

## Overview

**Component**: rustica-gestures
**Purpose**: Multi-touch gesture recognition and handling system for mobile/tablet interactions
**Language**: Rust
**Dependencies**: smithay, libc, dbus (zbus)

## Goals

1. **Native Multi-Touch Support**: Handle 2-10 simultaneous touch points
2. **Gesture Recognition**: Detect common gestures (tap, double-tap, long-press, swipe, pinch, rotate)
3. **Compositor Integration**: Seamlessly integrate with Smithay's touch event handling
4. **Customizable**: Allow per-app gesture configuration and disable gestures
5. **Accessibility**: Support gesture alternatives and reduced-sensitivity mode

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    rustica-comp                              │
│                  (Wayland Compositor)                        │
└────────────────────────┬────────────────────────────────────┘
                         │ Smithay touch events
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  rustica-gestures                            │
│                  (Gesture Engine)                            │
├─────────────────────────────────────────────────────────────┤
│  TouchTracker      │  GestureRecognizer  │  GestureDispatcher│
│  - Track touches   │  - State machine    │  - Emit gestures  │
│  - Compute deltas  │  - Pattern match    │  - Route to apps  │
│  - Detect timeout  │  - Threshold check  │  - Global hooks   │
└────────────────────────┬────────────────────────────────────┘
                         │ Gesture events
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Client Applications                             │
│         (receive gesture events via Wayland)                 │
└─────────────────────────────────────────────────────────────┘
```

## Core Data Structures

```rust
/// Touch point ID and tracking data
#[derive(Debug, Clone)]
pub struct TouchPoint {
    pub id: i32,
    pub position: Point<f64, Logical>,
    pub start_position: Point<f64, Logical>,
    pub pressure: Option<f32>,
    pub major: Option<f64>,  // Touch ellipse major axis
    pub minor: Option<f64>,  // Touch ellipse minor axis
    pub start_time: Instant,
    pub update_time: Instant,
}

/// Active touch session tracking multiple points
pub struct TouchSession {
    pub points: HashMap<i32, TouchPoint>,
    pub start_time: Instant,
    pub widget_under_touch: Option<Widget>,
    pub accepted: bool,  // Gesture accepted by app
}

/// Recognized gesture types
#[derive(Debug, Clone, PartialEq)]
pub enum Gesture {
    /// Single tap (quick down + up)
    Tap {
        position: Point<f64, Logical>,
        count: u32,  // 1, 2, 3 for multi-tap
    },

    /// Long press (hold without moving)
    LongPress {
        position: Point<f64, Logical>,
        duration: Duration,
    },

    /// Swipe in a direction
    Swipe {
        start_position: Point<f64, Logical>,
        end_position: Point<f64, Logical>,
        direction: SwipeDirection,
        velocity: f64,
    },

    /// Pinch to zoom (two fingers)
    Pinch {
        center: Point<f64, Logical>,
        scale: f64,  // 1.0 = no change, <1.0 = pinch in, >1.0 = pinch out
        phase: PinchPhase,
    },

    /// Two-finger rotation
    Rotate {
        center: Point<f64, Logical>,
        angle: f64,  // Radians, positive = clockwise
        phase: RotatePhase,
    },

    /// Three-finger swipe (for navigation)
    ThreeFingerSwipe {
        direction: SwipeDirection,
    },

    /// Edge swipe (from screen edge)
    EdgeSwipe {
        edge: ScreenEdge,
        progress: f64,  // 0.0 to 1.0
    },

    /// Pan/drag gesture
    Pan {
        delta: Vector2D<f64, Logical>,
        current_position: Point<f64, Logical>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PinchPhase {
    Begin,
    Update,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RotatePhase {
    Begin,
    Update,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScreenEdge {
    Top,
    Bottom,
    Left,
    Right,
}
```

## Gesture Recognition Engine

```rust
pub struct GestureRecognizer {
    config: GestureConfig,
    state: GestureState,
}

pub struct GestureConfig {
    /// Maximum movement for a tap (pixels)
    pub tap_slop: f64,

    /// Maximum time for a tap (ms)
    pub tap_timeout: u64,

    /// Minimum time for long press (ms)
    pub long_press_timeout: u64,

    /// Minimum distance for swipe (pixels)
    pub swipe_min_distance: f64,

    /// Minimum velocity for swipe (pixels/second)
    pub swipe_min_velocity: f64,

    /// Maximum time between taps for double/triple tap (ms)
    pub multi_tap_gap: u64,

    /// Edge swipe detection zone width (pixels)
    pub edge_zone_width: f64,

    /// Scale sensitivity for pinch gestures
    pub pinch_sensitivity: f64,

    /// Rotation sensitivity for rotate gestures
    pub rotation_sensitivity: f64,
}

impl Default for GestureConfig {
    fn default() -> Self {
        Self {
            tap_slop: 16.0,  // Android standard
            tap_timeout: 400,
            long_press_timeout: 500,
            swipe_min_distance: 80.0,
            swipe_min_velocity: 100.0,
            multi_tap_gap: 300,
            edge_zone_width: 24.0,
            pinch_sensitivity: 1.0,
            rotation_sensitivity: 1.0,
        }
    }
}

pub enum GestureState {
    Idle,
    TrackingTap {
        touch_id: i32,
        start_time: Instant,
        tap_count: u32,
        last_tap_time: Option<Instant>,
    },
    TrackingLongPress {
        touch_id: i32,
        start_time: Instant,
    },
    TrackingSwipe {
        touch_id: i32,
        start_position: Point<f64, Logical>,
        start_time: Instant,
    },
    TrackingPinch {
        touch_ids: (i32, i32),
        initial_distance: f64,
        initial_center: Point<f64, Logical>,
    },
    TrackingRotate {
        touch_ids: (i32, i32),
        initial_angle: f64,
        initial_center: Point<f64, Logical>,
    },
    TrackingEdgeSwipe {
        touch_id: i32,
        edge: ScreenEdge,
        start_position: Point<f64, Logical>,
    },
    TrackingPan {
        touch_id: i32,
        start_position: Point<f64, Logical>,
    },
}

impl GestureRecognizer {
    /// Process a touch down event
    pub fn handle_touch_down(
        &mut self,
        touch: &TouchPoint,
        session: &TouchSession,
    ) -> Option<Gesture> {
        match &self.state {
            GestureState::Idle => {
                // Check for edge swipe first
                if let Some(edge) = self.detect_edge_swipe(touch) {
                    self.state = GestureState::TrackingEdgeSwipe {
                        touch_id: touch.id,
                        edge,
                        start_position: touch.position,
                    };
                    return None;
                }

                // Start tracking as potential tap
                self.state = GestureState::TrackingTap {
                    touch_id: touch.id,
                    start_time: touch.start_time,
                    tap_count: 1,
                    last_tap_time: None,
                };
                None
            }

            GestureState::TrackingTap { touch_id, tap_count, last_tap_time, .. }
                if *touch_id != touch.id =>
            {
                // Second touch down - check for pinch/rotate
                if session.points.len() == 2 {
                    self.start_two_finger_gesture(session);
                }
                None
            }

            _ => None,
        }
    }

    /// Process a touch up event
    pub fn handle_touch_up(
        &mut self,
        touch: &TouchPoint,
        session: &TouchSession,
    ) -> Option<Gesture> {
        match &self.state {
            GestureState::TrackingTap { touch_id, start_time, tap_count, .. }
                if *touch_id == touch.id =>
            {
                let elapsed = touch.update_time.duration_since(*start_time);
                let moved = touch.position.distance(touch.start_position);

                if elapsed < Duration::from_millis(self.config.tap_timeout)
                    && moved < self.config.tap_slop
                {
                    // Valid tap
                    self.state = GestureState::Idle;
                    return Some(Gesture::Tap {
                        position: touch.position,
                        count: *tap_count,
                    });
                } else if elapsed > Duration::from_millis(self.config.long_press_timeout)
                    && moved < self.config.tap_slop
                {
                    // Long press
                    self.state = GestureState::Idle;
                    return Some(Gesture::LongPress {
                        position: touch.position,
                        duration: elapsed,
                    });
                }
            }

            GestureState::TrackingSwipe { touch_id, start_position, .. }
                if *touch_id == touch.id =>
            {
                let distance = start_position.distance(touch.position);
                if distance >= self.config.swipe_min_distance {
                    let direction = self.calculate_direction(start_position, &touch.position);
                    let duration = touch.update_time.duration_since(touch.start_time);
                    let velocity = distance / duration.as_secs_f64();

                    if velocity >= self.config.swipe_min_velocity {
                        self.state = GestureState::Idle;
                        return Some(Gesture::Swipe {
                            start_position: *start_position,
                            end_position: touch.position,
                            direction,
                            velocity,
                        });
                    }
                }
            }

            GestureState::TrackingPinch { .. } | GestureState::TrackingRotate { .. } => {
                // End two-finger gesture
                if session.points.len() <= 1 {
                    return self.end_two_finger_gesture(touch, session);
                }
            }

            _ => {}
        }

        None
    }

    /// Process a touch motion event
    pub fn handle_touch_motion(
        &mut self,
        touch: &TouchPoint,
        session: &TouchSession,
    ) -> Option<Gesture> {
        match &mut self.state {
            GestureState::TrackingTap { touch_id, start_time, .. }
                if *touch_id == touch.id =>
            {
                let elapsed = touch.update_time.duration_since(*start_time);
                let moved = touch.position.distance(touch.start_position);

                // Cancel tap if moved too far
                if moved >= self.config.tap_slop {
                    // Transition to swipe or pan tracking
                    if elapsed > Duration::from_millis(self.config.long_press_timeout) {
                        self.state = GestureState::TrackingPan {
                            touch_id: touch.id,
                            start_position: touch.start_position,
                        };
                    } else {
                        self.state = GestureState::TrackingSwipe {
                            touch_id: touch.id,
                            start_position: touch.start_position,
                            start_time: *start_time,
                        };
                    }
                }
            }

            GestureState::TrackingLongPress { touch_id, start_time }
                if *touch_id == touch.id =>
            {
                let moved = touch.position.distance(touch.start_position);
                let elapsed = touch.update_time.duration_since(*start_time);

                if moved >= self.config.tap_slop {
                    // Cancel long press, transition to pan
                    self.state = GestureState::TrackingPan {
                        touch_id: touch.id,
                        start_position: touch.start_position,
                    };
                } else if elapsed >= Duration::from_millis(self.config.long_press_timeout) {
                    // Emit long press gesture
                    self.state = GestureState::Idle;
                    return Some(Gesture::LongPress {
                        position: touch.position,
                        duration: elapsed,
                    });
                }
            }

            GestureState::TrackingPan { touch_id, start_position }
                if *touch_id == touch.id =>
            {
                let delta = touch.position - *start_position;
                return Some(Gesture::Pan {
                    delta,
                    current_position: touch.position,
                });
            }

            GestureState::TrackingSwipe { touch_id, start_position, start_time }
                if *touch_id == touch.id =>
            {
                let moved = start_position.distance(touch.position);
                if moved >= self.config.swipe_min_distance {
                    let duration = touch.update_time.duration_since(*start_time);
                    let velocity = moved / duration.as_secs_f64();

                    if velocity >= self.config.swipe_min_velocity {
                        let direction = self.calculate_direction(start_position, &touch.position);
                        self.state = GestureState::Idle;
                        return Some(Gesture::Swipe {
                            start_position: *start_position,
                            end_position: touch.position,
                            direction,
                            velocity,
                        });
                    }
                }
            }

            GestureState::TrackingPinch { touch_ids, initial_distance, .. } => {
                if let (Some(t1), Some(t2)) = (
                    session.points.get(&touch_ids.0),
                    session.points.get(&touch_ids.1),
                ) {
                    let current_distance = t1.position.distance(t2.position);
                    let center = (t1.position + t2.position) / 2.0;
                    let scale = current_distance / initial_distance;

                    return Some(Gesture::Pinch {
                        center,
                        scale,
                        phase: PinchPhase::Update,
                    });
                }
            }

            GestureState::TrackingRotate { touch_ids, initial_center, .. } => {
                if let (Some(t1), Some(t2)) = (
                    session.points.get(&touch_ids.0),
                    session.points.get(&touch_ids.1),
                ) {
                    let current_angle = (t2.position.y - t1.position.y)
                        .atan2(t2.position.x - t1.position.x);
                    let center = (t1.position + t2.position) / 2.0;

                    // This is simplified; need to track initial angle properly
                    return Some(Gesture::Rotate {
                        center,
                        angle: current_angle,
                        phase: RotatePhase::Update,
                    });
                }
            }

            GestureState::TrackingEdgeSwipe { touch_id, edge, start_position }
                if *touch_id == touch.id =>
            {
                let distance = start_position.distance(touch.position);
                let max_distance = match edge {
                    ScreenEdge::Left | ScreenEdge::Right => 400.0,  // Width
                    ScreenEdge::Top | ScreenEdge::Bottom => 800.0,  // Height
                };
                let progress = (distance / max_distance).min(1.0);

                return Some(Gesture::EdgeSwipe {
                    edge: *edge,
                    progress,
                });
            }

            _ => {}
        }

        None
    }

    /// Start tracking a two-finger gesture (pinch or rotate)
    fn start_two_finger_gesture(&mut self, session: &TouchSession) {
        let mut points: Vec<_> = session.points.values().collect();
        if points.len() != 2 {
            return;
        }

        let t1 = points[0];
        let t2 = points[1];

        let distance = t1.position.distance(t2.position);
        let center = (t1.position + t2.position) / 2.0;
        let angle = (t2.position.y - t1.position.y).atan2(t2.position.x - t1.position.x);

        // Start both pinch and rotate tracking simultaneously
        self.state = GestureState::TrackingPinch {
            touch_ids: (t1.id, t2.id),
            initial_distance: distance,
            initial_center: center,
        };
        // We can also track rotation here in a more complete implementation
    }

    /// End a two-finger gesture
    fn end_two_finger_gesture(&mut self, touch: &TouchPoint, session: &TouchSession) -> Option<Gesture> {
        if let GestureState::TrackingPinch { touch_ids, initial_distance, initial_center } = self.state {
            if let (Some(t1), Some(t2)) = (
                session.points.get(&touch_ids.0),
                session.points.get(&touch_ids.1),
            ) {
                let final_distance = t1.position.distance(t2.position);
                let scale = final_distance / initial_distance;

                self.state = GestureState::Idle;
                return Some(Gesture::Pinch {
                    center: *initial_center,
                    scale,
                    phase: PinchPhase::End,
                });
            }
        }

        self.state = GestureState::Idle;
        None
    }

    /// Detect if touch started in edge swipe zone
    fn detect_edge_swipe(&self, touch: &TouchPoint) -> Option<ScreenEdge> {
        let pos = touch.position;
        let zone = self.config.edge_zone_width;

        // Assume screen dimensions are available from compositor
        // This is simplified
        if pos.x < zone {
            Some(ScreenEdge::Left)
        } else if pos.x > 1920.0 - zone {  // Should use actual screen width
            Some(ScreenEdge::Right)
        } else if pos.y < zone {
            Some(ScreenEdge::Top)
        } else if pos.y > 1080.0 - zone {  // Should use actual screen height
            Some(ScreenEdge::Bottom)
        } else {
            None
        }
    }

    /// Calculate swipe direction from start to end position
    fn calculate_direction(
        &self,
        start: &Point<f64, Logical>,
        end: &Point<f64, Logical>,
    ) -> SwipeDirection {
        let dx = end.x - start.x;
        let dy = end.y - start.y;

        if dx.abs() > dy.abs() {
            if dx > 0.0 {
                SwipeDirection::Right
            } else {
                SwipeDirection::Left
            }
        } else {
            if dy > 0.0 {
                SwipeDirection::Down
            } else {
                SwipeDirection::Up
            }
        }
    }
}
```

## Wayland Protocol Integration

```rust
// Send gesture events to client surfaces
pub trait GestureHandler {
    fn tap(&self, surface: &WlSurface, position: Point<f64, Logical>, count: u32);

    fn long_press(&self, surface: &WlSurface, position: Point<f64, Logical>, duration: Duration);

    fn swipe(
        &self,
        surface: &WlSurface,
        start: Point<f64, Logical>,
        end: Point<f64, Logical>,
        direction: SwipeDirection,
        velocity: f64,
    );

    fn pinch(
        &self,
        surface: &WlSurface,
        center: Point<f64, Logical>,
        scale: f64,
        phase: PinchPhase,
    );

    fn rotate(
        &self,
        surface: &WlSurface,
        center: Point<f64, Logical>,
        angle: f64,
        phase: RotatePhase,
    );
}
```

## Configuration

### System-wide Gesture Settings

```toml
# /etc/rustica/gestures.conf
[general]
# Enable/disable gesture system
enabled = true

# Reduced sensitivity mode (for accessibility)
reduced_sensitivity = false

[tap]
# Maximum movement for tap recognition (pixels)
tap_slop = 16.0

# Maximum time for tap (ms)
tap_timeout = 400

[long_press]
# Minimum time for long press (ms)
timeout = 500

[swipe]
# Minimum distance for swipe (pixels)
min_distance = 80.0

# Minimum velocity for swipe (pixels/second)
min_velocity = 100.0

[edge_swipe]
# Width of edge detection zone (pixels)
zone_width = 24.0

[pinch]
# Pinch gesture sensitivity
sensitivity = 1.0

[rotate]
# Rotation gesture sensitivity
sensitivity = 1.0

[three_finger_swipe]
# Enable three-finger swipe for navigation
enabled = true
```

### Per-App Gesture Overrides

```toml
# ~/.config/rustica/gestures/apps.conf
["org.mozilla.firefox"]
# Disable pinch-to-zoom in Firefox (use built-in)
disable_pinch = true

["org.gnome.Nautilus"]
# Enable two-finger swipe for back/forward
enable_two_finger_swipe = true
```

## Accessibility Features

1. **Reduced Sensitivity Mode**: 2x slower gesture recognition
2. **Gesture Hold Delay**: Extra time before gesture is recognized
3. **Single-Finger Alternatives**: All multi-finger gestures have single-finger equivalents
4. **Gesture Feedback**: Visual/haptic feedback when gestures are recognized
5. **Disable Gesture Region**: Areas of screen where gestures don't trigger

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tap() {
        let mut recognizer = GestureRecognizer::new(GestureConfig::default());
        let mut session = TouchSession::new();

        let touch = TouchPoint {
            id: 0,
            position: Point::new(100.0, 100.0),
            start_position: Point::new(100.0, 100.0),
            pressure: None,
            major: None,
            minor: None,
            start_time: Instant::now(),
            update_time: Instant::now(),
        };

        session.add_touch(touch.clone());
        recognizer.handle_touch_down(&touch, &session);

        // Simulate quick tap (no movement, short duration)
        let touch_up = touch.with_update_time(Instant::now() + Duration::from_millis(200));
        let gesture = recognizer.handle_touch_up(&touch_up, &session);

        assert!(matches!(gesture, Some(Gesture::Tap { count: 1, .. })));
    }

    #[test]
    fn test_long_press() {
        let mut recognizer = GestureRecognizer::new(GestureConfig::default());
        let mut session = TouchSession::new();

        let touch = TouchPoint {
            id: 0,
            position: Point::new(100.0, 100.0),
            start_position: Point::new(100.0, 100.0),
            pressure: None,
            major: None,
            minor: None,
            start_time: Instant::now(),
            update_time: Instant::now(),
        };

        session.add_touch(touch.clone());
        recognizer.handle_touch_down(&touch, &session);

        // Motion after long press timeout should emit long press
        let touch_motion = touch.with_update_time(Instant::now() + Duration::from_millis(600));
        let gesture = recognizer.handle_touch_motion(&touch_motion, &session);

        assert!(matches!(gesture, Some(Gesture::LongPress { .. })));
    }

    #[test]
    fn test_swipe_right() {
        let mut recognizer = GestureRecognizer::new(GestureConfig::default());
        let mut session = TouchSession::new();

        let touch = TouchPoint {
            id: 0,
            position: Point::new(100.0, 100.0),
            start_position: Point::new(100.0, 100.0),
            pressure: None,
            major: None,
            minor: None,
            start_time: Instant::now(),
            update_time: Instant::now(),
        };

        session.add_touch(touch.clone());
        recognizer.handle_touch_down(&touch, &session);

        // Fast motion to the right
        let touch_motion = TouchPoint {
            id: 0,
            position: Point::new(300.0, 100.0),  // 200px right
            start_position: Point::new(100.0, 100.0),
            pressure: None,
            major: None,
            minor: None,
            start_time: Instant::now(),
            update_time: Instant::now() + Duration::from_millis(200),
        };

        let gesture = recognizer.handle_touch_up(&touch_motion, &session);

        assert!(matches!(
            gesture,
            Some(Gesture::Swipe { direction: SwipeDirection::Right, .. })
        ));
    }

    #[test]
    fn test_pinch_zoom() {
        let mut recognizer = GestureRecognizer::new(GestureConfig::default());
        let mut session = TouchSession::new();

        let touch1 = TouchPoint {
            id: 0,
            position: Point::new(100.0, 100.0),
            start_position: Point::new(100.0, 100.0),
            pressure: None,
            major: None,
            minor: None,
            start_time: Instant::now(),
            update_time: Instant::now(),
        };

        let touch2 = TouchPoint {
            id: 1,
            position: Point::new(200.0, 100.0),
            start_position: Point::new(200.0, 100.0),
            pressure: None,
            major: None,
            minor: None,
            start_time: Instant::now(),
            update_time: Instant::now(),
        };

        session.add_touch(touch1.clone());
        session.add_touch(touch2.clone());

        recognizer.handle_touch_down(&touch1, &session);
        recognizer.handle_touch_down(&touch2, &session);

        // Move fingers apart
        let touch1_out = TouchPoint {
            id: 0,
            position: Point::new(50.0, 100.0),
            start_position: Point::new(100.0, 100.0),
            pressure: None,
            major: None,
            minor: None,
            start_time: Instant::now(),
            update_time: Instant::now(),
        };

        let touch2_out = TouchPoint {
            id: 1,
            position: Point::new(250.0, 100.0),
            start_position: Point::new(200.0, 100.0),
            pressure: None,
            major: None,
            minor: None,
            start_time: Instant::now(),
            update_time: Instant::now(),
        };

        let gesture = recognizer.handle_touch_motion(&touch1_out, &session);

        assert!(matches!(gesture, Some(Gesture::Pinch { scale, .. } if scale > 1.0)));
    }
}
```

## Integration Points

1. **Compositor**: Receive Smithay touch events, forward to gesture engine
2. **Clients**: Receive gesture events via Wayland protocol
3. **Accessibility**: Reduced sensitivity mode, gesture alternatives
4. **Settings UI**: Configure gesture sensitivity, disable specific gestures
5. **Input Devices**: Touch device calibration and sensitivity

## Performance Considerations

1. **Event Batching**: Process multiple touch events per frame
2. **Gesture Coalescing**: Merge rapid pinch/rotate updates
3. **Early Rejection**: Quickly reject invalid gestures (e.g., too much movement for tap)
4. **Lazy Evaluation**: Only compute complex gestures when needed

## Security Considerations

1. **Gesture Isolation**: Apps can only receive gestures for their own surfaces
2. **Privacy**: No gesture logging or telemetry
3. **System Gestures**: Global gestures (edge swipes) have higher priority

## Future Enhancements

1. **Custom Gesture Registration**: Apps can register custom gesture patterns
2. **Gesture Macros**: Record and replay gesture sequences
3. **Machine Learning**: Adaptive gesture recognition based on user behavior
4. **Stylus Support**: Palm rejection and pressure-based gestures
5. **Gesture Preview**: Visual feedback before gesture completion

## Dependencies

```toml
[dependencies]
smithay = { git = "https://github.com/Smithay/smithay" }
zbus = "4"
libc = "0.2"
```