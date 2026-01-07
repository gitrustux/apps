# Session Manager (rustica-session) Specification

**Date**: 2025-01-07
**Status**: ✅ **APPROVED**
**Component**: Rustica Shell - Session Manager
**Phase**: 5.3 - System Applications (Session Management)

## Overview

The Session Manager handles **user login/logout**, **session startup/shutdown**, **autostart applications**, **environment setup**, and **session state management**. It runs as a **systemd user service** and coordinates all components of the Rustica Shell session.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Session Manager                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────────┐  │
│  │ Login       │  │ Session     │  │ Autostart   │  │ Environment   │  │
│  │ Manager     │  │ Controller  │  │ Manager     │  │ Setup         │  │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └───────┬───────┘  │
│         │                │                │                  │          │
│         ▼                ▼                ▼                  ▼          │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                      Session State                               │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐           │  │
│  │  │ Active   │ │ User     │ │ Apps     │ │ Clients  │           │  │
│  │  │ Sessions │ │ Info     │  │ State    │  │ Registry│           │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘           │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                           │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    Process Management                             │  │
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐      │  │
│  │  │ Shell        │ │ Components   │ │ User Applications   │      │  │
│  │  │ Processes    │  │ (panel,dock) │ │ (autostart,manual)  │      │  │
│  │  └──────────────┘ └──────────────┘ └──────────────────────┘      │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                           │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                  Lifecycle Management                             │  │
│  │  ┌─────────────┐ ┌─────────────┐ ┌────────────────────────────┐  │  │
│  │  │ Startup     │ │ Shutdown    │ │ Inhibitors                 │  │  │
│  │  │ Sequence    │ │ Sequence    │ │ (prevent logout)           │  │  │
│  │  └─────────────┘ └─────────────┘ └────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

## Session Manager Structure

```rust
pub struct SessionManager {
    /// Active sessions
    sessions: HashMap<SessionId, Session>,

    /// Current session
    current_session: Option<SessionId>,

    /// User database
    user_db: UserDatabase,

    /// PAM authenticator
    pam: PamAuthenticator,

    /// D-Bus connection
    dbus_connection: Connection,

    /// Autostart manager
    autostart: AutostartManager,

    /// Environment manager
    env: EnvironmentManager,

    /// Client registry (apps registered with session)
    clients: HashMap<String, SessionClient>,

    /// Logout inhibitors
    inhibitors: Vec<Inhibitor>,

    /// Running
    running: bool,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

pub struct Session {
    id: SessionId,
    user: UserInfo,
    display: String,
    display_type: DisplayType,
    state: SessionState,
    started_at: DateTime<Utc>,
    processes: Vec<Process>,
}

pub enum DisplayType {
    X11,
    Wayland,
    Headless,
}

pub enum SessionState {
    Starting,
    Active,
    Closing,
    Closed,
}

pub struct SessionClient {
    id: String,
    pid: Pid,
    name: String,
    desktop_id: String,
    restart_command: Vec<String>,
    restart_style: RestartStyle,
}

pub enum RestartStyle {
    Normal,      // Restart if crashes
    Immediately, // Always restart
    Once,        // Don't restart
}

pub struct Inhibitor {
    id: String,
    app_id: String,
    reason: String,
    who: String,
    flags: InhibitFlags,
}

pub struct InhibitFlags {
    logout: bool,
    switch_user: bool,
    suspend: bool,
    idle: bool,
}

impl SessionManager {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            sessions: HashMap::new(),
            current_session: None,
            user_db: UserDatabase::new()?,
            pam: PamAuthenticator::new()?,
            dbus_connection: Connection::session()?,
            autostart: AutostartManager::new()?,
            env: EnvironmentManager::new()?,
            clients: HashMap::new(),
            inhibitors: Vec::new(),
            running: false,
        })
    }

    pub fn run(&mut self) -> Result<(), Error> {
        self.running = true;

        // Register D-Bus service
        self.register_dbus_service()?;

        // Check for existing session
        if self.try_restore_session()? {
            info!("Restored existing session");
        } else {
            info!("No existing session, waiting for login");
        }

        // Main event loop
        while self.running {
            // Wait for D-Bus events
            self.dbus_connection.process_duration(Duration::from_millis(100))?;

            // Check session status
            self.check_session_health()?;

            // Handle inhibitors
            self.process_inhibitors()?;
        }

        Ok(())
    }
}
```

