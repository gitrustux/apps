# Phase 11.1: Developer Documentation

## Overview

**Component**: Developer Documentation
**Purpose**: Comprehensive documentation for GUI developers
**Format**: Markdown + Rustdoc
**Location**: `/var/www/rustux.com/prod/apps/gui/docs/`

## Goals

1. **Comprehensive Coverage**: Document all components, APIs, and workflows
2. **Easy Navigation**: Well-organized structure for quick information access
3. **Up-to-Date**: Documentation synchronized with code changes
4. **Code Examples**: Practical examples for all common tasks
5. **Architecture Guides**: Deep dives into design decisions and trade-offs

## Documentation Structure

```
docs/
├── README.md                           # Documentation overview
├── getting-started/
│   ├── installation.md                 # Development environment setup
│   ├── building.md                     # Building from source
│   ├── running.md                      # Running the compositor
│   ├── debugging.md                    # Debugging tips and tools
│   └── troubleshooting.md              # Common issues and solutions
├── architecture/
│   ├── overview.md                     # High-level architecture
│   ├── compositor.md                   # Compositor architecture
│   ├── shell.md                        # Shell architecture
│   ├── toolkit.md                      # Toolkit architecture
│   └── data-flow.md                    # Data flow diagrams
├── development/
│   ├── coding-standards.md             # Code style and conventions
│   ├── testing-guide.md                # Testing guidelines
│   ├── performance-guide.md            # Performance optimization
│   ├── security-guide.md               # Security considerations
│   └── accessibility-guide.md          # Accessibility requirements
├── components/
│   ├── compositor/                      # Compositor components
│   │   ├── surface-management.md
│   │   ├── rendering.md
│   │   ├── input-handling.md
│   │   └── output-management.md
│   ├── shell/                          # Shell components
│   │   ├── panel.md
│   │   ├── dock.md
│   │   ├── launcher.md
│   │   └── notifications.md
│   └── apps/                           # Applications
│       ├── terminal.md
│       ├── file-manager.md
│       ├── settings.md
│       └── text-editor.md
├── guides/
│   ├── creating-widgets.md             # Building custom widgets
│   ├── integrating-sensors.md          # Sensor integration
│   ├── adding-gestures.md              # Gesture support
│   ├── theming.md                      # Custom theming
│   └── packaging-apps.md               # App packaging
├── reference/
│   ├── apis/                           # API documentation
│   ├── protocols/                      # Protocol documentation
│   ├── configuration/                  # Configuration reference
│   └── dbus-interfaces/                # D-Bus interface docs
└── tools/
    ├── rustica-comp.md                 # Compositor tools
    ├── rustica-panel.md                # Panel tools
    └── profiling-tools.md              # Profiling and debugging
```

## Getting Started Guide

```markdown
# Getting Started with Rustica GUI Development

## Prerequisites

- Rust 1.70 or later
- Wayland development libraries
- Linux system (Ubuntu 22.04+ recommended)

## Installation

### Install Dependencies

\`\`\`bash
sudo apt update
sudo apt install -y \
  libwayland-dev \
  libegl1-mesa-dev \
  libgles2-mesa-dev \
  libxkbcommon-dev \
  libinput-dev \
  libudev-dev \
  libgbm-dev \
  libsystemd-dev \
  libpulse-dev \
  libdbus-1-dev
\`\`\`

### Clone Repository

\`\`\`bash
git clone https://github.com/rustux/rustica-gui.git
cd rustica-gui
\`\`\`

### Build

\`\`\`bash
# Build all components
cargo build --release

# Build specific component
cargo build --release -p rustica-comp
\`\`\`

## Running

### Run Compositor

\`\`\`bash
# Run in nested window for development
cargo run --release -p rustica-comp -- --nested

# Run on TTY
sudo cargo run --release -p rustica-comp
\`\`\`

### Run with Debug Logging

\`\`\`bash
RUSTICA_LOG=debug cargo run --release -p rustica-comp -- --nested
\`\`\`

## Development Workflow

1. Create feature branch: \`git checkout -b feature/my-feature\`
2. Make changes
3. Run tests: \`cargo test\`
4. Run linter: \`cargo clippy\`
5. Format code: \`cargo fmt\`
6. Commit changes: \`git commit\`
7. Push and create PR

## Debugging

### Enable Backtraces

\`\`\`bash
RUST_BACKTRACE=1 cargo run --release -p rustica-comp
\`\`\`

### Use GDB

\`\`\`bash
gdb --args target/release/rustica-comp --nested
\`\`\`

### Use LLDB

\`\`\`bash
lldb target/release/rustica-comp --nested
\`\`\`

## Common Issues

### "Failed to initialize Wayland display"

Make sure Wayland compositor is running:

\`\`\`bash
echo $XDG_SESSION_TYPE
\`\`\`

### "Permission denied when accessing /dev/dri/card0"

Add user to \`video\` group:

\`\`\`bash
sudo usermod -aG video $USER
\`\`\`

### Build errors

Clean and rebuild:

\`\`\`bash
cargo clean
cargo build --release
\`\`\`
```

