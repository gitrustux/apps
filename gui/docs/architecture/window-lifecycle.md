# Window Lifecycle Contract

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Window Management

## Overview

This specification defines the complete lifecycle of a window from creation to destruction in Rustica Shell. It ensures **clean resource management**, **proper focus tracking**, and **<100ms window creation time**.

## Window States

```rust
pub enum WindowState {
    /// Window is being created
    Creating,

    /// Window is visible and interactive
    Active,

    /// Window is visible but not focused
    Inactive,

    /// Window is minimized/hidden
    Hidden,

    /// Window is maximized within its workspace
    Maximized,

    /// Window occupies full screen (no decorations)
    Fullscreen,

    /// Window is being destroyed
    Destroying,

    /// Window has been destroyed
    Destroyed,
}
```

## State Machine

```
                    ┌───────────┐
                    │  Creating │
                    └─────┬─────┘
                          │
                          ▼
    ┌──────────────────────────────────────┐
    │           Active (Focused)            │◄────┐
    └──────────────────────────────────────┘     │
           │                    ▲               │
    user minimizes          user activates     │
           │                    │               │
           ▼                    │               │
    ┌───────────┐        ┌──────┴──────────────┘
    │  Hidden   │        │  Inactive (Unfocused)│
    └───────────┘        └─────────────────────┘
           │                    ▲
           │ user maximizes    │ user activates
           ▼                    │
    ┌───────────┐                │
    │ Maximized │────────────────┘
    └─────┬─────┘
          │ user requests fullscreen
          ▼
    ┌───────────┐
    │ Fullscreen │
    └─────┬─────┘
          │ user closes OR app requests destroy
          ▼
    ┌───────────┐
    │ Destroying│
    └─────┬─────┘
          │ cleanup complete
          ▼
    ┌───────────┐
    │ Destroyed │
    └───────────┘
```

## Window Creation Sequence

### Phase 1: Initial Creation (<50ms)

```rust
// 1. Application requests window creation
let surface_id = compositor.create_surface(CreateSurfaceRequest {
    app_id: "com.example.myapp",
    size: (800, 600),
    title: "My App".into(),
    capabilities: vec![Capability::GpuRendering],
});

// 2. Compositor creates window state
let window = Window::new(surface_id);
window.set_state(WindowState::Creating);
```

### Phase 2: Capability Handshake (<20ms)

```rust
// 3. Compositor requests capabilities from kernel
let granted = kernel.request_capabilities(
    app_pid,
    vec![Capability::GpuRendering { memory_mb: 256 }]
);

// 4. If granted, create graphics resources
if granted {
    window.create_surface();
    window.create_epg_context();
}
```

### Phase 3: Window Display (<30ms)

```rust
// 5. Configure window surface
compositor.configure_surface(surface_id, Configure {
    size: (800, 600),
    state: WindowState::Active,
    framerate: 60,
});

// 6. Add to window list
compositor.add_window(window);

// 7. Send first configure event to app
app.send_event(WindowEvent::Configure {
    surface: surface_id,
    size: (800, 600),
    state: WindowState::Active,
});

// 8. Window is now visible
window.set_state(WindowState::Active);
```

**Total Target: <100ms**

## Window Destruction Sequence

### Initiation

```rust
// Can be initiated by:
// 1. User closing window (close button, Alt+F4)
// 2. Application requesting close
// 3. Compositor terminating (shutdown)

enum CloseRequest {
    UserInitiated,
    AppInitiated,
    SystemShutdown,
}
```

### Graceful Shutdown Process

```rust
pub fn close_window(window: &mut Window, request: CloseRequest) {
    window.set_state(WindowState::Destroying);

    // 1. Notify application
    window.send_event(WindowEvent::Close {
        surface: window.surface_id(),
    });

    // 2. Wait for application acknowledgment (with timeout)
    let deadline = Duration::from_secs(5);
    match window.wait_for_ack(deadline) {
        Ok(_) => {
            // App acknowledged, destroy surface
            window.destroy_surface();
        }
        Err(_) => {
            // Timeout, force destroy
            log::warn!("Window close timeout, force destroying");
            window.destroy_surface_forcibly();
        }
    }

    // 3. Clean up resources
    window.release_graphics_resources();
    window.release_capabilities();

    // 4. Remove from window list
    compositor.remove_window(window.id());

    // 5. Final state
    window.set_state(WindowState::Destroyed);
}
```

## Parent-Child Window Relationships

