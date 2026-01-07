// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! bootctl - UEFI Boot Entry Management Utility
//!
//! Manage UEFI boot entries for multi-boot configurations.
//! Provides functionality for listing, adding, removing, and managing
//! boot loader entries in the UEFI firmware.

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// UEFI Boot Entry Management Utility
#[derive(Parser, Debug)]
#[command(name = "rustux-bootctl")]
#[command(author = "The Rustux Authors")]
#[command(version = "0.1.0")]
#[command(about = "UEFI boot entry management utility", long_about = None)]
struct BootctlArgs {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all boot entries
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Set boot order
    SetOrder {
        /// Boot order (comma-separated entry numbers)
        #[arg(required = true)]
        order: String,
    },

    /// Add a new boot entry
    Add {
        /// Entry title/name
        #[arg(short, long)]
        title: String,

        /// EFI executable path
        #[arg(short, long)]
        efi: PathBuf,

        /// ESP mount point
        #[arg(short, long, default_value = "/boot/efi")]
        esp: PathBuf,
    },

    /// Remove a boot entry
    Remove {
        /// Entry number (XXXX) to remove
        #[arg(required = true)]
        entry: String,
    },

    /// Set the next boot entry
    SetNextBoot {
        /// Entry number (XXXX) to boot next
        #[arg(required = true)]
        entry: String,
    },

    /// Detect other operating systems
    DetectOs {
        /// ESP mount point
        #[arg(short, long, default_value = "/boot/efi")]
        esp: PathBuf,
    },

    /// Export boot entries as JSON
    Export {
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Show boot manager status
    Status,
}

/// Boot entry information
#[derive(Debug, Clone, serde::Serialize)]
struct BootEntry {
    number: String,
    title: String,
    active: bool,
    efi_path: Option<String>,
}

fn main() -> Result<()> {
    let args = BootctlArgs::parse();

    // Check if running as root for commands that require it
    match &args.command {
        Commands::SetOrder { .. } | Commands::Add { .. } | Commands::Remove { .. } | Commands::SetNextBoot { .. } => {
            if !am_i_root() {
                eprintln!("ERROR: This command requires root privileges.");
                eprintln!("Please run: sudo rustux-bootctl <command>");
                std::process::exit(1);
            }
        }
        _ => {}
    }

    match args.command {
        Commands::List { verbose } => {
            list_boot_entries(verbose)?;
        }
        Commands::SetOrder { order } => {
            set_boot_order(&order)?;
        }
        Commands::Add { title, efi, esp } => {
            add_boot_entry(&title, &efi, &esp)?;
        }
        Commands::Remove { entry } => {
            remove_boot_entry(&entry)?;
        }
        Commands::SetNextBoot { entry } => {
            set_next_boot(&entry)?;
        }
        Commands::DetectOs { esp } => {
            detect_other_os(&esp)?;
        }
        Commands::Export { output } => {
            export_boot_entries(output.as_deref())?;
        }
        Commands::Status => {
            show_boot_status()?;
        }
    }

    Ok(())
}

/// Check if running as root
fn am_i_root() -> bool {
    std::env::var("USER").unwrap_or_default() == "root"
}

/// List all boot entries
fn list_boot_entries(verbose: bool) -> Result<()> {
    println!("UEFI Boot Entries:");
    println!();

    let entries = get_boot_entries()?;

    if entries.is_empty() {
        println!("  No boot entries found.");
        return Ok(());
    }

    for entry in &entries {
        let status = if entry.active { "[ACTIVE]" } else { "" };
        println!("  Boot{} - {} {}", entry.number, entry.title, status);

        if verbose {
            if let Some(efi) = &entry.efi_path {
                println!("    EFI: {}", efi);
            }
        }
    }

    // Show current boot order
    if let Ok(order) = get_boot_order() {
        println!();
        println!("Boot Order: {}", order.join(", "));
    }

    Ok(())
}

/// Get all boot entries from efibootmgr
fn get_boot_entries() -> Result<Vec<BootEntry>> {
    let output = Command::new("efibootmgr")
        .output()
        .context("efibootmgr not found. Please install efibootmgr.")?;

    if !output.status.success() {
        return Err(anyhow!("efibootmgr failed"));
    }

    let content = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();

    for line in content.lines() {
        if line.contains("Boot") && line.contains("*") {
            // Parse boot entry line
            // Format: BootXXXX (title)   ...
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let boot_str = parts[0];
                if let Some(entry_num) = boot_str.strip_prefix("Boot").and_then(|s| s.strip_suffix('*')) {
                    let title = parts.get(1).unwrap_or(&"Unknown").to_string();
                    let active = boot_str.ends_with('*');

                    // Extract EFI path if available
                    let efi_path = line.split("HD(").nth(1)
                        .and_then(|s| s.split(")/").nth(1))
                        .map(|s| format!("\\EFI\\{}", s.replace('/', "\\")));

                    entries.push(BootEntry {
                        number: entry_num.to_string(),
                        title,
                        active,
                        efi_path,
                    });
                }
            }
        }
    }

