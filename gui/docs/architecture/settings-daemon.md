# System Settings Daemon (rustica-settings-daemon) Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - System Settings Daemon
**Phase**: 5.2 - System Applications (Backend Service)

## Overview

The System Settings Daemon is a **background service** that manages all system settings via **D-Bus**, provides **persistent storage**, handles **hardware interactions**, and emits **change notifications**. It runs as a **systemd service** and serves as the **single source of truth** for all system settings.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         System Settings Daemon                          │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────────┐  │
│  │ D-Bus       │  │ Settings    │  │ Hardware    │  │ Event         │  │
│  │ Interface   │  │ Manager     │  │ Controllers │  │ Handlers      │  │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └───────┬───────┘  │
│         │                │                │                  │          │
│         ▼                ▼                ▼                  ▼          │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                      Settings Store                               │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │  │
│  │  │ Memory   │ │ Disk     │ │ Defaults │ │ Pending  │           │  │
│  │  │ Cache    │ │ Storage  │ │          │ │ Changes  │           │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘           │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                           │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    Change Notification                            │  │
│  │  ┌───────────┐ ┌─────────────┐ ┌────────────────┐                │  │
│  │  │ D-Bus     │ │ Inotify     │ │ Hardware       │                │  │
│  │  │ Signals   │ │ Watchers    │ │ Events         │                │  │
│  │  └───────────┘ └─────────────┘ └────────────────┘                │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

## Daemon Structure

```rust
pub struct SettingsDaemon {
    /// D-Bus connection
    dbus_connection: Connection,

    /// Settings store
    store: SettingsStore,

    /// Hardware controllers
    display_controller: Box<dyn DisplayController>,
    audio_controller: Box<dyn AudioController>,
    network_controller: Box<dyn NetworkController>,
    power_controller: Box<dyn PowerController>,
    input_controller: Box<dyn InputController>,

    /// Pending changes (not yet applied)
    pending_changes: HashMap<SettingKey, PendingChange>,

    /// Change listeners
    listeners: Vec<Box<dyn ChangeListener>>,

    /// Running
    running: bool,
}

pub struct PendingChange {
    value: SettingValue,
    timestamp: DateTime<Utc>,
    requires_restart: bool,
}

impl SettingsDaemon {
    pub fn new() -> Result<Self, Error> {
        // Connect to system bus
        let dbus_connection = Connection::system()?;

        // Load settings store
        let store = SettingsStore::load()?;

        // Initialize hardware controllers
        let display_controller = Box::new(DisplayController::new()?);
        let audio_controller = Box::new(AudioController::new()?);
        let network_controller = Box::new(NetworkController::new()?);
        let power_controller = Box::new(PowerController::new()?);
        let input_controller = Box::new(InputController::new()?);

        Ok(Self {
            dbus_connection,
            store,
            display_controller,
            audio_controller,
            network_controller,
            power_controller,
            input_controller,
            pending_changes: HashMap::new(),
            listeners: Vec::new(),
            running: false,
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        self.running = true;

        // Register D-Bus service
        self.register_dbus_service()?;

        // Load initial settings
        self.apply_initial_settings()?;

        // Main event loop
        while self.running {
            // Wait for D-Bus events
            self.dbus_connection.process_duration(Duration::from_millis(100))?;

            // Process pending changes
            self.process_pending_changes()?;

            // Handle hardware events
            self.handle_hardware_events()?;
        }

        Ok(())
    }
}
```

## Settings Store

