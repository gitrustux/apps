# Phase 10.3: UI/UX Testing

## Overview

**Component**: UI Test Framework
**Purpose**: Automated UI testing and user experience validation
**Language**: Rust
**Dependencies:** wayland-scanner, smithay, image

## Goals

1. **Visual Testing**: Automated visual regression testing
2. **Interaction Testing**: Test user interactions
3. **Layout Validation**: Verify responsive layouts
4. **User Flows**: Test complete user workflows
5. **Usability**: Validate usability standards

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   UI Test Framework                         │
├─────────────────────────────────────────────────────────────┤
│  VisualRegression  │  InteractionTest  │  LayoutValidator   │
│  - Screenshot      │  - Click/Type     │  - Responsive      │
│  - Compare         │  - Drag/Swipe     │  - Breakpoints     │
│  - Diff report     │  - Gesture        │  - Alignment       │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              Test Compositor (Headless/WLheadless)           │
└─────────────────────────────────────────────────────────────┘
```

## Visual Regression Testing

```rust
//! tests/ui/visual_regression.rs

use rustica_test::prelude::*;
use rustica_comp::Compositor;
use image::{DynamicImage, ImageFormat};
use std::path::PathBuf;

pub struct VisualRegression {
    baseline_dir: PathBuf,
    output_dir: PathBuf,
    diff_dir: PathBuf,
}

impl VisualRegression {
    pub fn new() -> Result<Self> {
        let baseline_dir = PathBuf::from("tests/ui/baselines");
        let output_dir = PathBuf::from("tests/ui/output");
        let diff_dir = PathBuf::from("tests/ui/diffs");

        std::fs::create_dir_all(&baseline_dir)?;
        std::fs::create_dir_all(&output_dir)?;
        std::fs::create_dir_all(&diff_dir)?;

        Ok(Self {
            baseline_dir,
            output_dir,
            diff_dir,
        })
    }

    /// Capture screenshot of component
    pub async fn capture_screenshot(
        &self,
        compositor: &Compositor,
        widget: &Widget,
        name: &str,
    ) -> Result<DynamicImage> {
        let output_path = self.output_dir.join(format!("{}.png", name));

        // Render widget to compositor
        widget.render(compositor)?;

        // Capture screenshot
        let screenshot = compositor.capture_screenshot().await?;

        // Save screenshot
        screenshot.save_with_format(&output_path, ImageFormat::Png)?;

        Ok(screenshot)
    }

    /// Compare screenshot with baseline
    pub async fn compare_with_baseline(
        &self,
        screenshot: &DynamicImage,
        name: &str,
        threshold: f64,
    ) -> Result<ComparisonResult> {
        let baseline_path = self.baseline_dir.join(format!("{}.png", name));

        // Check if baseline exists
        if !baseline_path.exists() {
            // Create new baseline
            screenshot.save_with_format(&baseline_path, ImageFormat::Png)?;
            return Ok(ComparisonResult::BaselineCreated);
        }

        // Load baseline
        let baseline = image::open(&baseline_path)?;

        // Compare images
        let diff = self.compute_difference(&baseline, screenshot)?;

        // Calculate similarity percentage
        let similarity = 100.0 - (diff.diff_pixel_count as f64
            / diff.total_pixels as f64 * 100.0);

        if similarity < (100.0 - threshold * 100.0) {
            // Save diff image
            let diff_path = self.diff_dir.join(format!("{}-diff.png", name));
            diff.diff_image.save_with_format(&diff_path, ImageFormat::Png)?;

            return Ok(ComparisonResult::Different {
                similarity,
                diff_image: diff_path,
            });
        }

        Ok(ComparisonResult::Similar { similarity })
    }

    fn compute_difference(
        &self,
        baseline: &DynamicImage,
        current: &DynamicImage,
    ) -> Result<ImageDiff> {
        let baseline_rgba = baseline.to_rgba8();
        let current_rgba = current.to_rgba8();

        let mut diff_image = DynamicImage::new_rgba8(
            baseline_rgba.width(),
            baseline_rgba.height(),
        );

        let mut diff_pixel_count = 0;

        for (x, y, pixel) in diff_image.to_rgba8().enumerate_pixels_mut() {
            let baseline_pixel = baseline_rgba.get_pixel(x, y);
            let current_pixel = current_rgba.get_pixel(x, y);

            // Calculate absolute difference
            let r_diff = (baseline_pixel[0] as i16 - current_pixel[0] as i16).abs() as u8;
            let g_diff = (baseline_pixel[1] as i16 - current_pixel[1] as i16).abs() as u8;
            let b_diff = (baseline_pixel[2] as i16 - current_pixel[2] as i16).abs() as u8;

            // Mark as red if difference > threshold
            if r_diff > 10 || g_diff > 10 || b_diff > 10 {
                *pixel = image::Rgba([255, 0, 0, 255]);
                diff_pixel_count += 1;
            } else {
                *pixel = image::Rgba([0, 0, 0, 0]);
            }
        }

        Ok(ImageDiff {
            diff_image,
            diff_pixel_count,
            total_pixels: baseline_rgba.width() as usize * baseline_rgba.height() as usize,
        })
    }

