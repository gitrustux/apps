# Theme System & Layout Units Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Design System

## Overview

This specification defines the **design system** for Rustica Shell, including spacing, typography, colors, and theming. It ensures **consistent spacing across all apps**, **seamless light/dark switching**, and **accessible color contrast ratios**.

## Design Philosophy

1. **Consistency First** - All apps use the same base units and scales
2. **Accessibility Built-in** - WCAG AA compliant colors, readable fonts
3. **Mobile-Ready** - Touch targets sized for mobile from day one
4. **Customizable** - Users can theme while maintaining consistency

## Spacing Scale

### Base Units (4px Grid)

All spacing is a multiple of **4 pixels**:

```rust
pub struct Spacing {
    // Micro spacing
    pub xxs: f32,  // 4px  - Micro padding/gap
    pub xs:  f32,  // 8px  - Small padding/gap
    pub sm:  f32,  // 12px - Compact padding/gap
    pub md:  f32,  // 16px - Default padding/gap
    pub lg:  f32,  // 24px - Medium padding/gap
    pub xl:  f32,  // 32px - Large padding/gap
    pub xxl: f32,  // 48px - Extra-large padding
    pub xxxl: f32, // 64px - Huge padding
}

impl Default for Spacing {
    fn default() -> Self {
        Self {
            xxs: 4.0,
            xs:  8.0,
            sm: 12.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
            xxl: 48.0,
            xxxl: 64.0,
        }
    }
}
```

### Usage Examples

```rust
// Button padding
button.padding(spacing.md);  // 16px padding

// Card margin
card.margin(spacing.lg);     // 24px margin

// List item gap
list.gap(spacing.sm);         // 12px between items

// Dialog padding
dialog.padding(spacing.xxl);   // 48px padding
```

## Typography Scale

### Font Sizes

```rust
pub struct Typography {
    // Display fonts
    pub h1: Font,     // 32px - Page title
    pub h2: Font,     // 24px - Section title
    pub h3: Font,     // 20px - Subsection title
    pub h4: Font,     // 16px - Header

    // Body fonts
    pub body: Font,  // 14px - Default text
    pub sm: Font,    // 12px - Small text
    pub xs: Font,    // 10px - Extra small
    pub caption: Font, // 11px - Captions/labels

    // Monospace
    pub mono: Font,  // 14px - Monospace text
    pub mono_sm: Font, // 12px - Small monospace
}

pub struct Font {
    pub size: f32,        // Size in points
    pub weight: FontWeight, // 300-900
    pub line_height: f32, // Line height ratio
}
```

### Font Weights

```rust
pub enum FontWeight {
    Light      = 300,
    Regular    = 400,
    Medium     = 500,
    SemiBold   = 600,
    Bold       = 700,
    ExtraBold  = 800,
}
```

### Line Heights

```rust
// Line heights as ratio of font size
pub enum LineHeight {
    Tight   = 1.2,   // 32px → 38.4px
    Normal  = 1.5,   // 14px → 21px (default)
    Relaxed = 1.75,  // 32px → 56px
}
```

### Font Families

```rust
pub struct FontFamily {
    pub sans: String,    // "Inter"
    pub serif: String,   // "Merriweather"
    pub mono: String,    // "JetBrains Mono"
    pub emoji: String,   // "Noto Color Emoji"
}
```

## Color System

### Semantic Colors

```rust
pub struct Colors {
    // Primary colors
    pub primary: Color,
    pub primary_variant: Color,
    pub on_primary: Color,    // Text/icons on primary

    // Secondary colors
    pub secondary: Color,
    pub secondary_variant: Color,
    pub on_secondary: Color,

    // Backgrounds
    pub background: Color,
    pub surface: Color,       // Cards, panels
    pub surface_variant: Color,

    // Error colors
    pub error: Color,
    pub on_error: Color,
    pub error_container: Color,

    // Warning colors
    pub warning: Color,
    pub on_warning: Color,
    pub warning_container: Color,

    // Success colors
    pub success: Color,
    pub on_success: Color,
    pub success_container: Color,

    // UI elements
    pub outline: Color,       // Borders, dividers
    pub shadow: Color,        // Drop shadows
    pub overlay: Color,      // Modals, tooltips
    pub scrim: Color,        // Backdrop scrim
}
```

### Color Tokens (Light Mode)

