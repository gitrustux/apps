# Phase 8.5: Mobile Battery Optimization (rustica-power)

## Overview

**Component**: rustica-power
**Purpose**: Power management and battery optimization for mobile devices
**Language**: Rust
**Dependencies**: libc, dbus (zbus), upower

## Goals

1. **Extended Battery Life**: Maximize battery runtime through intelligent power management
2. **Adaptive Performance**: Balance performance and power based on usage patterns
3. **User Control**: Give users granular control over power settings
4. **Transparent**: Show battery usage and power drain sources
5. **Graceful Degradation**: Maintain functionality when battery is low

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    User Applications                         │
│                   (request power hints)                      │
└────────────────────────┬────────────────────────────────────┘
                         │ D-Bus / Hints
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  rustica-power                               │
│                  (Power Manager)                             │
├─────────────────────────────────────────────────────────────┤
│  PowerProfile       │  BatteryMonitor   │  AppLimiter       │
│  - Power modes      │  - Battery status │  - Background limits│
│  - CPU governor     │  - Usage tracking │  - Wake lock mgmt  │
│  - Display control  │  - Health monitor │  - Job scheduling  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  Kernel / Hardware                           │
│           (CPUfreq, backlight, device power)                 │
└─────────────────────────────────────────────────────────────┘
```

## Core Data Structures

```rust
/// Battery health and status information
#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub percentage: u8,
    pub state: BatteryState,
    pub time_to_empty: Option<Duration>,
    pub time_to_full: Option<Duration>,
    pub energy: f64,        // Wh
    pub energy_full: f64,   // Wh (design capacity)
    pub energy_full_design: f64,  // Wh (original capacity)
    pub voltage: f64,       // V
    pub temperature: f64,   // °C
    pub cycle_count: u32,
    pub health: BatteryHealth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryState {
    Unknown,
    Charging,
    Discharging,
    Empty,
    FullyCharged,
    PendingCharge,
    PendingDischarge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryHealth {
    Unknown,
    Good,        // > 80% of design capacity
    Fair,        // 60-80% of design capacity
    Poor,        // 40-60% of design capacity
    Critical,    // < 40% of design capacity
}

/// Power profile (performance vs battery)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerProfile {
    /// Maximum performance, ignores battery
    Performance,

    /// Balanced mode (default)
    Balanced,

    /// Power saving, reduced performance
    PowerSaver,

    /// Maximum battery life
    BatterySaver,

    /// Adaptive profile based on usage
    Adaptive,
}

/// Application power state hint
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerHint {
    /// Normal operation
    Normal,

    /// App is performance-critical (game, video editing)
    PerformanceCritical,

    /// App is background and should minimize power
    Background,

    /// App should be suspended
    Suspend,

    /// App needs to keep running but can be throttled
    Throttled,
}

/// Wake lock to prevent system sleep
pub struct WakeLock {
    id: String,
    owner: String,
    lock_type: WakeLockType,
    created_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakeLockType {
    /// Prevent system sleep entirely
    System,

    /// Allow display sleep but keep system awake
    Partial,

    /// Allow system sleep but keep specific resource
    /// (e.g., network, audio)
    Specific,
}
```

## Power Manager

```rust
pub struct PowerManager {
    battery: BatteryMonitor,
    profile_manager: ProfileManager,
    app_limiter: AppLimiter,
    wake_lock_manager: WakeLockManager,
    usage_tracker: UsageTracker,
    config: PowerConfig,
}

impl PowerManager {
    pub fn new() -> Result<Self, Error> {
        let config = PowerConfig::load()?;

        Ok(Self {
            battery: BatteryMonitor::new()?,
            profile_manager: ProfileManager::new(),
            app_limiter: AppLimiter::new(),
            wake_lock_manager: WakeLockManager::new(),
            usage_tracker: UsageTracker::new(),
            config,
        })
    }

    /// Get current battery information
    pub fn battery_info(&self) -> BatteryInfo {
        self.battery.info()
    }

    /// Set active power profile
    pub fn set_profile(&mut self, profile: PowerProfile) -> Result<(), Error> {
        self.profile_manager.set_profile(profile)?;
        self.apply_profile_settings(profile)?;

        // Notify listeners
        self.notify_profile_change(profile);

        Ok(())
    }

