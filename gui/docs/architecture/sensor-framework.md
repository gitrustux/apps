# Phase 8.4: Sensor Integration Framework (rustica-sensors)

## Overview

**Component**: rustica-sensors
**Purpose**: Unified sensor access layer for mobile/touch devices
**Language**: Rust
**Dependencies**: libc, dbus (zbus), iio-sys (for Industrial I/O)

## Goals

1. **Unified API**: Single interface for all sensor types
2. **Power Efficient**: Minimize battery drain from sensor polling
3. **Privacy**: Clear permission model and data usage indicators
4. **Accessibility**: Sensor data for accessibility features
5. **Fallback**: Graceful degradation when sensors unavailable

## Supported Sensors

### Motion Sensors
- **Accelerometer**: 3-axis acceleration (m/s²)
- **Gyroscope**: 3-axis angular velocity (rad/s)
- **Magnetometer**: 3-axis magnetic field (µT)
- **Gravity**: Gravity vector (m/s²)
- **Linear Acceleration**: Acceleration without gravity (m/s²)
- **Rotation Vector**: Device orientation (quaternion)

### Environmental Sensors
- **Ambient Light**: Illuminance (lux)
- **Proximity**: Object distance (binary or cm)
- **Barometer**: Atmospheric pressure (hPa)
- **Temperature**: Ambient temperature (°C)
- **Humidity**: Relative humidity (%)
- **UV Index**: UV radiation level

### Position Sensors
- **GPS**: Location (lat, long, altitude)
- **Network Location**: WiFi/cell-based location