```rust
pub fn light_colors() -> Colors {
    Colors {
        // Primary (blue)
        primary: Color::rgb(0x1A, 0x73, 0xE8),
        primary_variant: Color::rgb(0x15, 0x5C, 0xBA),
        on_primary: Color::rgb(0xFF, 0xFF, 0xFF),

        // Secondary (teal)
        secondary: Color::rgb(0x00, 0x96, 0x88),
        secondary_variant: Color::rgb(0x00, 0x77, 0x6B),
        on_secondary: Color::rgb(0xFF, 0xFF, 0xFF),

        // Backgrounds
        background: Color::rgb(0xFF, 0xFF, 0xFF),
        surface: Color::rgb(0xF5, 0xF5, 0xF5),
        surface_variant: Color::rgb(0xE0, 0xE0, 0xE0),

        // Error (red)
        error: Color::rgb(0xBA, 0x1A, 0x1A),
        on_error: Color::rgb(0xFF, 0xFF, 0xFF),
        error_container: Color::rgb(0xFF, 0xBD, 0xBD),

        // Warning (orange)
        warning: Color::rgb(0xFF, 0x6D, 0x00),
        on_warning: Color::rgb(0xFF, 0xFF, 0xFF),
        warning_container: Color::rgb(0xFF, 0xE0, 0xB2),

        // Success (green)
        success: Color::rgb(0x00, 0xC8, 0x53),
        on_success: Color::rgb(0xFF, 0xFF, 0xFF),
        success_container: Color::rgb(0xB2, 0xFF, 0xCC),

        // UI elements
        outline: Color::rgba(0x00, 0x00, 0x00, 0x12),
        shadow: Color::rgba(0x00, 0x00, 0x00, 0x15),
        overlay: Color::rgba(0x00, 0x00, 0x00, 0x32),
        scrim: Color::rgba(0x00, 0x00, 0x00, 0x50),
    }
}
```

### Color Tokens (Dark Mode)

```rust
pub fn dark_colors() -> Colors {
    Colors {
        // Primary (lighter blue)
        primary: Color::rgb(0x64, 0xB5, 0xF6),
        primary_variant: Color::rgb(0x42, 0xA5, 0xF5),
        on_primary: Color::rgb(0x00, 0x00, 0x00),

        // Secondary (lighter teal)
        secondary: Color::rgb(0x80, 0xCB, 0xB4),
        secondary_variant: Color::rgb(0x4D, 0xB8, 0xA0),
        on_secondary: Color::rgb(0x00, 0x00, 0x00),

        // Backgrounds
        background: Color::rgb(0x12, 0x12, 0x12),
        surface: Color::rgb(0x1E, 0x1E, 0x1E),
        surface_variant: Color::rgb(0x2C, 0x2C, 0x2C),

        // Error (lighter red)
        error: Color::rgb(0xFF, 0x84, 0x84),
        on_error: Color::rgb(0x00, 0x00, 0x00),
        error_container: Color::rgb(0x66, 0x00, 0x00),

        // Warning (lighter orange)
        warning: Color::rgb(0xFF, 0xB7, 0x80),
        on_warning: Color::rgb(0x00, 0x00, 0x00),
        warning_container: Color::rgb(0x66, 0x3D, 0x00),

        // Success (lighter green)
        success: Color::rgb(0x69, 0xF0, 0xAE),
        on_success: Color::rgb(0x00, 0x00, 0x00),
        success_container: Color::rgb(0x00, 0x33, 0x1A),

        // UI elements
        outline: Color::rgba(0xFF, 0xFF, 0xFF, 0x12),
        shadow: Color::rgba(0x00, 0x00, 0x00, 0x25),
        overlay: Color::rgba(0x00, 0x00, 0x00, 0x40),
        scrim: Color::rgba(0x00, 0x00, 0x00, 0x60),
    }
}
```

### Accessibility (WCAG AA Compliance)

```rust
// Ensure color contrast ratios meet WCAG AA standards
// - Normal text: ≥ 4.5:1
// - Large text: ≥ 3:1
// - UI components: ≥ 3:1

pub fn check_contrast(foreground: Color, background: Color) -> bool {
    let ratio = foreground.contrast_ratio(background);
    ratio >= 4.5  // WCAG AA for normal text
}
```

## Border Radius