## Architecture Documentation

```markdown
# Compositor Architecture

## Overview

The Rustica compositor (rustica-comp) is built on top of [Smithay](https://github.com/Smithay/smithay), a pure-Rust Wayland compositor library.

## Core Components

### Display

The \`Display\` manages the Wayland display socket and client connections.

\`\`\`rust
use smithay::reexports::wayland_server::Display;

let display = Display::new()?;
\`\`\`

### Compositor State

The \`CompositorState\` handles Wayland protocols:

\`\`\`rust
use smithay::wayland::compositor::CompositorState;

let compositor_state = CompositorState::new::<State, _>(
    &display,
    compositor,
);
\`\`\`

### Space

The \`Space\` manages window layout and positioning:

\`\`\`rust
use smithay::desktop::Space;

let mut space = Space::<Window>::new();
space.map_window(window, (0, 0), None, true);
\`\`\`

## Data Flow

```
Client → Display → EventQueue → Compositor → Space → Renderer → Output
```

1. Client sends request via Wayland protocol
2. Display receives and queues event
3. EventQueue processes events
4. Compositor handles event
5. Space updates window state
6. Renderer renders frame
7. Output displays frame

## Key Design Decisions

### Why Smithay?

- **Pure Rust**: No C/C++ dependencies
- **Active Development**: Regular updates and bug fixes
- **Extensible**: Easy to customize protocols
- **Well-Documented**: Comprehensive API docs

### Why Not Other Compositors?

- **Weston**: C-based, harder to extend from Rust
- **Mutter**: Tightly coupled to GNOME
- **KWin**: Tightly coupled to KDE Plasma

### Event Loop

The compositor uses Smithay's event loop:

\`\`\`rust
use smithay::reexports::calloop::{EventLoop, LoopSignal};

let mut event_loop = EventLoop::try_new().unwrap();
let mut signal = event_loop.get_signal();

event_loop.run(
    None,
    &mut state,
    |state, _| {
        state.display.flush_clients().unwrap();
        state.space.refresh();
        state.render();
    },
    &mut signal,
)?;
\`\`\`

## Rendering Pipeline

```
┌─────────────┐
│   Input     │ → Touch/Mouse/Keyboard events
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Layout    │ → Calculate window positions
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Render     │ → Draw surfaces to off-screen buffers
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Compose   │ → Combine buffers into final frame
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Output    │ → Present to display
└─────────────┘
```

## Memory Management

The compositor uses Rust's ownership model:

- **Surfaces**: Owned by \`Space\`
- **Outputs**: Managed by \`OutputManager\`
- **Textures**: Reference counted via \`Arc\`

## Thread Safety

- **Main Thread**: Wayland event handling
- **Render Thread**: GPU operations (optional)
- **Worker Threads**: File I/O, network operations
```

## Development Guides

