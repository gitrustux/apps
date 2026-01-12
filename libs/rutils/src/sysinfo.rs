// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! System information utilities
//!
//! Provides access to system information like kernel version,
//! architecture, memory, CPU, and distribution details.

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;

/// System information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Kernel name (e.g., "Linux")
    pub kernel_name: String,
    /// Network node hostname
    pub hostname: String,
    /// Kernel release
    pub kernel_release: String,
    /// Kernel version
    pub kernel_version: String,
    /// Machine hardware name
    pub machine: String,
    /// Processor type
    pub processor: String,
    /// Hardware platform
    pub hardware_platform: String,
    /// Operating system name
    pub operating_system: String,
    /// Total memory in bytes
    pub total_memory: u64,
    /// Available memory in bytes
    pub available_memory: u64,
    /// Free memory in bytes
    pub free_memory: u64,
    /// Total swap in bytes
    pub total_swap: u64,
    /// Free swap in bytes
    pub free_swap: u64,
    /// Number of CPU cores
    pub cpu_cores: u32,
    /// System uptime in seconds
    pub uptime: u64,
}

impl SystemInfo {
    /// Get all system information
    pub fn gather() -> Result<Self> {
        Ok(Self {
            kernel_name: Self::get_kernel_name()?,
            hostname: Self::get_hostname()?,
            kernel_release: Self::get_kernel_release()?,
            kernel_version: Self::get_kernel_version()?,
            machine: Self::get_machine()?,
            processor: Self::get_processor()?,
            hardware_platform: Self::get_hardware_platform()?,
            operating_system: "Rustica".to_string(),
            total_memory: Self::get_total_memory()?,
            available_memory: Self::get_available_memory()?,
            free_memory: Self::get_free_memory()?,
            total_swap: Self::get_total_swap()?,
            free_swap: Self::get_free_swap()?,
            cpu_cores: Self::get_cpu_cores()?,
            uptime: Self::get_uptime()?,
        })
    }

    fn get_kernel_name() -> Result<String> {
        Self::read_sysinfo("kernel_name").or_else(|_| Ok("Linux".to_string()))
    }

    fn get_hostname() -> Result<String> {
        fs::read_to_string("/proc/sys/kernel/hostname")
            .context("Failed to read hostname")
            .map(|s| s.trim().to_string())
    }

    fn get_kernel_release() -> Result<String> {
        Self::read_sysinfo("kernel_release")
            .or_else(|_| fs::read_to_string("/proc/sys/kernel/osrelease")
                .context("Failed to read kernel release")
                .map(|s| s.trim().to_string()))
    }

    fn get_kernel_version() -> Result<String> {
        Self::read_sysinfo("kernel_version")
            .or_else(|_| fs::read_to_string("/proc/sys/kernel/version")
                .context("Failed to read kernel version")
                .map(|s| s.trim().to_string()))
    }

    fn get_machine() -> Result<String> {
        Self::read_sysinfo("machine")
            .or_else(|_| {
                #[cfg(target_arch = "x86_64")]
                { Ok("x86_64".to_string()) }
                #[cfg(target_arch = "aarch64")]
                { Ok("aarch64".to_string()) }
                #[cfg(target_arch = "riscv64")]
                { Ok("riscv64".to_string()) }
                #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
                { Ok("unknown".to_string()) }
            })
    }

    fn get_processor() -> Result<String> {
        Self::read_sysinfo("processor").or_else(|_| Self::get_machine())
    }

    fn get_hardware_platform() -> Result<String> {
        Self::read_sysinfo("hardware_platform")
            .or_else(|_| Self::get_machine())
    }

    fn get_total_memory() -> Result<u64> {
        Self::parse_meminfo("MemTotal")
    }

    fn get_available_memory() -> Result<u64> {
        Self::parse_meminfo("MemAvailable")
            .or_else(|_| Self::get_free_memory())
    }

    fn get_free_memory() -> Result<u64> {
        Self::parse_meminfo("MemFree")
    }

    fn get_total_swap() -> Result<u64> {
        Self::parse_meminfo("SwapTotal")
    }

    fn get_free_swap() -> Result<u64> {
        Self::parse_meminfo("SwapFree")
    }

    fn get_cpu_cores() -> Result<u32> {
        fs::read_to_string("/proc/cpuinfo")
            .context("Failed to read cpuinfo")
            .and_then(|content| {
                content.lines()
                    .filter(|line| line.trim().starts_with("processor"))
                    .count()
                    .try_into()
                    .map_err(|_| anyhow!("Invalid core count"))
            })
            .or_else(|_| Ok(1))
    }

