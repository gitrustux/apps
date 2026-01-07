# Wayland Compositor Implementation Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Compositor (rustica-comp)

## Overview

This specification defines the implementation of Rustica Compositor, a Wayland compositor built with Smithay. It provides **60 FPS stable rendering**, **multi-monitor support**, **hardware acceleration**, and **seamless mobile/desktop switching**.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Rustica Compositor                           │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │              Smithay Framework Layer                      │  │
│  │  - Wayland protocol handling                            │  │
│  │  - Surface management                                   │  │
│  │  - Input processing                                     │  │
│  └───────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                   Compositor State                        │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │  │
│  │  │   Windows   │  │  Displays   │  │    Input    │       │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘       │  │
│  └───────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                   Rustica Extensions                      │  │
│  │  - Security layer capabilities                           │  │
│  │  - Mobile-specific protocols                             │  │
│  │  - Workspace management                                  │  │
│  └───────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │              Rendering Backend                            │  │
│  │  ┌────────────┐  ┌────────────┐                          │  │
│  │  │ EGL/OpenGL │  │  Pixman    │                          │  │
│  │  │ (Hardware) │  │ (Software) │                          │  │
│  │  └────────────┘  └────────────┘                          │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    RUSTUX Microkernel                            │
│  - GPU capability requests                                      │
│  - Input device access                                          │
│  - Display mode setting                                         │
└─────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
/var/www/rustux.com/prod/apps/gui/rustica-comp/
├── Cargo.toml
├── src/
│   ├── main.rs                  # Entry point
│   ├── compositor.rs            # Main compositor struct
│   ├── state.rs                 # Compositor state
│   ├── backend/
│   │   ├── mod.rs
│   │   ├── drm.rs               # DRM/KMS backend
│   │   ├── egl.rs               # EGL rendering
│   │   └── pixman.rs            # Software rendering fallback
│   ├── window/
│   │   ├── mod.rs
│   │   ├── window.rs            # Window management
│   │   ├── tiling.rs            # Tiling layout
│   │   └── stacking.rs          # Stacking layout
│   ├── input/
│   │   ├── mod.rs
│   │   ├── keyboard.rs
│   │   ├── pointer.rs
│   │   └── touch.rs
│   ├── output/
│   │   ├── mod.rs
│   │   ├── display.rs           # Display management
│   │   └── scale.rs             # DPI scaling
│   ├── shell/
│   │   ├── mod.rs
│   │   ├── xdg_shell.rs         # XDG shell protocol
│   │   └── layer_shell.rs       # Layer shell protocol
│   ├── security/
│   │   ├── mod.rs
│   │   ├── capability.rs        # Kernel capability bridge
│   │   └── sandbox.rs           # Sandbox enforcement
│   └── mobile/
│       ├── mod.rs
│       ├── gestures.rs          # Touch gestures
│       └── rotation.rs          # Screen rotation
└── resources/
    └── backgrounds/             # Default wallpapers
        ├── wallpaper_d.png
        ├── wallpaper_m.png
        └── wallpaper_lock.png
```

## Cargo.toml

```toml
[package]
name = "rustica-comp"
version = "0.1.0"
edition = "2021"

[dependencies]
# Smithay - Wayland compositor framework
smithay = { version = "0.18", features = [
    "use_system_lib",  # Use system libraries for EGL, libinput, etc.
    "backend_drm",     # DRM/KMS backend for hardware rendering
    "backend_libinput",# libinput for input handling
    "backend_udev",    # udev for device discovery
    "renderer_gl",     # OpenGL/EGL renderer
    "renderer_pixman", # Software rendering fallback
] }

# Wayland protocols
wayland-server = "0.31"
wayland-sys = "0.31"

# Rendering
gl = "0.14"
egl = { package = "eglow", version = "0.6" }

# Input
libinput = "0.8"

# DRM/KMS
drm = "0.12"
gbm = "0.13"

# Utilities
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
thiserror = "1.0"

# RUSTUX kernel IPC
rustux-ipc = { path = "../../../kernel/src/ipc" }

