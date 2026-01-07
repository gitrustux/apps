# Phase 10.2: Integration Tests

## Overview

**Component**: Integration Test Suite
**Purpose**: Test interactions between GUI components
**Location**: `apps/gui/*/tests/integration/`
**Dependencies:** rustica-test, mock services

## Goals

1. **Component Integration**: Test how components work together
2. **End-to-End Scenarios**: Test complete user workflows
3. **Service Integration**: Test D-Bus, kernel, and hardware integration
4. **Error Recovery**: Test error handling and recovery
5. **Performance Under Load**: Test system behavior under stress

## Test Categories

```
tests/integration/
├── compositor/
│   ├── window_management.rs
│   ├── input_handling.rs
│   └── rendering.rs
├── shell/
│   ├── panel_integration.rs
│   ├── launcher_integration.rs
│   └── workspace_integration.rs
├── applications/
│   ├── file_manager.rs
│   ├── settings.rs
│   └── terminal.rs
├── mobile/
│   ├── gestures.rs
│   ├── keyboard.rs
│   └── sensors.rs
├── package/
│   ├── install.rs
│   ├── update.rs
│   └── remove.rs
└── system/
    ├── power.rs
    ├── themes.rs
    └── accessibility.rs
```

## Compositor Integration Tests

```rust
//! tests/integration/compositor/window_management.rs

use rustica_test::prelude::*;
use rustica_comp::{Compositor, Window, Workspace};
use std::time::Duration;

#[tokio_test]
async fn test_window_lifecycle() {
    // Setup test compositor
    let mut compositor = TestCompositor::new().await?;

    // Create window
    let window = compositor.create_window(800, 600).await?;
    assert!(compositor.window_exists(window.id()));

    // Map window
    compositor.map_window(window.id()).await?;
    assert!(compositor.is_window_mapped(window.id()));

    // Focus window
    compositor.focus_window(window.id()).await?;
    assert_eq!(compositor.focused_window(), Some(window.id()));

    // Unmap window
    compositor.unmap_window(window.id()).await?;
    assert!(!compositor.is_window_mapped(window.id()));

    // Destroy window
    compositor.destroy_window(window.id()).await?;
    assert!(!compositor.window_exists(window.id()));
}

#[tokio_test]
async fn test_workspace_switching() {
    let mut compositor = TestCompositor::new().await?;

    // Create multiple workspaces
    compositor.create_workspace().await?;
    compositor.create_workspace().await?;

    assert_eq!(compositor.workspace_count(), 3);

    // Add windows to different workspaces
    let window1 = compositor.create_window(800, 600).await?;
    compositor.assign_window_to_workspace(window1.id(), 0).await?;

    let window2 = compositor.create_window(800, 600).await?;
    compositor.assign_window_to_workspace(window2.id(), 1).await?;

    // Switch to workspace 1
    compositor.switch_to_workspace(1).await?;
    assert!(compositor.is_window_visible(window2.id()));
    assert!(!compositor.is_window_visible(window1.id()));

    // Switch to workspace 0
    compositor.switch_to_workspace(0).await?;
    assert!(compositor.is_window_visible(window1.id()));
    assert!(!compositor.is_window_visible(window2.id()));
}

#[tokio_test]
async fn test_window_focus_chain() {
    let mut compositor = TestCompositor::new().await?;

    // Create multiple windows
    let window1 = compositor.create_window(800, 600).await?;
    let window2 = compositor.create_window(800, 600).await?;
    let window3 = compositor.create_window(800, 600).await?;

    compositor.map_window(window1.id()).await?;
    compositor.map_window(window2.id()).await?;
    compositor.map_window(window3.id()).await?;

    // Focus windows in order
    compositor.focus_window(window1.id()).await?;
    compositor.focus_window(window2.id()).await?;
    compositor.focus_window(window3.id()).await?;

    // Test focus cycling
    compositor.cycle_focus_forward().await?;
    assert_eq!(compositor.focused_window(), Some(window1.id()));

    compositor.cycle_focus_forward().await?;
    assert_eq!(compositor.focused_window(), Some(window2.id()));

    compositor.cycle_focus_backward().await?;
    assert_eq!(compositor.focused_window(), Some(window1.id()));
}

#[tokio_test]
async fn test_fullscreen_window() {
    let mut compositor = TestCompositor::new().await?;

    let window = compositor.create_window(800, 600).await?;
    compositor.map_window(window.id()).await?;

    // Enter fullscreen
    compositor.set_fullscreen(window.id(), true).await?;
    assert!(compositor.is_window_fullscreen(window.id()));

    // Check window covers entire output
    let output = compositor.primary_output();
    let window_geometry = compositor.window_geometry(window.id());
    assert_eq!(window_geometry, output.geometry());

    // Exit fullscreen
    compositor.set_fullscreen(window.id(), false).await?;
    assert!(!compositor.is_window_fullscreen(window.id()));
}
```

