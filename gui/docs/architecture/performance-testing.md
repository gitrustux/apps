# Phase 10.4: Performance Testing

## Overview

**Component**: Performance Test Suite
**Purpose**: Benchmark and profile GUI performance
**Location:** `apps/gui/*/benches/`
**Dependencies:** criterion, flamegraph, perf

## Goals

1. **Baseline Performance**: Establish performance baselines
2. **Regression Detection**: Catch performance regressions
3. **Profiling**: Identify bottlenecks
4. **Stress Testing**: Test under load
5. **Power Measurement**: Measure power consumption

## Benchmark Categories

```
benches/
├── rendering/
│   ├── frame_time.rs
│   ├── draw_calls.rs
│   └── texture_upload.rs
├── input/
│   ├── event_latency.rs
│   ├── gesture_recognition.rs
│   └── keyboard_input.rs
├── compositor/
│   ├── window_management.rs
│   ├── surface_commit.rs
│   └── workspace_switch.rs
├── shell/
│   ├── panel_render.rs
│   ├── app_launch.rs
│   └── workspace_switch.rs
└── system/
    ├── memory_usage.rs
    ├── cpu_usage.rs
    └── battery_drain.rs
```

## Rendering Benchmarks

```rust
//! benches/rendering/frame_time.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rustica_comp::{Compositor, Renderer};
use std::time::Duration;

fn bench_frame_time(c: &mut Criterion) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let compositor = rt.block_on(async {
        TestCompositor::new().await.unwrap()
    });

    let mut group = c.benchmark_group("frame_time");

    for num_windows in &[1, 5, 10, 20, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_windows),
            num_windows,
            |b, &num_windows| {
                rt.block_on(async {
                    // Create windows
                    let mut windows = Vec::new();
                    for _ in 0..*num_windows {
                        let window = compositor.create_window(800, 600).await.unwrap();
                        compositor.map_window(window.id()).await.unwrap();
                        windows.push(window);
                    }

                    b.to_async(tokio::time::sleep(Duration::from_millis(16)))
                        .iter(|| {
                            compositor.render_frame().await.unwrap();
                        });
                });
            },
        );
    }

    group.finish();
}

fn bench_draw_calls(c: &mut Criterion) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let compositor = rt.block_on(async {
        TestCompositor::new().await.unwrap()
    });

    let mut group = c.benchmark_group("draw_calls");

    for num_rects in &[10, 100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_rects),
            num_rects,
            |b, &num_rects| {
                rt.block_on(async {
                    let window = compositor.create_window(1920, 1080).await.unwrap();

                    b.to_async(tokio::time::sleep(Duration::from_millis(16)))
                        .iter(|| {
                            compositor.render_test_rects(window.id(), num_rects).await.unwrap();
                        });
                });
            },
        );
    }

    group.finish();
}

fn bench_texture_upload(c: &mut Criterion) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let compositor = rt.block_on(async {
        TestCompositor::new().await.unwrap()
    });

    let mut group = c.benchmark_group("texture_upload");

    for size in &[256, 512, 1024, 2048, 4096] {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                rt.block_on(async {
                    b.to_async(tokio::time::sleep(Duration::from_millis(16)))
                        .iter(|| {
                            let texture = compositor.create_test_texture(size, size).unwrap();
                            compositor.upload_texture(&texture).await.unwrap();
                        });
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_frame_time, bench_draw_calls, bench_texture_upload);
criterion_main!(benches);
```

## Input Latency Benchmarks

