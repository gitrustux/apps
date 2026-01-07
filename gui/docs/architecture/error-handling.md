# Error Handling & Crash Recovery Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - System Stability

## Overview

This specification defines how Rustica Shell handles errors at all levels, from application crashes to compositor failures. It ensures **compositor crash auto-recovery in <3 seconds**, **app crashes don't affect system stability**, and **automatic crash reporting**.

## Error Types

```rust
pub enum GuiError {
    // Application errors
    AppCrashed { app_id: String, pid: Pid },
    AppHung { app_id: String, pid: Pid },
    AppMisbehaving { app_id: String, reason: String },

    // Compositor errors
    CompositorCrash,
    RenderingFailed,
    InputDeviceFailed { device: DeviceId },

    // Resource errors
    OutOfMemory { required: usize, available: usize },
    GpuCapabilityDenied { capability: Capability },
    SurfaceLost { surface: SurfaceId },

    // System errors
    DisplayRemoved { display: DisplayId },
    SessionTerminated,
}
```

## Error Handling Strategy

### Level 1: Application Errors (Isolated)

Applications can crash without affecting the system:

```rust
pub struct AppState {
    apps: HashMap<String, AppInstance>,
}

impl AppState {
    pub fn handle_app_crash(&mut self, pid: Pid, exit_code: i32) {
        // Find the app
        if let Some((app_id, app)) = self.find_app_by_pid(pid) {
            log::warn!("App {} (PID {}) crashed with exit code {}",
                app_id, pid, exit_code);

            // Generate crash report
            self.generate_crash_report(&app);

            // Notify user
            self.show_notification(Notification {
                title: format!("{} crashed", app.name()),
                body: "Restarting application...",
                urgency: Urgency::Error,
                actions: vec!["Restart".into(), "Dismiss".into()],
            });

            // Clean up
            self.cleanup_app(pid);

            // Auto-restart if user configured
            if self.should_autorestart(&app) {
                self.restart_app(&app_id);
            }
        }
    }

    fn cleanup_app(&mut self, pid: Pid) {
        // Remove from app list
        self.apps.remove(&pid.to_string());

        // Revoke capabilities
        kernel::revoke_all_capabilities(pid);

        // Clean up windows
        compositor.remove_app_windows(pid);
    }
}
```

### Level 2: Compositor Errors (Recoverable)

Compositor errors trigger auto-recovery:

```rust
pub struct Compositor {
    state: CompositorState,
    watchdog: Watchdog,
}

pub enum CompositorState {
    Running,
    Recovering,
    Failed,
}

impl Compositor {
    pub fn run(&mut self) -> ! {
        self.state = CompositorState::Running;
        self.watchdog.start(Duration::from_secs(5));

        loop {
            match self.state {
                CompositorState::Running => {
                    if let Err(e) = self.dispatch_events() {
                        self.handle_error(e);
                    }
                }
                CompositorState::Recovering => {
                    self.recover();
                }
                CompositorState::Failed => {
                    self.notify_failed();
                    std::process::exit(1);
                }
            }
        }
    }

    fn handle_error(&mut self, error: GuiError) {
        match error {
            GuiError::RenderingFailed => {
                // Try to reinitialize rendering
                log::error!("Rendering failed, reinitializing...");
                self.state = CompositorState::Recovering;
            }
            GuiError::OutOfMemory { .. } => {
                // Try to free memory
                self.free_memory();
            }
            GuiError::GpuCapabilityDenied { .. } => {
                // Fallback to software rendering
                self.use_software_rendering();
            }
            _ => {
                log::error!("Unhandled error: {:?}", error);
            }
        }
    }

    fn recover(&mut self) {
        log::info!("Attempting compositor recovery...");

        // 1. Save current state
        let state_snapshot = self.save_state();

        // 2. Reinitialize rendering
        if let Err(e) = self.reinitialize_rendering() {
            log::error!("Recovery failed: {}", e);
            self.state = CompositorState::Failed;
            return;
        }

        // 3. Restore state
        self.restore_state(state_snapshot);

        // 4. Notify user
        self.show_notification(Notification {
            title: "Display Recovered".into(),
            body: "The display system recovered from an error.".into(),
            urgency: Urgency::Warning,
        });

        self.state = CompositorState::Running;
        log::info!("Recovery successful");
    }
}
```

### Level 3: Watchdog System

```rust
pub struct Watchdog {
    interval: Duration,
    last_ping: Instant,
    timeout: Duration,
}

impl Watchdog {
    pub fn start(&mut self, interval: Duration) {
        self.interval = interval;
        self.last_ping = Instant::now();
        self.timeout = Duration::from_secs(15);

        thread::spawn(move || {
            loop {
                thread::sleep(interval);
                if Instant::now().duration_since(self.last_ping) > self.timeout {
                    // Compositor not responding, restart it
                    Self::trigger_restart();
                }
            }
        });
    }

    pub fn ping(&mut self) {
        self.last_ping = Instant::now();
    }
}
```