    /// Get current power profile
    pub fn current_profile(&self) -> PowerProfile {
        self.profile_manager.current_profile()
    }

    /// Application provides power hint
    pub fn set_app_hint(&mut self, app_id: String, hint: PowerHint) -> Result<(), Error> {
        match hint {
            PowerHint::PerformanceCritical => {
                // Grant higher CPU priority, prevent throttling
                self.app_limiter.set_priority(app_id, Priority::High)?;
            }
            PowerHint::Background => {
                // Allow aggressive throttling
                self.app_limiter.set_priority(app_id, Priority::Low)?;
                self.app_limiter.enable_throttling(app_id, 0.5)?;
            }
            PowerHint::Suspend => {
                // Suspend the app
                self.app_limiter.suspend(app_id)?;
            }
            PowerHint::Throttled => {
                // Moderate throttling
                self.app_limiter.enable_throttling(app_id, 0.75)?;
            }
            _ => {}
        }

        Ok(())
    }

    /// Acquire wake lock
    pub fn acquire_wake_lock(
        &mut self,
        owner: String,
        lock_type: WakeLockType,
        reason: String,
    ) -> Result<WakeLock, Error> {
        // Check if battery saver mode is active
        if self.current_profile() == PowerProfile::BatterySaver {
            // Restrict wake locks in battery saver
            return Err(Error::BatterySaverActive);
        }

        let wake_lock = self.wake_lock_manager.acquire(owner, lock_type, reason)?;

        // Prevent sleep if system wake lock
        if lock_type == WakeLockType::System {
            self.prevent_system_sleep();
        }

        Ok(wake_lock)
    }

    /// Release wake lock
    pub fn release_wake_lock(&mut self, id: String) -> Result<(), Error> {
        self.wake_lock_manager.release(id)?;

        // Re-enable sleep if no more system wake locks
        if !self.wake_lock_manager.has_system_locks() {
            self.allow_system_sleep();
        }

        Ok(())
    }

    /// Get battery usage breakdown
    pub fn battery_usage(&self) -> BatteryUsage {
        self.usage_tracker.usage_breakdown()
    }

    /// Check and apply battery thresholds
    pub fn check_battery_thresholds(&mut self) -> Result<(), Error> {
        let info = self.battery.info();

        // Low battery threshold
        if info.percentage <= self.config.low_battery_threshold {
            if self.current_profile() != PowerProfile::BatterySaver {
                // Auto-enable battery saver
                self.set_profile(PowerProfile::BatterySaver)?;
                self.notify_low_battery();
            }
        }

        // Critical battery threshold
        if info.percentage <= self.config.critical_battery_threshold {
            // Prepare for shutdown
            self.prepare_critical_shutdown()?;
        }

        Ok(())
    }

    fn apply_profile_settings(&mut self, profile: PowerProfile) -> Result<(), Error> {
        match profile {
            PowerProfile::Performance => {
                // Max CPU frequency
                self.set_cpu_governor("performance")?;

                // Max display brightness
                self.set_display_brightness(100)?;

                // Disable throttling
                self.app_limiter.disable_all_throttling()?;

                // Disable auto-suspend
                self.set_auto_suspend(false)?;
            }

            PowerProfile::Balanced => {
                // Balanced CPU governor
                self.set_cpu_governor("schedutil")?;

                // Auto display brightness
                self.set_auto_brightness(true)?;

                // Moderate throttling
                self.app_limiter.set_default_throttling(0.9)?;

                // Enable auto-suspend
                self.set_auto_suspend(true)?;
                self.set_suspend_timeout(Duration::from_secs(300))?;
            }

            PowerProfile::PowerSaver => {
                // Powersave CPU governor
                self.set_cpu_governor("powersave")?;

                // Lower display brightness
                self.set_display_brightness(60)?;

                // Enable throttling
                self.app_limiter.enable_global_throttling(0.7)?;

                // Aggressive auto-suspend
                self.set_auto_suspend(true)?;
                self.set_suspend_timeout(Duration::from_secs(120))?;
            }

            PowerProfile::BatterySaver => {
                // Min power CPU governor
                self.set_cpu_governor("powersave")?;

                // Low display brightness
                self.set_display_brightness(40)?;

                // Aggressive throttling
                self.app_limiter.enable_global_throttling(0.5)?;

                // Limit background activity
                self.app_limiter.suspend_background_apps()?;

                // Very aggressive auto-suspend
                self.set_auto_suspend(true)?;
                self.set_suspend_timeout(Duration::from_secs(60))?;

                // Reduce sensor sampling
                self.reduce_sensor_activity()?;
            }

            PowerProfile::Adaptive => {
                // Let system decide based on usage
                self.profile_manager.optimize_adaptively(&self.usage_tracker)?;
            }
        }

        Ok(())
    }

