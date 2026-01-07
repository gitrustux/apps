// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Process utilities

use anyhow::{Context, Result};
use nix::unistd::Pid;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub state: String,
    pub cmdline: String,
}

pub struct Process {
    pub pid: Pid,
}

impl Process {
    pub fn from_pid(pid: u32) -> Self {
        Self { pid: Pid::from_raw(pid as i32) }
    }

    pub fn current() -> Self {
        Self { pid: nix::unistd::getpid() }
    }

    pub fn fork(&self) -> Result<Option<Self>> {
        use nix::unistd::{fork, ForkResult};

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                Ok(Some(Process { pid: child }))
            }
            Ok(ForkResult::Child) => {
                Ok(None)
            }
            Err(e) => Err(anyhow::anyhow!("fork failed: {}", e)),
        }
    }

    pub fn exec(&self, path: &PathBuf, args: &[std::ffi::OsString]) -> Result<()> {
        use nix::unistd::execvp;
        use std::ffi::CString;

        let mut argv: Vec<CString> = vec![CString::new(path.to_string_lossy().as_ref())?];
        for arg in args {
            argv.push(CString::new(arg.to_string_lossy().as_ref())?);
        }

        execvp(&CString::new(path.to_string_lossy().as_ref())?, &argv).context("exec failed")?;
        unreachable!();
    }

    pub fn kill(&self, signal: nix::sys::signal::Signal) -> Result<()> {
        nix::sys::signal::kill(self.pid, signal).context("kill failed")?;
        Ok(())
    }

    pub fn wait(&self) -> Result<nix::sys::wait::WaitStatus> {
        nix::sys::wait::waitpid(self.pid, None).context("waitpid failed")
    }

    pub fn is_alive(&self) -> bool {
        nix::sys::signal::kill(self.pid, None).is_ok()
    }
}

pub fn get_process_info(pid: u32) -> Result<ProcessInfo> {
    let proc_path = PathBuf::from("/proc").join(pid.to_string());

    let stat_content = std::fs::read_to_string(proc_path.join("stat"))?;
    let cmdline_content = std::fs::read_to_string(proc_path.join("cmdline"))
        .unwrap_or_default()
        .replace('\0', " ");

    // Parse stat file (format: pid (name) state ppid ...)
    let parts: Vec<&str> = stat_content.split(')').collect();
    if parts.len() < 2 {
        anyhow::bail!("invalid stat format");
    }

    let pid_str = parts[0].trim().split_whitespace().next().unwrap_or("0");
    let name_parts: Vec<&str> = parts[0].split('(').collect();
    let name = name_parts.get(1).unwrap_or(&"").to_string();
    let rest = parts[1].trim();
    let rest_parts: Vec<&str> = rest.split_whitespace().collect();

    let state = rest_parts.get(0).unwrap_or(&"?").to_string();
    let ppid = rest_parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);

    Ok(ProcessInfo {
        pid: pid_str.parse().unwrap_or(0),
        ppid,
        name,
        state,
        cmdline: cmdline_content.trim().to_string(),
    })
}

pub fn list_processes() -> Result<Vec<ProcessInfo>> {
    let mut processes = Vec::new();

    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if let Ok(pid) = name.to_string_lossy().parse::<u32>() {
                if let Ok(info) = get_process_info(pid) {
                    processes.push(info);
                }
            }
        }
    }

    Ok(processes)
}
