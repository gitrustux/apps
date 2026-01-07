# Hi-DPI & Scaling Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Display Scaling

## Overview

This specification defines how Rustica Shell handles high-DPI displays and scaling. It ensures **crisp text on all displays**, **automatic DPI detection**, **fractional scaling support**, and **per-monitor scaling**.

## Design Philosophy

1. **Device Independent Pixels** - All layout uses logical pixels, not physical pixels
2. **Automatic Detection** - DPI detected from hardware and user preferences
3. **Fractional Scaling** - Support 1.25x, 1.5x, 1.75x scales (not just integers)
4. **Per-Monitor** - Different scaling for different displays

## DPI Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Physical Display                           │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  3840 × 2160 pixels (4K @ 27")                           │  │
│  │  Physical DPI: 163                                       │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Rustica Compositor                            │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Display Manager                                          │  │
│  │  - Detect physical size from EDID                        │  │
│  │  - Calculate DPI                                         │  │
│  │  - Determine scale factor                               │  │
│  └───────────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │  Scaling Layer                                            │  │
│  │  - Logical ↔ Physical conversion                        │  │
│  │  - Coordinate transformation                            │  │
│  │  - Buffer scaling                                       │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Logical Coordinate Space                      │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  1920 × 1080 logical pixels (scale: 2.0x)                │  │
│  │  UI renders at this resolution                          │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Application Surface                          │
│  - Widgets sized in logical pixels                              │
│  - Text sized in points                                         │
│  - All coordinates pre-scaled                                   │
└─────────────────────────────────────────────────────────────────┘
```

## Scale Factor System

### Scale Calculation

```rust
use std::sync::Arc;

pub struct DisplayInfo {
    pub name: String,
    pub physical_size: (u32, u32),    // Physical resolution (mm)
    pub resolution: (u32, u32),       // Pixel resolution
    pub refresh_rate: u32,            // Hz
    pub preferred_scale: f32,         // Auto-detected scale
    pub current_scale: f32,           // User-selected scale
}

impl DisplayInfo {
    /// Calculate DPI from physical size and resolution
    pub fn calculate_dpi(&self) -> (f32, f32) {
        let dpi_x = (self.resolution.0 as f32 / self.physical_size.0 as f32) * 25.4;
        let dpi_y = (self.resolution.1 as f32 / self.physical_size.1 as f32) * 25.4;
        (dpi_x, dpi_y)
    }

    /// Calculate ideal scale factor based on DPI
    pub fn calculate_ideal_scale(&self) -> f32 {
        let (dpi_x, dpi_y) = self.calculate_dpi();
        let dpi = (dpi_x + dpi_y) / 2.0;

        // Base scale on 96 DPI as standard (1.0x)
        // 192 DPI → 2.0x, 144 DPI → 1.5x, etc.
        let scale = dpi / 96.0;

        // Round to nearest 0.25 for common scales
        (scale * 4.0).round() / 4.0
    }

    /// Get effective scale (user preference or auto)
    pub fn effective_scale(&self) -> f32 {
        if self.current_scale > 0.0 {
            self.current_scale
        } else {
            self.preferred_scale
        }
    }

    /// Convert logical to physical pixels
    pub fn logical_to_physical(&self, logical: u32) -> u32 {
        (logical as f32 * self.effective_scale()) as u32
    }

    /// Convert physical to logical pixels
    pub fn physical_to_logical(&self, physical: u32) -> u32 {
        (physical as f32 / self.effective_scale()) as u32
    }
}
```

### Scale Manager

```rust
pub struct ScaleManager {
    displays: Vec<DisplayInfo>,
    primary_display: usize,
    global_scale: f32,
}

impl ScaleManager {
    pub fn new() -> Self {
        Self {
            displays: Vec::new(),
            primary_display: 0,
            global_scale: 1.0,
        }
    }

    /// Add or update a display
    pub fn add_display(&mut self, display: DisplayInfo) {
        let existing = self.displays.iter()
            .position(|d| d.name == display.name);

        match existing {
            Some(idx) => {
                self.displays[idx] = display;
            }
            None => {
                self.displays.push(display);
            }
        }
    }

    /// Remove a display
    pub fn remove_display(&mut self, name: &str) {
        self.displays.retain(|d| d.name != name);
    }

    /// Get scale factor for a display
    pub fn get_scale(&self, display_name: &str) -> f32 {
        self.displays.iter()
            .find(|d| d.name == display_name)
            .map(|d| d.effective_scale())
            .unwrap_or(self.global_scale)
    }

