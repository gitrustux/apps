# Screenshot Tool (rustica-screenshot) Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Screenshot Tool
**Phase**: 6.4 - Desktop Applications

## Overview

Rustica Screenshot is a **powerful screenshot and screen recording tool** with **region selection**, **window selection**, **multi-monitor support**, **delayed capture**, **annotation**, **screen recording**, **gif creation**, and **quick sharing**. It provides both **GUI** and **command-line interfaces**.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Screenshot Selection UI                                               │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                                                                 │   │
│  │   [Click and drag to select region]                             │   │
│  │   Press Enter to capture, Esc to cancel                         │   │
│  │                                                                 │   │
│  │   1920 × 1080 @ 60Hz                                            │   │
│  │                                                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                           │
│  [Capture Fullscreen] [Capture Window] [Capture Region]  [Delay: 5s▼]   │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│  Screenshot Preview                                                      │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                                                                 │   │
│  │   [Screenshot preview image]                                    │   │
│  │                                                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                           │
│  [Copy] [Save] [Annotate] [Record GIF] [Delete]                          │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│  Annotation Editor                                                       │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                                                                 │   │
│  │   [Annotated screenshot]                                        │   │
│  │                                                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                           │
│  [🖊️ Pen] [█ Rectangle] [○ Ellipse] [T Text] [🔴 Blur] [❌ Delete]    │
│  [Color: █] [Size: ———] [Undo] [Redo] [Apply] [Cancel]                │
└─────────────────────────────────────────────────────────────────────────┘
```

## Main Application Structure

```rust
pub struct ScreenshotApp {
    /// Mode
    mode: CaptureMode,

    /// Capture source
    source: CaptureSource,

    /// Delay
    delay: Duration,

    /// Output format
    format: ImageFormat,

    /// Output directory
    output_dir: PathBuf,

    /// Include cursor
    include_cursor: bool,

    /// Play sound
    play_sound: bool,

    /// Show notification
    show_notification: bool,

    /// Copy to clipboard
    copy_to_clipboard: bool,

    /// Auto-save
    auto_save: bool,

    /// Open in editor
    open_in_editor: bool,
}

pub enum CaptureMode {
    Screenshot,
    ScreenRecord,
    GifRecord,
}

pub enum CaptureSource {
    Fullscreen,
    CurrentOutput,
    Window { window_id: Option<WindowId> },
    Region { rect: Option<Rect> },
}

pub enum ImageFormat {
    PNG,
    JPEG,
    WebP,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct WindowId(String);

pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
```

## Screenshot Capture

```rust
impl ScreenshotApp {
    pub fn capture_screenshot(&self) -> Result<Image, Error> {
        // Apply delay if specified
        if self.delay.as_secs() > 0 {
            self.show_countdown()?;
        }

        // Get capture region
        let region = self.get_capture_region()?;

        // Capture screenshot
        let mut image = self.capture_region(&region)?;

        // Include cursor if requested
        if self.include_cursor {
            self.capture_cursor(&mut image, &region)?;
        }

        // Play sound
        if self.play_sound {
            self.play_shutter_sound()?;
        }

        // Show notification
        if self.show_notification {
            self.show_notification("Screenshot captured")?;
        }

        Ok(image)
    }

    fn get_capture_region(&self) -> Result<Rect, Error> {
        match self.source {
            CaptureSource::Fullscreen => {
                // Get all outputs combined
                let outputs = self.list_outputs()?;

                let mut min_x = i32::MAX;
                let mut min_y = i32::MAX;
                let mut max_x = i32::MIN;
                let mut max_y = i32::MIN;

                for output in &outputs {
                    min_x = min_x.min(output.position.0);
                    min_y = min_y.min(output.position.1);
                    max_x = max_x.max(output.position.0 + output.size.0 as i32);
                    max_y = max_y.max(output.position.1 + output.size.1 as i32);
                }

                Ok(Rect {
                    x: min_x,
                    y: min_y,
                    width: (max_x - min_x) as u32,
                    height: (max_y - min_y) as u32,
                })
            }

            CaptureSource::CurrentOutput => {
                // Get current output (where mouse is)
                let cursor_pos = self.get_cursor_position()?;
                let output = self.find_output_at(cursor_pos)?;

                Ok(Rect {
                    x: output.position.0,
                    y: output.position.1,
                    width: output.size.0,
                    height: output.size.1,
                })
            }

            CaptureSource::Window { ref window_id } => {
                // Get window geometry
                let window = self.find_window(window_id.as_ref())?;
                Ok(window.geometry)
            }

            CaptureSource::Region { ref rect } => {
                if let Some(rect) = rect {
                    Ok(rect.clone())
                } else {
                    // Show region selector
                    self.select_region()
                }
            }
        }
    }