```markdown
# Creating Custom Widgets

## Introduction

This guide shows how to create custom widgets using librustica, the native Rust UI toolkit for Rustica.

## Basic Widget

Every widget implements the \`Widget\` trait:

\`\`\`rust
use librustica::Widget;

struct MyWidget {
    text: String,
    background: Color,
}

impl Widget for MyWidget {
    fn render(&self, ctx: &mut RenderContext) -> Result<()> {
        // Draw background
        ctx.fill_rect(self.bounds(), self.background)?;

        // Draw text
        ctx.draw_text(&self.text, self.bounds().center())?;

        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> Result<bool> {
        match event {
            Event::MouseClick { position, .. } => {
                if self.bounds().contains(*position) {
                    // Handle click
                    return Ok(true);
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn bounds(&self) -> Rect {
        // Return widget bounds
    }
}
\`\`\`

## Properties

Use properties to make widgets configurable:

\`\`\`rust
use librustica::Property;

#[properties]
impl MyWidget {
    #[property]
    pub fn text(mut self, text: String) -> Self {
        self.text = text;
        self
    }

    #[property]
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }
}
\`\`\`

## Layout

Integrate with layout system:

\`\`\`rust
use librustica::layout::Layoutable;

impl Layoutable for MyWidget {
    fn measure(&self, constraints: SizeConstraints) -> Size {
        // Calculate desired size
    }

    fn arrange(&mut self, bounds: Rect) {
        // Position child elements
    }
}
\`\`\`

## Accessibility

Implement accessibility interface:

\`\`\`rust
use librustica::a11y::Accessible;

impl Accessible for MyWidget {
    fn accessible_role(&self) -> AccessibleRole {
        AccessibleRole::PushButton
    }

    fn accessible_name(&self) -> Option<String> {
        Some(self.text.clone())
    }

    fn accessible_description(&self) -> Option<String> {
        Some("A custom button".to_string())
    }
}
\`\`\`

## Example: Button Widget

\`\`\`rust
use librustica::{Widget, Property};

#[properties]
pub struct Button {
    text: String,
    background: Color,
    on_click: Box<dyn Fn()>,
    state: ButtonState,
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonState {
    Normal,
    Hover,
    Pressed,
}

impl Button {
    pub fn new(text: String) -> Self {
        Self {
            text,
            background: Color::from_hex("#6200EE"),
            on_click: Box::new(|| {}),
            state: ButtonState::Normal,
        }
    }

    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: Fn() + 'static,
    {
        self.on_click = Box::new(callback);
        self
    }
}

impl Widget for Button {
    fn render(&self, ctx: &mut RenderContext) -> Result<()> {
        let color = match self.state {
            ButtonState::Normal => self.background,
            ButtonState::Hover => self.background.lighten(0.1),
            ButtonState::Pressed => self.background.darken(0.1),
        };

        ctx.fill_rounded_rect(self.bounds(), 8.0, color)?;
        ctx.draw_text_centered(&self.text, self.bounds())?;

        Ok(())
    }

    fn handle_event(&mut self, event: &Event) -> Result<bool> {
        match event {
            Event::MouseEnter => {
                self.state = ButtonState::Hover;
            }
            Event::MouseLeave => {
                self.state = ButtonState::Normal;
            }
            Event::MouseButtonDown { .. } => {
                self.state = ButtonState::Pressed;
            }
            Event::MouseButtonUp { .. } => {
                self.state = ButtonState::Hover;
                (self.on_click)();
                return Ok(true);
            }
            _ => {}
        }
        Ok(false)
    }
}
\`\`\`

## Testing Widgets

\`\`\`rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_click() {
        let mut clicked = false;
        let mut button = Button::new("Click Me".to_string())
            .on_click(|| clicked = true);

        // Simulate click
        button.handle_event(&Event::MouseEnter).unwrap();
        button.handle_event(&Event::MouseButtonDown).unwrap();
        button.handle_event(&Event::MouseButtonUp).unwrap();

        assert!(clicked);
    }
}
\`\`\`
```

## Code Standards