```rust
pub struct SettingsStore {
    /// In-memory cache
    cache: HashMap<SettingKey, SettingValue>,

    /// Persistent storage path
    storage_path: PathBuf,

    /// Default values
    defaults: HashMap<SettingKey, SettingValue>,

    /// File watchers (for external changes)
    watchers: HashMap<SettingKey, INotifyWatcher>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum SettingKey {
    // Appearance
    ThemeMode,
    AccentColor,
    FontFamily,
    FontSize,
    IconSize,
    WindowDecorations,

    // Display
    Brightness,
    DisplayScale,
    RefreshRate,
    NightLightEnabled,
    NightLightTemperature,
    NightLightSchedule,

    // Network
    WiFiEnabled,
    WiFiKnownNetworks,
    HotspotEnabled,
    HotspotSSID,
    HotspotPassword,

    // Audio
    OutputVolume,
    OutputDevice,
    InputVolume,
    InputDevice,
    AudioBalance,
    SoundEffectsEnabled,

    // Power
    ScreenBlankTimeout,
    SuspendTimeout,
    PowerButtonAction,
    LidCloseAction,
    CriticalBatteryAction,
    BatterySaverEnabled,
    BatterySaverThreshold,

    // Input
    KeyboardLayout,
    KeyRepeatEnabled,
    KeyRepeatDelay,
    KeyRepeatInterval,
    PointerAcceleration,
    NaturalScrolling,
    TapToClick,

    // Date & Time
    AutomaticTime,
    NTPServers,
    ManualDateTime,
    Timezone,
    Hour24,
    ShowSeconds,
    DateFormat,

    // Accessibility
    HighContrast,
    ReduceAnimation,
    ScreenReader,
    ScreenReaderRate,
    ScreenReaderPitch,
    MagnifierEnabled,
    MagnifierScale,
    StickyKeys,
    SlowKeys,
}

#[derive(Clone)]
pub enum SettingValue {
    Bool(bool),
    Int(i64),
    UInt(u64),
    Float(f64),
    String(String),
    StringList(Vec<String>),
    Color(Color),
}

#[derive(Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl SettingsStore {
    pub fn load() -> Result<Self, Error> {
        let storage_path = dirs::config_dir()
            .ok_or(Error::NoConfigDir)?
            .join("rustica/settings.toml");

        // Create default storage if not exists
        if !storage_path.exists() {
            Self::create_default_storage(&storage_path)?;
        }

        // Load settings from TOML
        let config = std::fs::read_to_string(&storage_path)?;
        let parsed: toml::Value = toml::from_str(&config)?;

        // Parse into HashMap
        let mut cache = HashMap::new();
        if let Some(table) = parsed.as_table() {
            for (key, value) in table {
                if let Some(setting_key) = Self::parse_key(key) {
                    if let Some(setting_value) = Self::parse_value(value) {
                        cache.insert(setting_key, setting_value);
                    }
                }
            }
        }

        // Load defaults
        let defaults = Self::load_defaults();

        // Merge with defaults
        for (key, default_value) in &defaults {
            if !cache.contains_key(key) {
                cache.insert(key.clone(), default_value.clone());
            }
        }

        Ok(Self {
            cache,
            storage_path,
            defaults,
            watchers: HashMap::new(),
        })
    }

    pub fn get(&self, key: &SettingKey) -> SettingValue {
        self.cache.get(key)
            .or_else(|| self.defaults.get(key))
            .cloned()
            .unwrap_or(SettingValue::Bool(false))
    }

    pub fn set(&mut self, key: SettingKey, value: SettingValue) -> Result<(), Error> {
        // Update cache
        self.cache.insert(key.clone(), value);

        // Persist to disk
        self.persist()?;

        Ok(())
    }

    pub fn reset(&mut self, key: &SettingKey) -> Result<(), Error> {
        if let Some(default) = self.defaults.get(key) {
            self.cache.insert(key.clone(), default.clone());
            self.persist()?;
        }

        Ok(())
    }

    fn persist(&self) -> Result<(), Error> {
        // Convert cache to TOML
        let mut table = toml::value::Table::new();

        for (key, value) in &self.cache {
            let key_str = Self::key_to_string(key);
            let toml_value = Self::value_to_toml(value)?;
            table.insert(key_str, toml_value);
        }

        // Write to file atomically
        let content = toml::to_string_pretty(&table)?;
        let temp_path = self.storage_path.with_extension("tmp");

        std::fs::write(&temp_path, content)?;
        std::fs::rename(&temp_path, &self.storage_path)?;

        Ok(())
    }

    fn load_defaults() -> HashMap<SettingKey, SettingValue> {
        let mut defaults = HashMap::new();

        // Appearance defaults
        defaults.insert(
            SettingKey::ThemeMode,
            SettingValue::String("dark".into())
        );
        defaults.insert(
            SettingKey::AccentColor,
            SettingValue::Color(Color { r: 0x7B, g: 0x1F, b: 0xA2, a: 255 })
        );
        defaults.insert(
            SettingKey::FontFamily,
            SettingValue::String("System UI".into())
        );
        defaults.insert(
            SettingKey::FontSize,
            SettingValue::Int(12)
        );
        defaults.insert(
            SettingKey::IconSize,
            SettingValue::Int(24)
        );

        // Display defaults
        defaults.insert(
            SettingKey::Brightness,
            SettingValue::UInt(80)
        );
        defaults.insert(
            SettingKey::DisplayScale,
            SettingValue::Float(1.0)
        );
        defaults.insert(
            SettingKey::NightLightEnabled,
            SettingValue::Bool(false)
        );
        defaults.insert(
            SettingKey::NightLightTemperature,
            SettingValue::UInt(4500)
        );

        // Network defaults
        defaults.insert(
            SettingKey::WiFiEnabled,
            SettingValue::Bool(true)
        );
        defaults.insert(
            SettingKey::HotspotEnabled,
            SettingValue::Bool(false)
        );

        // Audio defaults
        defaults.insert(
            SettingKey::OutputVolume,
            SettingValue::UInt(100)
        );
        defaults.insert(
            SettingKey::InputVolume,
            SettingValue::UInt(80)
        );
        defaults.insert(
            SettingKey::SoundEffectsEnabled,
            SettingValue::Bool(true)
        );

        // Power defaults
        defaults.insert(
            SettingKey::ScreenBlankTimeout,
            SettingValue::UInt(600)  // 10 minutes
        );
        defaults.insert(
            SettingKey::SuspendTimeout,
            SettingValue::UInt(1800) // 30 minutes
        );
        defaults.insert(
            SettingKey::PowerButtonAction,
            SettingValue::String("suspend".into())
        );

        // Input defaults
        defaults.insert(
            SettingKey::KeyboardLayout,
            SettingValue::String("us".into())
        );
        defaults.insert(
            SettingKey::KeyRepeatEnabled,
            SettingValue::Bool(true)
        );
        defaults.insert(
            SettingKey::KeyRepeatDelay,
            SettingValue::UInt(500)
        );
        defaults.insert(
            SettingKey::KeyRepeatInterval,
            SettingValue::UInt(50)
        );

        // Date & Time defaults
        defaults.insert(
            SettingKey::AutomaticTime,
            SettingValue::Bool(true)
        );
        defaults.insert(
            SettingKey::Timezone,
            SettingValue::String("America/New_York".into())
        );
        defaults.insert(
            SettingKey::Hour24,
            SettingValue::Bool(true)
        );
        defaults.insert(
            SettingKey::ShowSeconds,
            SettingValue::Bool(false)
        );

        // Accessibility defaults
        defaults.insert(
            SettingKey::HighContrast,
            SettingValue::Bool(false)
        );
        defaults.insert(
            SettingKey::ReduceAnimation,
            SettingValue::Bool(false)
        );
        defaults.insert(
            SettingKey::ScreenReader,
            SettingValue::Bool(false)
        );

        defaults
    }
}
```

