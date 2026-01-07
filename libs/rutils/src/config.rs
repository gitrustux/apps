// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Configuration utilities

use anyhow::{Context, Result};
use std::path::PathBuf;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(all(feature = "serde", feature = "serde_json"))]
use serde_json;

#[cfg(feature = "serde")]
pub trait ConfigFile: Sized + Default + for<'de> Deserialize<'de> + Serialize {
    fn config_path() -> PathBuf;

    #[cfg(all(feature = "serde", feature = "serde_json"))]
    fn load() -> Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read config: {}", path.display()))?;

        serde_json::from_str(&content)
            .with_context(|| format!("failed to parse config: {}", path.display()))
    }

    #[cfg(all(feature = "serde", feature = "serde_json"))]
    fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create config dir: {}", parent.display()))?;
        }

        let content = serde_json::to_string_pretty(self)
            .context("failed to serialize config")?;

        std::fs::write(&path, content)
            .with_context(|| format!("failed to write config: {}", path.display()))
    }
}

#[cfg(feature = "serde")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub system: SystemConfig,
    pub network: NetworkConfig,
}

#[cfg(feature = "serde")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub hostname: String,
    pub timezone: String,
    pub locale: String,
}

#[cfg(feature = "serde")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub dhcp_enabled: bool,
    pub dns_servers: Vec<String>,
}

#[cfg(feature = "serde")]
impl Default for Config {
    fn default() -> Self {
        Self {
            system: SystemConfig {
                hostname: "rustica".to_string(),
                timezone: "UTC".to_string(),
                locale: "en_US.UTF-8".to_string(),
            },
            network: NetworkConfig {
                dhcp_enabled: true,
                dns_servers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
            },
        }
    }
}

#[cfg(feature = "serde")]
impl ConfigFile for Config {
    fn config_path() -> PathBuf {
        PathBuf::from("/etc/rustica/system.json")
    }
}

/// Get system configuration directory
pub fn get_config_dir() -> PathBuf {
    PathBuf::from("/etc/rustica")
}

/// Get cache directory
pub fn get_cache_dir() -> PathBuf {
    PathBuf::from("/var/cache/rustica")
}

/// Get state directory
pub fn get_state_dir() -> PathBuf {
    PathBuf::from("/var/lib/rustica")
}