## Login Flow

```rust
impl SessionManager {
    /// Authenticate user and start session
    pub fn login(&mut self, username: &str, password: &str, display: &str) -> Result<SessionId, Error> {
        info!("Attempting login for user: {}", username);

        // Authenticate via PAM
        if !self.pam.authenticate(username, password)? {
            return Err(Error::AuthenticationFailed);
        }

        // Get user info
        let user = self.user_db.get_user(username)
            .ok_or(Error::UserNotFound)?;

        // Check for existing session
        if let Some(session_id) = self.find_existing_session(username) {
            info!("User {} already has active session", username);
            self.activate_session(session_id.clone())?;
            return Ok(session_id);
        }

        // Create new session
        let session_id = SessionId(uuid::Uuid::new_v4().to_string());

        let session = Session {
            id: session_id.clone(),
            user: user.clone(),
            display: display.to_string(),
            display_type: DisplayType::Wayland,
            state: SessionState::Starting,
            started_at: Utc::now(),
            processes: Vec::new(),
        };

        self.sessions.insert(session_id.clone(), session);

        // Start session
        self.start_session(&session_id)?;

        info!("Session {} started for user {}", session_id, username);

        Ok(session_id)
    }

    /// Start a session (launch all components)
    fn start_session(&mut self, session_id: &SessionId) -> Result<(), Error> {
        let session = self.sessions.get(session_id)
            .ok_or(Error::SessionNotFound)?;

        // Set up environment
        self.env.setup_session(&session.user, &session.display)?;

        // Update session state
        self.sessions.get_mut(session_id).unwrap().state = SessionState::Active;

        // Mark as current
        self.current_session = Some(session_id.clone());

        // Launch shell components in order
        self.launch_shell_components(session_id)?;

        // Launch autostart applications
        self.autostart.launch_all(&session.user)?;

        // Emit session started signal
        self.session_started(session_id);

        Ok(())
    }

    /// Launch shell components in correct order
    fn launch_shell_components(&mut self, session_id: &SessionId) -> Result<(), Error> {
        let session = self.sessions.get(session_id)
            .ok_or(Error::SessionNotFound)?;

        let mut processes = Vec::new();

        // 1. Wayland compositor (rustica-comp)
        let comp = Process::spawn(
            "/usr/bin/rustica-comp",
            &["--display", &session.display],
            &session.user,
        )?;
        processes.push(comp);

        // Wait for compositor to be ready
        std::thread::sleep(Duration::from_millis(500));

        // 2. Settings daemon
        let settings = Process::spawn(
            "/usr/bin/rustica-settings-daemon",
            &[],
            &session.user,
        )?;
        processes.push(settings);

        // 3. Panel
        let panel = Process::spawn(
            "/usr/bin/rustica-panel",
            &[],
            &session.user,
        )?;
        processes.push(panel);

        // 4. Dock
        let dock = Process::spawn(
            "/usr/bin/rustica-dock",
            &[],
            &session.user,
        )?;
        processes.push(dock);

        // 5. Notification daemon
        let notifications = Process::spawn(
            "/usr/bin/rustica-notifications",
            &[],
            &session.user,
        )?;
        processes.push(notifications);

        // Update session processes
        self.sessions.get_mut(session_id).unwrap().processes = processes;

        Ok(())
    }
}
```

## Logout Flow

