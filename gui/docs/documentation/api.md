# Phase 11.3: API Documentation

## Overview

**Component**: API Documentation
**Purpose**: Comprehensive API reference for all public APIs
**Format**: Rustdoc + OpenAPI
**Location**: Generated from source code + `/var/www/rustux.com/prod/apps/gui/docs/api/`

## Goals

1. **Complete Coverage**: Document all public APIs
2. **Consistent Format**: Uniform documentation style
3. **Examples**: Code examples for every API
4. **Versioning**: Track API changes across versions
5. **Multiple Formats**: HTML, JSON, man pages

## API Documentation Structure

```
docs/api/
├── README.md                    # API documentation overview
├── components/                  # Component APIs
│   ├── compositor/              # Compositor API reference
│   │   ├── display.md
│   │   ├── surface.md
│   │   ├── output.md
│   │   └── input.md
│   ├── shell/                   # Shell APIs
│   │   ├── panel.md
│   │   ├── dock.md
│   │   └── launcher.md
│   ├── toolkit/                 # Toolkit API (librustica)
│   │   ├── widgets.md
│   │   ├── layout.md
│   │   ├── events.md
│   │   └── rendering.md
│   └── services/                # Service APIs
│       ├── settings-daemon.md
│       ├── notifications.md
│       └── power-management.md
├── protocols/                   # Protocol documentation
│   ├── wayland/                 # Wayland protocols
│   │   ├── rustica-shell.md
│   │   ├── rustica-layer.md
│   │   └── custom-protocols.md
│   └── dbus/                    # D-Bus interfaces
│       ├── org.rustica.Compositor.md
│       ├── org.rustica.Panel.md
│       └── org.rustica.Settings.md
└── formats/                     # Data formats
    ├── theme-format.md          # Theme file format
    ├── package-format.md         # Package metadata
    └── config-format.md          # Configuration files
```

## Rustdoc Standards

## Documentation Comments

### Module Documentation

```rust
//! Window management for the Rustica Wayland compositor.
//!
//! This module provides the core window management functionality including
//! window creation, positioning, sizing, and lifecycle management.
//!
//! # Architecture
//!
//! The window management system is built on Smithay's window management
//! capabilities and extends them with Rustica-specific features:
//!
//! - **Workspace Management**: Windows can be organized into virtual workspaces
//! - **Tiling**: Automatic tiling of windows in configurable layouts
//! - **Floating**: Traditional floating window management
//! - **Layer Shell**: Support for layer shell surfaces (panels, docks)
//!
//! # Core Types
//!
//! - [`Window`]: Represents a Wayland window/surface
//! - [`WindowManager`]: Manages all windows in the compositor
//! - [`Workspace`]: Represents a virtual workspace
//! - [`Layout`]: Window layout algorithm
//!
//! # Examples
//!
//! ## Creating a Window
//!
//! \`\`\`rust
//! use rustica_comp::Window;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let window = Window::builder()
//!     .title("My App")
//!     .size(800, 600)
//!     .create()?;
//! # Ok(())
//! # }
//! \`\`\`
//!
//! ## Managing Windows
//!
//! \`\`\`rust
//! use rustica_comp::{Compositor, WindowManager};
//!
//! # async fn example(compositor: &Compositor) -> Result<(), Box<dyn std::error::Error>> {
//! let manager = compositor.window_manager();
//!
//! // Map window to screen
//! manager.map_window(&window)?;
//!
//! // Focus window
//! manager.focus_window(&window)?;
//! # Ok(())
//! # }
//! \`\`\`
//!
//! # See Also
//!
//! - [`crate::surface`](surface) - Surface management
//! - [`crate::output`](output) - Output/monitor management
//! - [`crate::input`](input) - Input event handling
```

### Struct Documentation