```rust
//! benches/input/event_latency.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rustica_comp::{Compositor, InputEvent};
use std::time::{Duration, Instant};

fn bench_mouse_event_latency(c: &mut Criterion) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let compositor = rt.block_on(async {
        TestCompositor::new().await.unwrap()
    });

    c.bench_function("mouse_event_latency", |b| {
        rt.block_on(async {
            let window = compositor.create_window(800, 600).await.unwrap();
            compositor.map_window(window.id()).await.unwrap();
            compositor.focus_window(window.id()).await.unwrap();

            b.to_async(tokio::time::sleep(Duration::from_millis(1)))
                .iter(|| {
                    let start = Instant::now();

                    compositor
                        .send_mouse_event(window.id(), 100.0, 100.0)
                        .await
                        .unwrap();

                    // Wait for event to be processed
                    tokio::time::sleep(Duration::from_micros(100)).await;

                    let elapsed = start.elapsed();
                    assert!(elapsed < Duration::from_millis(16), "Event latency too high: {:?}", elapsed);

                    elapsed
                });
        });
    });
}

fn bench_keyboard_event_latency(c: &mut Criterion) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let compositor = rt.block_on(async {
        TestCompositor::new().await.unwrap()
    });

    c.bench_function("keyboard_event_latency", |b| {
        rt.block_on(async {
            let window = compositor.create_window(800, 600).await.unwrap();
            compositor.map_window(window.id()).await.unwrap();
            compositor.focus_window(window.id()).await.unwrap();

            b.to_async(tokio::time::sleep(Duration::from_millis(1)))
                .iter(|| {
                    let start = Instant::now();

                    compositor
                        .send_key_event(window.id(), KeyCode::KEY_A)
                        .await
                        .unwrap();

                    tokio::time::sleep(Duration::from_micros(100)).await;

                    start.elapsed()
                });
        });
    });
}

fn bench_gesture_recognition(c: &mut Criterion) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let compositor = rt.block_on(async {
        TestCompositor::new().await.unwrap()
    });

    let mut group = c.benchmark_group("gesture_recognition");

    // Bench various gesture types
    group.bench_function("tap", |b| {
        rt.block_on(async {
            b.to_async(tokio::time::sleep(Duration::from_millis(1)))
                .iter(|| {
                    compositor
                        .simulate_tap(100.0, 100.0)
                        .await
                        .unwrap()
                });
        });
    });

    group.bench_function("swipe", |b| {
        rt.block_on(async {
            b.to_async(tokio::time::sleep(Duration::from_millis(1)))
                .iter(|| {
                    compositor
                        .simulate_swipe((100.0, 500.0), (1800.0, 500.0))
                        .await
                        .unwrap()
                });
        });
    });

    group.bench_function("pinch", |b| {
        rt.block_on(async {
            b.to_async(tokio::time::sleep(Duration::from_millis(1)))
                .iter(|| {
                    compositor
                        .simulate_pinch((400.0, 300.0), 2.0)
                        .await
                        .unwrap()
                });
        });
    });

    group.finish();
}

criterion_group!(benches, bench_mouse_event_latency, bench_keyboard_event_latency, bench_gesture_recognition);
criterion_main!(benches);
```

## Memory Usage Benchmarks

```rust
//! benches/system/memory_usage.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use rustica_comp::Compositor;

fn bench_memory_per_window(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_per_window");

    for num_windows in &[1, 5, 10, 20, 50, 100] {
        group.throughput(Throughput::Elements(*num_windows as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(num_windows),
            num_windows,
            |b, &num_windows| {
                let mut rt = tokio::runtime::Runtime::new().unwrap();

                b.to_async(tokio::time::sleep(Duration::from_millis(100)))
                    .iter(|| {
                        rt.block_on(async {
                            let compositor = TestCompositor::new().await.unwrap();

                            // Create windows
                            for _ in 0..num_windows {
                                let window = compositor.create_window(800, 600).await.unwrap();
                                compositor.map_window(window.id()).await.unwrap();
                            }

                            // Measure memory
                            let memory_kb = get_process_memory();

                            memory_kb
                        });
                    });
            },
        );
    }

    group.finish();
}

fn bench_memory_leak_detection(c: &mut Criterion) {
    c.bench_function("memory_leak_window_create_destroy", |b| {
        let mut rt = tokio::runtime::Runtime::new().unwrap();

        b.to_async(tokio::time::sleep(Duration::from_millis(100)))
            .iter(|| {
                rt.block_on(async {
                    let mut compositor = TestCompositor::new().await.unwrap();

                    let initial_memory = get_process_memory();

                    // Create and destroy 100 windows
                    for _ in 0..100 {
                        let window = compositor.create_window(800, 600).await.unwrap();
                        compositor.map_window(window.id()).await.unwrap();
                        compositor.destroy_window(window.id()).await.unwrap();
                    }

                    let final_memory = get_process_memory();

                    // Memory should not grow significantly
                    let growth = final_memory - initial_memory;
                    assert!(growth < 10_000, "Potential memory leak: {} KB growth", growth);

                    growth
                });
            });
    });
}

fn get_process_memory() -> i64 {
    use std::fs;

    let status = fs::read_to_string("/proc/self/status")
        .expect("Failed to read /proc/self/status");

    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            return parts[1].parse::<i64>().unwrap_or(0);
        }
    }

    0
}

criterion_group!(benches, bench_memory_per_window, bench_memory_leak_detection);
criterion_main!(benches);
```

