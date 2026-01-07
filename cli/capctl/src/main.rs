// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! capctl - Capability Control Utility
//!
//! Manage file capabilities and map kernel object model capabilities to file permissions.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use nix::sys::stat::mode_t;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Capability Control Utility
#[derive(Parser, Debug)]
#[command(name = "capctl")]
#[command(author = "The Rustux Authors")]
#[command(version = "0.1.0")]
#[command(about = "Capability control utility", long_about = None)]
struct CapctlArgs {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List capabilities of a file
    Get {
        /// File path
        #[arg(required = true)]
        path: PathBuf,
    },

    /// Set capabilities on a file
    Set {
        /// File path
        #[arg(required = true)]
        path: PathBuf,

        /// Capabilities to set (comma-separated)
        #[arg(required = true)]
        capabilities: String,

        /// Set permissions instead of capabilities
        #[arg(short, long)]
        permissions: bool,
    },

    /// Remove capabilities from a file
    Remove {
        /// File path
        #[arg(required = true)]
        path: PathBuf,

        /// Capabilities to remove (comma-separated, or 'all')
        #[arg(required = true)]
        capabilities: String,
    },

    /// List all defined capabilities
    List {
        /// Filter by category
        category: Option<String>,
    },

    /// Convert Unix permissions to capabilities
    FromPerms {
        /// File path
        #[arg(required = true)]
        path: PathBuf,
    },

    /// Show capability database
    Database,

    /// Initialize capability database
    InitDB,
}

/// Capability definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Capability {
    name: String,
    description: String,
    category: String,
    required_level: u8, // 0-255 capability level
    equivalent_perms: String, // Unix permissions equivalent
}

/// File capabilities metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct FileCapabilities {
    capabilities: Vec<String>,
    permission_bits: u32,
    capability_level: u8,
}

/// Standard capability database
fn get_capability_database() -> HashMap<String, Capability> {
    let mut db = HashMap::new();

    // File operations
    db.insert("file_read".to_string(), Capability {
        name: "file_read".to_string(),
        description: "Read file contents".to_string(),
        category: "file".to_string(),
        required_level: 1,
        equivalent_perms: "r--r--r--".to_string(),
    });

    db.insert("file_write".to_string(), Capability {
        name: "file_write".to_string(),
        description: "Write to file".to_string(),
        category: "file".to_string(),
        required_level: 2,
        equivalent_perms: "-w--w--w-".to_string(),
    });

    db.insert("file_execute".to_string(), Capability {
        name: "file_execute".to_string(),
        description: "Execute file".to_string(),
        category: "file".to_string(),
        required_level: 2,
        equivalent_perms: "--x--x--x".to_string(),
    });

    db.insert("file_full".to_string(), Capability {
        name: "file_full".to_string(),
        description: "Full file access (read/write/execute)".to_string(),
        category: "file".to_string(),
        required_level: 3,
        equivalent_perms: "rwxrwxrwx".to_string(),
    });

    // Directory operations
    db.insert("dir_read".to_string(), Capability {
        name: "dir_read".to_string(),
        description: "List directory contents".to_string(),
        category: "directory".to_string(),
        required_level: 1,
        equivalent_perms: "r-xr-xr-x".to_string(),
    });

    db.insert("dir_write".to_string(), Capability {
        name: "dir_write".to_string(),
        description: "Write to directory".to_string(),
        category: "directory".to_string(),
        required_level: 2,
        equivalent_perms: "rwxr-xr-x".to_string(),
    });

    // Network operations
    db.insert("net_bind".to_string(), Capability {
        name: "net_bind".to_string(),
        description: "Bind to network ports (< 1024 requires elevated level)".to_string(),
        category: "network".to_string(),
        required_level: 3,
        equivalent_perms: "N/A".to_string(),
    });

    db.insert("net_connect".to_string(), Capability {
        name: "net_connect".to_string(),
        description: "Initiate network connections".to_string(),
        category: "network".to_string(),
        required_level: 2,
        equivalent_perms: "N/A".to_string(),
    });

    // System operations
    db.insert("sys_admin".to_string(), Capability {
        name: "sys_admin".to_string(),
        description: "System administration operations".to_string(),
        category: "system".to_string(),
        required_level: 5,
        equivalent_perms: "N/A".to_string(),
    });

    db.insert("sys_service".to_string(), Capability {
        name: "sys_service".to_string(),
        description: "Manage system services".to_string(),
        category: "system".to_string(),
        required_level: 4,
        equivalent_perms: "N/A".to_string(),
    });

    // Device operations
    db.insert("device_read".to_string(), Capability {
        name: "device_read".to_string(),
        description: "Read from device files".to_string(),
        category: "device".to_string(),
        required_level: 3,
        equivalent_perms: "N/A".to_string(),
    });

    db.insert("device_write".to_string(), Capability {
        name: "device_write".to_string(),
        description: "Write to device files".to_string(),
        category: "device".to_string(),
        required_level: 4,
        equivalent_perms: "N/A".to_string(),
    });

    // Process operations
    db.insert("proc_spawn".to_string(), Capability {
        name: "proc_spawn".to_string(),
        description: "Spawn processes".to_string(),
        category: "process".to_string(),
        required_level: 2,
        equivalent_perms: "N/A".to_string(),
    });

    db.insert("proc_kill".to_string(), Capability {
        name: "proc_kill".to_string(),
        description: "Terminate processes".to_string(),
        category: "process".to_string(),
        required_level: 3,
        equivalent_perms: "N/A".to_string(),
    });

    // Package management
    db.insert("pkg_install".to_string(), Capability {
        name: "pkg_install".to_string(),
        description: "Install packages".to_string(),
        category: "package".to_string(),
        required_level: 4,
        equivalent_perms: "N/A".to_string(),
    });

    db.insert("pkg_remove".to_string(), Capability {
        name: "pkg_remove".to_string(),
        description: "Remove packages".to_string(),
        category: "package".to_string(),
        required_level: 4,
        equivalent_perms: "N/A".to_string(),
    });

    db
}