## Shell Integration Tests

```rust
//! tests/integration/shell/panel_integration.rs

use rustica_test::prelude::*;
use rustica_panel::Panel;
use rustica_launcher::Launcher;
use rustica_comp::Compositor;

#[tokio_test]
async fn test_panel_window_launch() {
    let compositor = TestCompositor::new().await?;
    let mut panel = Panel::test_new(&compositor).await?;
    let launcher = Launcher::test_new(&compositor).await?;

    // Get app entry from panel
    let app_entry = panel.find_app_entry("firefox").await?;
    assert!(app_entry.is_some());

    // Simulate click
    let app_entry = app_entry.unwrap();
    panel.simulate_click(app_entry.id()).await?;

    // Wait for window to appear
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Check that window was launched
    let windows = compositor.list_windows().await?;
    assert!(windows.iter().any(|w| w.app_id() == Some("org.mozilla.firefox")));
}

#[tokio_test]
async fn test_panel_workspace_indicators() {
    let compositor = TestCompositor::new().await?;
    let mut panel = Panel::test_new(&compositor).await?;

    // Add workspaces
    compositor.create_workspace().await?;
    compositor.create_workspace().await?;

    // Check panel shows correct workspace count
    let indicators = panel.workspace_indicators().await?;
    assert_eq!(indicators.len(), 3);

    // Switch to workspace 1
    compositor.switch_to_workspace(1).await?;
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Check panel updates indicator
    let active = panel.active_workspace().await?;
    assert_eq!(active, 1);
}

#[tokio_test]
async fn test_panel_system_menu() {
    let compositor = TestCompositor::new().await?;
    let mut panel = Panel::test_new(&compositor).await?;

    // Open system menu
    panel.open_system_menu().await?;

    // Click "Settings"
    panel.click_system_menu_item("Settings").await?;

    // Wait for settings window
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Check settings window opened
    let windows = compositor.list_windows().await?;
    assert!(windows.iter().any(|w| w.app_id() == Some("org.rustica.Settings")));
}
```

## Application Integration Tests

```rust
//! tests/integration/applications/file_manager.rs

use rustica_test::prelude::*;
use rustica_files::FileManager;
use tempfile::TempDir;
use std::fs;

#[tokio_test]
async fn test_file_manager_browse() {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test content")?;

    let mut fm = FileManager::test_new(temp_dir.path()).await?;

    // Navigate to temp directory
    fm.navigate_to(temp_dir.path()).await?;

    // Check file list
    let files = fm.current_files().await?;
    assert!(files.iter().any(|f| f.name() == "test.txt"));

    // Select file
    fm.select_file("test.txt").await?;
    let selected = fm.selected_file().await?;
    assert_eq!(selected.unwrap().name(), "test.txt");
}

#[tokio_test]
async fn test_file_manager_create_folder() {
    let temp_dir = TempDir::new()?;
    let mut fm = FileManager::test_new(temp_dir.path()).await?;

    // Create new folder
    fm.create_folder("New Folder").await?;

    // Check folder was created
    let new_folder = temp_dir.path().join("New Folder");
    assert!(new_folder.exists());
    assert!(new_folder.is_dir());

    // Check file manager shows folder
    let files = fm.current_files().await?;
    assert!(files.iter().any(|f| f.name() == "New Folder"));
}

#[tokio_test]
async fn test_file_manager_delete() {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "test content")?;

    let mut fm = FileManager::test_new(temp_dir.path()).await?;

    // Select and delete file
    fm.select_file("test.txt").await?;
    fm.delete_selected().await?;

    // Check file was deleted
    assert!(!test_file.exists());

    // Check file manager no longer shows file
    let files = fm.current_files().await?;
    assert!(!files.iter().any(|f| f.name() == "test.txt"));
}
```

## Mobile Integration Tests