## Compositor Benchmarks

```rust
//! benches/compositor/window_management.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rustica_comp::Compositor;

fn bench_window_create(c: &mut Criterion) {
    let mut group = c.benchmark_group("window_create");

    for num_windows in &[1, 5, 10, 20] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_windows),
            num_windows,
            |b, &num_windows| {
                let mut rt = tokio::runtime::Runtime::new().unwrap();

                b.to_async(tokio::time::sleep(Duration::from_millis(10)))
                    .iter(|| {
                        rt.block_on(async {
                            let compositor = TestCompositor::new().await.unwrap();

                            for _ in 0..num_windows {
                                compositor.create_window(800, 600).await.unwrap();
                            }
                        });
                    });
            },
        );
    }

    group.finish();
}

fn bench_workspace_switch(c: &mut Criterion) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("workspace_switch", |b| {
        rt.block_on(async {
            let mut compositor = TestCompositor::new().await.unwrap();

            // Create 3 workspaces with windows
            for ws in 0..3 {
                compositor.create_workspace().await.unwrap();

                for _ in 0..5 {
                    let window = compositor.create_window(800, 600).await.unwrap();
                    compositor.assign_window_to_workspace(window.id(), ws).await.unwrap();
                }
            }

            b.to_async(tokio::time::sleep(Duration::from_millis(16)))
                .iter(|| {
                    let start = std::time::Instant::now();

                    for ws in 0..3 {
                        compositor.switch_to_workspace(ws).await.unwrap();
                    }

                    start.elapsed()
                });
        });
    });
}

fn bench_surface_commit(c: &mut Criterion) {
    let mut group = c.benchmark_group("surface_commit");

    for resolution in &[(800, 600), (1920, 1080), (3840, 2160)] {
        group.bench_with_input(
            BenchmarkId::from_parameter(resolution),
            resolution,
            |b, &(width, height)| {
                let mut rt = tokio::runtime::Runtime::new().unwrap();

                b.to_async(tokio::time::sleep(Duration::from_millis(16)))
                    .iter(|| {
                        rt.block_on(async {
                            let compositor = TestCompositor::new().await.unwrap();
                            let window = compositor.create_window(width, height).await.unwrap();

                            compositor.commit_surface(window.id()).await.unwrap()
                        });
                    });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_window_create, bench_workspace_switch, bench_surface_commit);
criterion_main!(benches);
```

## Stress Testing

```rust
//! benches/stress/load.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rustica_comp::Compositor;
use tokio::time::{sleep, Duration};

fn bench_high_input_load(c: &mut Criterion) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("high_input_load", |b| {
        rt.block_on(async {
            let compositor = TestCompositor::new().await.unwrap();
            let window = compositor.create_window(800, 600).await.unwrap();
            compositor.map_window(window.id()).await.unwrap();

            b.to_async(sleep(Duration::from_millis(16)))
                .iter(|| {
                    // Send 100 mouse events rapidly
                    for i in 0..100 {
                        let x = (i as f64 * 8.0) % 800.0;
                        let y = (i as f64 * 6.0) % 600.0;
                        compositor.send_mouse_event(window.id(), x, y).await.unwrap();
                    }

                    // Send 100 key events
                    for _ in 0..100 {
                        compositor.send_key_event(window.id(), KeyCode::KEY_A).await.unwrap();
                    }
                });
        });
    });
}

fn bench_many_windows(c: &mut Criterion) {
    let mut group = c.benchmark_group("many_windows");

    for num_windows in &[10, 50, 100, 200, 500] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_windows),
            num_windows,
            |b, &num_windows| {
                let mut rt = tokio::runtime::Runtime::new().unwrap();

                b.to_async(sleep(Duration::from_millis(100)))
                    .iter(|| {
                        rt.block_on(async {
                            let compositor = TestCompositor::new().await.unwrap();

                            // Create many windows
                            let mut windows = Vec::new();
                            for _ in 0..num_windows {
                                let window = compositor.create_window(800, 600).await.unwrap();
                                compositor.map_window(window.id()).await.unwrap();
                                windows.push(window);
                            }

                            // Measure frame time with many windows
                            let start = std::time::Instant::now();
                            compositor.render_frame().await.unwrap();
                            start.elapsed()
                        });
                    });
            },
        );
    }

    group.finish();
}

fn bench_rapid_window_create_destroy(c: &mut Criterion) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("rapid_window_create_destroy", |b| {
        rt.block_on(async {
            let mut compositor = TestCompositor::new().await.unwrap();

            b.to_async(sleep(Duration::from_millis(10)))
                .iter(|| {
                    for _ in 0..10 {
                        let window = compositor.create_window(800, 600).await.unwrap();
                        compositor.map_window(window.id()).await.unwrap();
                        compositor.destroy_window(window.id()).await.unwrap();
                    }
                });
        });
    });
}

criterion_group!(benches, bench_high_input_load, bench_many_windows, bench_rapid_window_create_destroy);
criterion_main!(benches);
```