```rust
/// A Wayland window/surface managed by the compositor.
///
/// Each [`Window`] represents a client's Wayland surface that has been
/// mapped to the screen. Windows have properties like title, size, and
/// position that can be controlled by both the client and the compositor.
///
/// # Lifecycle
///
/// Windows go through the following lifecycle:
///
/// 1. **Created**: Client creates a surface via Wayland protocol
/// 2. **Mapped**: Window is mapped to screen (visible)
/// 3. **Configured**: Window size/position is set
/// 4. **Unmapped**: Window is unmapped (hidden)
/// 5. **Destroyed**: Surface is destroyed
///
/// # Threading
///
/// [`Window`] methods can be called from any thread, but most operations
/// require being on the compositor's main event loop thread.
///
/// # Examples
///
/// \`\`\`rust
/// use rustica_comp::Window;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a new window
/// let window = Window::builder()
///     .title("Hello World")
///     .app_id("com.example.HelloWorld")
///     .size(800, 600)
///     .create()?;
///
/// // Check if window is mapped
/// if window.is_mapped() {
///     println!("Window is visible");
/// }
/// # Ok(())
/// # }
/// \`\`\`
///
/// # Panics
///
/// Panics can occur in the following situations:
///
/// - Calling surface methods after the surface has been destroyed
/// - Invalid size constraints (zero or negative dimensions)
/// - Thread safety violations (certain methods from wrong thread)
#[derive(Debug, Clone)]
pub struct Window {
    /// Unique identifier for this window
    id: u32,

    /// Window title as set by the client
    title: String,

    /// Application ID (e.g., "org.mozilla.firefox")
    app_id: String,

    /// Window geometry
    geometry: Geometry,

    /// Current window state
    state: WindowState,

    /// Wayland surface for this window
    surface: WlSurface,

    /// Output the window is currently displayed on
    output: Option<Output>,
}

impl Window {
    /// Creates a new [`Window`] with the specified properties.
    ///
    /// This is a convenience method for [`WindowBuilder::build`]. For more
    /// options, use [`WindowBuilder`] instead.
    ///
    /// # Arguments
    ///
    /// * `title` - Window title
    /// * `app_id` - Application ID in reverse DNS notation
    /// * `width` - Initial width in pixels
    /// * `height` - Initial height in pixels
    ///
    /// # Returns
    ///
    /// A [`Window`] handle or an error if window creation failed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Title or app_id is empty
    /// - Width or height is zero
    /// - Wayland display is not available
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// # use rustica_comp::Window;
    /// #
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let window = Window::new(
    ///     "My App",
    ///     "com.example.myapp",
    ///     800,
    ///     600,
    /// ).await?;
    /// # Ok(())
    /// # }
    /// \`\`\`
    pub async fn new(
        title: String,
        app_id: String,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        WindowBuilder::new()
            .title(title)
            .app_id(app_id)
            .size(width, height)
            .build()
            .await
    }

    /// Returns the window's unique identifier.
    ///
    /// This ID is unique for the lifetime of the compositor instance and
    /// can be used to look up the window from the [`WindowManager`].
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// let id = window.id();
    /// println!("Window ID: {}", id);
    /// \`\`\`
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the window's current title.
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// let title = window.title();
    /// println!("Window title: {}", title);
    /// \`\`\`
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the window's application ID.
    ///
    /// The app ID is in reverse DNS notation (e.g., "org.mozilla.firefox").
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// let app_id = window.app_id();
    /// println!("App ID: {}", app_id);
    /// \`\`\`
    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    /// Checks if the window is currently mapped (visible).
    ///
    /// A window is mapped when [`Window::map()`] has been called and has not
    /// been unmapped since.
    ///
    /// # Returns
    ///
    /// `true` if the window is visible, `false` otherwise.
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// if window.is_mapped() {
    ///     println!("Window is visible");
    /// } else {
    ///     println!("Window is hidden");
    /// }
    /// \`\`\`
    pub fn is_mapped(&self) -> bool {
        matches!(self.state, WindowState::Mapped)
    }

    /// Maps the window, making it visible on screen.
    ///
    /// This is typically called automatically by the compositor when a client
    /// commits its first surface.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The surface has been destroyed
    /// - The window is already mapped
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// # async fn example(window: &Window) -> Result<(), Box<dyn std::error::Error>> {
    /// window.map().await?;
    /// # Ok(())
    /// # }
    /// \`\`\`
    pub async fn map(&self) -> Result<()> {
        // Implementation
    }

    /// Unmaps the window, hiding it from screen.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The surface has been destroyed
    /// - The window is not mapped
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// # async fn example(window: &Window) -> Result<(), Box<dyn std::error::Error>> {
    /// window.unmap().await?;
    /// # Ok(())
    /// # }
    /// \`\`\`
    pub async fn unmap(&self) -> Result<()> {
        // Implementation
    }

    /// Sets the window position.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate in logical pixels
    /// * `y` - Y coordinate in logical pixels
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// window.set_position(100, 100)?;
    /// \`\`\`
    pub fn set_position(&mut self, x: i32, y: i32) -> Result<()> {
        // Implementation
    }

    /// Sets the window size.
    ///
    /// # Arguments
    ///
    /// * `width` - Width in logical pixels
    /// * `height` - Height in logical pixels
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// window.set_size(1024, 768)?;
    /// \`\`\`
    pub fn set_size(&mut self, width: u32, height: u32) -> Result<()> {
        // Implementation
    }

    /// Returns the window's current geometry.
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// let geometry = window.geometry();
    /// println!("Size: {}x{}", geometry.width, geometry.height);
    /// \`\`\`
    pub fn geometry(&self) -> Geometry {
        self.geometry.clone()
    }

    /// Focuses the window.
    ///
    /// This makes the window the target for keyboard input.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The window is not mapped
    /// - The window cannot be focused (e.g., popups)
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// # async fn example(window: &Window) -> Result<(), Box<dyn std::error::Error>> {
    /// window.focus().await?;
    /// # Ok(())
    /// # }
    /// \`\`\`
    pub async fn focus(&self) -> Result<()> {
        // Implementation
    }

    /// Closes (destroys) the window.
    ///
    /// # Errors
    ///
    /// Returns an error if the surface has already been destroyed.
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// # async fn example(window: &Window) -> Result<(), Box<dyn std::error::Error>> {
    /// window.close().await?;
    /// # Ok(())
    /// # }
    /// \`\`\`
    pub async fn close(&self) -> Result<()> {
        // Implementation
    }
}

/// Builder for creating [`Window`] instances with custom configuration.
///
/// # Examples
///
/// \`\`\`rust
/// use rustica_comp::Window;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let window = Window::builder()
///     .title("My Window")
///     .app_id("com.example.mywindow")
///     .size(1024, 768)
///     .position(100, 100)
///     .resizable(true)
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// \`\`\`
pub struct WindowBuilder {
    title: Option<String>,
    app_id: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    x: Option<i32>,
    y: Option<i32>,
    resizable: bool,
    decorations: bool,
}

impl WindowBuilder {
    /// Creates a new [`WindowBuilder`] with default values.
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// let builder = WindowBuilder::new();
    /// \`\`\`
    pub fn new() -> Self {
        Self {
            title: None,
            app_id: None,
            width: None,
            height: None,
            x: None,
            y: None,
            resizable: true,
            decorations: true,
        }
    }

    /// Sets the window title.
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// let builder = WindowBuilder::new()
    ///     .title("My App");
    /// \`\`\`
    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// Sets the application ID.
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// let builder = WindowBuilder::new()
    ///     .app_id("com.example.myapp");
    /// \`\`\`
    pub fn app_id(mut self, app_id: String) -> Self {
        self.app_id = Some(app_id);
        self
    }

    /// Sets the window size.
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// let builder = WindowBuilder::new()
    ///     .size(800, 600);
    /// \`\`\`
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Sets the window position.
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// let builder = WindowBuilder::new()
    ///     .position(100, 100);
    /// \`\`\`
    pub fn position(mut self, x: i32, y: i32) -> Self {
        self.x = Some(x);
        self.y = Some(y);
        self
    }

    /// Sets whether the window is resizable.
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// let builder = WindowBuilder::new()
    ///     .resizable(false);
    /// \`\`\`
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Sets whether the window has server-side decorations.
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// let builder = WindowBuilder::new()
    ///     .decorations(false);
    /// \`\`\`
    pub fn decorations(mut self, decorations: bool) -> Self {
        self.decorations = decorations;
        self
    }

    /// Builds the [`Window`] with the specified configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Title or app_id is not set
    /// - Size is not set
    /// - Window creation fails
    ///
    /// # Examples
    ///
    /// See [`Window`] for examples.
    pub async fn build(self) -> Result<Window> {
        let title = self.title.ok_or_else(|| {
            Error::BuilderError("title is required".to_string())
        })?;

        let app_id = self.app_id.ok_or_else(|| {
            Error::BuilderError("app_id is required".to_string())
        })?;

        let width = self.width.unwrap_or(800);
        let height = self.height.unwrap_or(600);

        // Implementation
        Ok(Window { /* ... */ })
    }
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self::new()
    }
}
```

