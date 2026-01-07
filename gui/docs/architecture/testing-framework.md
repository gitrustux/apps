# Phase 10.1: Testing Framework (rustica-test)

## Overview

**Component**: rustica-test
**Purpose**: Unified testing framework for all GUI components
**Language**: Rust
**Dependencies:** cargo, nextest, criterion

## Goals

1. **Fast Feedback**: Run tests quickly with parallel execution
2. **Comprehensive Coverage**: Unit, integration, UI, performance, and accessibility tests
3. **CI/CD Ready**: Easy integration with continuous integration
4. **Developer Friendly**: Simple test writing and debugging
5. **Reliable**: Flaky test detection and retry logic

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Test Runner                              │
│                  (cargo nextest)                             │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  rustica-test (Framework)                    │
├─────────────────────────────────────────────────────────────┤
│  TestHelpers       │  MockServices     │  TestFixtures      │
│  - Assertions      │  - Mock compositor│  - Test windows    │
│  - Matchers        │  - Mock D-Bus     │  - Test apps       │
│  - Test macros     │  - Mock sensors   │  - Test data       │
└────────────────────────┬────────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         ▼               ▼               ▼
    ┌─────────┐    ┌─────────┐    ┌─────────┐
    │ Unit    │    │Integration│   │ UI      │
    │ Tests   │    │ Tests   │    │ Tests   │
    └─────────┘    └─────────┘    └─────────┘
```

## Test Organization

```
apps/gui/
├── rustica-comp/
│   ├── tests/
│   │   ├── unit/          # Unit tests
│   │   ├── integration/   # Integration tests
│   │   └── fixtures/      # Test fixtures
│   └── benches/           # Benchmarks
├── rustica-test/          # Test framework
│   ├── src/
│   │   ├── lib.rs
│   │   ├── prelude.rs     # Test prelude
│   │   ├── assertions.rs  # Custom assertions
│   │   ├── matchers.rs    # Test matchers
│   │   └── fixtures.rs    # Test fixtures
│   └── mocks/             # Mock services
└── scripts/
    ├── test.sh            # Run all tests
    ├── test-unit.sh       # Run unit tests
    └── test-integration.sh # Run integration tests
```

## Core Test Framework

```rust
//! rustica-test - Unified testing framework for Rustica GUI

pub mod prelude;

pub mod assertions;
pub mod matchers;
pub mod fixtures;
pub mod mocks;

pub use rustica_test_macros::*;

#[cfg(test)]
mod tests {
    use super::prelude::*;
}
```

## Test Prelude

```rust
//! rustica-test::prelude - Common imports for testing

pub use std::time::Duration;

pub use anyhow::Result;
pub use tokio;

// Re-export common test items
pub use crate::assertions::*;
pub use crate::matchers::*;
pub use crate::fixtures::*;

// Test macros
pub use rustica_test_macros::{
    test, tokio_test, bench, fixture, async_fixture,
};

// Common test utilities
pub use crate::TestContext;
pub use crate::TestRunner;

/// Default test timeout
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default async test timeout
pub const DEFAULT_ASYNC_TIMEOUT: Duration = Duration::from_secs(60);
```

## Custom Assertions

```rust
//! Custom assertions for GUI testing

use std::path::Path;

/// Assert that a value is approximately equal
pub fn assert_approx_eq<T>(left: T, right: T, epsilon: T) where
    T: Float + PartialEq + std::fmt::Debug
{
    let diff = (left - right).abs();

    assert!(
        diff <= epsilon,
        "Assertion failed: {:?} is not approximately equal to {:?} (epsilon: {:?})",
        left, right, epsilon
    );
}

/// Assert that a file exists
pub fn assert_file_exists<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();

    assert!(
        path.exists(),
        "Assertion failed: File {:?} does not exist",
        path
    );

    assert!(
        path.is_file(),
        "Assertion failed: {:?} is not a file",
        path
    );
}

/// Assert that a directory exists
pub fn assert_dir_exists<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();

    assert!(
        path.exists(),
        "Assertion failed: Directory {:?} does not exist",
        path
    );

    assert!(
        path.is_dir(),
        "Assertion failed: {:?} is not a directory",
        path
    );
}