### Window Hierarchy

```rust
pub struct Window {
    id: WindowId,
    parent: Option<WindowId>,
    children: Vec<WindowId>,
    // ...
}

impl Window {
    // Create child window (dialog, menu, tooltip)
    pub fn create_child(&mut self, config: ChildConfig) -> WindowId {
        let child = Window::new_child(config, Some(self.id));
        self.children.push(child.id());
        compositor.add_window(child);
        child.id()
    }

    // When parent is hidden/minimized
    pub fn hide(&mut self) {
        self.set_state(WindowState::Hidden);

        // Hide all children
        for child_id in &self.children {
            if let Some(child) = compositor.get_window(*child_id) {
                child.hide();
            }
        }
    }
}
```

### Child Window Types

| Type | Behavior | Example |
|------|----------|---------|
| **Modal** | Blocks parent input, centered on parent | Dialog |
| **Popup** | Can be clicked outside, dismissible | Menu, Dropdown |
| **Tooltip** | No interaction, short-lived | Tooltip |
| **Utility** | No minimize/maximize, stays on top | Palette |

## Focus Management

### Focus Model

```rust
pub enum FocusTarget {
    /// Main window surface
    Surface(WindowId),

    /// Child surface (dialog, menu, etc.)
    Child(WindowId),

    /// Null (no focus - click to focus)
    Null,
}

pub fn set_focus(target: FocusTarget) {
    match target {
        FocusTarget::Surface(id) => {
            // Clear focus from previous
            if let Some(prev) = compositor.focused_window() {
                prev.send_event(WindowEvent::FocusOut);
            }

            // Set new focus
            if let Some(window) = compositor.get_window(id) {
                window.send_event(WindowEvent::FocusIn);
                compositor.set_focused_window(window);
            }
        }
        FocusTarget::Null => {
            // Clear all focus
            if let Some(prev) = compositor.focused_window() {
                prev.send_event(WindowEvent::FocusOut);
            }
            compositor.clear_focused_window();
        }
    }
}
```

### Focus Notifications

```rust
pub enum FocusEvent {
    /// Window gained focus
    In {
        reason: FocusReason,  // Click, key press, API request
    },

    /// Window lost focus
    Out {
        reason: FocusReason,  // Click outside, API request
    },
}

pub enum FocusReason {
    /// User clicked on window
    Pointer,

    /// Keyboard navigation (Tab, Alt+Tab, etc.)
    Keyboard,

    /// Application requested focus
    Application,

    /// Compositor-initiated (startup, window close)
    Compositor,

    /// Unknown (system-initiated)
    Other,
}
```

## Window Properties

### Mutable Properties

```rust
pub struct WindowProperties {
    // Appearance
    pub title: String,
    pub icon: Option<Icon>,
    pub resizable: bool,
    pub decorations: bool,      // Title bar, borders
    pub always_on_top: bool,

    // State
    pub position: Option<(i32, i32)>,
    pub size: (u32, u32),
    pub min_size: Option<(u32, u32)>,
    pub max_size: Option<(u32, u32)>,

    // Behavior
    pub accepts_focus: bool,
    pub close_on_escape: bool,

    // Workspace
    pub workspace: Option<WorkspaceId>,
    pub sticky: bool,  // Visible on all workspaces
}

impl Window {
    pub fn set_properties(&mut self, props: WindowProperties) {
        self.properties = props;

        // Apply to surface
        self.update_surface();

        // Notify app of changes
        self.send_event(WindowEvent::PropertiesChanged {
            properties: self.properties.clone(),
        });
    }
}
```

### Window Metadata

```rust
pub struct WindowMetadata {
    // Application info
    pub app_id: String,
    pub pid: Pid,

    // Creation time
    pub created_at: Instant,

    // Capabilities
    pub capabilities: Vec<Capability>,

    // Security context
    pub sandboxed: bool,
    pub trust_level: TrustLevel,
}
```

## XDG Shell Integration

### Surface Roles

```rust
pub enum SurfaceRole {
    /// Standard window
    Toplevel,

    /// Popup/child window
    Popup {
        parent: SurfaceId,
        position: (i32, i32),
    },

    /// Layer surface (panels, lock screens)
    Layer {
        layer: Layer,
        exclusive_zone: Option<u32>,
    },
}
```

### XDG Shell States