### Trait Documentation

```rust
/// A renderable UI component.
///
/// Widgets are the fundamental building blocks of the Rustica UI toolkit.
/// Each widget defines its own appearance and behavior by implementing
/// this trait.
///
/// # Lifecycle
///
/// Widgets follow a lifecycle managed by the toolkit:
///
/// 1. **Creation**: Widget is instantiated
/// 2. **Layout**: Size and position are calculated via [`Layoutable::measure`]
/// 3. **Rendering**: Widget is drawn via [`Widget::render`]
/// 4. **Event Handling**: User input is processed via [`Widget::handle_event`]
/// 5. **Destruction**: Widget is cleaned up
///
/// # Required Methods
///
/// - [`Widget::render`] - Draw the widget
/// - [`Widget::handle_event`] - Process user input
/// - [`Widget::bounds`] - Return widget bounds
/// - [`Widget::bounds_mut`] - Return mutable reference to bounds
///
/// # Optional Methods
///
/// - [`Widget::needs_render`] - Check if widget needs redraw
/// - [`Widget::focus`] - Handle focus gained/lost
/// - [`Widget::theme_changed`] - Respond to theme changes
///
/// # Examples
///
/// ## Simple Button Widget
///
/// \`\`\`rust
/// use librustica::{Widget, RenderContext, Event, Rect};
///
/// struct Button {
///     text: String,
///     clicked: bool,
/// }
///
/// impl Widget for Button {
///     fn render(&self, ctx: &mut RenderContext) -> Result<()> {
///         // Draw button background
///         ctx.fill_rect(self.bounds(), Color::BLUE)?;
///
///         // Draw button text
///         ctx.draw_text_centered(&self.text, self.bounds())?;
///
///         Ok(())
///     }
///
///     fn handle_event(&mut self, event: &Event) -> Result<bool> {
///         match event {
///             Event::MouseClick { .. } => {
///                 self.clicked = true;
///                 Ok(true)
///             }
///             _ => Ok(false),
///         }
///     }
///
///     fn bounds(&self) -> Rect {
///         // Return widget bounds
///     }
/// }
/// \`\`\`
///
/// # Accessibility
///
/// Widgets should implement [`Accessible`] to provide accessibility information.
///
/// # See Also
///
/// - [`Accessible`] - Accessibility interface
/// - [`Layoutable`] - Layout integration
/// - [`RenderContext`] - Rendering context
pub trait Widget: Any {
    /// Renders the widget to the given render context.
    ///
    /// This method is called when the widget needs to be redrawn. The widget
    /// should draw itself using the provided render context.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Mutable reference to the render context
    ///
    /// # Returns
    ///
    /// - `Ok(())` if rendering succeeded
    /// - `Err` if rendering failed
    ///
    /// # Performance
    ///
    /// This method may be called frequently (e.g., 60 times per second for animations).
    /// Implementations should be efficient and avoid unnecessary allocations.
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// # fn render_example(widget: &MyWidget, ctx: &mut RenderContext) -> Result<()> {
    /// widget.render(ctx)?;
    /// # Ok(())
    /// # }
    /// \`\`\`
    fn render(&self, ctx: &mut RenderContext) -> Result<()>;

    /// Handles a user input event.
    ///
    /// This method is called when user input (mouse, keyboard, touch) occurs
    /// within the widget's bounds. Return `true` if the event was handled,
    /// preventing it from propagating to parent widgets.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to handle
    ///
    /// # Returns
    ///
    /// - `Ok(true)` if the event was handled
    /// - `Ok(false)` if the event was not handled
    /// - `Err` if event processing failed
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// # fn handle_event_example(widget: &mut MyWidget, event: &Event) -> Result<bool> {
    /// match widget.handle_event(event)? {
    ///         true => println!("Event handled"),
    ///         false => println!("Event not handled"),
    ///     }
    /// # }
    /// \`\`\`
    fn handle_event(&mut self, event: &Event) -> Result<bool>;

    /// Returns the widget's bounding rectangle.
    ///
    /// The bounds are in the parent widget's coordinate system.
    ///
    /// # Returns
    ///
    /// A [`Rect`] representing the widget's position and size.
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// let bounds = widget.bounds();
    /// println!("Widget at: ({}, {}), size: {}x{}",
    ///     bounds.x, bounds.y, bounds.width, bounds.height);
    /// \`\`\`
    fn bounds(&self) -> Rect;

    /// Returns a mutable reference to the widget's bounds.
    ///
    /// This allows layout algorithms to reposition widgets.
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// widget.bounds_mut().x = 100.0;
    /// widget.bounds_mut().y = 200.0;
    /// \`\`\`
    fn bounds_mut(&mut self) -> &mut Rect;

    /// Checks if the widget needs to be re-rendered.
    ///
    /// This is an optional method that widgets can implement to optimize
    /// rendering. The default implementation always returns `true`.
    ///
    /// # Returns
    ///
    /// `true` if the widget needs rendering, `false` otherwise.
    fn needs_render(&self) -> bool {
        true
    }

    /// Called when the widget gains focus.
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// # fn focus_example(widget: &mut MyWidget) {
    /// widget.focus();
    /// }
    /// \`\`\`
    fn focus(&mut self) {
        // Default: do nothing
    }

    /// Called when the widget loses focus.
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// # fn blur_example(widget: &mut MyWidget) {
    /// widget.blur();
    /// }
    /// \`\`\`
    fn blur(&mut self) {
        // Default: do nothing
    }

    /// Called when the theme changes.
    ///
    /// Widgets should reload their styling when this is called.
    ///
    /// # Examples
    ///
    /// \`\`\`rust
    /// # fn theme_changed_example(widget: &mut MyWidget) {
    /// widget.theme_changed();
    /// }
    /// \`\`\`
    fn theme_changed(&mut self) {
        // Default: do nothing
    }
}
```