    /// Update baseline image
    pub async fn update_baseline(&self, screenshot: &DynamicImage, name: &str) -> Result<()> {
        let baseline_path = self.baseline_dir.join(format!("{}.png", name));
        screenshot.save_with_format(&baseline_path, ImageFormat::Png)?;
        Ok(())
    }
}

pub struct ImageDiff {
    pub diff_image: DynamicImage,
    pub diff_pixel_count: usize,
    pub total_pixels: usize,
}

pub enum ComparisonResult {
    BaselineCreated,
    Similar { similarity: f64 },
    Different { similarity: f64, diff_image: PathBuf },
}
```

## Interaction Testing

```rust
//! tests/ui/interaction.rs

use rustica_test::prelude::*;
use rustica_comp::{Compositor, Window};
use std::time::Duration;

pub struct InteractionTester {
    compositor: Compositor,
}

impl InteractionTester {
    pub fn new(compositor: Compositor) -> Self {
        Self { compositor }
    }

    /// Simulate mouse click on widget
    pub async fn click(&self, window_id: u32, x: f64, y: f64) -> Result<()> {
        self.compositor.send_mouse_move(window_id, x, y).await?;
        tokio::time::sleep(Duration::from_millis(50)).await;
        self.compositor.send_mouse_button(window_id, 1, true).await?;
        tokio::time::sleep(Duration::from_millis(50)).await;
        self.compositor.send_mouse_button(window_id, 1, false).await?;
        Ok(())
    }

    /// Simulate double click
    pub async fn double_click(&self, window_id: u32, x: f64, y: f64) -> Result<()> {
        self.click(window_id, x, y).await?;
        tokio::time::sleep(Duration::from_millis(50)).await;
        self.click(window_id, x, y).await?;
        Ok(())
    }

