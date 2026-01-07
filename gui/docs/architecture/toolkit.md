# UI Toolkit Strategy Decision

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - UI Toolkit

## Decision

**CHOSEN: Native Rust Widget Library (librustica) with COSMIC-inspired Design**

We will build a **native Rust UI toolkit** called **librustica** from scratch, taking design inspiration from COSMIC's libcosmic (which is GTK4-based) but implemented purely in Rust for our architecture.

### Core Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     librustica                              │
│                  (UI Toolkit Library)                       │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │   Widget    │  │  Layout     │  │   Style     │       │
│  │   Library   │  │   Engine    │  │   System    │       │
│  └─────────────┘  └─────────────┘  └─────────────┘       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │  Rendering  │  │   Events    │  │ Accessibility│       │
│  │   Backend   │  │   Handler   │  │    (A11y)   │       │
│  └─────────────┘  └─────────────┘  └─────────────┘       │
├─────────────────────────────────────────────────────────────┤
│                   Smithay (Wayland)                         │
└─────────────────────────────────────────────────────────────┘
```

## Widget Library

### Core Widgets (Tier 1 - Essential)
```rust
// Base widget trait
pub trait Widget {
    fn render(&mut self, ctx: &mut RenderContext);
    fn handle_event(&mut self, event: &Event) -> EventResult;
    fn layout(&mut self, constraints: Constraints) -> Size;
    fn children(&mut self) -> Vec<&mut dyn Widget>;
}

// Concrete widgets
pub struct Button { /* ... */ }
pub struct Label { /* ... */ }
pub struct TextField { /* ... */ }
pub struct Container { /* ... */ }
pub struct ScrollArea { /* ... */ }
pub struct Window { /* ... */ }
```

### Layout System
- **Flexbox** layout engine (like CSS, modern web approach)
- **Constraints** propagation (min/max width/height)
- **Alignment** and **spacing** properties
- **Overflow** handling (scroll, clip, visible)

```rust
pub enum Layout {
    Flex {
        direction: Direction,
        alignment: Alignment,
        spacing: f32,
        // ...
    },
    Grid {
        columns: usize,
        rows: usize,
        // ...
    },
    Absolute,
}
```

### Styling System

**Design Tokens** (from Phase 0.7):
```rust
pub struct Theme {
    // Spacing scale
    pub space_xxs: f32,  // 4px
    pub space_xs: f32,   // 8px
    pub space_sm: f32,   // 12px
    pub space_md: f32,   // 16px
    pub space_lg: f32,   // 24px
    pub space_xl: f32,   // 32px

    // Typography
    pub font_xs: Font,
    pub font_sm: Font,
    pub font_md: Font,
    pub font_lg: Font,
    pub font_xl: Font,
    pub font_h1: Font,
    pub font_h2: Font,

    // Colors
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub surface: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,