# librustica libraries
librustica = { path = "../libs/librustica" }
librustica-accessibility = { path = "../libs/librustica-accessibility" }
librustica-ime = { path = "../libs/librustica-ime" }
librustica-scaling = { path = "../libs/librustica-scaling" }
```

## Main Compositor Struct

```rust
use smithay::{
    backend::renderer::gles::GlesRenderer,
    delegate_compositor, delegate_data_device, delegate_layer_shell,
    delegate_output, delegate_seat, delegate_shm, delegate_xdg_shell,
    desktop::{utils::send_frames_surface_tree, Space},
    input::{Seat, SeatHandler, SeatState},
    reexports::wayland_server::{
        backend::GlobalId, protocol::wl_output::WlOutput,
    },
    wayland::{
        compositor::CompositorState,
        data_device::DataDeviceState,
        output::OutputState,
        shell::xdg::XdgShellState,
    },
};

pub struct RusticaCompositor {
    // Display
    display: Display<RusticaCompositor>,

    // Smithay state
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    layer_shell_state: LayerShellState,
    seat_state: SeatState<RusticaCompositor>,
    data_device_state: DataDeviceState,

    // Desktop space
    space: Space<WindowElement>,

    // Outputs (displays)
    outputs: Vec<Output>,

    // Seats (input devices)
    seats: Vec<Seat<RusticaCompositor>>,

    // Rendering
    renderer: Option<GlesRenderer>,

    // Kernel IPC
    kernel_bridge: KernelBridge,

    // Extensions
    mobile_mode: bool,
    scale_manager: ScaleManager,
    accessibility: AtSpiRegistry,
    ime_manager: ImeManager,
}

impl RusticaCompositor {
    pub fn new() -> Result<Self> {
        // Create Wayland display
        let display = Display::new()?;

        // Initialize Smithay state
        let compositor_state = CompositorState::new::<Self>(&display);
        let xdg_shell_state = XdgShellState::new::<Self>(&display);
        let layer_shell_state = LayerShellState::new::<Self>(&display);
        let seat_state = SeatState::new();
        let data_device_state = DataDeviceState::new::<Self>(&display);

        // Connect to kernel
        let kernel_bridge = KernelBridge::connect()?;

        // Initialize extensions
        let scale_manager = ScaleManager::new();
        let accessibility = AtSpiRegistry::new().await?;
        let ime_manager = ImeManager::new()?;

        // Detect mobile mode from kernel
        let mobile_mode = kernel_bridge.is_mobile_device()?;

        Ok(Self {
            display,
            compositor_state,
            xdg_shell_state,
            layer_shell_state,
            seat_state,
            data_device_state,
            space: Space::default(),
            outputs: Vec::new(),
            seats: Vec::new(),
            renderer: None,
            kernel_bridge,
            mobile_mode,
            scale_manager,
            accessibility,
            ime_manager,
        })
    }

    /// Run the compositor event loop
    pub fn run(&mut self) -> ! {
        // Initialize DRM backend
        let (mut backend, mut renderer) = self.init_drm_backend()?;

        // Initialize seats (input devices)
        self.init_seats();

        // Run event loop
        loop {
            // Handle Wayland events
            self.display.dispatch(&mut backend, &mut renderer)
                .expect("display.dispatch failed");

            // Render
            self.render(&mut backend, &mut renderer);
        }
    }

    fn init_drm_backend(
        &mut self,
    ) -> Result<(DrmBackend<GlesRenderer>, GlesRenderer)> {
        use smithay::backend::drm::{DrmBackend, DrmEvent, GlesRenderer};

        // Initialize udev
        let mut backend = UdevBackend::new("rustica-comp")?;

        // Create renderer
        let renderer = GlesRenderer::new(
            backend.renderer_formats(),
            backend.display_formats(),
        )?;

        // Listen for device events
        for (dev, _) in backend.device_events() {
            self.add_output(dev, &renderer)?;
        }

        Ok((backend, renderer))
    }

    fn init_seats(&mut self) {
        use smithay::backend::libinput::{Libinput, LibinputInputBackend};

        // Initialize libinput
        let mut context = Libinput::new_with_udev(
            UdevContext::new().unwrap()
        ).unwrap();

        // Add seat
        let seat = self.seat_state.new_wl_seat(
            &self.display,
            "seat0",
            None,
        );

        self.seats.push(seat);
    }
}
```

## Window Management

### Window Element

```rust
use smithay::desktop::{Window, WindowSurface};

pub enum WindowElement {
    Xdg(Window),
    Layer(LayerSurface),
}