## D-Bus Interface

```rust
// D-Bus service name: org.rustica.SettingsDaemon
// Object path: /org/rustica/SettingsDaemon

#[dbus_interface(name = "org.rustica.SettingsDaemon")]
impl SettingsDaemon {
    /// Get all settings
    fn get_all_settings(&self) -> HashMap<String, String> {
        self.store.cache.iter()
            .map(|(k, v)| (Self::key_to_string(k), Self::value_to_string(v)))
            .collect()
    }

    /// Get a single setting
    fn get_setting(&self, key: String) -> String {
        if let Ok(setting_key) = Self::parse_key_string(&key) {
            Self::value_to_string(self.store.get(&setting_key))
        } else {
            String::new()
        }
    }

    /// Set a setting
    fn set_setting(&mut self, key: String, value: String) -> Result<(), Error> {
        let setting_key = Self::parse_key_string(&key)?;
        let setting_value = Self::parse_value_string(&value)?;

        // Add to pending
        self.pending_changes.insert(
            setting_key.clone(),
            PendingChange {
                value: setting_value.clone(),
                timestamp: Utc::now(),
                requires_restart: Self::requires_restart(&setting_key),
            }
        );

        // Apply immediately if no restart needed
        if !Self::requires_restart(&setting_key) {
            self.apply_setting(&setting_key, &setting_value)?;
        }

        Ok(())
    }

    /// Reset a setting to default
    fn reset_setting(&mut self, key: String) -> Result<(), Error> {
        let setting_key = Self::parse_key_string(&key)?;
        self.store.reset(&setting_key)?;

        // Emit change signal
        self.emit_change(&setting_key, &self.store.get(&setting_key));

        Ok(())
    }

    /// Apply all pending changes
    fn apply_changes(&mut self) -> Result<(), Error> {
        for (key, change) in &self.pending_changes {
            self.apply_setting(key, &change.value)?;
        }

        self.pending_changes.clear();

        // Emit settings applied signal
        self.emit_settings_applied();

        Ok(())
    }

    /// Revert all pending changes
    fn revert_changes(&mut self) -> Result<(), Error> {
        for key in self.pending_changes.keys() {
            // Revert to stored value
            let stored = self.store.get(key);
            self.apply_setting(key, &stored)?;
        }

        self.pending_changes.clear();

        Ok(())
    }

    /// Signal: Setting changed
    #[dbus_interface(signal)]
    fn setting_changed(&self, key: String, value: String);

    /// Signal: All settings applied
    #[dbus_interface(signal)]
    fn settings_applied(&self);
}
```