```rust
//! tests/integration/mobile/gestures.rs

use rustica_test::prelude::*;
use rustica_gestures::{GestureManager, Gesture};
use rustica_comp::Compositor;

#[tokio_test]
async fn test_swipe_to_switch_workspace() {
    let compositor = TestCompositor::new().await?;
    let mut gesture_manager = GestureManager::test_new(&compositor).await?;

    // Create second workspace
    compositor.create_workspace().await?;

    // Simulate swipe gesture
    let swipe_start = (100.0, 500.0);
    let swipe_end = (1800.0, 500.0);  // Swipe right

    gesture_manager.simulate_swipe(swipe_start, swipe_end).await?;

    // Wait for gesture to process
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Check workspace switched
    let active = compositor.active_workspace().await?;
    assert_eq!(active, 1);
}

#[tokio_test]
async fn test_pinch_to_zoom() {
    let compositor = TestCompositor::new().await?;
    let mut gesture_manager = GestureManager::test_new(&compositor).await?;

    let window = compositor.create_window(800, 600).await?;
    compositor.map_window(window.id()).await?;

    // Get initial scale
    let initial_scale = compositor.window_scale_factor(window.id()).await?;

    // Simulate pinch zoom (2x)
    gesture_manager.simulate_pinch(
        (400.0, 300.0),  // Center
        2.0,             // Scale
    ).await?;

    // Wait for gesture to process
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Check scale factor changed
    let new_scale = compositor.window_scale_factor(window.id()).await?;
    assert_approx_eq!(new_scale, initial_scale * 2.0, 0.1);
}

#[tokio_test]
async fn test_double_tap_to_maximize() {
    let compositor = TestCompositor::new().await?;
    let mut gesture_manager = GestureManager::test_new(&compositor).await?;

    let window = compositor.create_window(800, 600).await?;
    compositor.map_window(window.id()).await?;

    // Get initial geometry
    let initial_geom = compositor.window_geometry(window.id()).await?;

    // Simulate double tap
    gesture_manager.simulate_double_tap((400.0, 300.0)).await?;

    // Wait for gesture to process
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Check window was maximized
    let new_geom = compositor.window_geometry(window.id()).await?;
    assert!(new_geom.width > initial_geom.width);
    assert!(new_geom.height > initial_geom.height);
}
```

## Package Integration Tests

```rust
//! tests/integration/package/install.rs

use rustica_test::prelude::*;
use rustica_store::PackageManager;

#[tokio_test]
async fn test_package_install() {
    let mut pm = PackageManager::test_new().await?;

    // Search for package
    let results = pm.search("test-app").await?;
    assert!(!results.is_empty());

    let package = &results[0];

    // Install package
    let handle = pm.install(&package.id, PackageType::Native).await?;

    // Wait for installation
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            match handle.progress().await {
                InstallProgress::Complete => break,
                InstallProgress::Error(e) => panic!("Installation failed: {}", e),
                _ => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }
    }).await.expect("Installation timed out");

    // Check package is installed
    let installed = pm.list_installed().await?;
    assert!(installed.iter().any(|p| p.id == package.id));

    // Cleanup
    pm.remove(&package.id, PackageType::Native).await?;
}

#[tokio_test]
async fn test_flatpak_install() {
    let mut pm = PackageManager::test_new().await?;

    // Install test Flatpak (using org.freedesktop.Platform as it's small)
    let handle = pm.install(
        "org.freedesktop.Platform",
        PackageType::Flatpak,
    ).await?;

    // Wait for installation
    tokio::time::timeout(Duration::from_secs(120), async {
        loop {
            match handle.progress().await {
                InstallProgress::Complete => break,
                InstallProgress::Error(e) => panic!("Installation failed: {}", e),
                _ => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }
    }).await.expect("Installation timed out");

    // Check Flatpak is installed
    let installed = pm.list_installed().await?;
    assert!(installed.iter().any(|p| p.id == "org.freedesktop.Platform"));
}

#[tokio_test]
async fn test_package_update() {
    let mut pm = PackageManager::test_new().await?;

    // Install an older version of a test package
    pm.install("test-app@1.0.0", PackageType::Native).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Check for updates
    let updates = pm.get_updates().await?;
    let update = updates.iter().find(|u| u.id == "test-app");
    assert!(update.is_some());

    // Update package
    pm.update("test-app", PackageType::Native).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Check version updated
    let installed = pm.list_installed().await?;
    let pkg = installed.iter().find(|p| p.id == "test-app").unwrap();
    assert_eq!(pkg.version, "2.0.0");
}
```

## System Integration Tests