    Ok(entries)
}

/// Get current boot order
fn get_boot_order() -> Result<Vec<String>> {
    let output = Command::new("efibootmgr")
        .output()
        .context("efibootmgr not found")?;

    if !output.status.success() {
        return Err(anyhow!("efibootmgr failed"));
    }

    let content = String::from_utf8_lossy(&output.stdout);

    for line in content.lines() {
        if line.contains("BootOrder:") {
            let order_str = line.split(':').nth(1).unwrap_or("");
            return Ok(order_str.split(',').map(|s| s.trim().to_string()).collect());
        }
    }

    Ok(vec![])
}

/// Set boot order
fn set_boot_order(order: &str) -> Result<()> {
    println!("Setting boot order to: {}", order);

    let status = Command::new("efibootmgr")
        .arg("--bootorder")
        .arg(order)
        .status()
        .context("Failed to set boot order")?;

    if status.success() {
        println!("Boot order updated successfully.");
    } else {
        return Err(anyhow!("Failed to set boot order"));
    }

    Ok(())
}

/// Add a new boot entry
fn add_boot_entry(title: &str, efi: &Path, esp: &Path) -> Result<()> {
    println!("Adding boot entry: {}", title);
    println!("  EFI: {}", efi.display());
    println!("  ESP: {}", esp.display());

    // Verify EFI file exists
    let efi_path = if efi.is_absolute() {
        efi.to_path_buf()
    } else {
        esp.join(efi)
    };

    if !efi_path.exists() {
        return Err(anyhow!("EFI file not found: {}", efi_path.display()));
    }

    // Create the boot entry using efibootmgr
    let disk = get_disk_from_esp(esp)?;
    let part = get_partition_from_esp(esp)?;

    // Create loader path relative to ESP
    let loader = efi.strip_prefix(esp)
        .unwrap_or(efi)
        .to_string_lossy()
        .replace('/', "\\");

    let status = Command::new("efibootmgr")
        .arg("--create")
        .arg("--disk")
        .arg(&disk)
        .arg("--part")
        .arg(&part)
        .arg("--label")
        .arg(title)
        .arg("--loader")
        .arg(&loader)
        .status()
        .context("Failed to create boot entry")?;

    if status.success() {
        println!("Boot entry created successfully.");
    } else {
        return Err(anyhow!("Failed to create boot entry"));
    }

    Ok(())
}

/// Get disk device from ESP mount point
fn get_disk_from_esp(esp: &Path) -> Result<String> {
    // Find mount point info
    let output = Command::new("findmnt")
        .arg("-n")
        .arg("-o")
        .arg("SOURCE")
        .arg(esp)
        .output()
        .context("findmnt not found")?;

    if !output.status.success() {
        return Err(anyhow!("Failed to find mount info"));
    }

    let source = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Extract disk (e.g., /dev/sda1 -> /dev/sda)
    if let Some(disk) = source.strip_suffix(|c: char| c.is_ascii_digit()) {
        Ok(disk.to_string())
    } else {
        Ok(source)
    }
}

/// Get partition number from ESP mount point
fn get_partition_from_esp(esp: &Path) -> Result<String> {
    let output = Command::new("findmnt")
        .arg("-n")
        .arg("-o")
        .arg("SOURCE")
        .arg(esp)
        .output()
        .context("findmnt not found")?;

    if !output.status.success() {
        return Err(anyhow!("Failed to find mount info"));
    }

    let source = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Extract partition number (e.g., /dev/sda1 -> 1)
    source.chars()
        .last()
        .and_then(|c| c.to_digit(10))
        .map(|n| n.to_string())
        .ok_or_else(|| anyhow!("Failed to determine partition number"))
}

/// Remove a boot entry
fn remove_boot_entry(entry: &str) -> Result<()> {
    println!("Removing boot entry: Boot{}", entry);

    let status = Command::new("efibootmgr")
        .arg("--bootnum")
        .arg(format!("{}{}", "Boot", entry))
        .arg("--delete-bootnum")
        .status()
        .context("Failed to remove boot entry")?;

    if status.success() {
        println!("Boot entry removed successfully.");
    } else {
        return Err(anyhow!("Failed to remove boot entry"));
    }

    Ok(())
}