## Hardware Controllers

### Display Controller

```rust
pub trait DisplayController: Send + Sync {
    /// Set brightness (0-100)
    fn set_brightness(&self, brightness: u8) -> Result<(), Error>;

    /// Get brightness
    fn get_brightness(&self) -> Result<u8, Error>;

    /// Set scale factor
    fn set_scale(&self, scale: f32) -> Result<(), Error>;

    /// Set refresh rate
    fn set_refresh_rate(&self, output: &str, rate: u32) -> Result<(), Error>;

    /// Enable night light
    fn set_night_light(&self, enabled: bool, temperature: u16) -> Result<(), Error>;

    /// Get display info
    fn get_displays(&self) -> Result<Vec<DisplayInfo>, Error>;
}

pub struct DisplayControllerImpl {
    drm_devices: Vec<DrmDevice>,
    backlights: Vec<BacklightDevice>,
}

impl DisplayController for DisplayControllerImpl {
    fn set_brightness(&self, brightness: u8) -> Result<(), Error> {
        for backlight in &self.backlights {
            backlight.set_brightness(brightness)?;
        }
        Ok(())
    }

    fn set_night_light(&self, enabled: bool, temperature: u16) -> Result<(), Error> {
        // Adjust gamma ramps for all displays
        for drm in &self.drm_devices {
            drm.set_gamma_ramp(enabled, temperature)?;
        }
        Ok(())
    }

    fn get_displays(&self) -> Result<Vec<DisplayInfo>, Error> {
        let mut displays = Vec::new();

        for drm in &self.drm_devices {
            for connector in &drm.connectors() {
                displays.push(DisplayInfo {
                    id: connector.id.clone(),
                    name: connector.name.clone(),
                    resolution: (connector.mode.hdisplay, connector.mode.vdisplay),
                    physical_size: (connector.width_mm, connector.height_mm),
                    refresh_rates: connector.modes.iter().map(|m| m.vrefresh).collect(),
                    current_refresh_rate: connector.mode.vrefresh,
                    primary: connector.is_primary,
                    enabled: connector.is_connected,
                    position: (connector.x, connector.y),
                });
            }
        }

        Ok(displays)
    }
}
```

### Audio Controller