## Crash Reporting

### Crash Report Structure

```rust
pub struct CrashReport {
    // Timestamp
    pub timestamp: DateTime<Utc>,

    // Application info
    pub app_id: String,
    pub app_version: String,
    pub pid: Pid,

    // Crash info
    pub signal: Signal,
    pub exit_code: Option<i32>,

    // System info
    pub os_version: String,
    pub kernel_version: String,
    pub rustica_version: String,

    // Stack trace
    pub backtrace: String,

    // Logs
    pub logs: String,

    // System state
    pub memory_usage: MemoryStats,
    pub gpu_info: GpuInfo,
}
```

### Automatic Crash Collection

```rust
pub fn generate_crash_report(app: &AppInstance) -> CrashReport {
    // Capture stack trace
    let backtrace = Backtrace::new()
        .frames()
        .iter()
        .map(|frame| format!("{:?}", frame))
        .collect::<Vec<_>>()
        .join("\n");

    // Collect logs
    let logs = app.collect_logs(
        app.started()..Instant::now()
    );

    // System information
    let os_version = read_os_release();
    let kernel_version = read_kernel_version();

    CrashReport {
        timestamp: Utc::now(),
        app_id: app.id().clone(),
        app_version: app.version().clone(),
        pid: app.pid(),
        signal: app.crash_signal(),
        exit_code: app.exit_code(),
        os_version,
        kernel_version,
        rustica_version: env!("CARGO_PKG_VERSION").into(),
        backtrace,
        logs,
        memory_usage: MemoryStats::collect(),
        gpu_info: GpuInfo::collect(),
    }
}
```

### Crash Report Storage

```rust
pub struct CrashStorage {
    reports_dir: PathBuf,
}

impl CrashStorage {
    pub fn save_report(&self, report: &CrashReport) -> Result<()> {
        let filename = format!(
            "crash_{}_{}.json",
            report.app_id,
            report.timestamp.format("%Y%m%d_%H%M%S")
        );
        let path = self.reports_dir.join(filename);

        let json = serde_json::to_string_pretty(report)?;
        fs::write(&path, json)?;

        log::info!("Crash report saved to {}", path.display());
        Ok(())
    }

    pub fn get_recent_reports(&self, limit: usize) -> Vec<CrashReport> {
        let mut reports = Vec::new();

        for entry in fs::read_dir(&self.reports_dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension() == Some("json".into()))
        {
            let path = entry.path();
            if let Ok(json) = fs::read_to_string(&path) {
                if let Ok(report) = serde_json::from_str::<CrashReport>(&json) {
                    reports.push(report);
                }
            }
        }

        reports.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        reports.truncate(limit);
        reports
    }
}
```

## App Hang Detection

```rust
pub struct HangDetector {
    timeout: Duration,
    last_activity: HashMap<Pid, Instant>,
}

impl HangDetector {
    pub fn check_hangs(&mut self) -> Vec<Pid> {
        let now = Instant::now();
        let mut hung_apps = Vec::new();

        for (&pid, &last_activity) in &self.last_activity {
            if now.duration_since(*last_activity) > self.timeout {
                hung_apps.push(*pid);
            }
        }

        hung_apps
    }

    pub fn kill_hung_app(&mut self, pid: Pid) {
        log::warn!("Killing hung app (PID {})", pid);

        // Try to kill gracefully
        if let Err(e) = nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(pid),
            nix::sys::signal::Signal::SIGTERM
        ) {
            log::error!("Failed to kill app: {}", e);
        }

        // Wait 5 seconds then SIGKILL
        thread::sleep(Duration::from_secs(5));
        if let Err(e) = nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(pid),
            nix::sys::signal::Signal::SIGKILL
        ) {
            log::error!("Failed to SIGKILL app: {}", e);
        }
    }
}
```

## Error Recovery Strategies

### Rendering Failure Recovery

```rust
impl Compositor {
    pub fn use_software_rendering(&mut self) {
        log::warn!("Switching to software rendering");

        // Save current surfaces
        let surfaces = self.surfaces.clone();

        // Reinitialize with Pixman
        self.backend = Box::new(PixmanBackend::new());

        // Restore surfaces
        for surface in surfaces {
            self.backend.create_surface(surface.config());
        }

        // Notify user
        self.show_notification(Notification {
            title: "Software Rendering".into(),
            body: "Using software rendering due to hardware failure.".into(),
            urgency: Urgency::Warning,
        });
    }
}
```

### Memory Pressure Handling