/// Assert that a Wayland surface has specific properties
pub fn assert_surface_properties(
    surface: &WaylandSurface,
    expected: SurfaceProperties,
) {
    let props = surface.properties();

    assert_eq!(
        props.width, expected.width,
        "Surface width mismatch"
    );

    assert_eq!(
        props.height, expected.height,
        "Surface height mismatch"
    );

    assert_eq!(
        props.scale_factor, expected.scale_factor,
        "Scale factor mismatch"
    );

    assert_eq!(
        props.title, expected.title,
        "Title mismatch"
    );
}

/// Assert that a gesture was recognized
pub fn assert_gesture_recognized(
    result: &GestureResult,
    expected: GestureType,
) {
    assert_eq!(
        result.gesture_type, expected,
        "Expected {:?} but got {:?}",
        expected, result.gesture_type
    );

    assert!(
        result.confidence > 0.8,
        "Gesture confidence too low: {}",
        result.confidence
    );
}

/// Assert that keyboard input was handled
pub fn assert_keyboard_handled(
    result: &KeyboardResult,
    expected: KeyEvent,
) {
    match result {
        KeyboardResult::Char(c) => {
            if let KeyEvent::Char(expected_char) = expected {
                assert_eq!(c, expected_char, "Character mismatch");
            } else {
                panic!("Expected char {:?} but got {:?}", expected, result);
            }
        }
        KeyboardResult::Command(cmd) => {
            if let KeyEvent::Command(expected_cmd) = expected {
                assert_eq!(cmd, expected_cmd, "Command mismatch");
            } else {
                panic!("Expected command {:?} but got {:?}", expected, result);
            }
        }
        _ => panic!("Expected {:?} but got {:?}", expected, result),
    }
}

/// Assert that sensor reading is within valid range
pub fn assert_sensor_valid<T>(reading: &SensorReading<T>, min: T, max: T)
where
    T: PartialOrd + std::fmt::Debug + Copy,
{
    assert!(
        reading.value >= min && reading.value <= max,
        "Sensor reading {:?} is outside valid range [{:?}, {:?}]",
        reading.value, min, max
    );

    assert!(
        reading.accuracy != Accuracy::Unreliable,
        "Sensor reading is marked as unreliable"
    );
}

/// Assert that package was installed successfully
pub fn assert_package_installed(package_id: &str) {
    let install_path = PathBuf::from("/opt/rustica/packages").join(package_id);

    assert_dir_exists(&install_path);

    let metadata_path = install_path.join("metadata.json");
    assert_file_exists(&metadata_path);
}

/// Assert that theme colors meet contrast requirements
pub fn assert_contrast_ratio(
    foreground: Color,
    background: Color,
    min_ratio: f64,
) {
    let ratio = calculate_contrast_ratio(foreground, background);

    assert!(
        ratio >= min_ratio,
        "Contrast ratio {:.2} is below minimum {:.2}",
        ratio, min_ratio
    );
}

fn calculate_contrast_ratio(foreground: Color, background: Color) -> f64 {
    let fg_luminance = relative_luminance(foreground);
    let bg_luminance = relative_luminance(background);

    let lighter = fg_luminance.max(bg_luminance);
    let darker = fg_luminance.min(bg_luminance);

    (lighter + 0.05) / (darker + 0.05)
}