```rust
impl SessionManager {
    /// Logout from current session
    pub fn logout(&mut self, force: bool) -> Result<(), Error> {
        let session_id = self.current_session.as_ref()
            .ok_or(Error::NoActiveSession)?;

        let session = self.sessions.get(session_id)
            .ok_or(Error::SessionNotFound)?;

        info!("Logging out from session {}", session_id);

        // Check for inhibitors
        if !force && !self.inhibitors.is_empty() {
            return Err(Error::LogoutInhibited {
                inhibitors: self.inhibitors.clone(),
            });
        }

        // Change state
        self.sessions.get_mut(session_id).unwrap().state = SessionState::Closing;

        // Emit session ending signal
        self.session_ending(session_id);

        // Stop autostart apps
        self.autostart.stop_all(&session.user)?;

        // Stop shell components in reverse order
        self.stop_shell_components(session_id)?;

        // Save session state
        self.save_session_state(session_id)?;

        // Remove session
        self.sessions.remove(session_id);
        self.current_session = None;

        // Emit session ended signal
        self.session_ended(session_id);

        info!("Session {} ended", session_id);

        Ok(())
    }

    /// Stop shell components in reverse order
    fn stop_shell_components(&mut self, session_id: &SessionId) -> Result<(), Error> {
        let session = self.sessions.get_mut(session_id)
            .ok_or(Error::SessionNotFound)?;

        // Stop in reverse order
        for process in session.processes.drain(..).rev() {
            debug!("Stopping process: {}", process.name());

            // Try graceful shutdown first
            process.terminate()?;

            // Wait up to 5 seconds
            let deadline = Instant::now() + Duration::from_secs(5);
            while !process.is_finished() {
                if Instant::now() > deadline {
                    // Force kill if not finished
                    process.kill()?;
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }

        Ok(())
    }
}
```

## Autostart Manager

```rust
pub struct AutostartManager {
    /// System autostart entries
    system_entries: Vec<AutostartEntry>,

    /// User autostart entries
    user_entries: Vec<AutostartEntry>,
}

pub struct AutostartEntry {
    path: PathBuf,
    name: String,
    hidden: bool,
    enabled: bool,
    command: Vec<String>,
    conditions: Vec<AutostartCondition>,
    delay: Option<Duration>,
}

pub enum AutostartCondition {
    Wayland,
    X11,
    DBusSession(String),
}

impl AutostartManager {
    pub fn new() -> Result<Self, Error> {
        // Load system entries
        let system_path = PathBuf::from("/etc/xdg/autostart");
        let system_entries = Self::load_entries(&system_path)?;

        // Load user entries
        let user_path = dirs::config_dir()
            .ok_or(Error::NoConfigDir)?
            .join("autostart");
        let user_entries = Self::load_entries(&user_path)?;

        Ok(Self {
            system_entries,
            user_entries,
        })
    }

    fn load_entries(dir: &Path) -> Result<Vec<AutostartEntry>, Error> {
        let mut entries = Vec::new();

        if !dir.exists() {
            return Ok(entries);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                continue;
            }

            // Parse .desktop file
            if let Some(desktop_entry) = Self::parse_desktop_entry(&path)? {
                entries.push(desktop_entry);
            }
        }

        Ok(entries)
    }

    fn parse_desktop_entry(path: &Path) -> Result<Option<AutostartEntry>, Error> {
        let content = std::fs::read_to_string(path)?;

        let mut name = None;
        let mut hidden = false;
        let mut enabled = true;
        let mut command = Vec::new();
        let mut conditions = Vec::new();
        let mut delay = None;

        // Parse .desktop file
        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("Name=") {
                name = Some(line[5..].to_string());
            } else if line.starts_with("Hidden=true") {
                hidden = true;
            } else if line.starts_with("X-GNOME-Autostart-enabled=false") {
                enabled = false;
            } else if line.starts_with("Exec=") {
                // Parse command (handle quoted strings)
                command = Self::parse_command(&line[5..]);
            } else if line.starts_with("OnlyShowIn=") {
                // Check if current environment is in list
                let envs = line[11..].split(';').collect::<Vec<_>>();
                if !envs.contains(&"Rustica") && !envs.contains(&"GNOME") {
                    enabled = false;
                }
            } else if line.starts_with("X-Rustica-Delay=") {
                if let Ok(seconds) = line[18..].parse::<u64>() {
                    delay = Some(Duration::from_secs(seconds));
                }
            }
        }

        let name = name.ok_or(Error::InvalidDesktopEntry)?;

        Ok(Some(AutostartEntry {
            path: path.to_path_buf(),
            name,
            hidden,
            enabled,
            command,
            conditions,
            delay,
        }))
    }

    pub fn launch_all(&self, user: &UserInfo) -> Result<(), Error> {
        info!("Launching autostart applications for user {}", user.name);

        // Merge system and user entries (user entries override)
        let mut all_entries = self.system_entries.clone();
        for user_entry in &self.user_entries {
            if let Some(system_entry) = all_entries.iter_mut().find(|e| e.name == user_entry.name) {
                *system_entry = user_entry.clone();
            } else {
                all_entries.push(user_entry.clone());
            }
        }

        // Launch enabled entries
        for entry in all_entries {
            if entry.enabled && !entry.hidden {
                self.launch_entry(&entry, user)?;
            }
        }

        Ok(())
    }

    fn launch_entry(&self, entry: &AutostartEntry, user: &UserInfo) -> Result<(), Error> {
        // Apply delay if specified
        if let Some(delay) = entry.delay {
            std::thread::sleep(delay);
        }

        info!("Launching autostart: {}", entry.name);

        // Spawn process
        let _child = std::process::Command::new(&entry.command[0])
            .args(&entry.command[1..])
            .uid(user.uid)
            .gid(user.gid)
            .spawn()
            .map_err(|e| Error::SpawnFailed {
                app: entry.name.clone(),
                reason: e.to_string(),
            })?;

        Ok(())
    }
}
```