```rust
impl Compositor {
    pub fn free_memory(&mut self) {
        log::warn!("Low memory, freeing resources...");

        // 1. Drop unused surfaces
        self.drop_hidden_surfaces();

        // 2. Reduce texture cache size
        self.shrink_texture_cache();

        // 3. Ask apps to reduce memory
        for app in self.apps.values() {
            app.send_event(SystemEvent::LowMemory);
        }

        // 4. If still low, restart compositor
        if self.memory_pressure() > 0.9 {
            log::error!("Critical memory pressure, restarting...");
            self.restart();
        }
    }

    fn drop_hidden_surfaces(&mut self) {
        self.surfaces.retain(|surface| {
            if surface.state() == WindowState::Hidden {
                self.backend.destroy_surface(surface.id());
                false
            } else {
                true
            }
        });
    }
}
```

## User Notifications

### Error Notification UI

```rust
pub fn show_error_notification(&self, error: GuiError) {
    match error {
        GuiError::AppCrashed { app_id, .. } => {
            self.show_notification(Notification {
                title: format!("{} Crashed", app_id),
                body: "Would you like to restart it?".into(),
                urgency: Urgency::Error,
                actions: vec![
                    "Restart".into(),
                    "Close".into(),
                    "Report".into(),
                ],
            });
        }
        GuiError::OutOfMemory { .. } => {
            self.show_notification(Notification {
                title: "Low Memory".into(),
                body: "Close some applications and try again.".into(),
                urgency: Urgency::Critical,
                actions: vec!["Open System Monitor".into()],
            });
        }
        _ => {}
    }
}
```

## Compositor Restart

### Clean Restart Process

```rust
impl Compositor {
    pub fn restart(&mut self) {
        log::info!("Initiating compositor restart...");

        // 1. Save state to disk
        self.save_state_to_disk();

        // 2. Notify all apps (graceful shutdown)
        for app in self.apps.values_mut() {
            app.send_event(SystemEvent::SessionTerminated);
        }

        // 3. Give apps time to save (5 seconds)
        thread::sleep(Duration::from_secs(5));

        // 4. Execute new compositor process
        let exe = env::current_exe().unwrap();
        Command::new(exe)
            .arg("--recover")
            .spawn()
            .expect("Failed to spawn new compositor");

        // 5. Exit this process
        std::process::exit(0);
    }

    fn recover_from_restart(&mut self) -> Result<()> {
        log::info!("Recovering from restart...");

        // Load saved state
        let state = self.load_state_from_disk()?;

        // Reinitialize
        self.reinitialize()?;

        // Restore state
        self.restore_state(state);

        log::info!("Recovery complete");
        Ok(())
    }
}
```

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Compositor crash recovery | <3 seconds | Crash to functional |
| App crash handling | <1 second | Crash to notification |
| Memory leak detection | Continuous | Monitor RSS |
| Crash report generation | <2 seconds | Crash to saved |

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-comp/src/
│   ├── error/
│   │   ├── mod.rs              # Error module
│   │   ├── recovery.rs         # Crash recovery
│   │   ├── watchdog.rs         # Watchdog system
│   │   └── crash.rs            # Crash reporting
│   └── ...
│
├── crash_reports/              # Crash report storage
│   └── *.json
│
└── system/
    ├── rustica-crash-handler   # Crash handler daemon
    └── rustica-error-monitor    # Error monitoring
```

## Example Usage

```rust
use librustica::error::*;

fn main() {
    let mut compositor = Compositor::new();

    // Enable error handling
    compositor.set_error_handler(|error| {
        match error {
            GuiError::AppCrashed { .. } => {
                // Handle app crash
                handle_app_crash(error);
            }
            GuiError::RenderingFailed => {
                // Try to recover
                compositor.recover();
            }
            _ => {}
        }
    });

    // Enable crash reporting
    compositor.set_crash_handler(|report| {
        crash_storage.save_report(report);
    });

    // Enable watchdog
    compositor.set_watchdog(Duration::from_secs(5));

    // Run compositor
    compositor.run();
}
```

## Success Criteria

- [ ] Compositor crash recovers in <3 seconds
- [ ] App crashes don't affect system stability
- [ ] Automatic crash reporting works
- [ ] Memory pressure is handled gracefully
- [ ] Watchdog detects and recovers from hangs
- [ ] Performance targets met

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Restart loop | Max restart attempts, notify user |
| Crash report size | Limit recent reports, compress old ones |
| False positive crash detection | Timeout before declaring crash |
| Recovery fails | Fallback to minimal mode |

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## References

- [systemd-coredump](https://www.freedesktop.org/software/systemd/man/systemd-coredump.html)
- [Wayland Error Handling](https://wayland.freedesktop.org/docs/html/ch04.html#sect-Protocol)
- [Watchdog Timers](https://docs.rs/watchdogs/0.1.0/watchdogs/)
- [Crash Reporting Best Practices](https://github.com/rust-lang/backtrace-rs)