    /// Set scale factor for a display
    pub fn set_scale(&mut self, display_name: &str, scale: f32) {
        if let Some(display) = self.displays.iter_mut().find(|d| d.name == display_name) {
            display.current_scale = scale;
        }
    }

    /// Set global scale (used when no display-specific scale)
    pub fn set_global_scale(&mut self, scale: f32) {
        self.global_scale = scale;
    }

    /// Get primary display scale
    pub fn primary_scale(&self) -> f32 {
        self.displays.get(self.primary_display)
            .map(|d| d.effective_scale())
            .unwrap_or(self.global_scale)
    }

    /// Detect displays and their properties
    pub fn detect_displays(&mut self, backend: &mut RenderBackend) {
        // Query backend for connected outputs
        let outputs = backend.get_outputs();

        for output in outputs {
            let display = DisplayInfo {
                name: output.name.clone(),
                physical_size: output.physical_size,
                resolution: output.resolution,
                refresh_rate: output.refresh_rate,
                preferred_scale: output.calculate_ideal_scale(),
                current_scale: 0.0,  // Auto initially
            };

            self.add_display(display);
        }
    }
}
```

## Coordinate Transformation

### Scaling Layer

```rust
pub struct ScalingContext {
    scale: f32,
    transform: GlobalTransform,
}

#[derive(Clone, Copy)]
pub struct GlobalTransform {
    pub scale: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl ScalingContext {
    pub fn new(scale: f32) -> Self {
        Self {
            scale,
            transform: GlobalTransform {
                scale,
                offset_x: 0.0,
                offset_y: 0.0,
            },
        }
    }

    /// Transform logical point to physical
    pub fn logical_to_physical(&self, point: Point) -> Point {
        Point {
            x: point.x * self.transform.scale + self.transform.offset_x,
            y: point.y * self.transform.scale + self.transform.offset_y,
        }
    }

    /// Transform physical point to logical
    pub fn physical_to_logical(&self, point: Point) -> Point {
        Point {
            x: (point.x - self.transform.offset_x) / self.transform.scale,
            y: (point.y - self.transform.offset_y) / self.transform.scale,
        }
    }

    /// Transform logical size to physical
    pub fn scale_size(&self, size: Size) -> Size {
        Size {
            width: size.width * self.transform.scale,
            height: size.height * self.transform.scale,
        }
    }

    /// Transform physical size to logical
    pub fn unscale_size(&self, size: Size) -> Size {
        Size {
            width: size.width / self.transform.scale,
            height: size.height / self.transform.scale,
        }
    }

    /// Transform rectangle
    pub fn scale_rect(&self, rect: Rectangle) -> Rectangle {
        Rectangle {
            x: rect.x * self.transform.scale + self.transform.offset_x,
            y: rect.y * self.transform.scale + self.transform.offset_y,
            width: rect.width * self.transform.scale,
            height: rect.height * self.transform.scale,
        }
    }

    /// Get logical scale factor
    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Update scale factor
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
        self.transform.scale = scale;
    }
}

#[derive(Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
```

## Rendering with Scaling

### Buffer Management

```rust
pub struct ScaledBuffer {
    // Logical buffer (what app renders to)
    logical_buffer: WlBuffer,
    logical_size: (u32, u32),

    // Physical buffer (what compositor displays)
    physical_buffer: WlBuffer,
    physical_size: (u32, u32),

    scale: f32,
}

impl ScaledBuffer {
    /// Create a new scaled buffer
    pub fn new(
        logical_size: (u32, u32),
        scale: f32,
        renderer: &mut Renderer,
    ) -> Result<Self> {
        // Calculate physical size
        let physical_size = (
            (logical_size.0 as f32 * scale) as u32,
            (logical_size.1 as f32 * scale) as u32,
        );

        // Create buffers
        let logical_buffer = renderer.create_buffer(logical_size)?;
        let physical_buffer = renderer.create_buffer(physical_size)?;

        Ok(Self {
            logical_buffer,
            logical_size,
            physical_buffer,
            physical_size,
            scale,
        })
    }

    /// Scale logical buffer to physical buffer
    pub fn scale_to_physical(&self, renderer: &mut Renderer) -> Result<()> {
        // Read from logical buffer
        let logical_data = renderer.read_buffer(&self.logical_buffer)?;

        // Scale using high-quality filter
        let physical_data = self.scale_image(&logical_data)?;

        // Write to physical buffer
        renderer.write_buffer(&self.physical_buffer, &physical_data)?;

        Ok(())
    }