    /// Simulate text input
    pub async fn type_text(&self, window_id: u32, text: &str) -> Result<()> {
        for ch in text.chars() {
            self.compositor.send_key(window_id, ch_to_keycode(ch)).await?;
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        Ok(())
    }

    /// Simulate key press
    pub async fn press_key(&self, window_id: u32, key: KeyCode) -> Result<()> {
        self.compositor.send_key(window_id, key).await?;
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    /// Simulate drag operation
    pub async fn drag(
        &self,
        window_id: u32,
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
    ) -> Result<()> {
        // Press at start
        self.compositor.send_mouse_move(window_id, start_x, start_y).await?;
        self.compositor.send_mouse_button(window_id, 1, true).await?;

        // Move to end
        let steps = 10;
        for i in 0..=steps {
            let x = start_x + (end_x - start_x) * (i as f64 / steps as f64);
            let y = start_y + (end_y - start_y) * (i as f64 / steps as f64);
            self.compositor.send_mouse_move(window_id, x, y).await?;
            tokio::time::sleep(Duration::from_millis(20)).await;
        }

        // Release
        self.compositor.send_mouse_button(window_id, 1, false).await?;

        Ok(())
    }

    /// Simulate swipe gesture
    pub async fn swipe(
        &self,
        window_id: u32,
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Touch down
        self.compositor.send_touch_down(window_id, 0, start_x, start_y).await?;

        // Move through points
        let steps = 10;
        for i in 1..=steps {
            let x = start_x + (end_x - start_x) * (i as f64 / steps as f64);
            let y = start_y + (end_y - start_y) * (i as f64 / steps as f64);
            self.compositor.send_touch_motion(window_id, 0, x, y).await?;
            tokio::time::sleep(Duration::from_millis(16)).await;  // 60fps
        }

        // Touch up
        self.compositor.send_touch_up(window_id, 0, end_x, end_y).await?;

        // Check gesture was recognized
        let duration = start_time.elapsed();
        if duration < Duration::from_millis(500) {
            // Fast swipe
            Ok(())
        } else {
            Err(anyhow::anyhow!("Swipe too slow"))
        }
    }

    /// Simulate pinch gesture
    pub async fn pinch(
        &self,
        window_id: u32,
        center_x: f64,
        center_y: f64,
        scale: f64,
    ) -> Result<()> {
        let initial_distance = 100.0;

        // Two fingers start
        self.compositor.send_touch_down(window_id, 0, center_x - 50.0, center_y).await?;
        self.compositor.send_touch_down(window_id, 1, center_x + 50.0, center_y).await?;

        // Move fingers apart/together
        let steps = 10;
        for i in 1..=steps {
            let offset = (initial_distance * scale) * (i as f64 / steps as f64);
            self.compositor.send_touch_motion(window_id, 0, center_x - offset / 2.0, center_y).await?;
            self.compositor.send_touch_motion(window_id, 1, center_x + offset / 2.0, center_y).await?;
            tokio::time::sleep(Duration::from_millis(16)).await;
        }

        // Lift fingers
        self.compositor.send_touch_up(window_id, 0, center_x, center_y).await?;
        self.compositor.send_touch_up(window_id, 1, center_x, center_y).await?;

        Ok(())
    }
}

fn ch_to_keycode(ch: char) -> KeyCode {
    // Simplified mapping
    match ch {
        'a'..='z' => KeyCode::KEY_A,
        'A'..='Z' => KeyCode::KEY_A,
        '0'..='9' => KeyCode::KEY_0,
        ' ' => KeyCode::KEY_SPACE,
        '\n' => KeyCode::KEY_ENTER,
        '\t' => KeyCode::KEY_TAB,
        _ => KeyCode::KEY_UNKNOWN,
    }
}
```

## Layout Validation

```rust
//! tests/ui/layout.rs

use rustica_test::prelude::*;
use rustica_comp::{Compositor, Window};
use std::collections::HashMap;

pub struct LayoutValidator {
    breakpoints: Vec<Breakpoint>,
}

#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
}

impl LayoutValidator {
    pub fn new() -> Self {
        Self {
            breakpoints: vec![
                Breakpoint {
                    name: "mobile".to_string(),
                    width: 360,
                    height: 640,
                    scale_factor: 2.0,
                },
                Breakpoint {
                    name: "tablet".to_string(),
                    width: 768,
                    height: 1024,
                    scale_factor: 1.5,
                },
                Breakpoint {
                    name: "desktop".to_string(),
                    width: 1920,
                    height: 1080,
                    scale_factor: 1.0,
                },
            ],
        }
    }

    /// Validate layout across all breakpoints
    pub async fn validate_layouts(
        &self,
        widget: &Widget,
        compositor: &Compositor,
    ) -> Result<LayoutReport> {
        let mut results = HashMap::new();

        for breakpoint in &self.breakpoints {
            // Set output size
            compositor.set_output_size(breakpoint.width, breakpoint.height).await?;
            compositor.set_scale_factor(breakpoint.scale_factor).await?;

            // Render widget
            widget.render(compositor)?;

            // Validate layout
            let validation = self.validate_widget_layout(widget, compositor).await?;
            results.insert(breakpoint.name.clone(), validation);
        }

        Ok(LayoutReport { results })
    }

    async fn validate_widget_layout(
        &self,
        widget: &Widget,
        compositor: &Compositor,
    ) -> Result<LayoutValidation> {
        let mut issues = Vec::new();

        // Check widget is within bounds
        let geom = widget.geometry();
        let output = compositor.output_geometry().await?;

        if geom.x < 0.0 || geom.y < 0.0 {
            issues.push(LayoutIssue::WidgetOutOfBounds);
        }

        if geom.x + geom.width > output.width as f64 {
            issues.push(LayoutIssue::WidgetOverflowsRight);
        }

        if geom.y + geom.height > output.height as f64 {
            issues.push(LayoutIssue::WidgetOverflowsBottom);
        }

        // Check for overlaps
        let children = widget.children();
        for (i, child1) in children.iter().enumerate() {
            for child2 in children.iter().skip(i + 1) {
                if self.widgets_overlap(child1.geometry(), child2.geometry()) {
                    issues.push(LayoutIssue::WidgetsOverlap {
                        widget1: child1.id().to_string(),
                        widget2: child2.id().to_string(),
                    });
                }
            }
        }

        // Check minimum touch targets (44x44px)
        for child in children {
            if child.is_interactive() {
                if child.geometry().width < 44.0 || child.geometry().height < 44.0 {
                    issues.push(LayoutIssue::TouchTargetTooSmall {
                        widget: child.id().to_string(),
                        size: (child.geometry().width, child.geometry().height),
                    });
                }
            }
        }

        // Check text readability
        self.validate_text(widget, &mut issues)?;

        Ok(LayoutValidation {
            valid: issues.is_empty(),
            issues,
        })
    }