fn relative_luminance(color: Color) -> f64 {
    // Convert to linear RGB and calculate luminance
    let r = srgb_to_linear(color.r);
    let g = srgb_to_linear(color.g);
    let b = srgb_to_linear(color.b);

    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn srgb_to_linear(c: f64) -> f64 {
    if c <= 0.03928 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
```

## Test Matchers

```rust
//! Test matchers for flexible assertions

use std::ops::Range;

/// Matcher for approximate equality
pub struct Approximately<T> {
    value: T,
    epsilon: T,
}

impl<T> Approximately<T> where
    T: Float + PartialEq + std::fmt::Debug
{
    pub fn new(value: T, epsilon: T) -> Self {
        Self { value, epsilon }
    }
}

impl<T> PartialEq<T> for Approximately<T> where
    T: Float + PartialEq + std::fmt::Debug + Copy
{
    fn eq(&self, other: &T) -> bool {
        (*other - self.value).abs() <= self.epsilon
    }
}

/// Matcher for ranges
pub struct InRange<T> {
    range: Range<T>,
}

impl<T> InRange<T> where
    T: PartialOrd + std::fmt::Debug + Copy
{
    pub fn new(range: Range<T>) -> Self {
        Self { range }
    }
}

impl<T> PartialEq<T> for InRange<T> where
    T: PartialOrd + std::fmt::Debug + Copy
{
    fn eq(&self, other: &T) -> bool {
        self.range.contains(other)
    }
}

/// Matcher for file existence
pub struct ExistingFile;

impl ExistingFile {
    pub fn new() -> Self {
        Self
    }
}

impl PartialEq<PathBuf> for ExistingFile {
    fn eq(&self, other: &PathBuf) -> bool {
        other.exists() && other.is_file()
    }
}

impl PartialEq<&str> for ExistingFile {
    fn eq(&self, other: &&str) -> bool {
        let path = PathBuf::from(other);
        path.exists() && path.is_file()
    }
}

/// Matcher for directory existence
pub struct ExistingDirectory;

impl ExistingDirectory {
    pub fn new() -> Self {
        Self
    }
}

impl PartialEq<PathBuf> for ExistingDirectory {
    fn eq(&self, other: &PathBuf) -> bool {
        other.exists() && other.is_dir()
    }
}

/// Matcher for valid gesture
pub struct ValidGesture;

impl ValidGesture {
    pub fn new() -> Self {
        Self
    }
}

impl PartialEq<GestureResult> for ValidGesture {
    fn eq(&self, other: &GestureResult) -> bool {
        other.confidence > 0.8 && other.gesture_type != GestureType::Invalid
    }
}

/// Trait for floating point operations
trait Float: Copy {
    fn abs(self) -> Self;
}

impl Float for f32 {
    fn abs(self) -> Self { self.abs() }
}

impl Float for f64 {
    fn abs(self) -> Self { self.abs() }
}
```

## Test Fixtures

```rust
//! Test fixtures for common test scenarios

use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

/// Temporary directory fixture
pub struct TempDirFixture {
    temp: TempDir,
}

impl TempDirFixture {
    pub fn new() -> Result<Self> {
        Ok(Self {
            temp: TempDir::new()?,
        })
    }

    pub fn path(&self) -> &Path {
        self.temp.path()
    }

    pub fn create_file(&self, name: &str, content: &str) -> PathBuf {
        let path = self.temp.path().join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    pub fn create_dir(&self, name: &str) -> PathBuf {
        let path = self.temp.path().join(name);
        std::fs::create_dir(&path).unwrap();
        path
    }
}

/// Test compositor fixture
pub struct TestCompositor {
    compositor: Compositor,
    temp_dir: TempDirFixture,
}

impl TestCompositor {
    pub fn new() -> Result<Self> {
        let temp_dir = TempDirFixture::new()?;

        let config = CompositorConfig {
            data_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let compositor = Compositor::test_new(config)?;

        Ok(Self {
            compositor,
            temp_dir,
        })
    }

    pub fn compositor(&self) -> &Compositor {
        &self.compositor
    }

    pub fn create_test_window(&self, width: u32, height: u32) -> TestWindow {
        self.compositor.create_test_window(width, height)
    }

    pub fn advance_time(&self, duration: Duration) {
        self.compositor.test_advance_time(duration);
    }
}

/// Test window fixture
pub struct TestWindow {
    surface: WaylandSurface,
}

impl TestWindow {
    pub fn new(surface: WaylandSurface) -> Self {
        Self { surface }
    }

    pub fn surface(&self) -> &WaylandSurface {
        &self.surface
    }

    pub fn send_touch(&self, x: f64, y: f64) {
        self.surface.test_send_touch(x, y);
    }

    pub fn send_key(&self, key: KeyCode) {
        self.surface.test_send_key(key);
    }
}

/// Mock sensor fixture
pub struct MockSensor {
    readings: Vec<SensorReading<f64>>,
    current_index: usize,
}

impl MockSensor {
    pub fn new(readings: Vec<SensorReading<f64>>) -> Self {
        Self {
            readings,
            current_index: 0,
        }
    }

    pub fn read(&mut self) -> Option<SensorReading<f64>> {
        if self.current_index < self.readings.len() {
            let reading = self.readings[self.current_index].clone();
            self.current_index += 1;
            Some(reading)
        } else {
            None
        }
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
    }
}

/// Test data fixture
pub struct TestDataFixture {
    data_dir: PathBuf,
}

impl TestDataFixture {
    pub fn new() -> Result<Self> {
        let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data");

        Ok(Self { data_dir })
    }

    pub fn load_image(&self, name: &str) -> Result<Image> {
        let path = self.data_dir.join("images").join(name);
        Image::load(&path)
    }

    pub fn load_theme(&self, name: &str) -> Result<Theme> {
        let path = self.data_dir.join("themes").join(format!("{}.json", name));
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn load_fixture(&self, name: &str) -> Result<String> {
        let path = self.data_dir.join(name);
        Ok(std::fs::read_to_string(&path)?)
    }
}
```

## Test Macros

```rust
//! rustica-test-macros - Testing procedural macros

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Attribute macro for standard tests
#[proc_macro_attribute]
pub fn test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let output = quote! {
        #[::core::prelude::v1::test]
        fn #input_name() {
            // Setup logging
            let _ = rustica_test::init_test_logger();

            // Run test with timeout
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async {
                    tokio::time::timeout(
                        rustica_test::DEFAULT_TIMEOUT,
                        async {
                            #test_body
                        }
                    ).await
                    .expect("Test timed out")
                })
        }
    };

    TokenStream::from(output)
}

/// Attribute macro for async tests
#[proc_macro_attribute]
pub fn tokio_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let output = quote! {
        #[::tokio::test(flavor = "multi_thread", worker_threads = 1)]
        async fn #test_name() {
            // Setup logging
            let _ = rustica_test::init_test_logger();

            // Run test with timeout
            tokio::time::timeout(
                rustica_test::DEFAULT_ASYNC_TIMEOUT,
                async #test_block
            ).await
            .expect("Test timed out")
        }
    };

    TokenStream::from(output)
}