impl WindowElement {
    /// Get window title
    pub fn title(&self) -> String {
        match self {
            Self::Xdg(w) => w.title(),
            Self::Layer(l) => l.layer().name(),
        }
    }

    /// Get window app ID
    pub fn app_id(&self) -> String {
        match self {
            Self::Xdg(w) => w.app_id(),
            Self::Layer(l) => String::new(),
        }
    }

    /// Check if window is alive
    pub fn alive(&self) -> bool {
        match self {
            Self::Xdg(w) => w.alive(),
            Self::Layer(l) => l.alive(),
        }
    }

    /// Send frame callback
    pub fn send_frame(&self, output: &Output, time: u32) {
        match self {
            Self::Xdg(w) => {
                send_frames_surface_tree(
                    w.toplevel().expect("no toplevel"),
                    output,
                    time,
                    None,
                );
            }
            Self::Layer(l) => {
                l.send_frame(output, time);
            }
        }
    }
}
```

### Tiling Layout

```rust
pub struct TilingLayout {
    gaps: Size,
    layout: LayoutDirection,
}

pub enum LayoutDirection {
    Horizontal,
    Vertical,
    Grid,
}

impl TilingLayout {
    pub fn new() -> Self {
        Self {
            gaps: Size { width: 8, height: 8 },
            layout: LayoutDirection::Horizontal,
        }
    }

    /// Arrange windows in tiling layout
    pub fn arrange(&self, windows: &[WindowElement], area: Rectangle) {
        if windows.is_empty() {
            return;
        }

        match self.layout {
            LayoutDirection::Horizontal => {
                let total_width = area.width - (self.gaps.width * (windows.len() - 1) as i32);
                let window_width = total_width / windows.len() as i32;

                for (i, window) in windows.iter().enumerate() {
                    let x = area.x + (i as i32 * (window_width + self.gaps.width));
                    let geometry = Rectangle {
                        x,
                        y: area.y,
                        width: window_width,
                        height: area.height,
                    };

                    window.set_geometry(geometry);
                }
            }
            LayoutDirection::Vertical => {
                let total_height = area.height - (self.gaps.height * (windows.len() - 1) as i32);
                let window_height = total_height / windows.len() as i32;

                for (i, window) in windows.iter().enumerate() {
                    let y = area.y + (i as i32 * (window_height + self.gaps.height));
                    let geometry = Rectangle {
                        x: area.x,
                        y,
                        width: area.width,
                        height: window_height,
                    };

                    window.set_geometry(geometry);
                }
            }
            LayoutDirection::Grid => {
                let cols = (windows.len() as f32).sqrt().ceil() as usize;
                let rows = (windows.len() + cols - 1) / cols;

                let window_width = (area.width - self.gaps.width * (cols - 1) as i32) / cols as i32;
                let window_height = (area.height - self.gaps.height * (rows - 1) as i32) / rows as i32;

                for (i, window) in windows.iter().enumerate() {
                    let col = i % cols;
                    let row = i / cols;

                    let x = area.x + (col as i32 * (window_width + self.gaps.width));
                    let y = area.y + (row as i32 * (window_height + self.gaps.height));

                    let geometry = Rectangle {
                        x,
                        y,
                        width: window_width,
                        height: window_height,
                    };

                    window.set_geometry(geometry);
                }
            }
        }
    }

    /// Cycle layout direction
    pub fn cycle_layout(&mut self) {
        self.layout = match self.layout {
            LayoutDirection::Horizontal => LayoutDirection::Vertical,
            LayoutDirection::Vertical => LayoutDirection::Grid,
            LayoutDirection::Grid => LayoutDirection::Horizontal,
        };
    }
}
```

## Input Handling

### Keyboard

```rust
impl SeatHandler for RusticaCompositor {
    fn keyboard_focus(&mut self, focus: Option<&KeyboardFocusTarget>) {
        let seat = self.seats.first_mut().unwrap();

        if let Some(target) = focus {
            seat.keyboard.set_focus(self, target);
        } else {
            seat.keyboard.clear_focus(self);
        }
    }