## D-Bus API Documentation

```markdown
# org.rustica.Compositor

## Description

The \`org.rustica.Compositor\` D-Bus interface provides methods for controlling the Rustica Wayland compositor from external applications.

## Bus Name

- **Session Bus**: \`org.rustica.Compositor\`
- **System Bus**: \`org.rustica.CompositorSystem\`

## Object Path

\`/org/rustica/Compositor\`

## Interface

### Methods

#### CreateWindow

Create a new window.

**Signature**:
\`\`\`
method CreateWindow(
    title: String,
    app_id: String,
    width: UInt32,
    height: UInt32
) -> (window_id: UInt32)
\`\`\`

**Parameters**:
- \`title\`: Window title
- \`app_id\`: Application ID (reverse DNS notation)
- \`width\`: Initial width in pixels
- \`height\`: Initial height in pixels

**Returns**:
- \`window_id\`: Unique identifier for the new window

**Errors**:
- \`org.freedesktop.DBus.Error.InvalidArgs\`: Invalid arguments (empty title/app_id, zero size)
- \`org.rustica.Compositor.Error.Failed\`: Window creation failed

**Example**:
\`\`\`bash
dbus-send --session --dest=org.rustica.Compositor \
  /org/rustica/Compositor \
  org.rustica.Compositor.CreateWindow \
  "My Window" "com.example.MyWindow" 800 600
\`\`\`

#### MapWindow

Map a window to screen (make it visible).

**Signature**:
\`\`\`method MapWindow(
    window_id: UInt32
) -> ()
\`\`\`

**Parameters**:
- \`window_id\`: Window identifier

**Errors**:
- \`org.rustica.Compositor.Error.NoSuchWindow\`: Window does not exist
- \`org.rustica.Compositor.Error.AlreadyMapped\`: Window is already mapped

#### UnmapWindow

Unmap a window (hide it).

**Signature**:
\`\`\`method UnmapWindow(
    window_id: UInt32
) -> ()
\`\`\`

**Parameters**:
- \`window_id\`: Window identifier

**Errors**:
- \`org.rustica.Compositor.Error.NoSuchWindow\`: Window does not exist
- \`org.rustica.Compositor.Error.NotMapped\`: Window is not mapped

#### FocusWindow

Focus a window.

**Signature**:
\`\`\`method FocusWindow(
    window_id: UInt32
) -> ()
\`\`\`

**Parameters**:
- \`window_id\`: Window identifier

**Errors**:
- \`org.rustica.Compositor.Error.NoSuchWindow\`: Window does not exist
- \`org.rustica.Compositor.Error.NotMapped\`: Window is not mapped

#### CloseWindow

Close (destroy) a window.

**Signature**:
\`\`\`method CloseWindow(
    window_id: UInt32
) -> ()
\`\`\`

**Parameters**:
- \`window_id\`: Window identifier

**Errors**:
- \`org.rustica.Compositor.Error.NoSuchWindow\`: Window does not exist

### Signals

#### WindowCreated

Emitted when a new window is created.

**Signal**:
\`\`\`signal WindowCreated(
    window_id: UInt32,
    title: String,
    app_id: String
)
\`\`\`

**Arguments**:
- \`window_id\`: Unique window identifier
- \`title\`: Window title
- \`app_id\`: Application ID

#### WindowMapped

Emitted when a window is mapped to screen.

**Signal**:
\`\`\`signal WindowMapped(
    window_id: UInt32
)
\`\`\`

**Arguments**:
- \`window_id\`: Window identifier

#### WindowUnmapped

Emitted when a window is unmapped.

**Signal**:
\`\`\`signal WindowUnmapped(
    window_id: UInt32
)
\`\`\`

**Arguments**:
- \`window_id\`: Window identifier

#### WindowClosed

Emitted when a window is closed.

**Signal**:
\`\`\`signal WindowClosed(
    window_id: UInt32
)
\`\`\`

**Arguments**:
- \`window_id\`: Window identifier

#### WindowFocused

Emitted when a window receives focus.

**Signal**:
\`\`\`signal WindowFocused(
    window_id: UInt32
)
\`\`\`

**Arguments**:
- \`window_id\`: Window identifier

### Properties

#### ActiveWindow

The currently focused window.

**Signature**:
\`\`\`readonly property ActiveWindow: UInt32
\`\`\`

#### WindowCount

Number of open windows.

**Signature**:
\`\`\`readonly property WindowCount: UInt32
\`\`\`

## Python Example

\`\`\`python
import dbus

# Connect to session bus
bus = dbus.SessionBus()

# Get compositor object
compositor = bus.get_object(
    "org.rustica.Compositor",
    "/org/rustica/Compositor"
)

# Create interface
compositor_interface = dbus.Interface(
    compositor,
    "org.rustica.Compositor"
)

# Create a window
window_id = compositor_interface.CreateWindow(
    "My Window",
    "com.example.MyWindow",
    800,
    600
)

print(f"Created window: {window_id}")

# Map the window
compositor_interface.MapWindow(window_id)

# Focus the window
compositor_interface.FocusWindow(window_id)
\`\`\`
```