    fn capture_region(&self, region: &Rect) -> Result<Image, Error> {
        // Use wayland screenshot protocol or pipewire
        use pipewire::screen_capture::ScreenCapture;

        let capture = ScreenCapture::new()?;

        // Request capture
        let frame = capture.capture_frame(region)?;

        // Convert to image
        Ok(Image::from_frame(frame))
    }

    fn capture_cursor(&self, image: &mut Image, region: &Rect) -> Result<(), Error> {
        // Get cursor position and image
        let (cursor_pos, cursor_image) = self.get_cursor_info()?;

        // Check if cursor is in region
        if cursor_pos.x >= region.x
            && cursor_pos.y >= region.y
            && cursor_pos.x < region.x + region.width as i32
            && cursor_pos.y < region.y + region.height as i32
        {
            // Draw cursor on image
            let x = (cursor_pos.x - region.x) as u32;
            let y = (cursor_pos.y - region.y) as u32;

            image.composite(&cursor_image, x, y);
        }

        Ok(())
    }
}
```

## Region Selection UI

```rust
pub struct RegionSelector {
    /// Window
    window: Window,

    /// Screen image
    screen_image: Image,

    /// Selection
    selection: Option<Rect>,

    /// Start position
    start_pos: Option<Point>,

    /// Current position
    current_pos: Point,

    /// Outputs
    outputs: Vec<OutputInfo>,

    /// Windows
    windows: Vec<WindowInfo>,
}

pub struct OutputInfo {
    pub name: String,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub refresh_rate: u32,
}

pub struct WindowInfo {
    pub id: WindowId,
    pub title: String,
    pub app_id: String,
    pub geometry: Rect,
}

impl RegionSelector {
    pub fn select_region(&mut self) -> Result<Rect, Error> {
        // Create fullscreen window
        self.window.create_fullscreen()?;
        self.window.set_input_region(None); // Accept input everywhere

        // Capture screen
        self.screen_image = self.capture_screen()?;

        // List outputs and windows
        self.outputs = self.list_outputs()?;
        self.windows = self.list_windows()?;

        // Run selection loop
        self.selection = None;
        self.start_pos = None;

        loop {
            let event = self.window.next_event()?;

            match event {
                Event::Quit => {
                    return Err(Error::Cancelled);
                }

                Event::PointerButton { button, pressed, position } => {
                    if button == 1 && pressed {
                        if let Some(start) = self.start_pos {
                            // Finished selection
                            let end = position;
                            self.selection = Some(self.calculate_rect(start, end));
                            break;
                        } else {
                            // Start selection
                            self.start_pos = Some(position);
                        }
                    } else if button == 3 && pressed {
                        // Cancel on right click
                        return Err(Error::Cancelled);
                    }
                }

                Event::PointerMotion { position } => {
                    self.current_pos = position;

                    // Update selection preview
                    if let Some(start) = self.start_pos {
                        self.selection = Some(self.calculate_rect(start, position));
                    }

                    // Re-render
                    self.render()?;
                }

                Event::Key { key, pressed } => {
                    if key == KeyCode::Return && pressed {
                        if let Some(ref selection) = self.selection {
                            return Ok(selection.clone());
                        }
                    } else if key == KeyCode::Escape && pressed {
                        return Err(Error::Cancelled);
                    }
                }

                _ => {}
            }
        }

        self.selection.ok_or(Error::NoSelection)
    }