```rust
// Standard XDG shell states
pub enum XdgState {
    /// Window is being resized by user or compositor
    Resizing(Size),

    /// Window has been activated (gained focus)
    Activated,

    /// Window has been deactivated (lost focus)
    Deactivated,

    /// Window should close
    Close,
}
```

## Performance Requirements

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Window create | <100ms | Request to visible |
| Window destroy | <50ms | Close to cleaned up |
| Focus switch | <50ms | Previous out to next in |
| State change | <100ms | Request to applied |
| Property update | <50ms | Request to notified |

## Resource Management

### Graphics Resources

```rust
pub struct GraphicsResources {
    // EGL surface
    egl_surface: Option<EGLSurface>,

    // Buffers
    buffers: Vec<Buffer>,

    // Textures
    textures: Vec<Texture>,
}

impl Window {
    pub fn release_graphics_resources(&mut self) {
        // Release in reverse order of creation
        if let Some(surface) = self.egl_surface.take() {
            egl.destroy_surface(surface);
        }

        self.buffers.clear();
        self.textures.clear();
    }
}
```

### Capability Release

```rust
impl Window {
    pub fn release_capabilities(&mut self) {
        // Tell kernel to revoke capabilities
        kernel::revoke_capabilities(
            self.metadata.pid,
            &self.metadata.capabilities
        );
    }
}
```

## Error Handling

```rust
pub enum WindowError {
    /// Creation failed
    CreationFailed(String),

    /// Graphics initialization failed
    GraphicsError(String),

    /// Capability request denied
    CapabilityDenied(Capability),

    /// Surface destroyed unexpectedly
    SurfaceLost,

    /// Application not responding
    AppTimeout,
}

impl Window {
    pub fn create_with_error_handling(
        config: WindowConfig,
    ) -> Result<Window, WindowError> {
        // Try to create window
        match Self::new(config) {
            Ok(window) => Ok(window),
            Err(e) => {
                // Log error
                log::error!("Window creation failed: {:?}", e);

                // Notify user (via notification)
                compositor.show_notification(Notification {
                    title: "Window Creation Failed".into(),
                    body: e.to_string(),
                    urgency: Urgency::Error,
                });

                Err(e)
            }
        }
    }
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-comp/src/
│   ├── window/
│   │   ├── mod.rs              # Window module
│   │   ├── window.rs           # Window struct
│   │   ├── state.rs            # State machine
│   │   ├── focus.rs            # Focus management
│   │   ├── properties.rs       # Window properties
│   │   ├── lifecycle.rs        # Creation/destruction
│   │   └── xdg_shell.rs        # XDG shell integration
│   └── ...
│
└── libs/librustica/src/
    └── window/
        ├── mod.rs
        ├── config.rs            # Window configuration
        ├── metadata.rs          # Window metadata
        └── event.rs             # Window events
```

## Example Usage

### Creating a Window

```rust
use librustica::window::*;

// Create window with configuration
let config = WindowConfig {
    title: "My Application".into(),
    size: (1024, 768),
    min_size: Some((640, 480)),
    resizable: true,
    decorations: true,
    accepts_focus: true,
};

let window = match Window::create(config) {
    Ok(w) => w,
    Err(e) => {
        eprintln!("Failed to create window: {}", e);
        std::process::exit(1);
    }
};

// Run the window
window.run(|event| {
    match event {
        WindowEvent::Configure { size, .. } => {
            println!("Window configured to {:?}", size);
        }
        WindowEvent::Close => {
            println!("Window close requested");
            window.close();
        }
        _ => {}
    }
});
```

## Success Criteria

- [ ] Window creation completes in <100ms
- [ ] Window destruction completes in <50ms with no leaks
- [ ] Focus tracking works correctly
- [ ] Parent-child relationships enforced
- [ ] All state transitions work
- [ ] Resource cleanup verified
- [ ] Error handling works
- [ ] Performance targets met

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Zombie windows | Force destroy after timeout, watchdog monitoring |
| Resource leaks | RAII patterns, automatic cleanup |
| Focus loops | Focus tracking algorithm, detect cycles |
| State corruption | State machine with validation |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [Wayland XDG Shell Protocol](https://wayland.freedesktop.org/docs/html/xdg-shell/xdg-shell-spec.html)
- [Smithay Window Management](https://docs.rs/smithay/*/smithay/wayland/xdg_shell/index.html)
- [GTK Window Lifecycle](https://docs.gtk.org/gtk4/gtk4/struct.Window.html)
- [Qt Window Management](https://doc.qt.io/qt-6/qwindow.html)