## JavaScript Example (node-dbus)

\`\`\`javascript
const dbus = require('dbus-next');

async function createWindow() {
    const bus = dbus.sessionBus();
    const obj = await bus.getProxyObject(
        'org.rustica.Compositor',
        '/org/rustica/Compositor'
    );
    const compositor = obj.getInterface('org.rustica.Compositor');

    const windowId = await compositor.CreateWindow(
        'My Window',
        'com.example.MyWindow',
        800,
        600
    );

    console.log(`Created window: ${windowId}`);
}

createWindow().catch(console.error);
\`\`\`
```
```

## Protocol Documentation

```markdown
# rustica-layer Protocol

## Introduction

The \`rustica-layer\` protocol is a Wayland protocol extension that allows
surfaces to request layer-shell positioning (as panels, docks, etc.) and
provides Rustica-specific layer features.

## Protocol Specification

### Summary

\`\`\`xml
<protocol name="rustica_layer">
  <interface name="zrustica_layer_v1" version="1">
    <request name="set_layer">
      <description><![CDATA[
        Set the surface as a layer surface.

        This requests that the surface be treated as a layer surface,
        which means it will be positioned by the compositor rather than
        by the client.

        The layer may be anchored to an edge of the output and have an
        exclusive zone reserved for it (other surfaces will not be placed
        in the zone).
      ]]></description>
      <arg name="layer" type="uint" summary="layer surface type"/>
      <arg name="anchor" type="string" enum="true" summary="edge anchor">
        <entry name="top" value="top"/>
        <entry name="bottom" value="bottom"/>
        <entry name="left" value="left"/>
        <entry name="right" value="right"/>
      </arg>
      <arg name="exclusive_zone" type="int" summary="exclusive zone size"/>
    </request>

    <request name="set_exclusive_zone">
      <description><![CDATA[
        Set the exclusive zone for the layer surface.

        The exclusive zone is the area around the anchor edge that is
        reserved for the layer surface. Other layer surfaces will not be
        placed in this zone.

        For example, a top panel might set an exclusive zone of 24 pixels
        from the top edge, ensuring nothing overlaps it.
      ]]></description>
      <arg name="size" type="int" summary="exclusive zone size in pixels"/>
    </request>

    <event name="configure">
      <description><![CDATA[
        The configure event asks the client to resize its surface.

        The width and height are the size the client should resize its
        surface to. The serial is an event serial number that should be
        used to surface.ack_configure.
      ]]></description>
      <arg name="serial" type="uint" summary="configure serial number"/>
      <arg name="width" type="int" summary="surface width"/>
      <arg name="height" type="int" summary="surface height"/>
    </event>

    <enum name="error">
      <entry name="invalid_layer" value="0"/>
      <entry name="invalid_anchor" value="1"/>
      <entry name="invalid_exclusive_zone" value="2"/>
    </enum>
  </interface>
</protocol>
\`\`\`

### Layer Types

| Value | Description |
|-------|-------------|
| 0 | Background - Behind all windows |
| 1 | Bottom - Above all windows, anchored to bottom |
| 2 | Top - Above all windows, anchored to top |
| 3 | Overlay - Above all windows, any position |

### Usage Example

\`\`\`rust
use smithay::wayland::shell::wlr_layer::LayerShell;

// Request layer shell
let layer = LayerShell::bind(&display, &surface)?;

// Set layer properties
layer.set_layer(Layer::Layer::Top)?;
layer.set_exclusive_zone(32)?;

// Commit surface
surface.commit()?;
\`\`\`
```