    /// Scale image data
    fn scale_image(&self, src: &[u8]) -> Result<Vec<u8>> {
        // Use image scaling library
        let src_image = image::RgbImage::from_raw(
            self.logical_size.0,
            self.logical_size.1,
            src,
        ).ok_or(Error::InvalidImageData)?;

        // Resize with high-quality filter
        let resized = image::imageops::resize(
            &src_image,
            self.physical_size.0,
            self.physical_size.1,
            image::imageops::FilterType::Lanczos3,
        );

        Ok(resized.into_raw())
    }
}
```

### EGL Scaling

```rust
pub struct EGLScaler {
    egl_context: EGLContext,
    shader_program: GLuint,
}

impl EGLScaler {
    pub fn new(egl: &EGL) -> Result<Self> {
        let egl_context = egl.create_context()?;
        let shader_program = Self::compile_shader()?;

        Ok(Self {
            egl_context,
            shader_program,
        })
    }

    /// Scale texture using GPU
    pub fn scale_texture(
        &self,
        src_texture: GLuint,
        src_size: (u32, u32),
        dst_size: (u32, u32),
    ) -> Result<GLuint> {
        unsafe {
            // Create framebuffer
            let mut fbo = 0;
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

            // Create destination texture
            let mut dst_texture = 0;
            gl::GenTextures(1, &mut dst_texture);
            gl::BindTexture(gl::TEXTURE_2D, dst_texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                dst_size.0 as i32,
                dst_size.1 as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );

            // Set texture parameters
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            // Attach to framebuffer
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                dst_texture,
                0,
            );

            // Render scaled texture
            gl::Viewport(0, 0, dst_size.0 as i32, dst_size.1 as i32);
            gl::UseProgram(self.shader_program);

            // Set up quad
            self.render_quad(src_texture, src_size, dst_size);

            // Cleanup
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::DeleteFramebuffers(1, &fbo);

            Ok(dst_texture)
        }
    }

    unsafe fn render_quad(&self, texture: GLuint, src: (u32, u32), dst: (u32, u32)) {
        // Bind source texture
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture);

        // Set uniforms
        let src_size_loc = gl::GetUniformLocation(self.shader_program, cstr!("srcSize").as_ptr());
        gl::Uniform2f(src_size_loc, src.0 as f32, src.1 as f32);

        // Draw quad
        gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
    }

    fn compile_shader() -> Result<GLuint> {
        const VERTEX_SHADER: &str = r#"
            #version 300 es
            in vec2 position;
            in vec2 texCoord;
            out vec2 vTexCoord;
            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                vTexCoord = texCoord;
            }
        "#;

        const FRAGMENT_SHADER: &str = r#"
            #version 300 es
            precision mediump float;
            in vec2 vTexCoord;
            uniform sampler2D tex;
            uniform vec2 srcSize;
            out vec4 fragColor;
            void main() {
                fragColor = texture(tex, vTexCoord);
            }
        "#;

        // Compile and link shaders...
        Ok(1)  // Simplified
    }
}
```

## Font Scaling

### Point-Based Sizing

```rust
pub struct FontScaler {
    scale: f32,
    dpi: f32,
}

impl FontScaler {
    pub fn new(scale: f32, dpi: f32) -> Self {
        Self { scale, dpi }
    }

    /// Convert points to pixels at given DPI
    pub fn points_to_pixels(&self, points: f32) -> f32 {
        // Points are 1/72 inch
        // Pixels = points * (DPI / 72)
        (points * self.dpi / 72.0) * self.scale
    }

    /// Get font size in pixels for a given point size
    pub fn get_font_size(&self, point_size: f32) -> f32 {
        self.points_to_pixels(point_size)
    }

    /// Scale font for display
    pub fn scale_font(&self, font: &Font) -> ScaledFont {
        ScaledFont {
            font: font.clone(),
            size: self.get_font_size(font.point_size),
            scale: self.scale,
        }
    }
}

pub struct Font {
    pub family: String,
    pub point_size: f32,
    pub weight: FontWeight,
    pub style: FontStyle,
}

pub struct ScaledFont {
    pub font: Font,
    pub size: f32,  // Size in pixels
    pub scale: f32,
}

// Standard font sizes in points
pub struct FontSizes;

impl FontSizes {
    pub const H1: f32 = 32.0;
    pub const H2: f32 = 24.0;
    pub const H3: f32 = 20.0;
    pub const H4: f32 = 16.0;
    pub const BODY: f32 = 14.0;
    pub const SMALL: f32 = 12.0;
    pub const CAPTION: f32 = 11.0;
}
```

## Per-Monitor Scaling

### Mixed DPI Support

```rust
pub struct MultiDisplayManager {
    displays: Vec<DisplayContext>,
}