    fn widgets_overlap(&self, geom1: Geometry, geom2: Geometry) -> bool {
        geom1.x < geom2.x + geom2.width
            && geom1.x + geom1.width > geom2.x
            && geom1.y < geom2.y + geom2.height
            && geom1.y + geom1.height > geom2.y
    }

    fn validate_text(&self, widget: &Widget, issues: &mut Vec<LayoutIssue>) -> Result<()> {
        // Check font size (minimum 12px)
        if let Some(font_size) = widget.font_size() {
            if font_size < 12.0 {
                issues.push(LayoutIssue::FontSizeTooSmall {
                    widget: widget.id().to_string(),
                    size: font_size,
                });
            }
        }

        // Check line height (1.4x font size minimum)
        if let Some((font_size, line_height)) = widget.line_height() {
            if line_height < font_size * 1.4 {
                issues.push(LayoutIssue::LineHeightTooSmall {
                    widget: widget.id().to_string(),
                    line_height,
                    font_size,
                });
            }
        }

        // Check text contrast
        if let Some((fg, bg)) = widget.text_colors() {
            let contrast = calculate_contrast_ratio(fg, bg);
            if contrast < 4.5 {
                issues.push(LayoutIssue::InsufficientContrast {
                    widget: widget.id().to_string(),
                    contrast,
                });
            }
        }

        Ok(())
    }
}

pub struct LayoutReport {
    pub results: HashMap<String, LayoutValidation>,
}

pub struct LayoutValidation {
    pub valid: bool,
    pub issues: Vec<LayoutIssue>,
}

#[derive(Debug, Clone)]
pub enum LayoutIssue {
    WidgetOutOfBounds,
    WidgetOverflowsRight,
    WidgetOverflowsBottom,
    WidgetsOverlap { widget1: String, widget2: String },
    TouchTargetTooSmall { widget: String, size: (f64, f64) },
    FontSizeTooSmall { widget: String, size: f64 },
    LineHeightTooSmall { widget: String, line_height: f64, font_size: f64 },
    InsufficientContrast { widget: String, contrast: f64 },
}
```

## User Flow Testing

```rust
//! tests/ui/user_flows.rs

use rustica_test::prelude::*;
use rustica_comp::Compositor;
use InteractionTester;

pub struct UserFlowTest {
    compositor: Compositor,
    interaction: InteractionTester,
}

impl UserFlowTest {
    pub fn new(compositor: Compositor) -> Self {
        let interaction = InteractionTester::new(compositor.clone());
        Self {
            compositor,
            interaction,
        }
    }

    /// Test: Launch application from panel
    pub async fn test_launch_app_from_panel(&self) -> Result<()> {
        // Open launcher
        self.interaction.click(1, 100, 500).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Type app name
        self.interaction.type_text(2, "firefox").await?;
        tokio::time::sleep(Duration::from_millis(300)).await;

        // Press Enter
        self.interaction.press_key(2, KeyCode::KEY_ENTER).await?;

        // Wait for window
        wait_for(|| async {
            let windows = self.compositor.list_windows().await?;
            Ok(windows.iter().any(|w| w.app_id() == Some("org.mozilla.firefox")))
        }, Duration::from_secs(5)).await?;

        Ok(())
    }

    /// Test: Open and close settings
    pub async fn test_open_settings(&self) -> Result<()> {
        // Click settings icon in panel
        self.interaction.click(1, 1800, 20).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Verify settings window opened
        let windows = self.compositor.list_windows().await?;
        let settings_window = windows.iter()
            .find(|w| w.app_id() == Some("org.rustica.Settings"))
            .ok_or(anyhow::anyhow!("Settings window not found"))?;

        // Navigate to Appearance panel
        self.interaction.click(settings_window.id(), 100, 100).await?;
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Verify panel changed
        // ...

        // Close window
        self.interaction.press_key(settings_window.id(), KeyCode::KEY_ESC).await?;

        // Verify window closed
        tokio::time::sleep(Duration::from_millis(200)).await;
        let windows = self.compositor.list_windows().await?;
        assert!(!windows.iter().any(|w| w.id() == settings_window.id()));

        Ok(())
    }