## Data Format Documentation

```markdown
# Theme File Format

## Introduction

Rustica uses JSON theme files to define color schemes, fonts, and styling.
Themes are stored in \`/usr/share/rustica/themes/\`.

## File Format

Theme files are JSON with the following structure:

\`\`\`json
{
  "name": "Theme Name",
  "version": "1.0",
  "description": "Theme description",

  "color_scheme": {
    "primary": "#6200EE",
    "on_primary": "#FFFFFF",
    "primary_container": "#3700B3",
    "on_primary_container": "#EADDFF",

    "secondary": "#03DAC6",
    "on_secondary": "#000000",
    "secondary_container": "#0097A7",
    "on_secondary_container": "#B2EBF2",

    "background": "#F5F5F5",
    "on_background": "#000000",
    "surface": "#FFFFFF",
    "on_surface": "#000000",

    "error": "#B00020",
    "on_error": "#FFFFFF",
    "success": "#4CAF50",
    "on_success": "#FFFFFF",
    "warning": "#FF9800",
    "on_warning": "#000000"
  },

  "typography": {
    "font_family": {
      "default": "Roboto",
      "monospace": "Roboto Mono",
      "serif": "Roboto Slab"
    },
    "font_size": {
      "default": 14,
      "h1": 32,
      "h2": 24,
      "h3": 20,
      "h4": 16,
      "h5": 14,
      "h6": 12
    },
    "line_height": {
      "default": 1.4,
      "heading": 1.2
    },
    "font_weight": {
      "light": 300,
      "regular": 400,
      "medium": 500,
      "bold": 700
    }
  },

  "spacing": {
    "unit": 4,
    "gaps": {
      "xxs": 4,
      "xs": 8,
      "sm": 16,
      "md": 24,
      "lg": 32,
      "xl": 48,
      "xxl": 64
    }
  },

  "border_radius": {
    "small": 4,
    "medium": 8,
    "large": 16
  },

  "shadows": {
    "elevation_1": {
      "offset_x": 0,
      "offset_y": 2,
      "blur_radius": 4,
      "spread_radius": 0,
      "color": "rgba(0, 0, 0, 0.14)"
    },
    "elevation_2": {
      "offset_x": 0,
      "offset_y": 4,
      "blur_radius": 8,
      "spread_radius": 0,
      "color": "rgba(0, 0, 0, 0.20)"
    },
    "elevation_4": {
      "offset_x": 0,
      "offset_y": 8,
      "blur_radius": 16,
      "spread_radius": 0,
      "color": "rgba(0, 0, 0, 0.25)"
    }
  }
}
\`\`\`

## Color Scheme Format

Colors use hex notation (\`#RRGGBB\` or \`#RRGGBBAA\` for with alpha).

### Color Roles

| Role | Description | Example |
|------|-------------|---------|
| \`primary\` | Primary brand color | \`#6200EE\` |
| \`on_primary\` | Color for content on primary | \`#FFFFFF\` |
| \`background\` | Background color | \`#FFFFFF\` |
| \`on_background\` | Color for content on background | \`#000000\` |
| \`surface\` | Surface color (cards, dialogs) | \`#FFFFFF\` |
| \`on_surface\` | Color for content on surface | \`#000000\` |
| \`error\` | Error indication | \`#B00020\` |
| \`success\` | Success indication | \`#4CAF50\` |
| \`warning\` | Warning indication | \`#FF9800\` |

## Example Theme

\`\`\`json
{
  "name": "Rustica Dark",
  "version": "1.0",
  "description": "Dark theme for Rustica OS",

  "color_scheme": {
    "primary": "#BB86FC",
    "on_primary": "#000000",
    "primary_container": "#3700B3",
    "on_primary_container": "#EADDFF",

    "secondary": "#03DAC6",
    "on_secondary": "#000000",
    "secondary_container": "#0097A7",
    "on_secondary_container": "#B2EBF2",

    "background": "#121212",
    "on_background": "#FFFFFF",
    "surface": "#1E1E1E",
    "on_surface": "#E0E0E0",

    "error": "#CF6679",
    "on_error": "#000000",
    "success": "#81C784",
    "on_success": "#000000",
    "warning": "#FFB74D",
    "on_warning": "#000000"
  },

  "typography": {
    "font_family": {
      "default": "Roboto",
      "monospace": "Roboto Mono",
      "serif": "Roboto Slab"
    },
    "font_size": {
      "default": 14,
      "h1": 32,
      "h2": 24,
      "h3": 20,
      "h4": 16,
      "h5": 14,
      "h6": 12
    },
    "line_height": {
      "default": 1.4,
      "heading": 1.2
    },
    "font_weight": {
      "light": 300,
      "regular": 400,
      "medium": 500,
      "bold": 700
    }
  }
}
\`\`\`
```