/// Set the next boot entry (one-time boot)
fn set_next_boot(entry: &str) -> Result<()> {
    println!("Setting next boot to: Boot{}", entry);

    let status = Command::new("efibootmgr")
        .arg("--bootnext")
        .arg(entry)
        .status()
        .context("Failed to set next boot")?;

    if status.success() {
        println!("Next boot set to Boot{}", entry);
    } else {
        return Err(anyhow!("Failed to set next boot"));
    }

    Ok(())
}

/// Detect other operating systems
fn detect_other_os(esp: &Path) -> Result<()> {
    println!("Detecting other operating systems...");
    println!();

    let efi_dir = esp.join("EFI");

    if !efi_dir.exists() {
        println!("ESP not found at {}", efi_dir.display());
        return Ok(());
    }

    let mut detected = Vec::new();

    // Check for common bootloaders
    let bootloaders = [
        ("Microsoft", "Windows Boot Manager", vec!["Microsoft/Boot/bootmgfw.efi"]),
        ("ubuntu", "Ubuntu", vec!["ubuntu/grubx64.efi", "ubuntu/shimx64.efi"]),
        ("fedora", "Fedora", vec!["fedora/shimx64.efi"]),
        ("debian", "Debian", vec!["debian/grubx64.efi"]),
        ("arch", "Arch Linux", vec!["arch/grubx64.efi"]),
        ("opensuse", "openSUSE", vec!["opensuse/shimx64.efi"]),
    ];

    for (dir, name, paths) in bootloaders {
        for path in paths {
            let efi_path = efi_dir.join(path);
            if efi_path.exists() {
                detected.push((name.to_string(), efi_path));
                break;
            }
        }
    }

    if detected.is_empty() {
        println!("No other operating systems detected.");
    } else {
        println!("Detected operating systems:");
        for (name, efi_path) in &detected {
            println!("  - {}", name);
            println!("    EFI: {}", efi_path.display());
        }
        println!();
        println!("To add a detected OS, use:");
        for (name, efi_path) in &detected {
            let relative_path = efi_path.strip_prefix(esp).unwrap_or(efi_path);
            println!("  sudo rustux-bootctl add --title '{}' --efi {}",
                name, relative_path.display());
        }
    }

    Ok(())
}

/// Export boot entries as JSON
fn export_boot_entries(output: Option<&Path>) -> Result<()> {
    let entries = get_boot_entries()?;

    let json = serde_json::to_string_pretty(&entries)?;

    match output {
        Some(path) => {
            fs::write(path, &json)?;
            println!("Exported {} entries to {}", entries.len(), path.display());
        }
        None => {
            println!("{}", json);
        }
    }

    Ok(())
}

/// Show boot manager status
fn show_boot_status() -> Result<()> {
    println!("UEFI Boot Manager Status:");
    println!();

    // Check if UEFI is being used
    let efivars = Path::new("/sys/firmware/efi/efivars");
    if !efivars.exists() {
        println!("  Boot mode: Legacy BIOS (not UEFI)");
        println!("  Note: This tool requires UEFI boot mode.");
        return Ok(());
    }

    println!("  Boot mode: UEFI");
    println!();

    // Check if secure boot is enabled
    let secure_boot_var = efivars.join("SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c");
    if secure_boot_var.exists() {
        if let Ok(data) = fs::read(&secure_boot_var) {
            if data.len() >= 5 {
                let enabled = data.get(4).map(|&b| b == 1).unwrap_or(false);
                println!("  Secure Boot: {}", if enabled { "Enabled" } else { "Disabled" });
            }
        }
    } else {
        println!("  Secure Boot: Not supported");
    }

    println!();

    // Show boot timeout
    let timeout_var = efivars.join("Timeout-8be4df61-93ca-11d2-aa0d-00e098032b8c");
    if timeout_var.exists() {
        if let Ok(data) = fs::read(&timeout_var) {
            if data.len() >= 5 {
                let timeout = data.get(4).map(|&b| b as u16 * 2).unwrap_or(0);
                println!("  Boot timeout: {} seconds", timeout);
            }
        }
    }

    println!();

    // Count boot entries
    let entries = get_boot_entries()?;
    let active_count = entries.iter().filter(|e| e.active).count();
    println!("  Total boot entries: {}", entries.len());
    println!("  Active entries: {}", active_count);

    if let Ok(order) = get_boot_order() {
        println!("  Boot order size: {}", order.len());
    }

    Ok(())
}
