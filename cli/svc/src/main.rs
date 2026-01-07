// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! svc - Service Manager
//!
//! Manages system services with real process tracking.

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use nix::sys::signal::{self, Signal};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{self, ForkResult, Pid};
use rutils::Process;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::os::fd::AsRawFd;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;

/// Service Manager
#[derive(Parser, Debug)]
#[command(name = "svc")]
#[command(about = "Service Manager", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all services
    List {
        /// Show all services including stopped
        #[arg(short, long)]
        all: bool,
    },

    /// Start service
    Start {
        /// Service name
        service: String,
    },

    /// Stop service
    Stop {
        /// Service name
        service: String,
    },

    /// Restart service
    Restart {
        /// Service name
        service: String,
    },

    /// Show service status
    Status {
        /// Service name
        service: String,
    },

    /// Enable service (start at boot)
    Enable {
        /// Service name
        service: String,
    },

    /// Disable service (don't start at boot)
    Disable {
        /// Service name
        service: String,
    },

    /// Show service logs
    Logs {
        /// Service name
        service: String,

        /// Number of lines
        #[arg(short = 'n', long, default_value_t = 50)]
        lines: usize,

        /// Follow logs
        #[arg(short = 'f', long)]
        follow: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceFile {
    name: String,
    description: String,
    exec_start: Vec<String>,
    exec_stop: Option<Vec<String>>,
    working_dir: Option<String>,
    user: Option<String>,
    group: Option<String>,
    environment: HashMap<String, String>,
    auto_start: bool,
    restart_on_failure: bool,
    dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServiceState {
    name: String,
    pid: Option<u32>,
    running: bool,
    enabled: bool,
    start_time: Option<String>,
    restart_count: u32,
}

struct ServiceManager {
    services_dir: PathBuf,
    state_dir: PathBuf,
    run_dir: PathBuf,
    log_dir: PathBuf,
    services: HashMap<String, ServiceFile>,
    states: HashMap<String, ServiceState>,
}

impl ServiceManager {
    fn new() -> Result<Self> {
        let services_dir = PathBuf::from("/etc/rustica/services");
        let state_dir = PathBuf::from("/var/lib/rustica/svc");
        let run_dir = PathBuf::from("/run/rustica/svc");
        let log_dir = PathBuf::from("/var/log/rustica");

        // Create directories
        fs::create_dir_all(&services_dir)?;
        fs::create_dir_all(&state_dir)?;
        fs::create_dir_all(&run_dir)?;
        fs::create_dir_all(&log_dir)?;

        let mut sm = Self {
            services_dir,
            state_dir,
            run_dir,
            log_dir,
            services: HashMap::new(),
            states: HashMap::new(),
        };

        sm.load_services()?;
        sm.load_states()?;

        Ok(sm)
    }

    fn load_services(&mut self) -> Result<()> {
        if !self.services_dir.exists() {
            // Create default services
            self.create_default_services()?;
            return Ok(());
        }

        // Load service files from directory
        for entry in fs::read_dir(&self.services_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("service") {
                if let Ok(service) = self.parse_service_file(&path) {
                    self.services.insert(service.name.clone(), service);
                }
            }
        }

        Ok(())
    }

    fn create_default_services(&mut self) -> Result<()> {
        // Create default network service
        let network = ServiceFile {
            name: "network".to_string(),
            description: "Network initialization".to_string(),
            exec_start: vec!["/usr/bin/network-init".to_string()],
            exec_stop: None,
            working_dir: None,
            user: Some("root".to_string()),
            group: Some("root".to_string()),
            environment: HashMap::new(),
            auto_start: true,
            restart_on_failure: false,
            dependencies: vec![],
        };

        // Create default firewall service
        let firewall = ServiceFile {
            name: "firewall".to_string(),
            description: "Firewall service".to_string(),
            exec_start: vec!["/usr/bin/fwctl".to_string(), "load".to_string()],
            exec_stop: Some(vec!["/usr/bin/fwctl".to_string(), "flush".to_string()]),
            working_dir: None,
            user: Some("root".to_string()),
            group: Some("root".to_string()),
            environment: HashMap::new(),
            auto_start: true,
            restart_on_failure: false,
            dependencies: vec!["network".to_string()],
        };

        self.services.insert(network.name.clone(), network);
        self.services.insert(firewall.name.clone(), firewall);

        // Save default service files
        self.save_service_files()?;

        Ok(())
    }

    fn save_service_files(&self) -> Result<()> {
        fs::create_dir_all(&self.services_dir)?;

        for service in self.services.values() {
            let service_path = self.services_dir.join(format!("{}.service", service.name));
            let content = serde_json::to_string_pretty(service)?;
            fs::write(&service_path, content)?;
        }

        Ok(())
    }

    fn parse_service_file(&self, path: &PathBuf) -> Result<ServiceFile> {
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Try JSON format first
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(service) = serde_json::from_str::<ServiceFile>(&content) {
                return Ok(service);
            }
        }

        // Fallback: parse systemd-style .service file (simplified)
        let content = fs::read_to_string(path)?;

        let mut description = String::new();
        let mut exec_start = Vec::new();
        let mut exec_stop = None;
        let mut working_dir = None;
        let mut user = None;
        let mut group = None;
        let mut auto_start = false;
        let mut restart_on_failure = false;
        let mut dependencies = Vec::new();

        let mut current_section = String::new();

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len()-1].to_string();
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match (current_section.as_str(), key) {
                    ("Unit", "Description") => description = value.to_string(),
                    ("Service", "ExecStart") => {
                        exec_start = shell_words::split(value).unwrap_or_default();
                    }
                    ("Service", "ExecStop") => {
                        exec_stop = Some(shell_words::split(value).unwrap_or_default());
                    }
                    ("Service", "WorkingDirectory") => working_dir = Some(value.to_string()),
                    ("Service", "User") => user = Some(value.to_string()),
                    ("Service", "Group") => group = Some(value.to_string()),
                    ("Service", "Restart") => {
                        restart_on_failure = value.to_lowercase() == "on-failure" || value.to_lowercase() == "always";
                    }
                    ("Install", "WantedBy") => {
                        auto_start = true;
                    }
                    ("Unit", "After") => {
                        dependencies = value.split_whitespace().map(|s| s.to_string()).collect();
                    }
                    _ => {}
                }
            }
        }

        Ok(ServiceFile {
            name,
            description,
            exec_start,
            exec_stop,
            working_dir,
            user,
            group,
            environment: HashMap::new(),
            auto_start,
            restart_on_failure,
            dependencies,
        })
    }

    fn load_states(&mut self) -> Result<()> {
        let state_file = self.state_dir.join("state.json");

        if state_file.exists() {
            let content = fs::read_to_string(&state_file)?;
            if let Ok(states) = serde_json::from_str::<HashMap<String, ServiceState>>(&content) {
                self.states = states;

                // Update running status based on actual PIDs
                self.update_running_status()?;
            }
        }

        // Initialize states for services without one
        for (name, service) in &self.services {
            if !self.states.contains_key(name) {
                self.states.insert(name.clone(), ServiceState {
                    name: name.clone(),
                    pid: None,
                    running: false,
                    enabled: service.auto_start,
                    start_time: None,
                    restart_count: 0,
                });
            }
        }

        Ok(())
    }

    fn save_states(&self) -> Result<()> {
        let state_file = self.state_dir.join("state.json");
        let content = serde_json::to_string_pretty(&self.states)?;
        fs::write(&state_file, content)?;
        Ok(())
    }

    fn update_running_status(&mut self) -> Result<()> {
        for state in self.states.values_mut() {
            if let Some(pid) = state.pid {
                // Check if process is still running
                let proc_path = PathBuf::from("/proc").join(pid.to_string());
                state.running = proc_path.exists();

                if !state.running {
                    state.pid = None;
                    state.start_time = None;
                }
            }
        }
        Ok(())
    }

    fn list_services(&self, show_all: bool) {
        println!("Loaded services:");
        println!();

        let mut names: Vec<_> = self.services.keys().collect();
        names.sort();

        for name in names {
            let service = self.services.get(name).unwrap();
            let state = self.states.get(name).unwrap();

            if !show_all && !state.running {
                continue;
            }

            let status = if state.running {
                "\x1b[1;32mrunning\x1b[0m"
            } else {
                "\x1b[1;31mstopped\x1b[0m"
            };

            let enabled = if state.enabled {
                "enabled"
            } else {
                "disabled"
            };

            println!("  {} - {} ({})", name, status, enabled);
            println!("    {}", service.description);
            if let Some(pid) = state.pid {
                println!("    PID: {}", pid);
            }
            println!();
        }
    }

    fn start_service(&mut self, name: &str) -> Result<()> {
        if !self.services.contains_key(name) {
            return Err(anyhow::anyhow!("Service not found: {}", name));
        }

        let service = self.services.get(name).unwrap().clone();

        // Check if already running
        {
            let state = self.states.get(name).unwrap();
            if state.running {
                if let Some(pid) = state.pid {
                    // Verify process is actually running
                    let proc_path = PathBuf::from("/proc").join(pid.to_string());
                    if proc_path.exists() {
                        println!("Service {} is already running (PID: {})", name, pid);
                        return Ok(());
                    }
                }
            }
        }

        println!("Starting {}...", name);

        // Start dependencies first
        for dep in &service.dependencies {
            if let Some(dep_state) = self.states.get(dep) {
                if !dep_state.running {
                    println!("  Starting dependency {}...", dep);
                    self.start_service(dep)?;
                }
            }
        }

        // Fork and execute
        let child = match unsafe { unistd::fork() }? {
            ForkResult::Parent { child } => child,
            ForkResult::Child => {
                return self.exec_service(&service);
            }
        };

        // Parent process - update state after fork
        let state = self.states.get_mut(name).unwrap();
        state.pid = Some(child.as_raw() as u32);
        state.running = true;
        state.start_time = Some(format_time(SystemTime::now()));

        // Write PID file
        let pid_file = self.run_dir.join(format!("{}.pid", name));
        fs::write(&pid_file, format!("{}", child.as_raw()))?;

        self.save_states()?;

        println!("Service {} started (PID: {})", name, child.as_raw());

        Ok(())
    }

    fn exec_service(&self, service: &ServiceFile) -> Result<()> {
        // Set working directory
        if let Some(ref dir) = service.working_dir {
            std::env::set_current_dir(dir)?;
        }

        // Set user/group
        if let Some(ref user) = service.user {
            if let Ok(u) = unistd::User::from_name(user) {
                if let Some(u) = u {
                    unistd::setuid(u.uid)?;
                    unistd::setgid(u.gid)?;
                }
            }
        }

        // Set environment variables
        for (key, value) in &service.environment {
            std::env::set_var(key, value);
        }

        // Redirect stdout/stderr to log file
        let log_file = self.log_dir.join(format!("{}.log", service.name));
        if let Ok(f) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
        {
            let fd = f.as_raw_fd();
            // Keep f alive so fd stays valid
            std::mem::forget(f);
            unsafe {
                libc::dup2(fd, 1);
                libc::dup2(fd, 2);
            }
        }

        // Execute service
        let (program, args) = if service.exec_start.len() > 1 {
            (&service.exec_start[0], &service.exec_start[1..])
        } else {
            (&service.exec_start[0], &[][..])
        };

        let mut cmd = Command::new(program);
        cmd.args(args);

        let err = cmd.exec();

        Err(anyhow::anyhow!("exec failed: {}", err))
    }

    fn stop_service(&mut self, name: &str) -> Result<()> {
        if !self.services.contains_key(name) {
            return Err(anyhow::anyhow!("Service not found: {}", name));
        }

        let service = self.services.get(name).unwrap().clone();
        let state = self.states.get_mut(name).unwrap();

        if !state.running {
            println!("Service {} is not running", name);
            return Ok(());
        }

        println!("Stopping {}...", name);

        // Send SIGTERM
        if let Some(pid) = state.pid {
            signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM)?;

            // Wait for process to exit
            let start = std::time::Instant::now();
            let timeout = std::time::Duration::from_secs(10);

            loop {
                match waitpid(Pid::from_raw(pid as i32), None) {
                    Ok(WaitStatus::Exited(_, _)) | Ok(WaitStatus::Signaled(_, _, _)) => {
                        break;
                    }
                    Err(_) => {
                        // Process doesn't exist
                        break;
                    }
                    Ok(_) => {
                        // Still running
                    }
                }

                if start.elapsed() > timeout {
                    // Force kill
                    signal::kill(Pid::from_raw(pid as i32), Signal::SIGKILL)?;
                    break;
                }

                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }

        // Run ExecStop if present
        if let Some(ref exec_stop) = service.exec_stop {
            let (program, args) = if exec_stop.len() > 1 {
                (&exec_stop[0], &exec_stop[1..])
            } else {
                (&exec_stop[0], &[][..])
            };

            let _ = Command::new(program)
                .args(args)
                .output();
        }

        state.running = false;
        state.pid = None;
        state.start_time = None;

        // Remove PID file
        let pid_file = self.run_dir.join(format!("{}.pid", name));
        fs::remove_file(pid_file).ok();

        self.save_states()?;

        println!("Service {} stopped", name);

        Ok(())
    }

    fn restart_service(&mut self, name: &str) -> Result<()> {
        println!("Restarting {}...", name);
        self.stop_service(name)?;
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.start_service(name)
    }

    fn show_status(&self, name: &str) -> Result<()> {
        if !self.services.contains_key(name) {
            return Err(anyhow::anyhow!("Service not found: {}", name));
        }

        let service = self.services.get(name).unwrap();
        let state = self.states.get(name).unwrap();

        println!("Service: {}", service.name);
        println!("Description: {}", service.description);
        println!("Status: {}", if state.running { "running" } else { "stopped" });
        println!("Enabled: {}", if state.enabled { "yes" } else { "no" });

        if let Some(pid) = state.pid {
            println!("PID: {}", pid);
        }

        if let Some(ref start_time) = state.start_time {
            println!("Started: {}", start_time);
        }

        if !service.dependencies.is_empty() {
            println!("Dependencies: {}", service.dependencies.join(", "));
        }

        Ok(())
    }

    fn enable_service(&mut self, name: &str) -> Result<()> {
        if let Some(state) = self.states.get_mut(name) {
            state.enabled = true;
            self.save_states()?;
            println!("Service {} enabled", name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Service not found: {}", name))
        }
    }

    fn disable_service(&mut self, name: &str) -> Result<()> {
        if let Some(state) = self.states.get_mut(name) {
            state.enabled = false;
            self.save_states()?;
            println!("Service {} disabled", name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Service not found: {}", name))
        }
    }

    fn show_logs(&self, name: &str, lines: usize, follow: bool) -> Result<()> {
        let log_path = self.log_dir.join(format!("{}.log", name));

        if !log_path.exists() {
            println!("No logs found for service: {}", name);
            return Ok(());
        }

        let content = fs::read_to_string(&log_path)?;
        let log_lines: Vec<&str> = content.lines().collect();

        let start = if log_lines.len() > lines {
            log_lines.len() - lines
        } else {
            0
        };

        for line in &log_lines[start..] {
            println!("{}", line);
        }

        if follow {
            println!("(Following logs - press Ctrl+C to exit)");
            // In production, would use inotify or polling
        }

        Ok(())
    }
}

fn format_time(time: SystemTime) -> String {
    time.duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| format!("{}", d.as_secs()))
        .unwrap_or_else(|_| "unknown".to_string())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut sm = ServiceManager::new()?;

    match args.command {
        Commands::List { all } => {
            sm.list_services(all);
        }
        Commands::Start { service } => {
            sm.start_service(&service)?;
        }
        Commands::Stop { service } => {
            sm.stop_service(&service)?;
        }
        Commands::Restart { service } => {
            sm.restart_service(&service)?;
        }
        Commands::Status { service } => {
            sm.show_status(&service)?;
        }
        Commands::Enable { service } => {
            sm.enable_service(&service)?;
        }
        Commands::Disable { service } => {
            sm.disable_service(&service)?;
        }
        Commands::Logs { service, lines, follow } => {
            sm.show_logs(&service, lines, follow)?;
        }
    }

    Ok(())
}