```rust
pub trait AudioController: Send + Sync {
    /// Set output volume (0-100)
    fn set_output_volume(&self, volume: u8) -> Result<(), Error>;

    /// Get output volume
    fn get_output_volume(&self) -> Result<u8, Error>;

    /// Set output device
    fn set_output_device(&self, device: &str) -> Result<(), Error>;

    /// Set input volume
    fn set_input_volume(&self, volume: u8) -> Result<(), Error>;

    /// Set audio balance (-1.0 to 1.0)
    fn set_balance(&self, balance: f32) -> Result<(), Error>;

    /// Get available devices
    fn get_devices(&self) -> Result<(Vec<AudioDevice>, Vec<AudioDevice>), Error>;
}

pub struct AudioControllerImpl {
    pulse_context: PulseAudioContext,
}

impl AudioController for AudioControllerImpl {
    fn set_output_volume(&self, volume: u8) -> Result<(), Error> {
        // Convert to PA volume (0-65536)
        let pa_volume = (volume as f32 / 100.0 * 65536.0) as u32;
        self.pulse_context.set_output_volume(pa_volume)?;
        Ok(())
    }

    fn set_balance(&self, balance: f32) -> Result<(), Error> {
        // Clamp to -1.0 to 1.0
        let balance = balance.clamp(-1.0, 1.0);

        // Calculate left/right balance
        let (left, right) = if balance < 0.0 {
            (1.0, 1.0 + balance)
        } else {
            (1.0 - balance, 1.0)
        };

        self.pulse_context.set_channel_volumes(left, right)?;
        Ok(())
    }

    fn get_devices(&self) -> Result<(Vec<AudioDevice>, Vec<AudioDevice>), Error> {
        let sinks = self.pulse_context.get_sinks()?;
        let sources = self.pulse_context.get_sources()?;

        let output_devices = sinks.into_iter().map(|s| AudioDevice {
            id: s.name.clone(),
            name: s.description,
            icon: s.icon_name,
            ports: s.ports.into_iter().map(|p| AudioPort {
                id: p.name,
                name: p.description,
                available: p.available,
            }).collect(),
        }).collect();

        let input_devices = sources.into_iter().map(|s| AudioDevice {
            id: s.name.clone(),
            name: s.description,
            icon: s.icon_name,
            ports: s.ports.into_iter().map(|p| AudioPort {
                id: p.name,
                name: p.description,
                available: p.available,
            }).collect(),
        }).collect();

        Ok((output_devices, input_devices))
    }
}
```

### Network Controller

```rust
pub trait NetworkController: Send + Sync {
    /// Enable/disable WiFi
    fn set_wifi_enabled(&self, enabled: bool) -> Result<(), Error>;

    /// Connect to network
    fn connect(&self, ssid: &str, password: Option<&str>) -> Result<(), Error>;

    /// Disconnect
    fn disconnect(&self) -> Result<(), Error>;

    /// Get available networks
    fn get_networks(&self) -> Result<Vec<Network>, Error>;

    /// Get connection status
    fn get_connection_status(&self) -> Result<Option<NetworkConnection>, Error>;

    /// Enable hotspot
    fn set_hotspot(&self, enabled: bool, ssid: &str, password: &str) -> Result<(), Error>;
}

pub struct NetworkControllerImpl {
    nm_client: NetworkManagerClient,
}

impl NetworkController for NetworkControllerImpl {
    fn set_wifi_enabled(&self, enabled: bool) -> Result<(), Error> {
        self.nm_client.set_wireless_enabled(enabled)?;
        Ok(())
    }

    fn connect(&self, ssid: &str, password: Option<&str>) -> Result<(), Error> {
        // Find network
        let network = self.nm_client.find_network(ssid)?;

        // Connect with password if secured
        if network.secured {
            if let Some(pwd) = password {
                self.nm_client.connect_with_password(&network.path, pwd)?;
            } else {
                return Err(Error::PasswordRequired);
            }
        } else {
            self.nm_client.connect(&network.path)?;
        }

        Ok(())
    }

    fn get_networks(&self) -> Result<Vec<Network>, Error> {
        let aps = self.nm_client.get_access_points()?;

        let networks = aps.into_iter().map(|ap| Network {
            ssid: ap.ssid,
            strength: ap.strength,
            secured: ap.flags.contains(NetworkFlags::PRIVACY),
            known: ap.is_known,
        }).collect();

        Ok(networks)
    }

    fn set_hotspot(&self, enabled: bool, ssid: &str, password: &str) -> Result<(), Error> {
        if enabled {
            self.nm_client.start_hotspot(ssid, password)?;
        } else {
            self.nm_client.stop_hotspot()?;
        }
        Ok(())
    }
}
```