```rust
//! tests/integration/system/power.rs

use rustica_test::prelude::*;
use rustica_power::PowerManager;
use std::time::Duration;

#[tokio_test]
async fn test_power_profile_switch() {
    let mut pm = PowerManager::test_new().await?;

    // Set performance profile
    pm.set_profile(PowerProfile::Performance).await?;
    assert_eq!(pm.current_profile(), PowerProfile::Performance);

    // Check CPU governor changed
    let governor = pm.cpu_governor().await?;
    assert_eq!(governor, "performance");

    // Switch to battery saver
    pm.set_profile(PowerProfile::BatterySaver).await?;
    assert_eq!(pm.current_profile(), PowerProfile::BatterySaver);

    // Check CPU governor changed
    let governor = pm.cpu_governor().await?;
    assert_eq!(governor, "powersave");
}

#[tokio_test]
async fn test_wake_lock() {
    let mut pm = PowerManager::test_new().await?;

    // Acquire wake lock
    let lock = pm.acquire_wake_lock(
        "test-app".to_string(),
        WakeLockType::System,
        "Testing wake lock".to_string(),
    ).await?;

    // Check wake lock is active
    assert!(pm.has_system_wake_locks().await);

    // Try to suspend (should fail)
    let suspend_result = pm.suspend().await;
    assert!(suspend_result.is_err());

    // Release wake lock
    pm.release_wake_lock(lock.id).await?;

    // Check wake lock is released
    assert!(!pm.has_system_wake_locks().await);
}

#[tokio_test]
async fn test_battery_thresholds() {
    let mut pm = PowerManager::test_new().await?;

    // Mock battery level at 15%
    pm.set_mock_battery_level(15).await;

    // Check low battery threshold triggered
    pm.check_battery_thresholds().await?;

    // Check profile switched to battery saver
    assert_eq!(pm.current_profile(), PowerProfile::BatterySaver);

    // Check low battery notification sent
    let notifications = pm.notifications().await;
    assert!(notifications.iter().any(|n| n.summary.contains("Low Battery")));
}

#[tokio_test]
async fn test_app_throttling() {
    let mut pm = PowerManager::test_new().await?;

    // Set battery saver profile
    pm.set_profile(PowerProfile::BatterySaver).await?;

    // Set app hint for background app
    pm.set_app_hint("test-app".to_string(), PowerHint::Background).await?;

    // Check app is throttled
    let throttle = pm.app_throttle_factor("test-app").await?;
    assert!(throttle < 1.0);

    // Change to performance-critical
    pm.set_app_hint("test-app".to_string(), PowerHint::PerformanceCritical).await?;

    // Check throttling removed
    let throttle = pm.app_throttle_factor("test-app").await?;
    assert_approx_eq!(throttle, 1.0, 0.01);
}
```

## Accessibility Integration Tests

```rust
//! tests/integration/system/accessibility.rs

use rustica_test::prelude::*;
use atspi::Accessible;

#[tokio_test]
async fn test_screen_reader_announcement() {
    let compositor = TestCompositor::new().await?;
    let window = compositor.create_window(800, 600).await?;
    compositor.map_window(window.id()).await?;

    // Get accessible interface
    let accessible = window.accessible_interface().await?;

    // Check name is announced
    let name = accessible.name().await?;
    assert_eq!(name, "Test Window");

    // Check role is correct
    let role = accessible.role().await?;
    assert_eq!(role, AccessibleRole::Window);

    // Check children are accessible
    let children = accessible.children().await?;
    assert!(!children.is_empty());
}

#[tokio_test]
async fn test_keyboard_navigation() {
    let compositor = TestCompositor::new().await?;
    let mut window = compositor.create_test_window_with_widgets().await?;

    // Get accessible interface
    let accessible = window.accessible_interface().await?;

    // Simulate Tab key
    window.send_key(KeyCode::Tab).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Check focus moved
    let focused = accessible.focused_child().await?;
    assert!(focused.is_some());

    // Navigate through widgets
    for _ in 0..5 {
        window.send_key(KeyCode::Tab).await;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Check focus cycled correctly
    let focused = accessible.focused_child().await?;
    assert!(focused.is_some());
}

#[tokio_test]
async fn test_high_contrast_theme() {
    let theme_engine = ThemeEngine::test_new().await?;

    // Load high contrast theme
    theme_engine.load_theme("high-contrast").await?;

    let theme = theme_engine.current_theme();

    // Check contrast ratios meet WCAG AAA
    assert_contrast_ratio!(theme.background, theme.foreground, 7.0);
    assert_contrast_ratio!(theme.primary, theme.on_primary, 7.0);
    assert_contrast_ratio!(theme.secondary, theme.on_secondary, 7.0);
}
```