pub struct DisplayContext {
    display: DisplayInfo,
    scaling: ScalingContext,
    output: WlOutput,
}

impl MultiDisplayManager {
    /// Handle window spanning multiple displays
    pub fn handle_cross_display_window(&self, window: &mut Window) {
        let rects = window.display_rects();

        for (display, rect) in rects {
            let scale = self.get_scale_for_display(display);
            let scaling = ScalingContext::new(scale);

            // Scale for this display
            let scaled_rect = scaling.scale_rect(rect);
            window.set_display_rect(display, scaled_rect);
        }
    }

    /// Get scale for display
    fn get_scale_for_display(&self, display_name: &str) -> f32 {
        self.displays.iter()
            .find(|d| d.display.name == display_name)
            .map(|d| d.display.effective_scale())
            .unwrap_or(1.0)
    }

    /// Handle window moved between displays
    pub fn handle_display_change(&self, window: &mut Window, from: &str, to: &str) {
        let from_scale = self.get_scale_for_display(from);
        let to_scale = self.get_scale_for_display(to);

        // Adjust window size for new scale
        let current_size = window.size();
        let new_size = Size {
            width: current_size.width * to_scale / from_scale,
            height: current_size.height * to_scale / from_scale,
        };

        window.set_size(new_size);
    }
}
```

## Configuration

### User Settings

```toml
# config.toml

[display]
# Scale factor for global scaling
scale = 1.0

# Auto-detect DPI
auto_detect_dpi = true

# Per-display scaling
[[display.displays]]
name = "HDMI-1"
scale = 2.0

[[display.displays]]
name = "eDP-1"
scale = 1.5

# Fractional scaling
display.fractional_scaling = true
display.allowed_scales = [1.0, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0]

# Text scaling (separate from UI scaling)
display.text_scale_dpi = 96.0
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Scale change | <100ms | Setting change to visible |
| Buffer scale | <16ms | Frame to scaled |
| Font load | <50ms | Scale change to rendered |
| Memory overhead | <100MB | With 2x scaling |
| GPU scaling | <5ms | GPU texture scale |

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── libs/librustica-scaling/
│   ├── src/
│   │   ├── mod.rs
│   │   ├── context.rs          # Scaling context
│   │   ├── display.rs          # Display info
│   │   ├── manager.rs          # Scale manager
│   │   ├── buffer.rs           # Scaled buffers
│   │   ├── egl.rs              # EGL scaling
│   │   └── font.rs             # Font scaling
│   └── Cargo.toml
│
└── rustica-comp/src/
    └── scaling/
        ├── mod.rs
        ├── protocol.rs         # Wayland scaling protocol
        └── renderer.rs         # Scaling renderer
```

## Example Usage

```rust
use librustica_scaling::*;

// Initialize scale manager
let mut manager = ScaleManager::new();
manager.detect_displays(&mut backend);

// Set scale for display
manager.set_scale("HDMI-1", 2.0);

// Get scaling context
let scale = manager.get_scale("HDMI-1");
let ctx = ScalingContext::new(scale);

// Transform coordinates
let physical = ctx.logical_to_physical(Point { x: 100.0, y: 100.0 });
let size = ctx.scale_size(Size { width: 800.0, height: 600.0 });

// Scale font
let font_scaler = FontScaler::new(scale, 96.0);
let font_size = font_scaler.get_font_size(14.0);  // 14pt
```

## Success Criteria

- [ ] DPI detection works for all displays
- [ ] Fractional scaling (1.25x, 1.5x, etc.) works
- [ ] Per-monitor scaling works
- [ ] Text is crisp at all scales
- [ ] Scale changes apply in <100ms
- [ ] GPU scaling works
- [ ] No visual artifacts when scaling
- [ ] Performance targets met
- [ ] Tests pass

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Blurry text at non-integer scales | Use fractional positioning, subpixel rendering |
| Performance impact on high-DPI | GPU scaling, texture caching |
| Apps assume 96 DPI | Virtual DPI, scale all coordinates |
| Multi-display complexity | Per-display contexts, automatic scaling |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [Wayland Scaling Protocol](https://wayland.freedesktop.org/docs/html/ch04.html#sect-Protocol-WlOutput)
- [Fractional Scaling in Wayland](https://wayland.freedesktop.org/docs/html/ch04.html#sect-Protocol-WlSurface)
- [GNOME HiDPI Support](https://wiki.gnome.org/HowDoI/HiDpi)
- [Windows DPI Scaling](https://docs.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows)
- [macOS Retina Displays](https://developer.apple.com/design/human-interface-guidelines/macos/visual-design/color/)