### Biometric Sensors
- **Fingerprint**: Fingerprint scanner
- **Face Unlock**: Front camera face recognition

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Applications                              │
│        (request sensor data via D-Bus / library)             │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  rustica-sensors                             │
│                  (Sensor Daemon)                             │
├─────────────────────────────────────────────────────────────┤
│  SensorManager      │  PermissionManager  │  PowerController│
│  - Device discovery │  - Permission check │  - Batch data    │
│  - Data streaming   │  - Permission UI    │  - Throttle rate │
│  - Event filtering  │  - Audit log        │  - Auto-suspend  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                  Hardware Abstraction                        │
│              (Linux IIO, evdev, hidraw)                      │
└─────────────────────────────────────────────────────────────┘
```

## Core Data Structures

```rust
/// Sensor identification and metadata
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SensorId {
    pub type_: SensorType,
    pub name: String,
    pub device_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SensorType {
    Accelerometer,
    Gyroscope,
    Magnetometer,
    Gravity,
    LinearAcceleration,
    RotationVector,
    AmbientLight,
    Proximity,
    Barometer,
    Temperature,
    Humidity,
    UvIndex,
    Gps,
    NetworkLocation,
    Fingerprint,
    FaceUnlock,
}

/// Sensor capability information
#[derive(Debug, Clone)]
pub struct SensorInfo {
    pub id: SensorId,
    pub vendor: String,
    pub version: u32,
    pub type_: SensorType,
    pub max_range: f64,       // Maximum range of sensor
    pub resolution: f64,      // Sensor resolution
    pub power: f64,           // Power consumption (mA)
    pub min_delay: u32,       // Minimum sampling period (µs)
    pub max_delay: u32,       // Maximum sampling period (µs)
    pub fifo_enabled: bool,   // FIFO buffering support
    pub wake_up: bool,        // Wake-up sensor
}

/// Sensor reading with timestamp
#[derive(Debug, Clone)]
pub struct SensorReading {
    pub sensor_id: SensorId,
    pub timestamp: DateTime<Utc>,
    pub accuracy: Accuracy,
    pub values: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accuracy {
    Unreliable,
    Low,
    Medium,
    High,
}

/// Sensor event stream
pub struct SensorEvent {
    pub reading: SensorReading,
    pub sequence_number: u64,
}
```

## Sensor Manager

```rust
pub struct SensorManager {
    sensors: HashMap<SensorId, SensorHandle>,
    active_streams: HashMap<SensorId, SensorStream>,
    permission_manager: PermissionManager,
    power_controller: PowerController,
    event_dispatcher: EventDispatcher,
}

impl SensorManager {
    pub fn new() -> Result<Self, Error> {
        let mut sensors = HashMap::new();

        // Discover available sensors
        sensors.extend(Accelerometer::discover()?);
        sensors.extend(Gyroscope::discover()?);
        sensors.extend(Magnetometer::discover()?);
        sensors.extend(AmbientLight::discover()?);
        sensors.extend(Proximity::discover()?);
        sensors.extend(Barometer::discover()?);

        Ok(Self {
            sensors,
            active_streams: HashMap::new(),
            permission_manager: PermissionManager::new(),
            power_controller: PowerController::new(),
            event_dispatcher: EventDispatcher::new(),
        })
    }

    /// List all available sensors
    pub fn list_sensors(&self) -> Vec<SensorInfo> {
        self.sensors.values()
            .map(|s| s.info())
            .collect()
    }

    /// Get information about specific sensor
    pub fn get_sensor_info(&self, id: &SensorId) -> Option<SensorInfo> {
        self.sensors.get(id).map(|s| s.info())
    }

    /// Start streaming sensor data
    pub fn start_stream(
        &mut self,
        id: SensorId,
        client: ClientId,
        sampling_period: Duration,
        batching: bool,
    ) -> Result<Receiver<SensorEvent>, Error> {
        // Check permissions
        self.permission_manager.check_permission(client, &id)?;

        // Get sensor handle
        let sensor = self.sensors.get(&id)
            .ok_or(Error::SensorNotFound)?;

        // Configure power management
        self.power_controller.activate_sensor(id, sampling_period);

        // Create stream
        let (sender, receiver) = channel();
        let stream = SensorStream {
            sender: sender.clone(),
            sampling_period,
            batching,
        };

        // Start reading from sensor
        sensor.start_reading(sampling_period, move |event| {
            let _ = sender.send(event);
        })?;

        self.active_streams.insert(id, stream);

        Ok(receiver)
    }

    /// Stop streaming sensor data
    pub fn stop_stream(&mut self, id: &SensorId, client: ClientId) -> Result<(), Error> {
        // Verify ownership
        let stream = self.active_streams.get(id)
            .ok_or(Error::NotStreaming)?;

        if stream.client != client {
            return Err(Error::PermissionDenied);
        }

        // Stop sensor
        if let Some(sensor) = self.sensors.get(id) {
            sensor.stop_reading()?;
        }

        // Update power management
        self.power_controller.deactivate_sensor(id);

        self.active_streams.remove(id);

        Ok(())
    }

    /// Read sensor once (single-shot)
    pub fn read_once(&mut self, id: &SensorId, client: ClientId) -> Result<SensorReading, Error> {
        // Check permissions
        self.permission_manager.check_permission(client, id)?;

        let sensor = self.sensors.get(id)
            .ok_or(Error::SensorNotFound)?;

        sensor.read_single()
    }

    /// Register for sensor change events (e.g., significant motion)
    pub fn register_trigger(
        &mut self,
        id: &SensorId,
        client: ClientId,
        trigger_type: TriggerType,
        callback: Box<dyn Fn(SensorReading)>,
    ) -> Result<TriggerHandle, Error> {
        self.permission_manager.check_permission(client, id)?;

        let sensor = self.sensors.get(id)
            .ok_or(Error::SensorNotFound)?;

        sensor.register_trigger(trigger_type, callback)
    }
}

/// Client identification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClientId {
    pub app_id: String,
    pub pid: u32,
}

/// Trigger types for event-based sensors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerType {
    SignificantMotion,
    WakeUp,
    FaceDown,
    FaceUp,
    Pickup,
    Shake,
}
```

## Sensor Implementations

### Accelerometer

```rust
pub struct Accelerometer {
    handle: SensorHandle,
    calibration: CalibrationData,
}