## Test Utilities

```rust
//! tests/integration/mod.rs - Common integration test utilities

use std::time::Duration;
use tokio::time::sleep;

/// Wait for a condition to be true
pub async fn wait_for<F, Fut>(condition: F, timeout: Duration) -> anyhow::Result<()>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();

    while start.elapsed() < timeout {
        if condition().await {
            return Ok(());
        }
        sleep(Duration::from_millis(50)).await;
    }

    Err(anyhow::anyhow!("Condition not met within timeout"))
}

/// Retry an operation with exponential backoff
pub async fn retry<F, Fut, T>(operation: F, max_retries: u32) -> anyhow::Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    let mut attempt = 0;
    let mut delay = Duration::from_millis(100);

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries => {
                attempt += 1;
                sleep(delay).await;
                delay *= 2;
            }
            Err(e) => return Err(e),
        }
    }
}

/// Take a screenshot for debugging
pub async fn screenshot(compositor: &Compositor, name: &str) -> anyhow::Result<()> {
    let output = compositor.capture_output().await?;
    output.save(format!("/tmp/screenshots/{}.png", name))?;
    Ok(())
}
```

## Configuration

```toml
# .config/integration-test.toml

[general]
# Timeout for each test
test_timeout = 60

# Delay between actions
action_delay = 50

# Screenshot on failure
screenshot_on_failure = true

[compositor]
# Use nested compositor for tests
use_nested = true

# Backend to use
backend = "linux-drm"  # or "headless"

[apps]
# Test app repository
test_repo = "/var/lib/rustica-test/packages"

# Mock services
use_mock_services = true

[mock]
# Mock sensor data
sensor_data_path = "tests/data/sensors"

# Mock network
network_latency_ms = 10
```

## Test Scripts

```bash
#!/bin/bash
# scripts/test-integration.sh

set -e

echo "Running Integration Tests..."

# Build test binaries
cargo nextest run --no-run

# Run compositor integration tests
echo "=== Compositor Integration ==="
cargo nextest run --integration compositor

# Run shell integration tests
echo "=== Shell Integration ==="
cargo nextest run --integration shell

# Run application integration tests
echo "=== Application Integration ==="
cargo nextest run --integration applications

# Run mobile integration tests
echo "=== Mobile Integration ==="
cargo nextest run --integration mobile

# Run package integration tests
echo "=== Package Integration ==="
cargo nextest run --integration package

# Run system integration tests
echo "=== System Integration ==="
cargo nextest run --integration system

echo "All integration tests passed!"
```

## CI/CD Integration

```yaml
# .github/workflows/integration-test.yml
name: Integration Tests

on: [push, pull_request]

jobs:
  integration:
    runs-on: ubuntu-latest
    container:
      image: rustux/rustica-test:latest

    services:
      dbus:
        image: rustux/dbus-test

    steps:
      - uses: actions/checkout@v3
      - name: Run integration tests
        run: ./scripts/test-integration.sh
      - name: Upload screenshots
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: screenshots
          path: /tmp/screenshots/
```

## Best Practices

1. **Isolation**: Each test should clean up after itself
2. **Deterministic**: Avoid timing-dependent tests
3. **Fast**: Keep tests under 60 seconds
4. **Clear Failures**: Provide helpful error messages
5. **Retry Logic**: Handle transient failures
6. **Mock Services**: Use mocks when possible
7. **Resource Limits**: Set memory/CPU limits
8. **Debug Output**: Save logs on failure
9. **Screenshots**: Capture screenshots on failure
10. **Parallel Execution**: Design tests to run in parallel

## Dependencies

```toml
[dev-dependencies]
rustica-test = { path = "../rustica-test" }
tokio = { version = "1", features = ["full"] }
tempfile = "3"
anyhow = "1"
```

## Future Enhancements

1. **Visual Regression**: Automated screenshot comparison
2. **Network Simulation**: Simulate network conditions
3. **Hardware Simulation**: Mock hardware devices
4. **Stress Testing**: Load testing for components
5. **Chaos Testing**: Random failure injection
6. **Multi-User**: Test concurrent user sessions
7. **Upgrade Testing**: Test system upgrade paths
8. **Backup/Restore**: Test backup and restore
9. **Crash Recovery**: Test recovery from crashes
10. **Performance Regression**: Detect performance regressions