## API Versioning

```markdown
# API Versioning Policy

## Version Numbers

Rustica GUI uses [semantic versioning](https://semver.org/):

\`\`\`text
MAJOR.MINOR.PATCH
\`\`\`

- **MAJOR**: Incompatible API changes
- **MINOR**: New functionality in a backwards-compatible manner
- **PATCH**: Backwards-compatible bug fixes

## Deprecation Policy

APIs are deprecated for at least one major version before removal.

### Deprecation Process

1. Mark API as deprecated:
   \`\`\`rust
   #[deprecated(since = "0.5.0", note = "Use Window::set_geometry instead")]
   pub fn set_size(&mut self, width: u32, height: u32) -> Result<()> {
       // ...
   }
   \`\`\`

2. Document replacement in release notes

3. Update migration guide

4. Remove in next major version

## API Stability

### Stable APIs

Marked with \`@rustfmt::skip\` and version:
\`\`\`rust
/// # Stability
///
/// This API is stable and will not break except for major version bumps.
///
/// Stability: Stable (since 0.1.0)
\`\`\`

### Experimental APIs

Marked with experimental:
\`\`\`rust
/// # Stability
///
/// This API is experimental and may change in future versions.
///
/// Stability: Experimental
\`\`\`

### Internal APIs

Not documented and may change at any time.

## Breaking Changes

Breaking changes require:
1. Major version bump
2. Update migration guide
3. Release notes explaining changes
4. Update all dependent code

## Example Migration

### 0.x → 1.0

\`\`\`rust
// Old API (0.x)
window.set_size(800, 600)?;
window.set_position(100, 100)?;

// New API (1.0)
window.set_geometry(Rect::new((100, 100), (800, 600)))?;
\`\`\`
```