## Power Measurement

```rust
//! benches/system/power.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

#[cfg(target_os = "linux")]
fn measure_power_during_operation<F>(duration: Duration, operation: F) -> f64
where
    F: FnOnce(),
{
    use std::fs;

    // Read initial power
    let initial_energy = read_battery_energy();

    // Run operation
    let start = std::time::Instant::now();
    operation();
    let elapsed = start.elapsed();

    // Wait for remaining duration
    if elapsed < duration {
        std::thread::sleep(duration - elapsed);
    }

    // Read final power
    let final_energy = read_battery_energy();

    // Calculate power (J/s = W)
    let energy_diff = initial_energy - final_energy;
    energy_diff / duration.as_secs_f64()
}

#[cfg(target_os = "linux")]
fn read_battery_energy() -> f64 {
    use std::fs;

    // Read from /sys/class/power_supply/BAT0/energy_now
    if let Ok(energy_now) = fs::read_to_string("/sys/class/power_supply/BAT0/energy_now") {
        let energy_uwh: u64 = energy_now.trim().parse().unwrap_or(0);
        energy_uwh as f64 / 1_000_000.0  // Convert to Wh
    } else {
        0.0
    }
}

fn bench_idle_power(c: &mut Criterion) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("idle_power", |b| {
        rt.block_on(async {
            let compositor = TestCompositor::new().await.unwrap();

            b.to_async(tokio::time::sleep(Duration::from_secs(10)))
                .iter(|| {
                    #[cfg(target_os = "linux")]
                    let power = measure_power_during_operation(Duration::from_secs(5), || {
                        // Just idle
                    });

                    #[cfg(not(target_os = "linux"))]
                    let power = 0.0;

                    power
                });
        });
    });
}

fn bench_rendering_power(c: &mut Criterion) {
    let mut group = c.benchmark_group("rendering_power");

    for fps in &[30, 60, 120] {
        group.bench_with_input(
            BenchmarkId::from_parameter(fps),
            fps,
            |b, &fps| {
                let mut rt = tokio::runtime::Runtime::new().unwrap();

                b.to_async(tokio::time::sleep(Duration::from_secs(10)))
                    .iter(|| {
                        rt.block_on(async {
                            let compositor = TestCompositor::new().await.unwrap();
                            let window = compositor.create_window(1920, 1080).await.unwrap();
                            compositor.map_window(window.id()).await.unwrap();

                            #[cfg(target_os = "linux")]
                            let power = measure_power_during_operation(Duration::from_secs(5), || {
                                rt.block_on(async {
                                    let frame_duration = Duration::from_secs_f64(1.0 / fps as f64);
                                    for _ in 0..(fps * 5) {
                                        compositor.render_frame().await.unwrap();
                                        tokio::time::sleep(frame_duration).await;
                                    }
                                }).unwrap();
                            });

                            #[cfg(not(target_os = "linux"))]
                            let power = 0.0;

                            power
                        });
                    });
            },
        );
    }

    group.finish();
}

#[cfg(target_os = "linux")]
criterion_group!(benches, bench_idle_power, bench_rendering_power);
#[cfg(not(target_os = "linux"))]
criterion_group!(benches, bench_idle_power);
criterion_main!(benches);
```