    /// Test: File operations in file manager
    pub async fn test_file_operations(&self) -> Result<()> {
        // Open file manager
        self.interaction.click(1, 100, 500).await?;
        self.interaction.type_text(2, "files").await?;
        self.interaction.press_key(2, KeyCode::KEY_ENTER).await?;

        tokio::time::sleep(Duration::from_millis(500)).await;

        // Find file manager window
        let windows = self.compositor.list_windows().await?;
        let fm_window = windows.iter()
            .find(|w| w.app_id() == Some("org.rustica.Files"))
            .ok_or(anyhow::anyhow!("File manager window not found"))?;

        // Right-click to create new folder
        self.interaction.click(fm_window.id(), 200, 300).await?;
        self.interaction.press_key(fm_window.id(), KeyCode::KEY_CONTEXT_MENU).await?;
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Type "New Folder"
        self.interaction.type_text(fm_window.id(), "Test Folder").await?;
        self.interaction.press_key(fm_window.id(), KeyCode::KEY_ENTER).await?;

        // Verify folder created (check with file manager API)
        // ...

        Ok(())
    }

    /// Test: Workspace switching
    pub async fn test_workspace_switching(&self) -> Result<()> {
        // Create new workspace
        self.interaction.click(1, 500, 20).await?;  // Workspace indicator
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Click "Add Workspace"
        self.interaction.click(1, 600, 100).await?;
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Verify workspace count increased
        let workspace_count = self.compositor.workspace_count().await?;
        assert_eq!(workspace_count, 2);

        // Switch to workspace 1
        self.interaction.click(1, 550, 20).await?;
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Verify active workspace changed
        let active_workspace = self.compositor.active_workspace().await?;
        assert_eq!(active_workspace, 1);

        Ok(())
    }

    /// Test: Mobile gesture to switch workspace
    pub async fn test_gestures_workspace_switch(&self) -> Result<()> {
        // Create second workspace
        self.compositor.create_workspace().await?;

        // Swipe from left edge
        self.interaction.swipe(1, 10.0, 500.0, 1900.0, 500.0).await?;

        // Wait for animation
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Verify workspace switched
        let active_workspace = self.compositor.active_workspace().await?;
        assert_eq!(active_workspace, 1);

        Ok(())
    }
}
```

## Usability Testing

```rust
//! tests/ui/usability.rs

use rustica_test::prelude::*;

pub struct UsabilityValidator;

impl UsabilityValidator {
    /// Validate Fitts's Law compliance
    pub fn validate_fitts_law(widget: &Widget) -> Result<FittsLawReport> {
        let target_size = widget.target_size();
        let distance_to_edge = widget.distance_to_nearest_edge();

        // Calculate effective width (W)
        let effective_width = target_size.min(distance_to_edge * 2.0);

        // Index of difficulty (ID) = log2(1 + distance / width)
        let id = (2.0_f64).log2(1.0 + distance_to_edge / effective_width);

        // Expected time (ms) = a + b * ID
        // Typical values: a = 50ms, b = 100ms
        let expected_time = 50.0 + 100.0 * id;

        Ok(FittsLawReport {
            effective_width,
            distance_to_edge,
            index_of_difficulty: id,
            expected_time_ms: expected_time,
        })
    }

    /// Validate click target size
    pub fn validate_click_target(widget: &Widget) -> Result<TargetValidation> {
        let size = widget.target_size();

        let min_size = 44.0;  // WCAG AAA minimum
        let recommended_size = 48.0;  // Material Design recommendation

        let status = if size >= recommended_size {
            TargetStatus::Excellent
        } else if size >= min_size {
            TargetStatus::Acceptable
        } else {
            TargetStatus::TooSmall
        };

        Ok(TargetValidation {
            size,
            status,
        })
    }

    /// Validate response time
    pub async fn validate_response_time(
        interaction: &Interaction,
        expected_max_ms: u64,
    ) -> Result<ResponseTimeValidation> {
        let start = std::time::Instant::now();

        // Perform interaction
        interaction.execute().await?;

        let elapsed = start.elapsed();

        let acceptable = elapsed.as_millis() <= expected_max_ms as u128;

        Ok(ResponseTimeValidation {
            elapsed_ms: elapsed.as_millis() as u64,
            expected_max_ms,
            acceptable,
        })
    }