## Configuration

```toml
# API documentation generation configuration

[package.metadata.docs.rs]
features = ["all"]
all-features = false
rustdoc-args = ["--cfg", "docsrs"]
default-target = "x86_64-unknown-linux-gnu"

[doc]
html-root-url = "/rustica-gui/api/"
html-logo-url = "/rustica-gui/logo.png"
favicon = "/rustica-gui/favicon.ico"
```

## Best Practices

1. **Document Everything**: All public APIs must have docs
2. **Provide Examples**: Every API should have at least one example
3. **Explain Errors**: Document all possible errors
4. **Cross-Reference**: Link to related APIs
5. **Version Notes**: Document when features were added
6. **Performance Notes**: Document performance characteristics
7. **Thread Safety**: Document thread safety guarantees
8. **Panics**: Document any situations that cause panics
9. **Platform-Specific**: Note platform-specific behavior
10. **Keep Updated**: Update docs with code changes

## Tools

- **rustdoc**: Built-in Rust documentation generator
- **cargo-doc**: Generate and view documentation
- **mdBook**: Publishable book format
- **sphinx**: Alternative documentation tool
- **swagger**: For REST API documentation
- **dbus-doc**: Generate D-Bus interface docs

## CI/CD Integration

```yaml
# .github/workflows/docs-api.yml
name: API Documentation

on:
  push:
    branches: [main]
    paths: ['**/*.rs']

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Generate documentation
        run: |
          cargo doc --no-deps --all-features
          cargo doc --package rustica-comp --no-deps
          cargo doc --package librustica --no-deps
      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./target/doc
          publish_branch: gh-pages
```