```markdown
# Coding Standards

## Rust Style Guide

Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

### Naming

- **Types**: \`PascalCase\`
- **Functions**: \`snake_case\`
- **Constants**: \`SCREAMING_SNAKE_CASE\`
- **Acronyms**: Keep consistent (e.g., \`Http\`, not \`HTTP\`)

### Example

\`\`\`rust
// Good
struct WindowManager {
    max_windows: usize,
    window_list: Vec<Window>,
}

impl WindowManager {
    pub fn new(max_windows: usize) -> Self {
        Self {
            max_windows,
            window_list: Vec::new(),
        }
    }

    pub fn add_window(&mut self, window: Window) -> Result<()> {
        if self.window_list.len() >= self.max_windows {
            return Err(Error::TooManyWindows);
        }
        self.window_list.push(window);
        Ok(())
    }
}
\`\`\`

## Error Handling

Use \`Result<T, Error>\` for fallible operations:

\`\`\`rust
use anyhow::Result;

pub fn create_surface(display: &Display) -> Result<Surface> {
    let surface = display.create_surface()
        .context("Failed to create Wayland surface")?;

    Ok(surface)
}
\`\`\`

### Error Types

Define custom error types:

\`\`\`rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompositorError {
    #[error("No output available")]
    NoOutput,

    #[error("Failed to initialize renderer: {0}")]
    RendererInit(String),

    #[error("Surface {0} not found")]
    SurfaceNotFound(u32),
}
\`\`\`

## Documentation

### Public API

Document all public items:

\`\`\`rust
/// Creates a new window with the specified dimensions.
///
/// # Arguments
///
/// * \`width\` - Window width in pixels
/// * \`height\` - Window height in pixels
///
/// # Returns
///
/// A \`Window\` handle
///
/// # Errors
///
/// Returns an error if:
/// - The compositor is not running
/// - Width or height is zero
///
/// # Examples
///
/// \`\`\`rust
/// let window = create_window(800, 600)?;
/// \`\`\`
pub fn create_window(width: u32, height: u32) -> Result<Window> {
    // ...
}
\`\`\`

### Module Docs

Add module documentation:

\`\`\`rust
//! Window management for the Rustica compositor.
//!
//! This module provides functionality for creating, managing, and destroying
//! Wayland windows.
//!
//! # Example
//!
//! \`\`\`rust
//! use rustica_comp::window;
//!
//! let window = window::create(800, 600)?;
//! window.show()?;
//! \`\`\`
\`\`\`

## Performance Guidelines

### Memory Allocation

Minimize allocations in hot paths:

\`\`\`rust
// Bad
pub fn process_events(&self) {
    for event in &self.events {
        let data = event.to_string();  // Allocation
        self.handle(data);
    }
}

// Good
pub fn process_events(&self) {
    for event in &self.events {
        self.handle(event);  // No allocation
    }
}
\`\`\`

### Cloning

Avoid unnecessary clones:

\`\`\`rust
// Bad
pub fn get_name(&self) -> String {
    self.name.clone()  // Unnecessary clone
}

// Good
pub fn get_name(&self) -> &str {
    &self.name  // Return reference
}
\`\`\`

### Iterators

Use iterators where possible:

\`\`\`rust
// Bad
pub fn find_window(&self, id: u32) -> Option<&Window> {
    for window in &self.windows {
        if window.id() == id {
            return Some(window);
        }
    }
    None
}

// Good
pub fn find_window(&self, id: u32) -> Option<&Window> {
    self.windows.iter()
        .find(|w| w.id() == id)
}
\`\`\`

## Testing

### Unit Tests

Write unit tests alongside code:

\`\`\`rust
impl Window {
    pub fn is_visible(&self) -> bool {
        self.visible
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_visibility() {
        let window = Window::new();
        assert!(!window.is_visible());

        window.show();
        assert!(window.is_visible());
    }
}
\`\`\`

### Integration Tests

Place in \`tests/\` directory:

\`\`\`rust
// tests/window_creation.rs

use rustica_comp::Compositor;

#[tokio::test]
async fn test_create_window() {
    let compositor = Compositor::test_new().await.unwrap();
    let window = compositor.create_window(800, 600).await.unwrap();

    assert_eq!(window.width(), 800);
    assert_eq!(window.height(), 600);
}
\`\`\`

## Code Review Checklist

- [ ] Code follows style guidelines
- [ ] Public API is documented
- [ ] Errors are properly handled
- [ ] Tests are included
- [ ] No unnecessary dependencies added
- [ ] Performance implications considered
- [ ] Security implications considered
- [ ] Accessibility requirements met
```

## API Documentation Standards