## Environment Manager

```rust
pub struct EnvironmentManager {
    /// Environment variables
    env_vars: HashMap<String, String>,
}

impl EnvironmentManager {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            env_vars: HashMap::new(),
        })
    }

    pub fn setup_session(&mut self, user: &UserInfo, display: &str) -> Result<(), Error> {
        // Clear existing environment
        self.env_vars.clear();

        // Basic variables
        self.set("USER", &user.name);
        self.set("LOGNAME", &user.name);
        self.set("HOME", &user.home_dir);
        self.set("SHELL", &user.shell);

        // XDG variables
        self.set("XDG_RUNTIME_DIR", &format!("/run/user/{}", user.uid));
        self.set("XDG_CONFIG_HOME", &user.home_dir.join(".config").to_string_lossy());
        self.set("XDG_DATA_HOME", &user.home_dir.join(".local/share").to_string_lossy());
        self.set("XDG_CACHE_HOME", &user.home_dir.join(".cache").to_string_lossy());
        self.set("XDG_STATE_HOME", &user.home_dir.join(".local/state").to_string_lossy());
        self.set("XDG_DATA_DIRS", "/usr/local/share:/usr/share");
        self.set("XDG_CONFIG_DIRS", "/etc/xdg");

        // Wayland-specific
        self.set("WAYLAND_DISPLAY", display);
        self.set("DISPLAY", format!(":{}", display.trim_start_matches("wayland-")));
        self.set("XDG_SESSION_TYPE", "wayland");
        self.set("XDG_CURRENT_DESKTOP", "Rustica");

        // Rustica-specific
        self.set("RUSTICA_VERSION", env!("CARGO_PKG_VERSION"));

        // Qt
        self.set("QT_QPA_PLATFORM", "wayland");

        // GTK
        self.set("GTK_BACKEND", "wayland");

        // SDL
        self.set("SDL_VIDEODRIVER", "wayland");

        // Electron
        self.set("ELECTRON_OZONE_PLATFORM_HINT", "auto");

        // Mozilla
        self.set("MOZ_ENABLE_WAYLAND", "1");

        // Fontconfig
        self.set("FONTCONFIG_PATH", "/etc/fonts");

        // Apply to current process
        for (key, value) in &self.env_vars {
            std::env::set_var(key, value);
        }

        info!("Session environment configured for user {}", user.name);

        Ok(())
    }

    fn set(&mut self, key: &str, value: &str) {
        self.env_vars.insert(key.to_string(), value.to_string());
    }
}
```

## Logout Inhibition