impl Accelerometer {
    pub fn discover() -> Result<HashMap<SensorId, Self>, Error> {
        let mut sensors = HashMap::new();

        // Search IIO devices for accelerometer
        for device in iio_discover("accel")? {
            let info = device.read_info()?;

            let id = SensorId {
                type_: SensorType::Accelerometer,
                name: info.name.clone(),
                device_path: device.path(),
            };

            let handle = SensorHandle::new(device)?;
            let calibration = Self::load_calibration(&id)?;

            sensors.insert(id, Self {
                handle,
                calibration,
            });
        }

        Ok(sensors)
    }

    fn load_calibration(id: &SensorId) -> Result<CalibrationData, Error> {
        // Load calibration from /var/lib/rustica/sensors/
        let cal_path = format!(
            "/var/lib/rustica/sensors/{}_calibration.json",
            id.name
        );

        if let Ok(file) = std::fs::File::open(&cal_path) {
            Ok(serde_json::from_reader(file)?)
        } else {
            // Default calibration
            Ok(CalibrationData::default())
        }
    }

    /// Get gravity-compensated acceleration
    pub fn read_linear_acceleration(&self) -> Result<[f64; 3], Error> {
        let raw = self.read_raw()?;

        // Subtract gravity (approximate at 9.81 m/s² down)
        let gravity = self.estimate_gravity()?;

        Ok([
            raw[0] - gravity[0],
            raw[1] - gravity[1],
            raw[2] - gravity[2],
        ])
    }

    /// Estimate gravity vector using low-pass filter
    fn estimate_gravity(&self) -> Result<[f64; 3], Error> {
        // Alpha = t / (t + dT)
        // where t = low-pass filter time constant
        //       dT = sampling period
        let alpha = 0.8;

        let raw = self.read_raw()?;

        // Apply low-pass filter
        // gravity[i] = alpha * gravity[i] + (1 - alpha) * raw[i]
        // This is simplified; real implementation tracks state
        Ok([
            raw[0] * (1.0 - alpha),
            raw[1] * (1.0 - alpha),
            raw[2] * (1.0 - alpha) + 9.81 * alpha,  // Z is up
        ])
    }
}

impl Sensor for Accelerometer {
    fn info(&self) -> SensorInfo {
        self.handle.info()
    }

    fn read_raw(&self) -> Result<Vec<f64>, Error> {
        self.handle.read_raw()
    }