    fn render(&self) -> Result<(), Error> {
        let mut ctx = self.window.render_context();

        // Draw dimmed screen
        ctx.fill_rect(self.window.rect(), Color::rgba(0, 0, 0, 180));

        // Draw bright selection
        if let Some(ref selection) = self.selection {
            ctx.draw_image_rect(
                Rect {
                    x: selection.x,
                    y: selection.y,
                    width: selection.width,
                    height: selection.height,
                },
                &self.screen_image,
            );

            // Draw selection border
            ctx.stroke_rect(
                *selection,
                2.0,
                Color::rgb(0, 122, 255),
            );
        }

        // Draw magnifier near cursor
        self.draw_magnifier(&mut ctx)?;

        // Draw info
        self.draw_info(&mut ctx)?;

        ctx.commit()?;

        Ok(())
    }

    fn draw_magnifier(&self, ctx: &mut RenderContext) -> Result<(), Error> {
        let magnifier_size = 200;
        let zoom = 3.0;

        let magnifier_rect = Rect {
            x: self.current_pos.x + 20,
            y: self.current_pos.y + 20,
            width: magnifier_size,
            height: magnifier_size,
        };

        // Draw magnifier background
        ctx.fill_rounded_rect(magnifier_rect, 8.0, Color::rgb(30, 30, 30));

        // Draw zoomed image
        let src_size = magnifier_size as f32 / zoom;
        let src_rect = Rect {
            x: self.current_pos.x - (src_size / 2.0) as i32,
            y: self.current_pos.y - (src_size / 2.0) as i32,
            width: src_size as u32,
            height: src_size as u32,
        };

        ctx.draw_image_zoomed(magnifier_rect, &self.screen_image, src_rect);

        // Draw pixel info
        let pixel = self.screen_image.get_pixel(
            self.current_pos.x as u32,
            self.current_pos.y as u32,
        );

        let info = format!(
            "({}, {})\nRGB({}, {}, {})",
            self.current_pos.x,
            self.current_pos.y,
            pixel.r,
            pixel.g,
            pixel.b
        );

        ctx.draw_text(
            Rect {
                x: magnifier_rect.x + 10,
                y: magnifier_rect.y + magnifier_rect.height - 50,
                width: magnifier_rect.width - 20,
                height: 40,
            },
            &info,
            theme.typography.caption,
            Color::rgb(255, 255, 255),
        );

        Ok(())
    }

    fn draw_info(&self, ctx: &mut RenderContext) -> Result<(), Error> {
        if let Some(ref selection) = self.selection {
            // Find output
            if let Some(output) = self.find_output_for(selection) {
                let info = format!(
                    "{} × {} @ {}Hz",
                    output.size.0,
                    output.size.1,
                    output.refresh_rate
                );

                ctx.draw_text(
                    Rect {
                        x: selection.x,
                        y: selection.y - 30,
                        width: 200,
                        height: 20,
                    },
                    &info,
                    theme.typography.caption,
                    Color::rgb(255, 255, 255),
                );
            }

            // Draw selection size
            let size_info = format!(
                "{} × {}",
                selection.width,
                selection.height
            );

            ctx.draw_text(
                Rect {
                    x: selection.x + selection.width as i32 - 100,
                    y: selection.y + selection.height as i32 + 10,
                    width: 100,
                    height: 20,
                },
                &size_info,
                theme.typography.caption,
                Color::rgb(255, 255, 255),
            );
        }

        Ok(())
    }

    fn calculate_rect(&self, start: Point, end: Point) -> Rect {
        let x = start.x.min(end.x);
        let y = start.y.min(end.y);
        let width = (end.x - start.x).abs() as u32;
        let height = (end.y - start.y).abs() as u32;

        Rect { x, y, width, height }
    }
}
```

## Window Selection

```rust
pub struct WindowSelector {
    /// Windows
    windows: Vec<WindowInfo>,

    /// Highlighted window
    highlighted: Option<WindowId>,