fn main() -> Result<()> {
    let args = CapctlArgs::parse();

    match args.command {
        Commands::Get { path } => {
            get_capabilities(&path)?;
        }
        Commands::Set { path, capabilities, permissions } => {
            if permissions {
                set_permissions(&path, &capabilities)?;
            } else {
                set_capabilities(&path, &capabilities)?;
            }
        }
        Commands::Remove { path, capabilities } => {
            remove_capabilities(&path, &capabilities)?;
        }
        Commands::List { category } => {
            list_capabilities(category.as_deref())?;
        }
        Commands::FromPerms { path } => {
            convert_from_permissions(&path)?;
        }
        Commands::Database => {
            show_database();
        }
        Commands::InitDB => {
            init_database()?;
        }
    }

    Ok(())
}

fn get_capabilities(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", path.display()));
    }

    let metadata = fs::metadata(path)?;
    let mode = metadata.mode();
    let perms: u32 = mode.into();

    println!("File: {}", path.display());
    println!("Type: {}", if path.is_dir() { "Directory" } else { "File" });
    println!("Unix permissions: {:o}", perms & 0o777);

    // Try to read extended capabilities
    let xattr_name = "user.rustica_capabilities";
    match get_xattr(path, xattr_name) {
        Ok(cap_data) => {
            if let Ok(caps) = serde_json::from_str::<FileCapabilities>(&cap_data) {
                println!("Capabilities: {}", caps.capabilities.join(", "));
                println!("Capability level: {}", caps.capability_level);
            } else {
                println!("Extended capabilities: {}", cap_data);
            }
        }
        Err(_) => {
            println!("Extended capabilities: None");
            // Suggest capabilities based on Unix permissions
            suggest_capabilities(perms);
        }
    }

    Ok(())
}

fn set_capabilities(path: &Path, capabilities_str: &str) -> Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", path.display()));
    }

    let db = get_capability_database();
    let caps: Vec<String> = capabilities_str.split(',').map(|s| s.trim().to_string()).collect();

    // Validate capabilities
    for cap in &caps {
        if !db.contains_key(cap) {
            return Err(anyhow::anyhow!("Unknown capability: {}", cap));
        }
    }

    // Calculate capability level
    let mut max_level = 0;
    for cap in &caps {
        if let Some(cap_def) = db.get(cap) {
            if cap_def.required_level > max_level {
                max_level = cap_def.required_level;
            }
        }
    }

    let metadata = fs::metadata(path)?;
    let mode: mode_t = metadata.mode().into();

    let file_caps = FileCapabilities {
        capabilities: caps.clone(),
        permission_bits: mode,
        capability_level: max_level,
    };

    let cap_json = serde_json::to_string_pretty(&file_caps)?;
    set_xattr(path, "user.rustica_capabilities", &cap_json)?;

    println!("Set capabilities on {}", path.display());
    println!("Capabilities: {}", caps.join(", "));
    println!("Capability level: {}", max_level);

    Ok(())
}

fn set_permissions(path: &Path, permissions_str: &str) -> Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", path.display()));
    }

    // Parse Unix permissions (octal)
    let perms = u32::from_str_radix(permissions_str, 8)
        .context("Invalid permissions format (use octal, e.g., 755)")?;

    fs::set_permissions(path, fs::Permissions::from_mode(perms))?;

    println!("Set permissions {:o} on {}", perms, path.display());

    Ok(())
}

fn remove_capabilities(path: &Path, capabilities_str: &str) -> Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", path.display()));
    }

    if capabilities_str == "all" {
        // Remove all extended capabilities
        remove_xattr(path, "user.rustica_capabilities")?;
        println!("Removed all capabilities from {}", path.display());
    } else {
        // Remove specific capabilities
        let xattr_name = "user.rustica_capabilities";
        let cap_data = get_xattr(path, xattr_name)?;

        let mut file_caps: FileCapabilities = serde_json::from_str(&cap_data)
            .context("Failed to parse capabilities")?;

        let caps_to_remove: Vec<String> = capabilities_str.split(',')
            .map(|s| s.trim().to_string())
            .collect();

        file_caps.capabilities.retain(|c| !caps_to_remove.contains(c));

        let cap_json = serde_json::to_string_pretty(&file_caps)?;
        set_xattr(path, xattr_name, &cap_json)?;

        println!("Removed capabilities from {}", path.display());
        println!("Remaining: {}", file_caps.capabilities.join(", "));
    }

    Ok(())
}