    /// Validate visual feedback
    pub fn validate_visual_feedback(
        widget: &Widget,
        interaction: InteractionType,
    ) -> Result<VisualFeedbackValidation> {
        let feedback = widget.get_feedback(interaction);

        let has_visual = feedback.has_visual();
        let has_animation = feedback.has_animation();
        let duration_ms = feedback.duration().as_millis() as u64;

        // Check if feedback duration is appropriate (100-300ms)
        let duration_ok = duration_ms >= 100 && duration_ms <= 300;

        Ok(VisualFeedbackValidation {
            has_visual,
            has_animation,
            duration_ms,
            duration_ok,
        })
    }
}

pub struct FittsLawReport {
    pub effective_width: f64,
    pub distance_to_edge: f64,
    pub index_of_difficulty: f64,
    pub expected_time_ms: f64,
}

pub struct TargetValidation {
    pub size: f64,
    pub status: TargetStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetStatus {
    Excellent,
    Acceptable,
    TooSmall,
}

pub struct ResponseTimeValidation {
    pub elapsed_ms: u64,
    pub expected_max_ms: u64,
    pub acceptable: bool,
}

pub struct VisualFeedbackValidation {
    pub has_visual: bool,
    pub has_animation: bool,
    pub duration_ms: u64,
    pub duration_ok: bool,
}
```

## Test Scripts

```bash
#!/bin/bash
# scripts/test-ui.sh

set -e

echo "Running UI Tests..."

# Start virtual display
Xvfb :99 -screen 0 1920x1080x24 &
export DISPLAY=:99

# Start test compositor
cargo run --bin rustica-comp -- --test-mode &
sleep 2

# Run visual regression tests
echo "=== Visual Regression ==="
cargo nextest run --visual-regression

# Run interaction tests
echo "=== Interaction Tests ==="
cargo nextest run --ui-interaction

# Run layout tests
echo "=== Layout Tests ==="
cargo nextest run --ui-layout

# Run user flow tests
echo "=== User Flow Tests ==="
cargo nextest run --user-flows

# Run usability tests
echo "=== Usability Tests ==="
cargo nextest run --usability

# Cleanup
killall rustica-comp
killall Xvfb

echo "All UI tests passed!"
```

## Configuration

```toml
# .config/ui-test.toml

[visual_regression]
# Acceptable difference threshold (0.0-1.0)
threshold = 0.01

# Update baselines automatically
auto_update_baselines = false

[interaction]
# Delay between actions (ms)
action_delay = 50

# Animation timeout (ms)
animation_timeout = 1000

[layout]
# Breakpoints to test
breakpoints = ["mobile", "tablet", "desktop"]

# Minimum touch target size (px)
min_touch_target = 44

# Minimum font size (px)
min_font_size = 12

# Minimum contrast ratio
min_contrast_ratio = 4.5

[usability]
# Maximum response time (ms)
max_response_time = 300

# Minimum animation duration (ms)
min_animation_duration = 100

# Maximum animation duration (ms)
max_animation_duration = 300
```

## Best Practices

1. **Deterministic**: Avoid flaky tests with random timing
2. **Isolation**: Each test should clean up after itself
3. **Fast**: Keep UI tests under 5 seconds each
4. **Clear Names**: Test names should describe the user action
5. **Visual Baselines**: Store baselines in version control
6. **Parallel Execution**: Design tests to run in parallel
7. **Screenshot on Failure**: Always capture screenshots on failure
8. **CI/CD**: Run tests in CI/CD pipeline
9. **Regular Updates**: Update baselines when design changes
10. **User Perspective**: Test from user's perspective

## Dependencies

```toml
[dev-dependencies]
rustica-test = { path = "../rustica-test" }
image = "0.24"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

## Future Enhancements

1. **AI-Powered Testing**: ML to detect UI issues
2. **Eye Tracking**: Simulate eye movement patterns
3. **Accessibility Scanning**: Automated accessibility checks
4. **Performance Profiling**: Measure render times
5. **User Recording**: Record and replay user sessions
6. **Cross-Device Testing**: Test on multiple device types
7. **Internationalization**: Test across locales
8. **Dark Mode**: Test both light and dark themes
9. **Gesture Library**: Pre-built gesture test library
10. **Real User Monitoring**: Collect real usage data