    /// Selected window
    selected: Option<WindowId>,
}

impl WindowSelector {
    pub fn select_window(&mut self) -> Result<WindowInfo, Error> {
        // List windows
        self.windows = self.list_windows()?;

        // Show selection UI
        self.show_selector()?;

        loop {
            let event = self.window.next_event()?;

            match event {
                Event::Quit => {
                    return Err(Error::Cancelled);
                }

                Event::PointerButton { button, pressed, position } => {
                    if button == 1 && pressed {
                        // Find window under cursor
                        if let Some(window) = self.find_window_at(position) {
                            return Ok(window);
                        }
                    }
                }

                Event::PointerMotion { position } => {
                    // Update highlight
                    self.highlighted = self.find_window_at(position).map(|w| w.id.clone());
                    self.render()?;
                }

                Event::Key { key, pressed } => {
                    if key == KeyCode::Escape && pressed {
                        return Err(Error::Cancelled);
                    }
                }

                _ => {}
            }
        }
    }

    fn render(&self) -> Result<(), Error> {
        let mut ctx = self.window.render_context();

        // Dim background
        ctx.fill_rect(self.window.rect(), Color::rgba(0, 0, 0, 100));

        // Draw window highlights
        for window in &self.windows {
            let is_highlighted = self.highlighted.as_ref() == Some(&window.id);

            // Draw window border
            let rect = window.geometry;

            if is_highlighted {
                ctx.stroke_rect(rect, 4.0, Color::rgb(0, 122, 255));
            } else {
                ctx.stroke_rect(rect, 2.0, Color::rgba(255, 255, 255, 100));
            }

            // Draw window title
            if is_highlighted {
                let title_rect = Rect {
                    x: rect.x,
                    y: rect.y - 30,
                    width: 300,
                    height: 24,
                };

                ctx.fill_rounded_rect(title_rect, 4.0, Color::rgba(0, 0, 0, 200));
                ctx.draw_text(
                    title_rect,
                    &window.title,
                    theme.typography.caption,
                    Color::rgb(255, 255, 255),
                );
            }
        }

        ctx.commit()?;

        Ok(())
    }

    fn find_window_at(&self, position: Point) -> Option<&WindowInfo> {
        self.windows.iter()
            .find(|w| {
                position.x >= w.geometry.x
                    && position.y >= w.geometry.y
                    && position.x < w.geometry.x + w.geometry.width as i32
                    && position.y < w.geometry.y + w.geometry.height as i32
            })
    }
}
```

## Annotation Editor

```rust
pub struct AnnotationEditor {
    /// Image
    image: Image,

    /// Annotations
    annotations: Vec<Annotation>,

    /// Current tool
    tool: AnnotationTool,

    /// Color
    color: Color,

    /// Stroke width
    stroke_width: f32,

    /// Undo stack
    undo_stack: Vec<Vec<Annotation>>,