    fn keyboard(&mut self, keysym: Keysym, state: KeyState) {
        // Handle compositor shortcuts
        if state == KeyState::Pressed {
            match keysym {
                Keysym::Super_L => {
                    // Super key pressed - track for shortcuts
                    self.super_key_pressed = true;
                }
                Keysym::Return if self.super_key_pressed => {
                    // Super+Enter - open terminal
                    self.launch_terminal();
                }
                Keysym::d if self.super_key_pressed => {
                    // Super+D - toggle layout
                    self.tiling_layout.cycle_layout();
                    self.arrange_windows();
                }
                _ => {}
            }
        }

        // Forward to focused window
        let seat = self.seats.first_mut().unwrap();
        seat.keyboard.input(keysym, state);
    }
}
```

### Pointer

```rust
impl SeatHandler for RusticaCompositor {
    fn pointer_motion(&mut self, position: Point<f32>) {
        let seat = self.seats.first_mut().unwrap();
        seat.pointer.motion(
            self,
            position,
            None,  // serial
            Duration::from_millis(0),
        );
    }

    fn pointer_button(&mut self, button: MouseButton, state: ButtonState) {
        let seat = self.seats.first_mut().unwrap();

        if state == ButtonState::Pressed {
            // Find surface under pointer
            let under = self.space.element_under(self.pointer_location);

            if let Some(window) = under {
                // Focus window
                self.focus_window(window);

                // Move to top
                self.space.raise_element(window, true);
            }
        }

        seat.pointer.button(button, state);
    }

    fn pointer_axis(&mut self, axis: Axis, amount: AxisAmount) {
        let seat = self.seats.first_mut().unwrap();
        seat.pointer.axis(self, axis, amount);
    }
}
```

### Touch

```rust
impl SeatHandler for RusticaCompositor {
    fn touch_down(
        &mut self,
        slot: TouchSlot,
        location: Point<f32>,
    ) {
        let seat = self.seats.first_mut().unwrap();

        // Find surface under touch
        let under = self.space.element_under(location);

        if let Some(window) = under {
            // Focus window
            self.focus_window(window);

            // Move to top
            self.space.raise_element(window, true);
        }

        seat.touch.down(slot, location);
    }

    fn touch_motion(&mut self, slot: TouchSlot, location: Point<f32>) {
        let seat = self.seats.first_mut().unwrap();
        seat.touch.motion(slot, location);
    }

    fn touch_up(&mut self, slot: TouchSlot) {
        let seat = self.seats.first_mut().unwrap();
        seat.touch.up(slot);
    }
}
```

## Mobile Gestures

```rust
pub struct GestureRecognizer {
    gestures: Vec<ActiveGesture>,
}

pub struct ActiveGesture {
    gesture_type: GestureType,
    start_point: Point<f32>,
    current_point: Point<f32>,
    start_time: Instant,
}

pub enum GestureType {
    Swipe(SwipeDirection),
    Pinch(f32),  // scale factor
    Rotate(f32), // angle in degrees
}

pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}

impl GestureRecognizer {
    pub fn new() -> Self {
        Self {
            gestures: Vec::new(),
        }
    }

    /// Handle touch down
    pub fn touch_down(&mut self, slot: TouchSlot, position: Point<f32>) {
        self.gestures.push(ActiveGesture {
            gesture_type: GestureType::Swipe(SwipeDirection::Left), // Will be determined
            start_point: position,
            current_point: position,
            start_time: Instant::now(),
        });
    }

    /// Handle touch motion
    pub fn touch_motion(&mut self, slot: TouchSlot, position: Point<f32>) {
        if let Some(gesture) = self.gestures.last_mut() {
            gesture.current_point = position;

            // Calculate delta
            let delta = Point {
                x: position.x - gesture.start_point.x,
                y: position.y - gesture.start_point.y,
            };

            // Determine gesture type based on movement
            if delta.x.abs() > delta.y.abs() {
                // Horizontal swipe
                gesture.gesture_type = if delta.x > 0 {
                    GestureType::Swipe(SwipeDirection::Right)
                } else {
                    GestureType::Swipe(SwipeDirection::Left)
                };
            } else {
                // Vertical swipe
                gesture.gesture_type = if delta.y > 0 {
                    GestureType::Swipe(SwipeDirection::Down)
                } else {
                    GestureType::Swipe(SwipeDirection::Up)
                };
            }
        }
    }