```markdown
# API Documentation Standards

## Rustdoc Comments

All public APIs must have rustdoc comments:

### Functions

\`\`\`rust
/// Performs a gesture recognition on the provided touch points.
///
/// This function analyzes the sequence of touch points and determines
/// if they match a known gesture pattern (tap, swipe, pinch, rotate).
///
/// # Arguments
///
/// * \`session\` - Active touch session with recorded touch points
/// * \`config\` - Gesture recognition configuration
///
/// # Returns
///
/// A \`GestureResult\` containing the recognized gesture, if any.
///
/// # Errors
///
/// Returns \`Error::InvalidSession\` if the session has no touch points.
///
/// # Examples
///
/// \`\`\`
/// # use rustica_gestures::GestureRecognizer;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let mut recognizer = GestureRecognizer::new(GestureConfig::default());
///     let result = recognizer.handle_touch_down(&touch, &session)?;
/// # Ok(())
/// # }
/// \`\`\`
///
/// # Performance
///
/// This function is O(n) where n is the number of touch points in the session.
pub fn recognize_gesture(
    session: &TouchSession,
    config: &GestureConfig,
) -> Result<GestureResult, Error> {
    // ...
}
\`\`\`

### Structs

\`\`\`rust
/// Manages keyboard input and provides virtual keyboard functionality.
///
/// The \`KeyboardManager\` handles keyboard layout switching, touch input
/// processing, and text prediction. It integrates with the compositor
/// via the text-input-v3 protocol.
///
/// # State
///
/// The manager maintains:
/// - Current keyboard layout
/// - Shift and modifier state
/// - Active prediction model
/// - Gesture trail for swipe typing
///
/// # Examples
///
/// \`\`\`rust
/// use rustica_keyboard::KeyboardManager;
///
/// let manager = KeyboardManager::new()?;
/// manager.show().await?;
/// \`\`\`
pub struct KeyboardManager {
    /// Current keyboard layout (e.g., QWERTY, AZERTY)
    layout: KeyboardLayout,
    /// Current shift state
    shift_state: ShiftState,
    /// Active modifier flags
    modifiers: ModifierState,
}
\`\`\`

### Traits

\`\`\`rust
/// A renderable UI component.
///
/// Widgets are the basic building blocks of the UI. Every widget
/// implements the \`Widget\` trait to define its appearance and behavior.
///
/// # Lifecycle
///
/// Widgets go through the following lifecycle:
/// 1. **Creation**: Widget is instantiated
/// 2. **Layout**: Size and position are calculated
/// 3. **Rendering**: Widget is drawn to screen
/// 4. **Event Handling**: User input is processed
/// 5. **Destruction**: Widget is cleaned up
///
/// # Required Methods
///
/// - \`render()\` - Draw the widget
/// - \`handle_event()\` - Process user input
/// - \`bounds()\` - Return widget bounds
///
/// # Examples
///
/// Implementing a simple button:
///
/// \`\`\`rust
/// use librustica::Widget;
///
/// struct Button {
///     text: String,
///     clicked: bool,
/// }
///
/// impl Widget for Button {
///     fn render(&self, ctx: &mut RenderContext) -> Result<()> {
///         ctx.fill_rect(self.bounds(), Color::BLUE)?;
///         ctx.draw_text(&self.text, self.bounds().center())?;
///         Ok(())
///     }
///
///     fn handle_event(&mut self, event: &Event) -> Result<bool> {
///         match event {
///             Event::Click { .. } => {
///                 self.clicked = true;
///                 Ok(true)
///             }
///             _ => Ok(false),
///         }
///     }
///
///     fn bounds(&self) -> Rect {
///         // ...
///     }
/// }
/// \`\`\`
pub trait Widget {
    /// Render the widget to the given render context.
    ///
    /// # Arguments
    ///
    /// * \`ctx\` - Mutable reference to render context
    ///
    /// # Returns
    ///
    /// \`Ok(())\` if rendering succeeded, \`Err\` otherwise
    fn render(&self, ctx: &mut RenderContext) -> Result<()>;

    /// Handle a user input event.
    ///
    /// # Arguments
    ///
    /// * \`event\` - The event to handle
    ///
    /// # Returns
    ///
    /// \`Ok(true)\` if the event was handled, \`Ok(false)\` if not
    fn handle_event(&mut self, event: &Event) -> Result<bool>;

    /// Return the widget's bounding rectangle.
    ///
    /// # Returns
    ///
    /// A \`Rect\` representing the widget's bounds
    fn bounds(&self) -> Rect;
}
\`\`\`

## Module Documentation

Each module should have a top-level module doc:

\`\`\`rust
//! Touch gesture recognition for mobile and touch devices.
//!
//! This module provides gesture recognition capabilities including:
//!
//! - **Tap**: Single and multi-tap detection
//! - **Swipe**: Directional swipe detection (up, down, left, right)
//! - **Pinch**: Two-finger pinch-to-zoom
//! - **Rotate**: Two-finger rotation
//!
//! # Architecture
//!
//! The gesture system consists of:
//!
//! - \`GestureRecognizer\`: Main gesture engine
//! - \`TouchSession\`: Tracks active touch points
//! - \`GestureConfig\`: Configurable thresholds and timeouts
//!
//! # Example
//!
//! \`\`\`rust
//! use rustica_gestures::{GestureRecognizer, GestureConfig};
//!
//! let recognizer = GestureRecognizer::new(GestureConfig::default());
//! let result = recognizer.handle_touch_down(&touch, &session)?;
//!
//! match result {
//!     Some(Gesture::Tap { count, .. }) => {
//!         println!("Tapped {} times", count);
//!     }
//!     Some(Gesture::Swipe { direction, .. }) => {
//!         println!("Swiped {:?}", direction);
//!     }
//!     _ => {}
//! }
//! \`\`\`
\`\`\`

## Documentation Build

### Generate Documentation

\`\`\`bash
# Generate documentation
cargo doc --no-deps

# Open in browser
cargo doc --no-deps --open
\`\`\`

### Include Private Items

\`\`\`bash
cargo doc --no-deps --document-private-items
\`\`\`

### Custom CSS

Add \`docs/\` to project root with custom.css:

\`\`\`css
/* Add Rustica branding */
.doc {
    border-left: 4px solid #6200EE;
}
\`\`\`
```