    fn read_single(&self) -> Result<SensorReading, Error> {
        let values = self.read_raw()?;
        let calibrated = self.apply_calibration(&values);

        Ok(SensorReading {
            sensor_id: self.info().id,
            timestamp: Utc::now(),
            accuracy: Accuracy::High,
            values: calibrated,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationData {
    pub offset: [f64; 3],
    pub scale: [f64; 3],
    pub temperature_coefficients: [[f64; 3]; 2],
}

impl Default for CalibrationData {
    fn default() -> Self {
        Self {
            offset: [0.0; 3],
            scale: [1.0; 3],
            temperature_coefficients: [[0.0; 3]; 2],
        }
    }
}
```

### Ambient Light Sensor

```rust
pub struct AmbientLight {
    handle: SensorHandle,
}

impl AmbientLight {
    pub fn discover() -> Result<HashMap<SensorId, Self>, Error> {
        let mut sensors = HashMap::new();

        for device in iio_discover("illuminance")? {
            let info = device.read_info()?;

            let id = SensorId {
                type_: SensorType::AmbientLight,
                name: info.name.clone(),
                device_path: device.path(),
            };

            let handle = SensorHandle::new(device)?;

            sensors.insert(id, Self { handle });
        }

        Ok(sensors)
    }

    /// Get current illuminance in lux
    pub fn read_illuminance(&self) -> Result<f64, Error> {
        let reading = self.read_single()?;
        Ok(reading.values[0])
    }

    /// Get recommended screen brightness (0-100%)
    pub fn recommended_brightness(&self) -> Result<u8, Error> {
        let lux = self.read_illuminance()?;

        // HLG (Hybrid Log-Gamma) curve mapping lux to brightness
        let brightness = if lux < 10.0 {
            // Very dark
            10
        } else if lux < 100.0 {
            // Indoor
            30
        } else if lux < 1000.0 {
            // Bright indoor
            60
        } else if lux < 10000.0 {
            // Outdoor shade
            80
        } else {
            // Direct sunlight
            100
        };

        Ok(brightness)
    }
}
```

### Proximity Sensor

```rust
pub struct Proximity {
    handle: SensorHandle,
    max_range: f64,
}

impl Proximity {
    pub fn discover() -> Result<HashMap<SensorId, Self>, Error> {
        let mut sensors = HashMap::new();

        for device in iio_discover("distance")? {
            let info = device.read_info()?;
            let max_range = info.max_range;

            let id = SensorId {
                type_: SensorType::Proximity,
                name: info.name.clone(),
                device_path: device.path(),
            };

            let handle = SensorHandle::new(device)?;

            sensors.insert(id, Self { handle, max_range });
        }

        Ok(sensors)
    }

    /// Check if object is near (binary proximity)
    pub fn is_near(&self) -> Result<bool, Error> {
        let reading = self.read_single()?;
        Ok(reading.values[0] < self.max_range * 0.5)
    }

    /// Get distance in cm (if supported)
    pub fn read_distance(&self) -> Result<Option<f64>, Error> {
        let reading = self.read_single()?;

        if reading.values[0] >= self.max_range {
            Ok(None)
        } else {
            Ok(Some(reading.values[0]))
        }
    }
}
```

## Permission Manager

```rust
pub struct PermissionManager {
    permissions: HashMap<ClientId, HashMap<SensorType, PermissionState>>,
    audit_log: AuditLog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionState {
    NotAsked,
    Denied,
    Granted,
    GrantedOnce,  // One-time permission
}

impl PermissionManager {
    pub fn new() -> Self {
        Self {
            permissions: Self::load_permissions(),
            audit_log: AuditLog::new(),
        }
    }

    fn load_permissions() -> HashMap<ClientId, HashMap<SensorType, PermissionState>> {
        let path = "/var/lib/rustica/sensors/permissions.json";

        if let Ok(file) = std::fs::File::open(path) {
            serde_json::from_reader(file).unwrap_or_default()
        } else {
            HashMap::new()
        }
    }

    pub fn check_permission(&mut self, client: ClientId, sensor: &SensorId) -> Result<(), Error> {
        let client_perms = self.permissions.entry(client.clone())
            .or_insert_with(HashMap::new);

        let state = client_perms.entry(sensor.type_)
            .or_insert(PermissionState::NotAsked);

        match state {
            PermissionState::Granted => {
                self.audit_log.log_access(client, sensor.type_);
                Ok(())
            }
            PermissionState::Denied => Err(Error::PermissionDenied),
            PermissionState::NotAsked => {
                // Request permission
                self.request_permission(client, sensor.type_)?;
                Err(Error::PermissionPending)
            }
            PermissionState::GrantedOnce => {
                // Consume one-time permission
                *state = PermissionState::NotAsked;
                self.audit_log.log_access(client, sensor.type_);
                Ok(())
            }
        }
    }

    fn request_permission(&mut self, client: ClientId, sensor: SensorType) -> Result<(), Error> {
        // Show permission dialog to user
        let dialog = SensorPermissionDialog {
            app_name: client.app_id.clone(),
            sensor_type: sensor,
            rationale: Self::rationale_for(sensor),
        };

        let granted = dialog.show()?;

        let state = if granted {
            PermissionState::Granted
        } else {
            PermissionState::Denied
        };

        self.permissions
            .entry(client)
            .or_insert_with(HashMap::new)
            .insert(sensor, state);

        self.save_permissions();

        Ok(())
    }

    fn rationale_for(sensor: SensorType) -> String {
        match sensor {
            SensorType::Accelerometer => {
                "Used for screen rotation and shake gestures".to_string()
            }
            SensorType::Gyroscope => {
                "Used for motion-controlled gaming and VR".to_string()
            }
            SensorType::Gps => {
                "Used for location-based features and weather".to_string()
            }
            SensorType::Microphone => {
                "Used for voice commands and calls".to_string()
            }
            _ => "Required for this feature to function".to_string(),
        }
    }

    fn save_permissions(&self) {
        let path = "/var/lib/rustica/sensors/permissions.json";

        if let Ok(file) = std::fs::File::create(path) {
            serde_json::to_writer_pretty(file, &self.permissions).ok();
        }
    }

    pub fn revoke_permission(&mut self, client: &ClientId, sensor: SensorType) {
        if let Some(client_perms) = self.permissions.get_mut(client) {
            client_perms.insert(sensor, PermissionState::Denied);
        }

        self.save_permissions();
    }

    pub fn revoke_all(&mut self, client: &ClientId) {
        self.permissions.remove(client);
        self.save_permissions();
    }
}

pub struct AuditLog {
    log_file: File,
}

impl AuditLog {
    fn new() -> Self {
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/var/log/rustica/sensors.log")
            .unwrap();

        Self { log_file }
    }

    fn log_access(&mut self, client: ClientId, sensor: SensorType) {
        let entry = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "app": client.app_id,
            "pid": client.pid,
            "sensor": format!("{:?}", sensor),
        });

        writeln!(self.log_file, "{}", entry).ok();
    }
}
```

## Power Controller

```rust
pub struct PowerController {
    sensor_states: HashMap<SensorId, SensorPowerState>,
    batch_configs: HashMap<SensorId, BatchConfig>,
}

struct SensorPowerState {
    active: bool,
    sampling_period: Duration,
    last_activity: Instant,
}

struct BatchConfig {
    enabled: bool,
    max_batch_latency: Duration,
    fifo_size: usize,
}

impl PowerController {
    pub fn new() -> Self {
        Self {
            sensor_states: HashMap::new(),
            batch_configs: HashMap::new(),
        }
    }

    pub fn activate_sensor(&mut self, id: SensorId, period: Duration) {
        let state = SensorPowerState {
            active: true,
            sampling_period: period,
            last_activity: Instant::now(),
        };

        self.sensor_states.insert(id, state);

        // Configure kernel for optimal power
        self.configure_kernel_power(id, period);
    }

    pub fn deactivate_sensor(&mut self, id: &SensorId) {
        self.sensor_states.remove(id);
        self.batch_configs.remove(id);

        // Put sensor to sleep via kernel interface
        self.put_sensor_to_sleep(id);
    }

    fn configure_kernel_power(&self, id: SensorId, period: Duration) {
        // Write to IIO sysfs to set sampling rate
        let sampling_freq_path = format!(
            "{}/sampling_frequency",
            id.device_path.to_string_lossy()
        );

        let hz = 1_000_000.0 / period.as_micros() as f64;

        std::fs::write(&sampling_freq_path, hz.to_string()).ok();
    }

    fn put_sensor_to_sleep(&self, id: &SensorId) {
        let power_state_path = format!(
            "{}/buffer/enable",
            id.device_path.to_string_lossy()
        );

        std::fs::write(&power_state_path, "0").ok();
    }

    /// Auto-suspend inactive sensors
    pub fn check_idle_sensors(&mut self, idle_threshold: Duration) {
        let now = Instant::now();

        for (id, state) in self.sensor_states.iter_mut() {
            if now.duration_since(state.last_activity) > idle_threshold {
                // Suspend sensor
                self.deactivate_sensor(id);
            }
        }
    }

    /// Enable batching for sensor
    pub fn enable_batching(
        &mut self,
        id: SensorId,
        max_latency: Duration,
        fifo_size: usize,
    ) {
        let config = BatchConfig {
            enabled: true,
            max_batch_latency: max_latency,
            fifo_size,
        };

        self.batch_configs.insert(id, config);

        // Configure kernel FIFO
        let buffer_length_path = format!(
            "{}/buffer/length",
            id.device_path.to_string_lossy()
        );

        std::fs::write(&buffer_length_path, fifo_size.to_string()).ok();
    }
}
```

## D-Bus Interface

```rust
// D-Bus service for sensor access
pub struct SensorService {
    manager: Arc<Mutex<SensorManager>>,
}

#[dbus_interface(name = "org.rustica.Sensors")]
impl SensorService {
    /// List all available sensors
    fn list_sensors(&self) -> Vec<SensorInfo> {
        self.manager.lock().unwrap().list_sensors()
    }

    /// Get information about specific sensor
    fn get_sensor_info(&self, name: String) -> Result<SensorInfo, Error> {
        let id = SensorId::from_name(name)?;
        self.manager.lock().unwrap()
            .get_sensor_info(&id)
            .ok_or(Error::SensorNotFound)
    }

    /// Start streaming sensor data
    fn start_stream(
        &self,
        sensor_name: String,
        sampling_period_ms: u32,
        batching: bool,
    ) -> Result<dbus::channel::Sender, Error> {
        let id = SensorId::from_name(sensor_name)?;
        let client = self.calling_client()?;

        let mut manager = self.manager.lock().unwrap();
        let receiver = manager.start_stream(
            id,
            client,
            Duration::from_millis(sampling_period_ms as u64),
            batching,
        )?;

        // Return D-Bus channel for streaming
        Ok(self.create_dbus_stream(receiver))
    }

    /// Stop streaming
    fn stop_stream(&self, sensor_name: String) -> Result<(), Error> {
        let id = SensorId::from_name(sensor_name)?;
        let client = self.calling_client()?;

        self.manager.lock().unwrap()
            .stop_stream(&id, client)
    }

    /// Single sensor reading
    fn read_once(&self, sensor_name: String) -> Result<SensorReading, Error> {
        let id = SensorId::from_name(sensor_name)?;
        let client = self.calling_client()?;

        self.manager.lock().unwrap()
            .read_once(&id, client)
    }

    /// Register trigger event
    fn register_trigger(
        &self,
        sensor_name: String,
        trigger_type: TriggerType,
    ) -> Result<TriggerHandle, Error> {
        let id = SensorId::from_name(sensor_name)?;
        let client = self.calling_client()?;

        // Create callback that sends D-Bus signal
        let callback = {
            let connection = self.connection.clone();
            Box::new(move |reading| {
                connection.send(
                    dbus::message::Message::signal(
                        "/org/rustica/Sensors",
                        "org.rustica.Sensors",
                        "TriggerFired",
                    ).append1(reading)
                ).ok();
            })
        };

        self.manager.lock().unwrap()
            .register_trigger(&id, client, trigger_type, callback)
    }
}
```

## Configuration

```toml
# /etc/rustica/sensors.conf
[general]
# Enable sensor daemon
enabled = true

# Auto-suspend idle sensors (seconds)
idle_suspend_timeout = 30

# Default sampling period for streaming (ms)
default_sampling_period = 100

[permissions]
# Default permission state for new apps
default_permission = "ask"

# Show permission rationale
show_rationale = true

# Remember permission decisions
remember_permissions = true

[power]
# Enable sensor batching
batching_enabled = true

# Maximum batch latency (ms)
max_batch_latency = 1000

# Auto-adjust sampling rate based on usage
adaptive_sampling = true

[accelerometer]
# Enable accelerometer
enabled = true

# Screen rotation threshold (degrees)
rotation_threshold = 45

# Shake detection threshold
shake_threshold = 15.0

[ambient_light]
# Enable auto-brightness
auto_brightness = true

# Brightness update interval (ms)
update_interval = 500

[proximity]
# Disable screen on proximity
disable_screen = true

# Distance threshold (cm)
near_threshold = 5.0

[location]
# Enable high-accuracy location
high_accuracy = false

# Location update interval (ms)
update_interval = 10000

# Background location updates
background_updates = false
```

## Privacy Features

1. **Permission Dialogs**: Clear explanation of why sensor is needed
2. **Usage Indicators**: Show when sensors are active (status icon)
3. **Audit Logging**: Track all sensor access
4. **One-Time Permissions**: Grant for single session
5. **Revocation**: Revoke permissions at any time
6. **Data Minimization**: Only request data actually needed
7. **Location Fuzzing**: Add random noise to location data

## Integration with System

```rust
// Auto-brightness based on ambient light
pub struct AutoBrightnessManager {
    light_sensor: AmbientLight,
    backlight: BacklightDevice,
    target_brightness: u8,
    current_brightness: u8,
}

impl AutoBrightnessManager {
    pub fn update(&mut self) -> Result<(), Error> {
        let target = self.light_sensor.recommended_brightness()?;

        // Smooth transition
        if target > self.current_brightness {
            self.current_brightness = self.current_brightness.saturating_add(1);
        } else if target < self.current_brightness {
            self.current_brightness = self.current_brightness.saturating_sub(1);
        }

        self.backlight.set_brightness(self.current_brightness)?;

        Ok(())
    }
}

// Screen rotation based on accelerometer
pub struct RotationManager {
    accelerometer: Accelerometer,
    current_orientation: Orientation,
    rotation_threshold: f64,  // degrees
}

impl RotationManager {
    pub fn check_rotation(&mut self) -> Result<Option<Orientation>, Error> {
        let reading = self.accelerometer.read_single()?;

        // Calculate device orientation from accelerometer
        let (pitch, roll) = self.calculate_orientation(&reading.values);

        let new_orientation = self.orientation_from_angles(pitch, roll);

        if new_orientation != self.current_orientation {
            self.current_orientation = new_orientation;
            return Ok(Some(new_orientation));
        }

        Ok(None)
    }

    fn calculate_orientation(&self, values: &[f64]) -> (f64, f64) {
        // Calculate pitch and roll from acceleration vector
        let x = values[0];
        let y = values[1];
        let z = values[2];

        let pitch = (y / z).atan().to_degrees();
        let roll = (-x / z).atan().to_degrees();

        (pitch, roll)
    }
}
```

## Dependencies

```toml
[dependencies]
zbus = "4"
libc = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"

# Industrial I/O for sensors
iio-sys = { path = "../iio-sys", optional = true }
evdev = "0.12"

# GPS
gps-rs = { path = "../gps-rs", optional = true }
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_discovery() {
        let sensors = Accelerometer::discover().unwrap();
        assert!(!sensors.is_empty());
    }

    #[test]
    fn test_accelerometer_read() {
        let sensors = Accelerometer::discover().unwrap();
        let (_, sensor) = sensors.into_iter().next().unwrap();

        let reading = sensor.read_single().unwrap();
        assert_eq!(reading.values.len(), 3);  // X, Y, Z
    }

    #[test]
    fn test_permission_grant() {
        let mut perm_manager = PermissionManager::new();
        let client = ClientId {
            app_id: "test.app".to_string(),
            pid: 1234,
        };

        // Initially not asked
        let sensor_id = SensorId::test_accelerometer();
        assert!(matches!(
            perm_manager.check_permission(client.clone(), &sensor_id),
            Err(Error::PermissionPending)
        ));

        // Grant permission
        perm_manager.request_permission(client.clone(), SensorType::Accelerometer).unwrap();

        // Now should succeed
        assert!(perm_manager.check_permission(client, &sensor_id).is_ok());
    }

    #[test]
    fn test_power_controller_suspend() {
        let mut power_controller = PowerController::new();
        let id = SensorId::test_accelerometer();

        power_controller.activate_sensor(id.clone(), Duration::from_millis(100));

        // Check auto-suspend
        power_controller.check_idle_sensors(Duration::from_secs(60));

        // Sensor should still be active (threshold not met)
        assert!(power_controller.sensor_states.contains_key(&id));
    }
}
```

## Future Enhancements

1. **Sensor Fusion**: Combine multiple sensors for better accuracy
2. **Machine Learning**: On-device activity recognition
3. **Calibration UI**: User-guided sensor calibration
4. **Virtual Sensors**: Software-defined sensors (e.g., pedometer)
5. **Sensor Recording**: Record and replay sensor data
6. **Low-Power Mode**: Reduced sampling when battery low