    fn set_cpu_governor(&self, governor: &str) -> Result<(), Error> {
        for cpu in 0..num_cpus::get() {
            let path = format!(
                "/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor",
                cpu
            );

            std::fs::write(&path, governor)?;
        }

        Ok(())
    }

    fn set_display_brightness(&self, percent: u8) -> Result<(), Error> {
        // Write to backlight sysfs
        let max_brightness_path = "/sys/class/backlight/*/max_brightness";
        let brightness_path = "/sys/class/backlight/*/brightness";

        // Read max brightness
        let max = std::fs::read_to_string(max_brightness_path)?
            .trim()
            .parse::<u32>()?;

        // Calculate brightness value
        let value = (max as f64 * percent as f64 / 100.0) as u32;

        std::fs::write(brightness_path, value.to_string())?;

        Ok(())
    }

    fn set_auto_brightness(&self, enabled: bool) -> Result<(), Error> {
        // Use ambient light sensor for auto-brightness
        self.battery.set_auto_brightness(enabled)
    }

    fn set_auto_suspend(&self, enabled: bool) -> Result<(), Error> {
        // Configure systemd logind
        let connection = zbus::Connection::system()?;

        let proxy = LogindProxy::new(&connection)?;
        proxy.set_suspend_on_lid_switch(enabled)?;

        Ok(())
    }

    fn set_suspend_timeout(&self, timeout: Duration) -> Result<(), Error> {
        // Configure screen blanking timeout
        let config = format!(
            "[Desktop]
Session idle-delay={}",
            timeout.as_secs()
        );

        std::fs::write(
            "/var/lib/rustica/power/suspend.conf",
            config,
        )?;

        Ok(())
    }

    fn reduce_sensor_activity(&self) -> Result<(), Error> {
        // Reduce sensor sampling rates via D-Bus
        let connection = zbus::Connection::session()?;

        let proxy = SensorProxy::new(&connection)?;
        proxy.set_global_sampling_period(Duration::from_millis(1000))?;

        Ok(())
    }

    fn notify_low_battery(&self) {
        // Show low battery notification
        Notification::new()
            .summary("Low Battery")
            .body(&format!(
                "Battery at {}%. Connect to power soon.",
                self.battery.info().percentage
            ))
            .icon("battery-low")
            .show();
    }