```rust
impl SessionManager {
    /// Inhibit logout
    pub fn inhibit_logout(&mut self, app_id: String, reason: String, who: String, flags: InhibitFlags) -> Result<String, Error> {
        let inhibitor_id = uuid::Uuid::new_v4().to_string();

        let inhibitor = Inhibitor {
            id: inhibitor_id.clone(),
            app_id,
            reason,
            who,
            flags,
        };

        self.inhibitors.push(inhibitor);

        info!("Logout inhibited: {} ({})", inhibitor_id, inhibitor.reason);

        // Emit inhibitor added signal
        self.inhibitor_added(&inhibitor_id);

        Ok(inhibitor_id)
    }

    /// Uninhibit logout
    pub fn uninhibit_logout(&mut self, inhibitor_id: String) -> Result<(), Error> {
        self.inhibitors.retain(|i| i.id != inhibitor_id);

        info!("Logout uninhibited: {}", inhibitor_id);

        // Emit inhibitor removed signal
        self.inhibitor_removed(&inhibitor_id);

        Ok(())
    }

    fn process_inhibitors(&mut self) -> Result<(), Error> {
        // Remove inhibitors for apps that are no longer running
        self.inhibitors.retain(|inhibitor| {
            if let Ok(_result) = std::process::Command::new("pidof")
                .arg(&inhibitor.app_id)
                .output()
            {
                true
            } else {
                warn!("Removing stale inhibitor: {}", inhibitor.id);
                false
            }
        });

        Ok(())
    }
}
```

## Session State Management

```rust
impl SessionManager {
    /// Save session state
    fn save_session_state(&self, session_id: &SessionId) -> Result<(), Error> {
        let session = self.sessions.get(session_id)
            .ok_or(Error::SessionNotFound)?;

        let state_path = dirs::state_dir()
            .ok_or(Error::NoStateDir)?
            .join("rustica/session.toml");

        // Create state
        let state = toml::toml! {
            [session]
            id = session_id.to_string()
            user = session.user.name
            display = session.display
            started_at = session.started_at.to_rfc3339()

            [clients]
        };

        // Add clients
        let mut state = state;
        for client in &self.clients {
            state["clients"][&client.id] = toml::toml! {
                id = client.id
                name = client.name
                desktop_id = client.desktop_id
                restart_command = client.restart_command
                restart_style = match client.restart_style {
                    RestartStyle::Normal => "normal",
                    RestartStyle::Immediately => "immediately",
                    RestartStyle::Once => "once",
                }
            };
        }

        // Write state
        std::fs::write(&state_path, state.to_string())?;

        info!("Session state saved to {}", state_path.display());

        Ok(())
    }

    /// Try to restore previous session
    fn try_restore_session(&mut self) -> Result<bool, Error> {
        let state_path = dirs::state_dir()
            .ok_or(Error::NoStateDir)?
            .join("rustica/session.toml");

        if !state_path.exists() {
            return Ok(false);
        }

        // Load state
        let content = std::fs::read_to_string(&state_path)?;
        let state: toml::Value = toml::from_str(&content)?;

        // Check session
        if let Some(session) = state.get("session") {
            info!("Found previous session: {:?}", session);
        }

        Ok(true)
    }
}
```

## D-Bus Interface