    /// Redo stack
    redo_stack: Vec<Vec<Annotation>>,
}

pub enum AnnotationTool {
    Pen,
    Line,
    Arrow,
    Rectangle,
    Ellipse,
    Text,
    Blur,
    Pixelate,
}

pub enum Annotation {
    Pen { points: Vec<Point>, color: Color, width: f32 },
    Line { start: Point, end: Point, color: Color, width: f32 },
    Arrow { start: Point, end: Point, color: Color, width: f32 },
    Rectangle { rect: Rect, color: Color, width: f32, filled: bool },
    Ellipse { rect: Rect, color: Color, width: f32, filled: bool },
    Text { position: Point, text: String, color: Color, size: f32 },
    Blur { rect: Rect, radius: f32 },
    Pixelate { rect: Rect, size: u32 },
}

impl AnnotationEditor {
    pub fn new(image: Image) -> Self {
        Self {
            image,
            annotations: Vec::new(),
            tool: AnnotationTool::Pen,
            color: Color::rgb(255, 0, 0),
            stroke_width: 3.0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn render(&self, ctx: &mut RenderContext) {
        // Draw image
        ctx.draw_image(ctx.screen_rect(), &self.image);

        // Draw annotations
        for annotation in &self.annotations {
            self.draw_annotation(ctx, annotation);
        }

        // Draw current tool preview
        if let Some(current) = &self.current_annotation {
            self.draw_annotation(ctx, current);
        }
    }

    fn draw_annotation(&self, ctx: &mut RenderContext, annotation: &Annotation) {
        match annotation {
            Annotation::Pen { points, color, width } => {
                if points.len() < 2 {
                    return;
                }

                for window in points.windows(2) {
                    ctx.stroke_line(
                        window[0],
                        window[1],
                        *width,
                        *color,
                    );
                }
            }

            Annotation::Line { start, end, color, width } => {
                ctx.stroke_line(*start, *end, *width, *color);
            }

            Annotation::Arrow { start, end, color, width } => {
                ctx.stroke_line(*start, *end, *width, *color);

                // Draw arrowhead
                let angle = (end.y - start.y).atan2(end.x - start.x);
                let arrow_len = 15.0;

                let left_wing = Point {
                    x: end.x - arrow_len * (angle - std::f32::consts::PI / 6.0).cos(),
                    y: end.y - arrow_len * (angle - std::f32::consts::PI / 6.0).sin(),
                };

                let right_wing = Point {
                    x: end.x - arrow_len * (angle + std::f32::consts::PI / 6.0).cos(),
                    y: end.y - arrow_len * (angle + std::f32::consts::PI / 6.0).sin(),
                };

                ctx.stroke_line(*end, left_wing, *width, *color);
                ctx.stroke_line(*end, right_wing, *width, *color);
            }

            Annotation::Rectangle { rect, color, width, filled } => {
                if *filled {
                    ctx.fill_rect(*rect, color.with_alpha(0.3));
                }
                ctx.stroke_rect(*rect, *width, *color);
            }

            Annotation::Ellipse { rect, color, width, filled } => {
                if *filled {
                    ctx.fill_ellipse(*rect, color.with_alpha(0.3));
                }
                ctx.stroke_ellipse(*rect, *width, *color);
            }

            Annotation::Text { position, text, color, size } => {
                ctx.draw_text(
                    Rect {
                        x: position.x,
                        y: position.y,
                        width: 500,
                        height: *size as u32 * 2,
                    },
                    text,
                    Font::system(),
                    *color,
                );
            }

            Annotation::Blur { rect, radius } => {
                ctx.draw_blur(*rect, *radius);
            }

            Annotation::Pixelate { rect, size } => {
                ctx.draw_pixelate(*rect, *size);
            }
        }
    }

    pub fn apply(&mut self) -> Image {
        // Render annotations to image
        let mut result = self.image.clone();

        for annotation in &self.annotations {
            self.apply_annotation(&mut result, annotation);
        }

        result
    }

    fn apply_annotation(&self, image: &mut Image, annotation: &Annotation) {
        match annotation {
            // Apply annotation to image pixels
            _ => {}
        }
    }

    pub fn undo(&mut self) {
        if !self.annotations.is_empty() {
            self.redo_stack.push(self.annotations.clone());
            self.annotations = self.undo_stack.pop().unwrap_or_default();
        }
    }

    pub fn redo(&mut self) {
        if let Some(annotations) = self.redo_stack.pop() {
            self.undo_stack.push(self.annotations.clone());
            self.annotations = annotations;
        }
    }
}
```

## Screen Recording

```rust
pub struct ScreenRecorder {
    /// Region
    region: Rect,

    /// FPS
    fps: u32,

    /// Output format
    format: RecordingFormat,

    /// Quality
    quality: u32,

    /// Audio
    include_audio: bool,

    /// Show cursor
    show_cursor: bool,
}

pub enum RecordingFormat {
    MP4,
    WebM,
    Gif,
}

impl ScreenRecorder {
    pub fn start(&mut self) -> Result<(), Error> {
        match self.format {
            RecordingFormat::MP4 | RecordingFormat::WebM => {
                self.start_video_recording()?;
            }

            RecordingFormat::Gif => {
                self.start_gif_recording()?;
            }
        }

        Ok(())
    }

    fn start_video_recording(&mut self) -> Result<(), Error> {
        use ffmpeg::{format, codec, encoder};

        // Initialize ffmpeg
        ffmpeg::init()?;

        // Create output
        let output_name = self.generate_output_name();
        let mut output = format::output(&output_name)?;

        // Create video encoder
        let codec = codec::find_by_name("libx264")
            .ok_or(Error::CodecNotFound)?;

        let mut encoder = encoder::open_as(
            codec,
            "mp4",
            codec::Flags::ENCODE,
        )?;

        encoder.set_width(self.region.width);
        encoder.set_height(self.region.height);
        encoder.set_framerate(self.fps, 1);
        encoder.set_bit_rate(4_000_000);

        // Start recording loop
        let mut frame_count = 0u64;

        loop {
            // Capture frame
            let frame = self.capture_frame()?;

            // Encode frame
            encoder.encode(&frame)?;

            frame_count += 1;

            // Check for stop signal
            if self.should_stop() {
                break;
            }
        }

        // Flush encoder
        encoder.flush();

        // Write trailer
        output.write_trailer()?;

        Ok(())
    }

    fn start_gif_recording(&mut self) -> Result<(), Error> {
        use gif::{Encoder, Repeat};

        let output_name = self.generate_output_name();
        let file = std::fs::File::create(output_name)?;

        let mut encoder = Encoder::new(
            file,
            self.region.width as u16,
            self.region.height as u16,
            &[],
        )?;

        encoder.set_repeat(Repeat::Infinite)?;

        loop {
            // Capture frame
            let frame = self.capture_frame()?;

            // Quantize to 256 colors
            let quantized = self.quantize_frame(&frame, 256)?;

            // Encode frame
            encoder.write_frame(&quantized)?;

            // Check for stop signal
            if self.should_stop() {
                break;
            }

            // Delay based on FPS
            std::thread::sleep(Duration::from_millis(1000 / self.fps as u64));
        }

        Ok(())
    }

    fn capture_frame(&self) -> Result<Image, Error> {
        // Capture region
        let capture = ScreenCapture::new()?;
        let frame = capture.capture_frame(&self.region)?;
        Ok(Image::from_frame(frame))
    }

    fn quantize_frame(&self, frame: &Image, max_colors: usize) -> Result<Vec<u8>, Error> {
        // Use median cut algorithm or similar
        // This is a placeholder
        Ok(Vec::new())
    }
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-screenshot/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── app.rs
│       ├── capture.rs
│       ├── region_selector.rs
│       ├── window_selector.rs
│       ├── annotation.rs
│       ├── recorder.rs
│       ├── clipboard.rs
│       └── cli.rs
```

## Dependencies

```toml
[package]
name = "rustica-screenshot"
version = "1.0.0"
edition = "2021"

[dependencies]
# GUI framework
librustica = { path = "../../../libs/librustica" }

# Image handling
image = "0.24"

# Screen capture
pipewire = "0.1"

# Video encoding
ffmpeg = { version = "0.6", optional = true }

# GIF encoding
gif = "0.12"

# Wayland
wayland-client = "0.31"
wayland-protocols = "0.31"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# XDG
dirs = "5.0"

# Date/time
chrono = "0.4"
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Screenshot capture | <100ms | Fullscreen |
| Region selection | 60fps | Interactive |
| Annotation render | 60fps | With annotations |
| GIF encoding | <5s | 10s at 30fps |
| Memory | <100MB | Base usage |

## Success Criteria

- [ ] Screenshot capture works
- [ ] Region selection works
- [ ] Window selection works
- [ ] Annotation editor works
- [ ] Screen recording works
- [ ] GIF creation works
- [ ] Copy to clipboard works
- [ ] Quick actions work
- [ ] CLI interface works
- [ ] Performance targets met

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## Timeline

- Week 1: Basic screenshot capture
- Week 2: Region selection UI
- Week 3: Window selection + delay
- Week 4: Annotation editor
- Week 5: Screen recording
- Week 6: GIF encoding + polish

**Total**: 6 weeks