    fn prepare_critical_shutdown(&mut self) -> Result<(), Error> {
        // Show critical battery warning
        Notification::new()
            .summary("Critical Battery")
            .body(&format!(
                "Battery critically low at {}%. System will shutdown soon.",
                self.battery.info().percentage
            ))
            .icon("battery-empty")
            .urgency(critical)
            .show();

        // Gracefully shutdown apps
        self.app_limiter.prepare_shutdown()?;

        Ok(())
    }
}
```

## Battery Monitor

```rust
pub struct BatteryMonitor {
    upower: UPowerClient,
    history: VecDeque<BatterySnapshot>,
    auto_brightness: AmbientLightSensor,
}

impl BatteryMonitor {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            upower: UPowerClient::new()?,
            history: VecDeque::with_capacity(1440),  // 24 hours of minute data
            auto_brightness: AmbientLightSensor::new()?,
        })
    }

    pub fn info(&self) -> BatteryInfo {
        self.upower.get_battery_info()
    }

    pub fn set_auto_brightness(&mut self, enabled: bool) -> Result<(), Error> {
        if enabled {
            let lux = self.auto_brightness.read_illuminance()?;
            let brightness = self.recommended_brightness(lux);
            self.set_backlight(brightness)?;
        }

        Ok(())
    }

    fn recommended_brightness(&self, lux: f64) -> u8 {
        // HLG (Hybrid Log-Gamma) curve
        if lux < 10.0 {
            10
        } else if lux < 100.0 {
            30
        } else if lux < 1000.0 {
            60
        } else if lux < 10000.0 {
            80
        } else {
            100
        }
    }

    /// Estimate battery health based on capacity degradation
    pub fn calculate_health(&self) -> BatteryHealth {
        let info = self.info();

        if info.energy_full_design == 0.0 {
            return BatteryHealth::Unknown;
        }

        let capacity_percent = (info.energy_full / info.energy_full_design) * 100.0;

        if capacity_percent > 80.0 {
            BatteryHealth::Good
        } else if capacity_percent > 60.0 {
            BatteryHealth::Fair
        } else if capacity_percent > 40.0 {
            BatteryHealth::Poor
        } else {
            BatteryHealth::Critical
        }
    }

    /// Record battery snapshot for history
    pub fn record_snapshot(&mut self) {
        let snapshot = BatterySnapshot {
            timestamp: Utc::now(),
            info: self.info(),
        };

        self.history.push_back(snapshot);

        // Keep only 24 hours
        while self.history.len() > 1440 {
            self.history.pop_front();
        }
    }

    /// Calculate battery drain rate (%/hour)
    pub fn drain_rate(&self) -> f64 {
        if self.history.len() < 2 {
            return 0.0;
        }

        let oldest = &self.history[0];
        let newest = &self.history[self.history.len() - 1];

        let percent_diff = oldest.info.percentage as f64 - newest.info.percentage as f64;
        let hours = newest.timestamp.signed_duration_since(oldest.timestamp)
            .num_seconds() as f64 / 3600.0;

        if hours > 0.0 {
            percent_diff / hours
        } else {
            0.0
        }
    }

    /// Estimate remaining battery time
    pub fn estimate_remaining(&self) -> Option<Duration> {
        let info = self.info();

        if info.time_to_empty.is_some() {
            return info.time_to_empty;
        }

        // Estimate from drain rate
        let rate = self.drain_rate();
        if rate > 0.0 {
            let hours = info.percentage as f64 / rate;
            Some(Duration::from_secs_f64(hours * 3600.0))
        } else {
            None
        }
    }
}

struct BatterySnapshot {
    timestamp: DateTime<Utc>,
    info: BatteryInfo,
}
```

## Profile Manager

```rust
pub struct ProfileManager {
    current_profile: PowerProfile,
    learning_enabled: bool,
    usage_patterns: UsagePatterns,
}

impl ProfileManager {
    pub fn new() -> Self {
        Self {
            current_profile: PowerProfile::Balanced,
            learning_enabled: true,
            usage_patterns: UsagePatterns::load(),
        }
    }

    pub fn set_profile(&mut self, profile: PowerProfile) -> Result<(), Error> {
        self.current_profile = profile;
        self.save_profile(profile);
        Ok(())
    }

    pub fn current_profile(&self) -> PowerProfile {
        self.current_profile
    }

    /// Adaptive optimization based on usage patterns
    pub fn optimize_adaptively(&mut self, tracker: &UsageTracker) -> Result<(), Error> {
        if !self.learning_enabled {
            return Ok(());
        }

        // Learn from usage patterns
        let usage = tracker.analyze_usage();

        // Determine optimal profile
        let optimal_profile = if usage.performance_critical {
            PowerProfile::Performance
        } else if usage.background_heavy {
            PowerProfile::PowerSaver
        } else if usage.battery_critical {
            PowerProfile::BatterySaver
        } else {
            PowerProfile::Balanced
        };

        self.set_profile(optimal_profile)?;

        Ok(())
    }

    fn save_profile(&self, profile: PowerProfile) {
        std::fs::write(
            "/var/lib/rustica/power/current-profile",
            format!("{:?}", profile),
        ).ok();
    }
}