```rust
pub struct BorderRadius {
    pub xs: f32,  // 2px  - Subtle rounding
    pub sm: f32,  // 4px  - Small radius (buttons, inputs)
    pub md: f32,  // 8px  - Medium radius (cards)
    pub lg: f32,  // 12px - Large radius (dialogs)
    pub xl: f32,  // 16px - Extra large (sheets)
    pub full: f32, // 9999px - Pill/circle
}

impl Default for BorderRadius {
    fn default() -> Self {
        Self {
            xs: 2.0,
            sm: 4.0,
            md: 8.0,
            lg: 12.0,
            xl: 16.0,
            full: 9999.0,
        }
    }
}
```

## Shadows

```rust
pub struct Shadows {
    pub sm: BoxShadow,   // Subtle elevation
    pub md: BoxShadow,   // Medium elevation
    pub lg: BoxShadow,   // High elevation
    pub xl: BoxShadow,   // Highest elevation
}

pub struct BoxShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

impl Default for Shadows {
    fn default() -> Self {
        Self {
            sm: BoxShadow {
                offset_x: 0.0,
                offset_y: 1.0,
                blur: 2.0,
                spread: 0.0,
                color: Color::rgba(0, 0, 0, 0.1),
            },
            md: BoxShadow {
                offset_x: 0.0,
                offset_y: 2.0,
                blur: 4.0,
                spread: 0.0,
                color: Color::rgba(0, 0, 0, 0.15),
            },
            lg: BoxShadow {
                offset_x: 0.0,
                offset_y: 4.0,
                blur: 8.0,
                spread: 0.0,
                color: Color::rgba(0, 0, 0, 0.2),
            },
            xl: BoxShadow {
                offset_x: 0.0,
                offset_y: 8.0,
                blur: 16.0,
                spread: 0.0,
                color: Color::rgba(0, 0, 0, 0.25),
            },
        }
    }
}
```

## Touch Targets

### Minimum Sizes

```rust
pub struct TouchTargets {
    pub min_tap_size: Size,  // 44×44px minimum (iOS standard)
    pub min_drag_size: Size, // 48×48px for drag handles
}

impl Default for TouchTargets {
    fn default() -> Self {
        Self {
            min_tap_size: Size { width: 44.0, height: 44.0 },
            min_drag_size: Size { width: 48.0, height: 48.0 },
        }
    }
}
```

### Target Application

```rust
// Buttons must be at least 44×44px
button.min_size(Size { width: 44.0, height: 44.0 });

// Checkbox hit area
checkbox.hitbox(Size { width: 44.0, height: 44.0 });

// List items
list_item.min_height(44.0);

// Icons in toolbar
toolbar.icon_size(24.0);
toolbar.icon_padding(10.0);  // 44 = 24 + 10 + 10
```

## Theme Structure

```rust
pub struct Theme {
    // Scale factor (for DPI scaling)
    pub scale: f32,

    // Spacing scale
    pub spacing: Spacing,

    // Typography
    pub typography: Typography,
    pub font_family: FontFamily,

    // Colors
    pub colors: Colors,

    // Borders
    pub border_radius: BorderRadius,

    // Shadows
    pub shadows: Shadows,

    // Touch targets
    pub touch: TouchTargets,

    // Mode
    pub mode: ThemeMode,
}

pub enum ThemeMode {
    Light,
    Dark,
    HighContrast,
}

impl Theme {
    pub fn light() -> Self {
        Self {
            scale: 1.0,
            spacing: Spacing::default(),
            typography: Typography::default(),
            font_family: FontFamily::default(),
            colors: light_colors(),
            border_radius: BorderRadius::default(),
            shadows: Shadows::default(),
            touch: TouchTargets::default(),
            mode: ThemeMode::Light,
        }
    }

    pub fn dark() -> Self {
        Self {
            scale: 1.0,
            spacing: Spacing::default(),
            typography: Typography::default(),
            font_family: FontFamily::default(),
            colors: dark_colors(),
            border_radius: BorderRadius::default(),
            shadows: Shadows::default(),
            touch: TouchTargets::default(),
            mode: ThemeMode::Dark,
        }
    }

    // Scaled theme for DPI
    pub fn with_scale(&self, scale: f32) -> Self {
        let mut theme = self.clone();
        theme.scale = scale;
        theme.spacing.spacing *= scale;
        theme.typography.font_size *= scale;
        theme
    }
}
```

## Theme Switching