## Profiling

```bash
#!/bin/bash
# scripts/profile.sh

set -e

echo "Profiling Rustica GUI..."

# CPU profiling with perf
echo "=== CPU Profiling ==="
cargo build --release
perf record --call-graph dwarf --freq 99 \
    ./target/release/rustica-comp --test-mode

# Generate flamegraph
perf script | FlameGraph/stackcollapse-perf.pl | \
    FlameGraph/flamegraph.pl > profile-flamegraph.svg

# Memory profiling with valgrind
echo "=== Memory Profiling ==="
valgrind --tool=massif --massif-out-file=massif.out \
    ./target/release/rustica-comp --test-mode

# Generate massif graph
ms_print massif.out > massif.txt

# Generate timeline graph
ms_plot massif.out massif.png

echo "Profiling complete!"
echo "Flamegraph: profile-flamegraph.svg"
echo "Memory graph: massif.png"
```

## Configuration

```toml
# .config/criterion.toml

[ci]
# CI-specific settings
bench_mode = "compare"
threshold = 5.0  # 5% regression threshold

[local]
# Local development settings
bench_mode = "normal"
plotting = true
output_directory = "target/criterion"

[profile]
# Custom profiles
[profile.quick]
inherits = "common"
sample_size = 20
warm_up_time = Duration::from_millis(500)
measurement_time = Duration::from_secs(2)

[profile.comprehensive]
inherits = "common"
sample_size = 100
warm_up_time = Duration::from_secs(1)
measurement_time = Duration::from_secs(10)

[common]
# Common settings
output_format = "quiet"
```

## Continuous Benchmarking

```yaml
# .github/workflows/benchmark.yml
name: Benchmarks

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run benchmarks
        run: |
          cargo install cargo-criterion
          cargo criterion --message-format = bench
      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: false
```

## Performance Targets

```toml
# .config/performance-targets.toml

[rendering]
# Maximum frame time (16.6ms = 60fps)
max_frame_time_ms = 16.6

# Maximum texture upload time
max_texture_upload_ms = 5.0

[input]
# Maximum input latency
max_input_latency_ms = 8.0

# Maximum gesture recognition time
max_gesture_time_ms = 16.0

[memory]
# Maximum memory per window (MB)
max_memory_per_window_mb = 50

# Maximum memory growth per hour (MB)
max_memory_growth_mb_per_hour = 100

[compositor]
# Maximum workspace switch time (ms)
max_workspace_switch_ms = 100

# Maximum window creation time (ms)
max_window_create_ms = 50

[power]
# Maximum idle power (W)
max_idle_power_w = 5.0

# Maximum rendering power at 60fps (W)
max_rendering_power_w = 15.0
```

## Best Practices

1. **Consistent Environment**: Run benchmarks in consistent conditions
2. **Warm-up**: Always warm up before measuring
3. **Multiple Runs**: Run multiple iterations for accuracy
4. **Isolation**: Close other applications during benchmarks
5. **Hardware**: Document hardware used for benchmarks
6. **Baseline**: Establish performance baseline
7. **Regression Detection**: Alert on performance regressions
8. **Profiling**: Profile before optimizing
9. **Realistic**: Benchmark realistic workloads
10. **Document**: Document benchmark methodology

## Dependencies

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
pprof = { version = "0.11", features = ["flamegraph", "criterion"] }
flamegraph = "0.6"

[target.'cfg(target_os = "linux")'.dev-dependencies]
perf-record = "0.1"
```

## Future Enhancements

1. **Real-time Monitoring**: Continuous performance monitoring
2. **Automated Alerts**: Alert on performance regressions
3. **Cross-platform Profiling**: Support macOS and Windows profiling
4. **Power Profiling**: Detailed power consumption analysis
5. **Thermal Profiling**: Measure thermal performance
6. **Network Profiling**: Profile network-dependent operations
7. **GPU Profiling**: GPU-specific profiling tools
8. **Automated Tuning**: Suggest performance improvements
9. **Historical Tracking**: Track performance over time
10. **Comparison Tools**: Compare with other desktop environments