/// Attribute macro for benchmarks
#[proc_macro_attribute]
pub fn bench(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let output = quote! {
        #[::criterion::criterion]
        fn #bench_name(c: &mut ::criterion::Criterion) {
            c.bench_function(stringify!(#bench_name), |b| {
                b.iter(|| {
                    #bench_body
                })
            });
        }
    };

    TokenStream::from(output)
}

/// Attribute macro for fixtures
#[proc_macro_attribute]
pub fn fixture(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let output = quote! {
        #[::core::prelude::v1::allow(dead_code)]
        fn #fixture_name() -> ::anyhow::Result<#return_type> {
            #fixture_body
        }
    };

    TokenStream::from(output)
}
```

## Test Runner

```rust
//! Test runner configuration

use std::path::PathBuf;
use std::time::Duration;

pub struct TestRunner {
    config: TestConfig,
}

pub struct TestConfig {
    /// Number of test threads
    pub test_threads: Option<usize>,

    /// Test timeout
    pub timeout: Duration,

    /// Retry failed tests
    pub retry_failed: bool,

    /// Number of retries
    pub retries: usize,

    /// Show test output
    pub show_output: bool,

    /// Fail fast on first failure
    pub fail_fast: bool,

    /// Run tests in random order
    pub randomize: bool,

    /// Filter tests to run
    pub filter: Option<String>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            test_threads: Some(num_cpus::get()),
            timeout: Duration::from_secs(30),
            retry_failed: true,
            retries: 3,
            show_output: false,
            fail_fast: false,
            randomize: false,
            filter: None,
        }
    }
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            config: TestConfig::default(),
        }
    }

    pub fn with_config(mut self, config: TestConfig) -> Self {
        self.config = config;
        self
    }

    pub fn run_tests(&self, workspace_dir: PathBuf) -> Result<TestResults> {
        let mut nextest_cmd = std::process::Command::new("cargo");

        nextest_cmd
            .arg("nextest")
            .arg("run")
            .current_dir(&workspace_dir);

        // Configure test threads
        if let Some(threads) = self.config.test_threads {
            nextest_cmd.arg("--test-threads").arg(threads.to_string());
        }

        // Configure retries
        if self.config.retry_failed {
            nextest_cmd.arg("--retries").arg(self.config.retries.to_string());
        }

        // Fail fast
        if self.config.fail_fast {
            nextest_cmd.arg("--fail-fast");
        }

        // Randomize
        if self.config.randomize {
            nextest_cmd.arg("--randomize");
        }

        // Filter
        if let Some(filter) = &self.config.filter {
            nextest_cmd.arg("--filter").arg(filter);
        }

        // Show output
        if self.config.show_output {
            nextest_cmd.arg("--success-output");
        }

        // Run tests
        let output = nextest_cmd.output()?;

        if output.status.success() {
            Ok(TestResults::from_stdout(&String::from_utf8_lossy(&output.stdout))?)
        } else {
            Err(anyhow::anyhow!("Tests failed"))
        }
    }
}