### Power Controller

```rust
pub trait PowerController: Send + Sync {
    /// Get battery percentage
    fn get_battery_percentage(&self) -> Result<u8, Error>;

    /// Get power state
    fn get_power_state(&self) -> Result<PowerState, Error>;

    /// Suspend system
    fn suspend(&self) -> Result<(), Error>;

    /// Hibernate system
    fn hibernate(&self) -> Result<(), Error>;

    /// Shutdown system
    fn shutdown(&self) -> Result<(), Error>;

    /// Set screen blank timeout (seconds)
    fn set_blank_timeout(&self, timeout: u32) -> Result<(), Error>;

    /// Set suspend timeout (seconds)
    fn set_suspend_timeout(&self, timeout: u32) -> Result<(), Error>;

    /// Enable battery saver
    fn set_battery_saver(&self, enabled: bool, threshold: u8) -> Result<(), Error>;
}

pub struct PowerControllerImpl {
    upower_client: UPowerClient,
    logind: LoginManager,
}

impl PowerController for PowerControllerImpl {
    fn get_battery_percentage(&self) -> Result<u8, Error> {
        Ok(self.upower_client.get_battery_percentage()?)
    }

    fn get_power_state(&self) -> Result<PowerState, Error> {
        let on_battery = self.upower_client.on_battery()?;
        let charging = self.upower_client.battery_charging()?;
        let fully_charged = self.upower_client.battery_fully_charged()?;

        Ok(if on_battery && !charging {
            PowerState::Discharging
        } else if charging {
            PowerState::Charging
        } else if fully_charged {
            PowerState::FullyCharged
        } else {
            PowerState::AC
        })
    }

    fn suspend(&self) -> Result<(), Error> {
        self.logind.suspend()?;
        Ok(())
    }

    fn set_blank_timeout(&self, timeout: u32) -> Result<(), Error> {
        // Configure via org.freedesktop.ScreenSaver
        Ok(())
    }

    fn set_battery_saver(&self, enabled: bool, threshold: u8) -> Result<(), Error> {
        // Enable power saving modes
        if enabled {
            // Reduce CPU frequency
            self.set_cpu_governor("powersave")?;

            // Reduce brightness
            self.set_max_brightness(50)?;

            // Disable some services
        }

        Ok(())
    }
}
```

### Input Controller

```rust
pub trait InputController: Send + Sync {
    /// Set keyboard layout
    fn set_keyboard_layout(&self, layout: &str) -> Result<(), Error>;

    /// Set key repeat
    fn set_key_repeat(&self, enabled: bool, delay: u32, interval: u32) -> Result<(), Error>;

    /// Set pointer acceleration
    fn set_pointer_acceleration(&self, accel: f32) -> Result<(), Error>;

    /// Set natural scrolling
    fn set_natural_scrolling(&self, enabled: bool) -> Result<(), Error>;

    /// Set tap to click
    fn set_tap_to_click(&self, enabled: bool) -> Result<(), Error>;

    /// Get available keyboard layouts
    fn get_layouts(&self) -> Result<Vec<KeyboardLayout>, Error>;
}

pub struct InputControllerImpl {
    libinput: LibinputContext,
    xkb: XkbContext,
}

impl InputController for InputControllerImpl {
    fn set_keyboard_layout(&self, layout: &str) -> Result<(), Error> {
        // Set via XKB and Wayland virtual keyboard
        self.xkb.set_layout(layout)?;
        Ok(())
    }

    fn set_key_repeat(&self, enabled: bool, delay: u32, interval: u32) -> Result<(), Error> {
        // Configure via Wayland text input protocol
        if enabled {
            self.libinput.set_key_repeat_delay(delay)?;
            self.libinput.set_key_repeat_rate(interval)?;
        } else {
            self.libinput.disable_key_repeat()?;
        }
        Ok(())
    }

    fn set_pointer_acceleration(&self, accel: f32) -> Result<(), Error> {
        // Set via libinput
        for device in self.libinput.pointers() {
            device.set_accel_speed(accel)?;
        }
        Ok(())
    }

    fn set_natural_scrolling(&self, enabled: bool) -> Result<(), Error> {
        for device in self.libinput.touchpads() {
            device.set_natural_scrolling(enabled)?;
        }
        Ok(())
    }

    fn set_tap_to_click(&self, enabled: bool) -> Result<(), Error> {
        for device in self.libinput.touchpads() {
            device.set_tap_to_click(enabled)?;
        }
        Ok(())
    }

    fn get_layouts(&self) -> Result<Vec<KeyboardLayout>, Error> {
        Ok(self.xkb.available_layouts()?)
    }
}
```