    /// Handle touch up
    pub fn touch_up(&mut self, slot: TouchSlot) -> Option<Gesture> {
        if let Some(gesture) = self.gestures.pop() {
            let delta = Point {
                x: gesture.current_point.x - gesture.start_point.x,
                y: gesture.current_point.y - gesture.start_point.y,
            };

            let duration = gesture.start_time.elapsed();

            // Minimum swipe distance: 100px
            // Maximum swipe duration: 500ms
            let distance = (delta.x.powi(2) + delta.y.powi(2)).sqrt();

            if distance > 100.0 && duration < Duration::from_millis(500) {
                return Some(match gesture.gesture_type {
                    GestureType::Swipe(dir) => Gesture::Swipe {
                        start: gesture.start_point,
                        end: gesture.current_point,
                        direction: dir,
                        velocity: distance / duration.as_secs_f32(),
                    },
                    _ => unimplemented!(),
                });
            }
        }

        None
    }

    /// Handle two-finger pinch
    pub fn pinch(&mut self, scale: f32) -> Option<Gesture> {
        // Pinch to zoom
        if scale > 1.1 || scale < 0.9 {
            Some(Gesture::Pinch {
                scale,
                center: self.gestures.first()?.start_point,
            })
        } else {
            None
        }
    }
}
```

## Kernel IPC Bridge

```rust
pub struct KernelBridge {
    connection: Connection,
}

impl KernelBridge {
    pub fn connect() -> Result<Self> {
        let connection = Connection::connect_to_kernel()?;
        Ok(Self { connection })
    }

    /// Request GPU capability from kernel
    pub fn request_gpu(&self, pid: Pid, limits: GpuLimits) -> Result<GpuCapability> {
        let request = GuiRequest::RequestGpu {
            pid,
            limits,
        };

        self.connection.send(request)?;

        match self.connection.receive()? {
            GuiResponse::Granted { capability } => Ok(capability),
            GuiResponse::Denied { reason } => Err(Error::CapabilityDenied(reason)),
            GuiResponse::Error { message } => Err(Error::KernelError(message)),
        }
    }

    /// Check if device is mobile
    pub fn is_mobile_device(&self) -> Result<bool> {
        let request = GuiRequest::QueryDeviceType;

        self.connection.send(request)?;

        match self.connection.receive()? {
            GuiResponse::DeviceType { is_mobile } => Ok(is_mobile),
            _ => Ok(false),
        }
    }

    /// Set display mode
    pub fn set_display_mode(
        &self,
        output: &Output,
        mode: DisplayMode,
    ) -> Result<()> {
        let request = GuiRequest::SetDisplayMode {
            connector: output.name(),
            mode,
        };

        self.connection.send(request)?;

        match self.connection.receive()? {
            GuiResponse::Success => Ok(()),
            GuiResponse::Error { message } => Err(Error::KernelError(message)),
            _ => Err(Error::UnexpectedResponse),
        }
    }
}
```

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Frame time | <16ms | Render loop iteration |
| Input latency | <10ms | Input to display |
| Window create | <100ms | Request to visible |
| Memory base | <50MB | Idle compositor |
| FPS stability | 60 FPS ±1 | No drops below 55 |

## Build Instructions

```bash
# Build compositor
cd /var/www/rustux.com/prod/apps/gui/rustica-comp
cargo build --release

# Install system-wide
sudo cargo install --path .

# Run compositor
rustica-comp
```

## Success Criteria

- [ ] Smithay compositor initializes
- [ ] DRM/KMS backend works
- [ ] EGL rendering works
- [ ] Input handling works (keyboard, mouse, touch)
- [ ] Window management works (tiling, stacking)
- [ ] Multi-monitor support works
- [ ] DPI scaling works
- [ ] Performance targets met
- [ ] Mobile gestures work
- [ ] Kernel IPC works

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| DRM/KMS complexity | Start with single monitor, add multi-monitor later |
| GPU driver issues | Provide Pixman software fallback |
| Performance problems | Profile hot paths, optimize rendering |
| Mobile gesture conflicts | Make gesture recognition configurable |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [Smithay Documentation](https://docs.rs/smithay/)
- [Smithay Examples](https://github.com/Smithay/smithay/tree/main/examples)
- [Wayland Protocol](https://wayland.freedesktop.org/docs/html/)
- [DRM/KMS](https://www.kernel.org/doc/html/latest/gpu/drm-kms.html)
- [libinput](https://wayland.freedesktop.org/libinput/doc/latest/)