struct UsagePatterns {
    time_of_day_profiles: HashMap<TimeOfDay, PowerProfile>,
    app_profiles: HashMap<String, PowerProfile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TimeOfDay {
    Morning,    // 6-12
    Afternoon,  // 12-18
    Evening,    // 18-24
    Night,      // 0-6
}

impl UsagePatterns {
    fn load() -> Self {
        let path = "/var/lib/rustica/power/usage-patterns.json";

        if let Ok(file) = std::fs::File::open(path) {
            serde_json::from_reader(file).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    fn save(&self) {
        let path = "/var/lib/rustica/power/usage-patterns.json";

        if let Ok(file) = std::fs::File::create(path) {
            serde_json::to_writer_pretty(file, self).ok();
        }
    }
}

impl Default for UsagePatterns {
    fn default() -> Self {
        Self {
            time_of_day_profiles: HashMap::new(),
            app_profiles: HashMap::new(),
        }
    }
}
```

## App Limiter

```rust
pub struct AppLimiter {
    app_priorities: HashMap<String, Priority>,
    throttled_apps: HashMap<String, f64>,  // app_id -> throttle factor (0.0-1.0)
    suspended_apps: HashSet<String>,
    background_apps: HashSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Normal,
    High,
}

impl AppLimiter {
    pub fn new() -> Self {
        Self {
            app_priorities: HashMap::new(),
            throttled_apps: HashMap::new(),
            suspended_apps: HashSet::new(),
            background_apps: HashSet::new(),
        }
    }

    pub fn set_priority(&mut self, app_id: String, priority: Priority) -> Result<(), Error> {
        self.app_priorities.insert(app_id.clone(), priority);

        // Apply via cgroups
        self.apply_cgroup_priority(&app_id, priority)?;

        Ok(())
    }

    pub fn enable_throttling(&mut self, app_id: String, factor: f64) -> Result<(), Error> {
        if factor < 0.0 || factor > 1.0 {
            return Err(Error::InvalidThrottleFactor);
        }

        self.throttled_apps.insert(app_id.clone(), factor);

        // Apply CPU quota via cgroups
        self.apply_cpu_quota(&app_id, factor)?;

        Ok(())
    }

    pub fn enable_global_throttling(&mut self, factor: f64) -> Result<(), Error> {
        for app_id in self.background_apps.iter() {
            self.enable_throttling(app_id.clone(), factor)?;
        }

        Ok(())
    }

    pub fn set_default_throttling(&mut self, factor: f64) -> Result<(), Error> {
        // Throttle all non-essential apps
        for app_id in self.app_priorities.keys() {
            if self.app_priorities.get(app_id) != Some(&Priority::High) {
                self.enable_throttling(app_id.clone(), factor)?;
            }
        }

        Ok(())
    }

    pub fn disable_all_throttling(&mut self) -> Result<(), Error> {
        for app_id in self.throttled_apps.keys() {
            self.apply_cpu_quota(app_id, 1.0)?;
        }

        self.throttled_apps.clear();

        Ok(())
    }

    pub fn suspend(&mut self, app_id: String) -> Result<(), Error> {
        // Send SIGSTOP to process
        if let Some(pid) = self.get_app_pid(&app_id) {
            unsafe {
                libc::kill(pid as libc::pid_t, libc::SIGSTOP);
            }
        }

        self.suspended_apps.insert(app_id);

        Ok(())
    }

    pub fn resume(&mut self, app_id: String) -> Result<(), Error> {
        // Send SIGCONT to process
        if let Some(pid) = self.get_app_pid(&app_id) {
            unsafe {
                libc::kill(pid as libc::pid_t, libc::SIGCONT);
            }
        }

        self.suspended_apps.remove(&app_id);

        Ok(())
    }

    pub fn suspend_background_apps(&mut self) -> Result<(), Error> {
        for app_id in self.background_apps.iter() {
            self.suspend(app_id.clone())?;
        }

        Ok(())
    }

    pub fn prepare_shutdown(&mut self) -> Result<(), Error> {
        // Gracefully shutdown all apps except critical system apps
        for app_id in self.app_priorities.keys() {
            if self.app_priorities.get(app_id) != Some(&Priority::High) {
                self.graceful_shutdown(app_id.clone())?;
            }
        }

        Ok(())
    }

    fn apply_cgroup_priority(&self, app_id: &str, priority: Priority) -> Result<(), Error> {
        // Use cgroups v2 to set CPU weight
        let cpu_weight = match priority {
            Priority::Low => 1,
            Priority::Normal => 100,
            Priority::High => 1000,
        };

        let cgroup_path = format!("/sys/fs/cgroup/user.slice/apps-{}/cpu.weight", app_id);

        std::fs::write(&cgroup_path, cpu_weight.to_string())?;

        Ok(())
    }

    fn apply_cpu_quota(&self, app_id: &str, factor: f64) -> Result<(), Error> {
        // CPU.max = quota $period
        // quota = period * factor
        let period = 100000;  // 100ms
        let quota = (period as f64 * factor) as u64;

        let cgroup_path = format!("/sys/fs/cgroup/user.slice/apps-{}/cpu.max", app_id);

        std::fs::write(&cgroup_path, format!("{} {}", quota, period))?;

        Ok(())
    }

    fn get_app_pid(&self, app_id: &str) -> Option<u32> {
        // Look up PID via D-Bus or cgroup
        // This is simplified
        Some(1234)  // Placeholder
    }

    fn graceful_shutdown(&self, app_id: String) -> Result<(), Error> {
        // Send SIGTERM for graceful shutdown
        if let Some(pid) = self.get_app_pid(&app_id) {
            unsafe {
                libc::kill(pid as libc::pid_t, libc::SIGTERM);
            }
        }

        Ok(())
    }
}
```

## Wake Lock Manager

```rust
pub struct WakeLockManager {
    locks: HashMap<String, WakeLock>,
    next_id: u64,
}

impl WakeLockManager {
    pub fn new() -> Self {
        Self {
            locks: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn acquire(
        &mut self,
        owner: String,
        lock_type: WakeLockType,
        reason: String,
    ) -> Result<WakeLock, Error> {
        let id = format!("wl-{}", self.next_id);
        self.next_id += 1;

        let wake_lock = WakeLock {
            id: id.clone(),
            owner,
            lock_type,
            created_at: Instant::now(),
        };

        self.locks.insert(id.clone(), wake_lock.clone());

        // Log wake lock acquisition
        log::info!("Wake lock acquired: {} by {} for {:?}", id, wake_lock.owner, lock_type);

        Ok(wake_lock)
    }

    pub fn release(&mut self, id: String) -> Result<(), Error> {
        let wake_lock = self.locks.remove(&id)
            .ok_or(Error::WakeLockNotFound)?;

        log::info!(
            "Wake lock released: {} by {} (held for {:?})",
            id,
            wake_lock.owner,
            wake_lock.created_at.elapsed()
        );

        Ok(())
    }

    pub fn has_system_locks(&self) -> bool {
        self.locks.values()
            .any(|lock| lock.lock_type == WakeLockType::System)
    }

    /// Clean up expired locks
    pub fn cleanup_expired(&mut self, max_age: Duration) {
        let now = Instant::now();

        self.locks.retain(|_, lock| {
            now.duration_since(lock.created_at) < max_age
        });
    }

    /// Get all active locks
    pub fn active_locks(&self) -> Vec<&WakeLock> {
        self.locks.values().collect()
    }
}
```

## Usage Tracker

```rust
pub struct UsageTracker {
    app_energy: HashMap<String, EnergyUsage>,
    screen_usage: ScreenUsageTracker,
    sensor_usage: SensorUsageTracker,
}

impl UsageTracker {
    pub fn new() -> Self {
        Self {
            app_energy: HashMap::new(),
            screen_usage: ScreenUsageTracker::new(),
            sensor_usage: SensorUsageTracker::new(),
        }
    }

    pub fn track_app_energy(&mut self, app_id: String, energy_joules: f64) {
        let usage = self.app_energy.entry(app_id).or_insert_with(EnergyUsage::default);
        usage.total_joules += energy_joules;
        usage.last_update = Utc::now();
    }

    pub fn analyze_usage(&self) -> UsageAnalysis {
        let total_energy: f64 = self.app_energy.values()
            .map(|e| e.total_joules)
            .sum();

        let performance_critical = self.app_energy.values()
            .any(|usage| usage.total_joules / total_energy > 0.3);

        let background_heavy = self.screen_usage.background_time() > Duration::from_secs(3600);

        let battery_critical = total_energy < 1000.0;  // Less than 1000J remaining

        UsageAnalysis {
            performance_critical,
            background_heavy,
            battery_critical,
        }
    }

    pub fn usage_breakdown(&self) -> BatteryUsage {
        let mut app_breakdown = Vec::new();

        for (app_id, usage) in &self.app_energy {
            app_breakdown.push(AppUsage {
                app_id: app_id.clone(),
                energy_joules: usage.total_joules,
                percentage: 0.0,  // Calculated from total
            });
        }

        // Sort by energy usage
        app_breakdown.sort_by(|a, b| b.energy_joules.partial_cmp(&a.energy_joules).unwrap());

        let total_energy: f64 = app_breakdown.iter()
            .map(|u| u.energy_joules)
            .sum();

        // Calculate percentages
        for usage in &mut app_breakdown {
            usage.percentage = (usage.energy_joules / total_energy) * 100.0;
        }

        BatteryUsage {
            apps: app_breakdown,
            screen: self.screen_usage.usage(),
            sensors: self.sensor_usage.usage(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct EnergyUsage {
    total_joules: f64,
    last_update: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UsageAnalysis {
    pub performance_critical: bool,
    pub background_heavy: bool,
    pub battery_critical: bool,
}

#[derive(Debug, Clone)]
pub struct BatteryUsage {
    pub apps: Vec<AppUsage>,
    pub screen: ScreenUsage,
    pub sensors: SensorUsage,
}

#[derive(Debug, Clone)]
pub struct AppUsage {
    pub app_id: String,
    pub energy_joules: f64,
    pub percentage: f64,
}

#[derive(Debug, Clone)]
pub struct ScreenUsage {
    pub on_time: Duration,
    pub brightness_factor: f64,
}

#[derive(Debug, Clone)]
pub struct SensorUsage {
    pub accelerometer_joules: f64,
    pub gps_joules: f64,
    pub other_joules: f64,
}
```

## D-Bus Interface

```rust
#[dbus_interface(name = "org.rustica.Power")]
impl PowerManager {
    /// Get battery information
    fn battery_info(&self) -> BatteryInfo {
        self.battery_info()
    }

    /// Set power profile
    fn set_profile(&mut self, profile: PowerProfile) -> Result<(), Error> {
        self.set_profile(profile)
    }

    /// Get current profile
    fn current_profile(&self) -> PowerProfile {
        self.current_profile()
    }

    /// Acquire wake lock
    fn acquire_wake_lock(
        &mut self,
        owner: String,
        lock_type: WakeLockType,
        reason: String,
    ) -> Result<WakeLock, Error> {
        self.acquire_wake_lock(owner, lock_type, reason)
    }

    /// Release wake lock
    fn release_wake_lock(&mut self, id: String) -> Result<(), Error> {
        self.release_wake_lock(id)
    }

    /// Get battery usage breakdown
    fn battery_usage(&self) -> BatteryUsage {
        self.battery_usage()
    }

    /// Set app power hint
    fn set_app_hint(&mut self, app_id: String, hint: PowerHint) -> Result<(), Error> {
        self.set_app_hint(app_id, hint)
    }

    /// Battery percentage changed signal
    #[dbus(signal)]
    fn battery_percentage_changed(&self, percentage: u8);

    /// Power profile changed signal
    #[dbus(signal)]
    fn profile_changed(&self, profile: PowerProfile);

    /// Low battery warning signal
    #[dbus(signal)]
    fn low_battery(&self, percentage: u8);
}
```

## Configuration

```toml
# /etc/rustica/power.conf
[general]
# Default power profile
default_profile = "balanced"

# Enable adaptive power management
adaptive_mode = true

[battery]
# Low battery threshold (%)
low_battery_threshold = 20

# Critical battery threshold (%)
critical_battery_threshold = 5

# Show battery percentage in status bar
show_percentage = true

# Show battery time remaining
show_time_remaining = true

[profiles.performance]
# CPU governor
cpu_governor = "performance"

# Display brightness (%)
display_brightness = 100

# Disable throttling
throttle_factor = 1.0

[profiles.balanced]
# CPU governor
cpu_governor = "schedutil"

# Auto display brightness
auto_brightness = true

# Minimal throttling
throttle_factor = 0.9

[profiles.power_saver]
# CPU governor
cpu_governor = "powersave"

# Display brightness (%)
display_brightness = 60

# Moderate throttling
throttle_factor = 0.7

[profiles.battery_saver]
# CPU governor
cpu_governor = "powersave"

# Display brightness (%)
display_brightness = 40

# Aggressive throttling
throttle_factor = 0.5

# Suspend background apps
suspend_background = true

[throttling]
# Enable app throttling
enabled = true

# Background app throttle factor
background_throttle = 0.5

# Minimum CPU quota for any app (%)
min_cpu_quota = 5

[wake_locks]
# Maximum wake lock duration (seconds)
max_duration = 3600

# Auto-expire wake locks
auto_expire = true

[tracking]
# Track per-app energy usage
track_app_energy = true

# Update interval (seconds)
update_interval = 60

# Keep usage history (days)
history_days = 30
```

## Dependencies

```toml
[dependencies]
zbus = "4"
upower = "0.9"
libc = "0.2"
num_cpus = "1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"
log = "0.4"
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery_info() {
        let monitor = BatteryMonitor::new().unwrap();
        let info = monitor.info();

        assert!(info.percentage <= 100);
    }

    #[test]
    fn test_profile_switch() {
        let mut manager = PowerManager::new().unwrap();

        manager.set_profile(PowerProfile::Performance).unwrap();
        assert_eq!(manager.current_profile(), PowerProfile::Performance);

        manager.set_profile(PowerProfile::BatterySaver).unwrap();
        assert_eq!(manager.current_profile(), PowerProfile::BatterySaver);
    }

    #[test]
    fn test_wake_lock_acquire() {
        let mut manager = PowerManager::new().unwrap();

        let lock = manager.acquire_wake_lock(
            "test.app".to_string(),
            WakeLockType::System,
            "Testing".to_string(),
        ).unwrap();

        assert!(manager.wake_lock_manager.has_system_locks());

        manager.release_wake_lock(lock.id).unwrap();
        assert!(!manager.wake_lock_manager.has_system_locks());
    }

    #[test]
    fn test_throttling() {
        let mut limiter = AppLimiter::new();

        limiter.enable_throttling("test.app".to_string(), 0.5).unwrap();

        // Should be throttled to 50%
        assert_eq!(
            limiter.throttled_apps.get("test.app"),
            Some(&0.5)
        );
    }
}
```

## Future Enhancements

1. **Machine Learning**: Predict battery drain patterns
2. **Smart Charging**: Learn charging habits and optimize battery health
3. **App Energy Reports**: Weekly battery usage reports per app
4. **Battery Calibration**: Automated battery calibration procedure
5. **Thermal Management**: Integrate with thermal throttling
6. **Solar Charging**: Support for solar charging accessories
7. **Battery Replacement**: Detect and prompt for battery replacement

## Power Saving Tips

User-facing suggestions for extending battery life:

1. **Lower Display Brightness**: Display is the biggest power drain
2. **Enable Auto-Brightness**: Adjusts to ambient light conditions
3. **Use Power Saver Mode**: Reduces performance for battery life
4. **Close Unused Apps**: Background apps consume power
5. **Disable Location Services**: GPS is power-intensive
6. **Reduce Screen Timeout**: Turn off screen sooner when idle
7. **Enable Dark Mode**: On OLED displays, dark pixels use no power
8. **Limit Background Activity**: Restrict background refresh
9. **Use WiFi Instead of Mobile Data**: WiFi is more power-efficient
10. **Keep Battery Cool**: Heat degrades battery capacity