## Change Notification System

```rust
pub trait ChangeListener: Send + Sync {
    fn on_setting_changed(&self, key: &SettingKey, value: &SettingValue);
    fn on_settings_applied(&self);
}

impl SettingsDaemon {
    pub fn register_listener(&mut self, listener: Box<dyn ChangeListener>) {
        self.listeners.push(listener);
    }

    fn emit_change(&self, key: &SettingKey, value: &SettingValue) {
        // D-Bus signal
        self.setting_changed(
            Self::key_to_string(key),
            Self::value_to_string(value)
        );

        // Internal listeners
        for listener in &self.listeners {
            listener.on_setting_changed(key, value);
        }
    }

    fn emit_settings_applied(&self) {
        // D-Bus signal
        self.settings_applied();

        // Internal listeners
        for listener in &self.listeners {
            listener.on_settings_applied();
        }
    }
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-settings-daemon/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── daemon.rs
│       ├── store.rs
│       ├── dbus/
│       │   ├── interface.rs
│       │   └── service.rs
│       ├── controllers/
│       │   ├── mod.rs
│       │   ├── display.rs
│       │   ├── audio.rs
│       │   ├── network.rs
│       │   ├── power.rs
│       │   └── input.rs
│       └── listeners.rs
└── systemd/
    └── rustica-settings-daemon.service
```

## Systemd Service

```ini
[Unit]
Description=Rustica Settings Daemon
Documentation=man:rustica-settings-daemon(8)
After=dbus.service network.target
Wants=dbus.service

[Service]
Type=dbus
BusName=org.rustica.SettingsDaemon
ExecStart=/usr/bin/rustica-settings-daemon
Restart=on-failure
RestartSec=5

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/etc/rustica /var/lib/rustica

# Performance
OOMScoreAdjust=-500

[Install]
WantedBy=multi-user.target
```

## Dependencies

```toml
[package]
name = "rustica-settings-daemon"
version = "1.0.0"
edition = "2021"

[dependencies]
# D-Bus
zbus = "3.0"
zvariant = "3.0"

# Serialization
serde = "1.0"
toml = "0.8"

# Hardware
libdrm = "0.1"
pulseaudio = "0.1"
network-manager = "0.1"
upower_dbus = "0.1"
libinput = "0.1"

# Input
xkbcommon = "0.5"

# Time
chrono = "0.4"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# File watching
notify = "5.0"

# Systemd
systemd = "0.10"

# XDG
dirs = "5.0"
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Settings read | <10ms | Get operation |
| Settings write | <50ms | Set + persist |
| Apply changes | <500ms | All hardware |
| Memory | <20MB | Daemon usage |
| Startup time | <1s | Service active |

## Success Criteria

- [ ] All hardware controllers functional
- [ ] D-Bus interface complete
- [ ] Settings persist correctly
- [ ] Change notifications work
- [ ] Systemd service runs correctly
- [ ] Performance targets met
- [ ] All settings apply immediately

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## Timeline

- Week 1: Daemon structure + D-Bus interface + settings store
- Week 2: Hardware controllers (display, audio)
- Week 3: Hardware controllers (network, power, input)
- Week 4: Change notifications + testing + systemd integration

**Total**: 4 weeks