fn list_capabilities(category: Option<&str>) -> Result<()> {
    let db = get_capability_database();

    println!("Defined capabilities:");
    println!();

    for (name, cap) in db.iter() {
        if let Some(cat) = category {
            if cap.category != cat {
                continue;
            }
        }

        println!("  {} ({})", name, cap.category);
        println!("    Description: {}", cap.description);
        println!("    Required level: {}", cap.required_level);
        println!("    Unix equivalent: {}", cap.equivalent_perms);
        println!();
    }

    Ok(())
}

fn convert_from_permissions(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", path.display()));
    }

    let metadata = fs::metadata(path)?;
    let mode: mode_t = metadata.mode().into();
    let perms = mode & 0o777;

    println!("File: {}", path.display());
    println!("Unix permissions: {:o}", perms);
    println!();

    // Suggest capabilities based on permissions
    suggest_capabilities(perms);

    Ok(())
}

fn suggest_capabilities(perms: u32) {
    let read = perms & 0o444 != 0;
    let write = perms & 0o222 != 0;
    let execute = perms & 0o111 != 0;

    println!("Suggested capabilities:");

    if read && write && execute {
        println!("  file_full - Full file access");
    } else if read && write {
        println!("  file_read - Read file");
        println!("  file_write - Write file");
    } else if read {
        println!("  file_read - Read file");
    } else if write {
        println!("  file_write - Write file");
    } else if execute {
        println!("  file_execute - Execute file");
    }

    if perms & 0o111 != 0 {
        println!("  file_execute - Execute file");
    }
}

fn show_database() {
    let db = get_capability_database();

    println!("Capability Database:");
    println!();
    println!("Total capabilities: {}", db.len());
    println!();

    let mut categories: std::collections::HashMap<String, Vec<&Capability>> = std::collections::HashMap::new();

    for cap in db.values() {
        categories.entry(cap.category.clone())
            .or_insert_with(Vec::new)
            .push(cap);
    }

    let mut cat_names: Vec<_> = categories.keys().collect();
    cat_names.sort();

    for cat in cat_names {
        println!("{}:", cat);
        if let Some(caps) = categories.get(cat) {
            for cap in caps {
                println!("  - {} (level {})", cap.name, cap.required_level);
            }
        }
        println!();
    }
}

fn init_database() -> Result<()> {
    let config_dir = PathBuf::from("/etc/rustica/capctl");
    let db_file = config_dir.join("capabilities.json");

    fs::create_dir_all(&config_dir)
        .context("Failed to create config directory")?;

    let db = get_capability_database();
    let db_json = serde_json::to_string_pretty(&db)?;

    fs::write(&db_file, db_json)
        .context("Failed to write database file")?;

    println!("Initialized capability database at {}", db_file.display());
    println!("Total capabilities: {}", db.len());

    Ok(())
}

// Extended attributes helper functions
fn get_xattr(path: &Path, name: &str) -> Result<String> {
    let path_cstr = std::ffi::CString::new(path.to_string_lossy().as_ref())?;
    let name_cstr = std::ffi::CString::new(name)?;

    let mut buffer = [0u8; 1024];
    let result = unsafe {
        libc::getxattr(
            path_cstr.as_ptr(),
            name_cstr.as_ptr(),
            buffer.as_mut_ptr() as *mut libc::c_void,
            buffer.len(),
        )
    };

    if result < 0 {
        return Err(anyhow::anyhow!("Failed to get xattr"));
    }

    let data = String::from_utf8_lossy(&buffer[..result as usize]).to_string();
    Ok(data.trim_end_matches('\0').to_string())
}

fn set_xattr(path: &Path, name: &str, data: &str) -> Result<()> {
    let path_cstr = std::ffi::CString::new(path.to_string_lossy().as_ref())?;
    let name_cstr = std::ffi::CString::new(name)?;
    let data_bytes = data.as_bytes();

    let result = unsafe {
        libc::setxattr(
            path_cstr.as_ptr(),
            name_cstr.as_ptr(),
            data_bytes.as_ptr() as *const libc::c_void,
            data_bytes.len(),
            0,
        )
    };

    if result < 0 {
        return Err(anyhow::anyhow!("Failed to set xattr"));
    }

    Ok(())
}

fn remove_xattr(path: &Path, name: &str) -> Result<()> {
    let path_cstr = std::ffi::CString::new(path.to_string_lossy().as_ref())?;
    let name_cstr = std::ffi::CString::new(name)?;

    let result = unsafe {
        libc::removexattr(
            path_cstr.as_ptr(),
            name_cstr.as_ptr(),
        )
    };

    if result < 0 {
        return Err(anyhow::anyhow!("Failed to remove xattr"));
    }

    Ok(())
}