pub struct TestResults {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration: Duration,
}
```

## Configuration

```toml
# .config/nextest.toml

[profile.default]
# Show test output
success-output = "immediate"
failure-output = "immediate-final"

# Test timeout (seconds)
test-timeout = 30
test-threads = { count = 0, percent = false }

# Retry failed tests
retries = 3

# Slowest tests timeout
slow-timeout = "60s"

[profile.ci]
# CI-specific settings
fail-fast = true
retries = 0

[profile.dev]
# Development settings
show-output = true
slow-timeout = "120s"
```

## Test Scripts

```bash
#!/bin/bash
# scripts/test.sh - Run all tests

set -e

echo "Running Rustica GUI Test Suite..."
echo ""

# Run unit tests
echo "=== Unit Tests ==="
./scripts/test-unit.sh

# Run integration tests
echo ""
echo "=== Integration Tests ==="
./scripts/test-integration.sh

# Run UI tests
echo ""
echo "=== UI Tests ==="
./scripts/test-ui.sh

# Run performance benchmarks
echo ""
echo "=== Performance Benchmarks ==="
./scripts/bench.sh

# Check code coverage
echo ""
echo "=== Code Coverage ==="
./scripts/coverage.sh

echo ""
echo "All tests passed!"
```

## Dependencies

```toml
[dev-dependencies]
# Test framework
rustica-test = { path = "../rustica-test" }
rustica-test-macros = { path = "../rustica-test-macros" }

# Async runtime
tokio = { version = "1", features = ["test-util", "macros"] }

# Test utilities
tempfile = "3"
pretty_assertions = "1"
proptest = "1"

# Mocking
mockall = "0.11"

# Benchmarking
criterion = "0.5"

# Coverage
tarpaulin = "0.27"

# Test runner
cargo-nextest = "0.9"

# Utilities
anyhow = "1"
```

## Usage Examples

```rust
// Example unit test
#[rustica_test::test]
fn test_window_creation() {
    let compositor = TestCompositor::new().unwrap();
    let window = compositor.create_test_window(800, 600);

    assert_eq!(window.surface().width(), 800);
    assert_eq!(window.surface().height(), 600);
}

// Example async test
#[rustica_test::tokio_test]
async fn test_package_install() {
    let mut pm = MockPackageManager::new().await;

    let result = pm.install("test-package").await;

    assert!(result.is_ok());
    assert_package_installed("test-package");
}

// Example benchmark
#[rustica_test::bench]
fn benchmark_rendering(c: &mut criterion::Criterion) {
    c.bench_function("render_1000_rects", |b| {
        b.iter(|| {
            let mut renderer = Renderer::new();
            renderer.render_test_rects(1000);
        })
    });
}

// Example fixture
#[rustica_test::fixture]
fn test_compositor() -> TestCompositor {
    TestCompositor::new().unwrap()
}

// Using fixture in test
#[test]
fn test_with_fixture(test_compositor: TestCompositor) {
    let window = test_compositor.create_test_window(640, 480);
    // ...
}
```

## Testing Best Practices

1. **Fast Tests**: Unit tests should run in milliseconds
2. **Isolation**: Each test should be independent
3. **Deterministic**: Tests should produce the same results every time
4. **Clear Names**: Test names should describe what they test
5. **Single Assertion**: Each test should verify one thing
6. **Setup/Teardown**: Use fixtures for common setup
7. **Mock External Dependencies**: Don't depend on external services
8. **Test Edge Cases**: Test boundary conditions and error cases
9. **Readable**: Tests should be easy to understand
10. **Maintainable**: Update tests when code changes

## CI/CD Integration

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - uses: taiki-e/install-action@nextest
      - name: Run tests
        run: ./scripts/test.sh
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

## Future Enhancements

1. **Property-Based Testing**: More proptest integration
2. **Fuzz Testing**: Fuzz testing for parsers
3. **Snapshot Testing**: UI snapshot testing
4. **Golden Master Testing**: Comparison test framework
5. **Visual Regression**: Screenshot comparison
6. **Mutation Testing**: Code mutation analysis
7. **Test Metrics**: Track test metrics over time
8. **Flaky Test Detection**: Automatic flaky test detection
9. **Parallel Integration Tests**: Better parallelization
10. **Distributed Testing**: Run tests across multiple machines