    fn get_uptime() -> Result<u64> {
        fs::read_to_string("/proc/uptime")
            .context("Failed to read uptime")
            .and_then(|content| {
                content.split_whitespace()
                    .next()
                    .and_then(|s| s.parse::<f64>().ok())
                    .map(|uptime| uptime as u64)
                    .ok_or_else(|| anyhow!("Invalid uptime format"))
            })
    }

    fn parse_meminfo(key: &str) -> Result<u64> {
        fs::read_to_string("/proc/meminfo")
            .context("Failed to read meminfo")
            .and_then(|content| {
                content.lines()
                    .find(|line| line.starts_with(key))
                    .and_then(|line| {
                        line.split_whitespace()
                            .nth(1)
                            .and_then(|s| s.parse::<u64>().ok())
                    })
                    .ok_or_else(|| anyhow!("{} not found in meminfo", key))
            })
    }

    fn read_sysinfo(field: &str) -> Result<String> {
        let output = std::process::Command::new("uname")
            .arg(match field {
                "kernel_name" => "-s",
                "kernel_release" => "-r",
                "kernel_version" => "-v",
                "machine" => "-m",
                "processor" => "-p",
                "hardware_platform" => "-i",
                _ => return Err(anyhow!("Unknown field: {}", field)),
            })
            .output()
            .context("Failed to run uname")?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(anyhow!("uname failed"))
        }
    }
}

/// Distribution information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionInfo {
    /// Distributor ID
    pub distributor_id: String,
    /// Distribution description
    pub description: String,
    /// Distribution release
    pub release: String,
    /// Distribution codename
    pub codename: String,
}

impl DistributionInfo {
    /// Get distribution information
    pub fn gather() -> Result<Self> {
        let os_release = Self::parse_os_release()?;

        Ok(Self {
            distributor_id: os_release.get("id")
                .or_else(|| os_release.get("NAME"))
                .unwrap_or(&"rustica".to_string())
                .to_string(),
            description: os_release.get("PRETTY_NAME")
                .unwrap_or(&"Rustica OS".to_string())
                .to_string(),
            release: os_release.get("VERSION_ID")
                .unwrap_or(&"0.1.0".to_string())
                .to_string(),
            codename: os_release.get("VERSION_CODENAME")
                .unwrap_or(&"dev".to_string())
                .to_string(),
        })
    }

    fn parse_os_release() -> Result<std::collections::HashMap<String, String>> {
        let content = fs::read_to_string("/etc/os-release")
            .or_else(|_| fs::read_to_string("/usr/lib/os-release"))
            .context("Failed to read os-release")?;

        let mut map = std::collections::HashMap::new();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let value = value.trim_matches('"').trim_matches('\'').to_string();
                map.insert(key.to_string(), value);
            }
        }

        Ok(map)
    }
}

/// CPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// CPU model name
    pub model: String,
    /// CPU vendor
    pub vendor: String,
    /// CPU cores
    pub cores: u32,
    /// CPU frequency in MHz
    pub frequency_mhz: Option<f64>,
    /// Cache size in KB
    pub cache_size_kb: Option<u32>,
}

impl CpuInfo {
    /// Get CPU information
    pub fn gather() -> Result<Self> {
        let content = fs::read_to_string("/proc/cpuinfo")
            .context("Failed to read cpuinfo")?;

        let mut model = String::new();
        let mut vendor = String::new();
        let mut frequency_mhz = None;
        let mut cache_size_kb = None;
        let mut cores = 0u32;

        for line in content.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "model name" => model = value.to_string(),
                    "vendor_id" => vendor = value.to_string(),
                    "cpu MHz" => frequency_mhz = value.parse().ok(),
                    "cache size" => {
                        cache_size_kb = value.split_whitespace()
                            .next()
                            .and_then(|s| s.parse().ok())
                    }
                    "processor" => cores += 1,
                    _ => {}
                }
            }
        }

        Ok(Self {
            model: if model.is_empty() { "Unknown CPU".to_string() } else { model },
            vendor: if vendor.is_empty() { "Unknown".to_string() } else { vendor },
            cores,
            frequency_mhz,
            cache_size_kb,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info() {
        let info = SystemInfo::gather();
        assert!(info.is_ok());
        let info = info.unwrap();
        assert_eq!(info.operating_system, "Rustica");
        assert!(!info.kernel_release.is_empty() || info.kernel_release == "unknown");
    }

    #[test]
    fn test_cpu_cores() {
        let cores = SystemInfo::get_cpu_cores();
        assert!(cores.is_ok());
        assert!(cores.unwrap() > 0);
    }
}