```rust
pub struct ThemeManager {
    current: Theme,
    listeners: Vec<Box<dyn ThemeChangeListener>>,
}

pub trait ThemeChangeListener {
    fn on_theme_changed(&self, theme: &Theme);
}

impl ThemeManager {
    pub fn set_mode(&mut self, mode: ThemeMode) {
        let new_theme = match mode {
            ThemeMode::Light => Theme::light(),
            ThemeMode::Dark => Theme::dark(),
            ThemeMode::HighContrast => Theme::high_contrast(),
        };

        self.current = new_theme;

        // Notify all listeners
        for listener in &self.listeners {
            listener.on_theme_changed(&self.current);
        }
    }

    pub fn register_listener(&mut self, listener: Box<dyn ThemeChangeListener>) {
        self.listeners.push(listener);
    }
}
```

## Styling API

### Widget Styling

```rust
pub struct Style {
    // Size
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,

    // Spacing
    pub padding: SpacingRect,
    pub margin: SpacingRect,

    // Appearance
    pub background: Option<Color>,
    pub border: Option<Border>,
    pub border_radius: Option<f32>,
    pub shadow: Option<BoxShadow>,

    // Typography
    pub font: Option<Font>,
    pub color: Option<Color>,

    // Effects
    pub opacity: Option<f32>,
    pub blur: Option<f32>,
}

impl Style {
    pub fn themed(theme: &Theme) -> Self {
        Self {
            background: Some(theme.colors.background),
            color: Some(theme.colors.on_primary),
            border_radius: Some(theme.border_radius.md),
            ..Default::default()
        }
    }
}
```

### Style Inheritance

```rust
impl Widget {
    pub fn inherit_style(&self, parent: &Style) -> Style {
        Style {
            font: parent.font,          // Inherit font
            color: parent.color,        // Inherit color
            // ... other inherited properties
            ..self.style()
        }
    }
}
```

## File Structure

```
/var/www/rustux.com/prod/libs/librustica/src/
├── style/
│   ├── mod.rs              # Style module
│   ├── theme.rs            # Theme struct
│   ├── colors.rs           # Color definitions
│   ├── spacing.rs          # Spacing scale
│   ├── typography.rs       # Typography scale
│   ├── shadows.rs          # Shadow definitions
│   └── border.rs           # Border radius
│
└── resources/
    └── fonts/              # Font files
        ├── Inter/
        ├── JetBrains Mono/
        ├── Noto Color Emoji/
        └── Merriweather/
```

## Example Usage

```rust
use librustica::style::*;

// Create themed widget
fn build_button(theme: &Theme) -> Button {
    Button::new("Click Me")
        .style(Style {
            background: Some(theme.colors.primary),
            color: Some(theme.colors.on_primary),
            padding: SpacingRect {
                top: theme.spacing.md,
                bottom: theme.spacing.md,
                left: theme.spacing.lg,
                right: theme.spacing.lg,
            },
            border_radius: Some(theme.border_radius.sm),
            shadow: Some(theme.shadows.md),
            min_width: Some(theme.touch.min_tap_size.width),
            min_height: Some(theme.touch.min_tap_size.height),
        })
}

// Apply theme dynamically
fn apply_theme(theme: &Theme) {
    for widget in all_widgets() {
        widget.update_theme(theme);
    }
}
```

## Success Criteria

- [ ] All spacing follows 4px grid
- [ ] All fonts use defined typography scale
- [ ] Color contrast meets WCAG AA (4.5:1 minimum)
- [ ] Light/dark theme switching works
- [ ] Touch targets are ≥44×44px
- [ ] Theme is customizable
- [ ] All apps use consistent spacing

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Inconsistent app styles | Provide style presets, enforce via guidelines |
| Poor accessibility | Automated color contrast checking |
| Mobile touch too small | Minimum size enforcement in toolkit |
| Performance (theme switch) | Lazy theme application, optimize updates |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [Material Design 3 Spacing](https://m3.material.io/styles/spacing/overview)
- [Apple Human Interface Guidelines](https://developer.apple.com/design/human-interface-guidelines/layout)
- [Fluent Design Spacing](https://fluent2.microsoft.com/design/spacing/)
- [WCAG 2.1 Contrast Requirements](https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum)
- [Inter Font Family](https://rsms.me/inter/)
- [JetBrains Mono](https://www.jetbrains.com/lp/mono/)