## Contributing Guide Structure

```markdown
# Contributing to Rustica GUI

Thank you for your interest in contributing! This document provides guidelines for contributing to the Rustica GUI project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing Requirements](#testing-requirements)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)

## Code of Conduct

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on constructive feedback

## Getting Started

1. Fork the repository
2. Clone your fork: \`git clone https://github.com/your-username/rustica-gui.git\`
3. Add upstream remote: \`git remote add upstream https://github.com/rustux/rustica-gui.git\`

## Development Workflow

\`\`\`bash
# Create feature branch
git checkout -b feature/my-feature

# Make changes
# ...

# Run tests
cargo test

# Run linter
cargo clippy

# Format code
cargo fmt

# Commit
git commit -m "Add my feature"

# Push
git push origin feature/my-feature
\`\`\`

## Coding Standards

See [Coding Standards](./development/coding-standards.md)

## Testing Requirements

See [Testing Guide](./development/testing-guide.md)

## Commit Messages

Follow conventional commits:

\`\`\`
feat: add gesture recognition

- Implement tap detection
- Implement swipe detection
- Add unit tests

Closes #123
\`\`\`

Allowed types:
- \`feat:\` - New feature
- \`fix:\` - Bug fix
- \`docs:\` - Documentation
- \`style:\` - Code style changes
- \`refactor:\` - Code refactoring
- \`perf:\` - Performance improvements
- \`test:\` - Adding tests
- \`chore:\` - Maintenance tasks

## Pull Request Process

1. Update documentation
2. Ensure all tests pass
3. Update CHANGELOG.md
4. Submit PR with description
5. Address review feedback
6. PR is merged!

## Getting Help

- GitHub Issues: https://github.com/rustux/rustica-gui/issues
- Matrix: #rustica-dev:matrix.org
- Email: dev@rustux.com
```

## Configuration

```toml
# Documentation build configuration

[package.metadata.docs.rs]
# Build documentation on docs.rs
all-features = true
rustdoc-args = ["--cfg", "docsrs"]

[doc]
# Include private items in dev builds
private-docs = ["--document-private-items"]

# Output directory
output-dir = "/var/www/rustux.com/prod/apps/gui/target/doc"

# CSS customizations
css = ["docs/custom.css"]
```

## Best Practices

1. **Write as You Code**: Document while implementing
2. **Keep it Current**: Update docs when code changes
3. **Use Examples**: Every API should have examples
4. **Cross-Reference**: Link between related docs
5. **Visual Aids**: Use diagrams where helpful
6. **Version Notes**: Document version-specific changes
7. **Search Friendly**: Use descriptive terms
8. **Multiple Formats**: Support HTML, PDF, and man pages
9. **Screenshots**: Include UI screenshots
10. **Videos**: Add screencasts for complex topics

## Tools

- **rustdoc**: Built-in Rust documentation
- **mdBook**: For books and guides
- **sphinx**: For reStructuredText docs
- **plantuml**: For UML diagrams
- **graphviz**: For architecture diagrams
- **pandoc**: For format conversion

## Dependencies

```toml
[dependencies]
# Documentation tools
mdbook = "0.4"
mdbook-katex = "0.5"  # Math support
mdbook-mermaid = "0.12"  # Diagrams
```

## CI/CD Integration

```yaml
# .github/workflows/docs.yml
name: Documentation

on:
  push:
    branches: [main]
    paths: ['**.md', '**.rs']

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build documentation
        run: |
          cargo doc --no-deps --all-features
          mdbook build docs/
      - name: Deploy
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./docs/book
          publish_branch: gh-pages
```

## Future Enhancements

1. **Interactive Tutorials**: Step-by-step guides
2. **API Explorer**: Browse and test APIs interactively
3. **Changelog Generator**: Automatic changelog from commits
4. **Documentation Search**: Full-text search
5. **Video Tutorials**: Screencast demonstrations
6. **Code Annotations**: Inline code explanations
7. **Translations**: Multi-language documentation
8. **Offline Docs**: Downloadable documentation
9. **API Versioning**: Document API changes over time
10. **Documentation Metrics**: Track documentation coverage