```rust
// D-Bus service name: org.rustica.SessionManager
// Object path: /org/rustica/SessionManager

#[dbus_interface(name = "org.rustica.SessionManager")]
impl SessionManager {
    /// Login and start session
    fn login(&mut self, username: String, password: String, display: String) -> Result<String, Error> {
        let session_id = self.login(&username, &password, &display)?;
        Ok(session_id.to_string())
    }

    /// Logout from current session
    fn logout(&mut self, force: bool) -> Result<(), Error> {
        self.logout(force)?;
        Ok(())
    }

    /// Inhibit logout
    fn inhibit_logout(
        &mut self,
        app_id: String,
        reason: String,
        who: String,
        flags: (bool, bool, bool, bool), // logout, switch_user, suspend, idle
    ) -> Result<String, Error> {
        let inhibit_flags = InhibitFlags {
            logout: flags.0,
            switch_user: flags.1,
            suspend: flags.2,
            idle: flags.3,
        };

        self.inhibit_logout(app_id, reason, who, inhibit_flags)
    }

    /// Uninhibit logout
    fn uninhibit_logout(&mut self, inhibitor_id: String) -> Result<(), Error> {
        self.uninhibit_logout(inhibitor_id)
    }

    /// Get current session
    fn current_session(&self) -> Option<String> {
        self.current_session.as_ref().map(|s| s.to_string())
    }

    /// Get active sessions
    fn list_sessions(&self) -> Vec<String> {
        self.sessions.keys().map(|s| s.to_string()).collect()
    }

    /// Register client with session
    fn register_client(
        &mut self,
        app_id: String,
        pid: u32,
        desktop_id: String,
        restart_command: Vec<String>,
    ) -> Result<(), Error> {
        let client = SessionClient {
            id: uuid::Uuid::new_v4().to_string(),
            pid: Pid::from_raw(pid as i32),
            name: app_id.clone(),
            desktop_id,
            restart_command,
            restart_style: RestartStyle::Normal,
        };

        self.clients.insert(client.id.clone(), client);

        info!("Client registered: {} ({})", app_id, client.id);

        Ok(())
    }

    /// Signal: Session started
    #[dbus_interface(signal)]
    fn session_started(&self, session_id: String);

    /// Signal: Session ending
    #[dbus_interface(signal)]
    fn session_ending(&self, session_id: String);

    /// Signal: Session ended
    #[dbus_interface(signal)]
    fn session_ended(&self, session_id: String);

    /// Signal: Inhibitor added
    #[dbus_interface(signal)]
    fn inhibitor_added(&self, inhibitor_id: String);

    /// Signal: Inhibitor removed
    #[dbus_interface(signal)]
    fn inhibitor_removed(&self, inhibitor_id: String);
}
```

## File Structure

```
/var/www/rustux.com/prod/apps/gui/
├── rustica-session/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── session.rs
│       ├── login.rs
│       ├── logout.rs
│       ├── autostart.rs
│       ├── environment.rs
│       ├── inhibitors.rs
│       └── dbus/
│           └── interface.rs
└── systemd/
    └── user/
        └── rustica-session.service
```

## Systemd User Service

```ini
[Unit]
Description=Rustica Session Manager
Documentation=man:rustica-session(8)
BindsTo=graphical-session.target
Wants=graphical-session.target
After=graphical-session.target

[Service]
Type=dbus
BusName=org.rustica.SessionManager
ExecStart=/usr/bin/rustica-session
Restart=on-failure
RestartSec=5

# Security
NoNewPrivileges=true
PrivateTmp=true

# Performance
OOMScoreAdjust=-500

[Install]
WantedBy=graphical-session.target
```

## Dependencies

```toml
[package]
name = "rustica-session"
version = "1.0.0"
edition = "2021"

[dependencies]
# D-Bus
zbus = "3.0"
zvariant = "3.0"

# Authentication
pam = "0.7"

# Serialization
serde = "1.0"
toml = "0.8"

# UUID
uuid = { version = "1.0", features = ["v4"] }

# Time
chrono = "0.4"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# XDG
dirs = "5.0"

# Process
nix = "0.26"

# User info
users = "0.11"
```

## Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Login time | <3s | Enter to desktop |
| Logout time | <2s | Click to gone |
| Session restore | <5s | Start to restored |
| Memory | <30MB | Manager usage |
| Autostart app | <500ms each | Spawn delay |

## Success Criteria

- [ ] Login/logout works correctly
- [ ] Session startup sequence is correct
- [ ] Autostart applications launch
- [ ] Logout inhibition works
- [ ] Session state persists
- [ ] D-Bus interface complete
- [ ] Systemd service runs correctly
- [ ] Performance targets met

## Sign-Off

**Architect**: Claude (AI Assistant)
**Date**: 2025-01-07
**Status**: APPROVED ✅

---

## Timeline

- Week 1: Session structure + login flow
- Week 2: Autostart manager + environment setup
- Week 3: Logout flow + inhibitors
- Week 4: Session state + D-Bus + systemd

**Total**: 4 weeks