    // Border radius
    pub radius_sm: f32,  // 4px
    pub radius_md: f32,  // 8px
    pub radius_lg: f32,  // 12px
}
```

**Theming Approach**:
- CSS-like styling with property inheritance
- Runtime theme switching (light/dark mode)
- Per-widget style overrides
- Animated theme transitions

## Why Native Toolkit?

### Advantages

1. **Full Control**
   - No external dependencies we don't understand
   - Optimize for our use case
   - Integrate with RUSTUX kernel capabilities

2. **Rust Ecosystem**
   - Memory safety guarantees
   - No FFI overhead
   - Compile-time optimization

3. **Tailored to Rustica**
   - Built for our architecture from day one
   - Accessibility first (not bolted on)
   - Mobile/desktop from the start

4. **Learning Opportunity**
   - We understand every line of code
   - No black boxes
   - Can fix bugs ourselves

### Why Not Wrapping Existing Toolkits?

| Toolkit | Rejection Reason |
|---------|------------------|
| **GTK4 Rust bindings** | FFI overhead, GTK design doesn't match our needs, hard to customize |
| **egui** | Immediate mode, not suitable for complex apps, game-like paradigm |
| **iced** | Less mature, limited widget set, immediate mode limitations |
| **Slint** | Proprietary license concerns, less flexibility, DSL-based |
| **Flutter** | Dart-based, heavy, not Rust-native |

## Implementation Strategy

### Phase 1: Core (Weeks 1-3)
```rust
// Minimal viable widget set
Widget trait + Button + Label + Container
Flexbox layout engine
Basic event handling (click, hover)
Simple rendering (solid colors, borders)
```

### Phase 2: Essential Widgets (Weeks 4-8)
```rust
TextField, ScrollArea, Checkbox, RadioButton
Dropdown, Slider, Progress Bar
Modal, Popover, Tooltip
Keyboard navigation focus handling
```

### Phase 3: Advanced (Weeks 9+)
```rust
TreeView, TabView, SplitView
Rich text rendering
Animations and transitions
Accessibility tree (AT-SPI)
Drag-and-drop
Clipboard handling
```

## Component Library Plan

### Pre-built Components (for app developers)

```rust
// librustica::components
pub mod components {
    pub mod buttons {     // PrimaryButton, SecondaryButton, IconButton
    pub mod inputs {       // TextField, TextArea, NumberInput
    pub mod navigation {   // Breadcrumb, Tabs, Sidebar
    pub mod feedback {     // ProgressBar, Spinner, Alert
    pub mod layout {       // Card, Dialog, Sheet
    pub mod lists {        // List, ListItem, Table
}
```

## File Structure

```
/var/www/rustux.com/prod/
├── libs/librustica/           # UI toolkit library
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── widgets/            # Widget implementations
│   │   │   ├── mod.rs
│   │   │   ├── button.rs
│   │   │   ├── label.rs
│   │   │   ├── container.rs
│   │   │   ├── window.rs
│   │   │   └── ...
│   │   ├── layout/             # Layout engine
│   │   │   ├── mod.rs
│   │   │   ├── flexbox.rs
│   │   │   ├── grid.rs
│   │   │   └── constraints.rs
│   │   ├── style/              # Styling system
│   │   │   ├── mod.rs
│   │   │   ├── theme.rs
│   │   │   ├── color.rs
│   │   │   └── painter.rs
│   │   ├── render/             # Rendering backend
│   │   │   ├── mod.rs
│   │   │   ├── context.rs
│   │   │   └── surface.rs
│   │   ├── events/             # Event handling
│   │   │   ├── mod.rs
│   │   │   ├── pointer.rs
│   │   │   ├── keyboard.rs
│   │   │   └── touch.rs
│   │   ├── a11y/               # Accessibility
│   │   │   ├── mod.rs
│   │   │   ├── tree.rs
│   │   │   └── at_spi.rs
│   │   └── components/         # Pre-built components
│   │       ├── mod.rs
│   │       ├── button.rs
│   │       ├── dialog.rs
│   │       └── ...
│   └── examples/               # Example apps
│       ├── basic.rs
│       ├── theme_switcher.rs
│       └── todos.rs
│
└── apps/gui/
    ├── rustica-settings/        # Uses librustica
    ├── rustica-files/           # Uses librustica
    ├── rustica-term/            # Uses librustica
    └── ...
```

## Example Usage

```rust
use librustica::prelude::*;
use librustica::widgets::*;
use librustica::components::*;

fn build_ui() -> impl Widget {
    Container::new()
        .style(Style {
            padding: 16.0,
            background: theme().surface,
            ..default
        })
        .child(
            Flex::column()
                .spacing(16.0)
                .child(
                    Label::new("Rustica Settings")
                        .font(theme().font_h2)
                        .color(theme().primary)
                )
                .child(
                    Button::new("Save")
                        .on_click(||
                            println!("Saved!")
                        })
                )
        )
}
```

## Dependencies

### System Libraries
```
libpango1.0-dev      # Text layout (optional, can use rustic)
libfreetype6-dev     # Font rendering
libfontconfig1-dev   # Font discovery
```

### Rust Crates
```toml
[dependencies]
# Rendering
smithay = "0.18"

# Text rendering
rustybuzz = "0.14"          # Shaping
fontdb = "0.22"              # Font database

# Events
winit = "0.30"               # Window events (or use smithay's)
touch = "0.1"                # Touch handling

# Accessibility
atspi = "0.20"              # AT-SPI bindings

# Utils
unicode-segmentation = "1.11"
unicode-bidi = "0.3"
```

## Performance Targets

| Metric | Target |
|--------|--------|
| Widget creation | <1ms per widget |
| Layout calculation | <5ms for 1000 widgets |
| Render pass | <16ms (60 FPS) |
| Theme switch | <100ms for full tree |

## Accessibility (Built-in)

Every widget implements accessibility:

```rust
pub trait Widget {
    fn accessibility_node(&self) -> A11yNode {
        A11yNode {
            role: self.role(),           // Button, Label, etc.
            label: self.label(),
            description: self.description(),
            state: self.state(),         // Checked, disabled, etc.
            actions: self.actions(),     // Click, focus, etc.
        }
    }
}
```

## Success Criteria

- [ ] Core widget set renders correctly
- [ ] Flexbox layout handles all cases
- [ ] Theme switching works (light/dark)
- [ ] Keyboard navigation is complete
- [ ] Touch input works alongside pointer
- [ ] Accessibility tree is exposed via AT-SPI
- [ ] Performance targets met
- [ ] Example apps demonstrate all features

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Reinventing the wheel | Start minimal, expand as needed |
| Text complexity | Use existing Rust crates (rustybuzz) |
| Performance bottlenecks | Profile early, optimize hot paths |
| Feature creep | Stick to MVP first, add advanced later |
| Mobile vs desktop divergence | Shared codebase, responsive widgets |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [COSMIC libcosmic](https://github.com/pop-os/libcosmic)
- [GTK4 Rust Bindings](https://gtk-rs.org/)
- [Smithay Toolkit](https://docs.rs/smithay/*/smithay/toolkit/index.html)
- [Flutter Engine Architecture](https://flutter.dev/docs/resources/architectural-overview)
- [Druid (Rust UI)](https://github.com/linebender/druid)
