// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! rustux-install - Rustica OS Installer
//!
//! Text/CLI installer for Rustica OS with support for:
//! - Language/region selection
//! - Disk detection & partitioning
//! - Network configuration
//! - Device-type detection
//! - Architecture detection
//! - OS profile selection
//! - Base system installation
//! - Bootloader installation

use anyhow::{Context, Result};
use clap::Parser;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Rustica OS Installer
#[derive(Parser, Debug)]
#[command(name = "rustux-install")]
#[command(author = "The Rustux Authors")]
#[command(version = "0.1.0")]
#[command(about = "Rustica OS Installer", long_about = None)]
struct Args {
    /// Automated installation mode
    #[arg(short, long)]
    auto: bool,

    /// Configuration file for automated installation
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Target device (e.g., /dev/sda)
    #[arg(short, long)]
    device: Option<String>,

    /// Skip confirmation prompts
    #[arg(short, long)]
    yes: bool,
}

/// Installer configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct InstallerConfig {
    language: String,
    region: String,
    timezone: String,
    keyboard: String,
    device: String,
    device_type: DeviceType,
    profile: Profile,
    filesystem: FilesystemType,
    hostname: String,
    username: String,
    use_swap: bool,
    use_home_partition: bool,
    network_config: NetworkConfig,
}

/// Device type detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
enum DeviceType {
    Desktop,
    Laptop,
    Mobile,
    Server,
}

/// Installation profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
enum Profile {
    Desktop,
    Laptop,
    Mobile,
    Server,
    Minimal,
}

/// Filesystem type
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
enum FilesystemType {
    Ext4,
    F2FS,  // Flash-Friendly File System for mobile
    Btrfs,
}

/// Network configuration
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
enum NetworkConfig {
    Dhcp,
    Static { ip: String, gateway: String, dns: String },
}

/// Main installation function
fn main() -> Result<()> {
    let args = Args::parse();

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                                                           ║");
    println!("║              Rustica OS Installer v.0.1.0                ║");
    println!("║                                                           ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    // Check if running as root
    if !am_i_root() {
        eprintln!("ERROR: This installer must be run as root.");
        eprintln!("Please run: sudo rustux-install");
        std::process::exit(1);
    }

    let config = if args.auto {
        run_auto_install(args)?
    } else if let Some(config_path) = args.config {
        load_config_from_file(&config_path)?
    } else {
        run_interactive_install(args)?
    };

    // Perform installation
    perform_installation(&config)?;

    println!();
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                                                           ║");
    println!("║           Installation completed successfully!            ║");
    println!("║                                                           ║");
    println!("║           Remove installation media and reboot.           ║");
    println!("║                                                           ║");
    println!("╚═══════════════════════════════════════════════════════════╝");

    Ok(())
}

/// Check if running as root
fn am_i_root() -> bool {
    env::var("USER").unwrap_or_default() == "root"
}

/// Run automated installation
fn run_auto_install(args: Args) -> Result<InstallerConfig> {
    println!("Running in automated mode...");

    let device = args.device.ok_or_else(|| {
        anyhow::anyhow!("Device must be specified in automated mode. Use --device <device>")
    })?;

    let arch = detect_architecture()?;
    println!("Detected architecture: {}", arch);

    let device_type = detect_device_type()?;
    println!("Detected device type: {:?}", device_type);

    // Determine profile based on device type
    let profile = match device_type {
        DeviceType::Desktop => Profile::Desktop,
        DeviceType::Laptop => Profile::Laptop,
        DeviceType::Mobile => Profile::Mobile,
        DeviceType::Server => Profile::Server,
    };

    // Determine filesystem based on profile
    let filesystem = match profile {
        Profile::Mobile => FilesystemType::F2FS,
        Profile::Desktop | Profile::Laptop => FilesystemType::Btrfs,
        _ => FilesystemType::Ext4,
    };

    Ok(InstallerConfig {
        language: "en_US".to_string(),
        region: "US".to_string(),
        timezone: "UTC".to_string(),
        keyboard: "us".to_string(),
        device,
        device_type,
        profile,
        filesystem,
        hostname: "rustux".to_string(),
        username: "user".to_string(),
        use_swap: true,
        use_home_partition: false,
        network_config: NetworkConfig::Dhcp,
    })
}

/// Load configuration from file
fn load_config_from_file(path: &Path) -> Result<InstallerConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config: InstallerConfig = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

    Ok(config)
}

/// Run interactive installation
fn run_interactive_install(args: Args) -> Result<InstallerConfig> {
    println!("Welcome to Rustica OS!");
    println!();

    // Language selection
    println!("Language selection:");
    println!("  1. English (US)");
    println!("  2. English (UK)");
    println!("  3. Spanish");
    println!("  4. French");
    println!("  5. German");
    let lang = prompt_choice("Select language", 1..=5)?;
    let (language, keyboard) = match lang {
        1 => ("en_US", "us"),
        2 => ("en_GB", "uk"),
        3 => ("es_ES", "es"),
        4 => ("fr_FR", "fr"),
        5 => ("de_DE", "de"),
        _ => ("en_US", "us"),
    };

    // Region/Timezone selection
    let timezone = prompt_input("Timezone (default: UTC)", Some("UTC".to_string()))?;

    // Device detection
    println!();
    println!("Detecting storage devices...");
    let devices = list_block_devices()?;
    if devices.is_empty() {
        anyhow::bail!("No block devices found!");
    }

    println!("Available devices:");
    for (i, dev) in devices.iter().enumerate() {
        println!("  {}. {} ({} GB)", i + 1, dev.device, dev.size_gb);
    }

    let device_idx = prompt_choice("Select target device", 1..=devices.len())?;
    let device = devices[device_idx - 1].device.clone();

    // Confirm destructive action
    println!();
    println!("WARNING: This will ERASE ALL DATA on {}!", device);
    if !args.yes {
        let confirm = prompt_yes_no("Continue?")?;
        if !confirm {
            anyhow::bail!("Installation cancelled by user.");
        }
    }

    // Detect device type
    let device_type = detect_device_type()?;
    println!("Detected device type: {:?}", device_type);

    // Profile selection
    println!();
    println!("Installation profile:");
    println!("  1. Desktop (full GUI, productivity apps)");
    println!("  2. Laptop (power-optimized, laptop-specific features)");
    println!("  3. Mobile (touch-optimized, mobile-specific features)");
    println!("  4. Server (minimal, server-oriented)");
    println!("  5. Minimal (base system only)");

    let profile_choice = prompt_choice("Select profile", 1..=5)?;
    let profile = match profile_choice {
        1 => Profile::Desktop,
        2 => Profile::Laptop,
        3 => Profile::Mobile,
        4 => Profile::Server,
        5 => Profile::Minimal,
        _ => Profile::Desktop,
    };

    // Filesystem selection
    println!();
    println!("Filesystem selection:");
    println!("  1. ext4 (stable, compatible)");
    println!("  2. f2fs (Flash-Friendly File System, optimized for mobile/storage)");
    println!("  3. btrfs (advanced features, snapshots)");

    // Recommend filesystem based on profile
    let default_fs = match profile {
        Profile::Mobile => 2,  // F2FS for mobile
        Profile::Desktop | Profile::Laptop => 3,  // Btrfs for desktop/laptop
        _ => 1,  // Ext4 for server/minimal
    };

    println!("  Recommended: {}", match default_fs {
        1 => "ext4",
        2 => "f2fs",
        3 => "btrfs",
        _ => "ext4",
    });

    let fs_choice = prompt_choice_with_default("Select filesystem", 1..=3, default_fs)?;
    let filesystem = match fs_choice {
        1 => FilesystemType::Ext4,
        2 => FilesystemType::F2FS,
        3 => FilesystemType::Btrfs,
        _ => FilesystemType::Ext4,
    };

    // Hostname
    println!();
    let hostname = prompt_input("Hostname", Some("rustux".to_string()))?;

    // User setup
    println!();
    let username = prompt_input("Username", Some("user".to_string()))?;

    // Partitioning options
    println!();
    let use_swap = prompt_yes_no("Create swap partition?")?;
    let use_home = prompt_yes_no("Create separate /home partition?")?;

    // Network configuration
    println!();
    println!("Network configuration:");
    println!("  1. DHCP (automatic)");
    println!("  2. Static IP");
    let net_choice = prompt_choice("Select network type", 1..=2)?;

    let network_config = if net_choice == 2 {
        let ip = prompt_input("IP address", None)?;
        let gateway = prompt_input("Gateway", None)?;
        let dns = prompt_input("DNS server", Some("8.8.8.8".to_string()))?;
        NetworkConfig::Static { ip, gateway, dns }
    } else {
        NetworkConfig::Dhcp
    };

    Ok(InstallerConfig {
        language: language.to_string(),
        region: language[3..5].to_string(),
        timezone,
        keyboard: keyboard.to_string(),
        device,
        device_type,
        profile,
        filesystem,
        hostname,
        username,
        use_swap,
        use_home_partition: use_home,
        network_config,
    })
}

/// Block device information
#[derive(Debug, Clone)]
struct BlockDevice {
    device: String,
    size_gb: f64,
}

/// List available block devices
fn list_block_devices() -> Result<Vec<BlockDevice>> {
    let mut devices = Vec::new();

    // Read from /sys/block
    let sys_block = Path::new("/sys/block");
    if sys_block.exists() {
        for entry in fs::read_dir(sys_block)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Skip loop devices and other special devices
            if name_str.starts_with("loop") || name_str.starts_with("ram") {
                continue;
            }

            let device_path = Path::new("/dev").join(&*name_str);
            if !device_path.exists() {
                continue;
            }

            // Get size
            let size_path = entry.path().join("size");
            if let Ok(size_str) = fs::read_to_string(size_path) {
                if let Ok(sector_count) = size_str.trim().parse::<u64>() {
                    let size_bytes = sector_count * 512;
                    let size_gb = size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

                    devices.push(BlockDevice {
                        device: format!("/dev/{}", name_str),
                        size_gb,
                    });
                }
            }
        }
    }

    Ok(devices)
}

/// Detect system architecture
fn detect_architecture() -> Result<String> {
    let output = Command::new("uname")
        .arg("-m")
        .output()
        .context("Failed to detect architecture")?;

    let arch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let arch_str = match arch.as_str() {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "riscv64" => "riscv64",
        _ => &arch,
    };

    Ok(arch_str.to_string())
}

/// Detect device type
fn detect_device_type() -> Result<DeviceType> {
    // Check for battery (laptop/mobile)
    if Path::new("/sys/class/power_supply/BAT0").exists() {
        // Check for touchscreen (mobile)
        if Path::new("/proc/bus/input/devices").exists() {
            if let Ok(content) = fs::read_to_string("/proc/bus/input/devices") {
                if content.contains("touchscreen") || content.contains("Touchscreen") {
                    return Ok(DeviceType::Mobile);
                }
            }
        }
        return Ok(DeviceType::Laptop);
    }

    // Check if we're in a VM (likely a server/desktop test environment)
    if Path::new("/sys/class/dmi/id/product_name").exists() {
        if let Ok(product) = fs::read_to_string("/sys/class/dmi/id/product_name") {
            let product_lower = product.to_lowercase();
            if product_lower.contains("virtual") || product_lower.contains("qemu") {
                // Default to server for VMs unless specified otherwise
                return Ok(DeviceType::Server);
            }
        }
    }

    // Default to desktop
    Ok(DeviceType::Desktop)
}

/// Prompt for a choice from a range
fn prompt_choice(prompt: &str, range: std::ops::RangeInclusive<usize>) -> Result<usize> {
    loop {
        print!("{} [{}-{}]: ", prompt, range.start(), range.end());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<usize>() {
            Ok(n) if range.contains(&n) => return Ok(n),
            _ => {
                println!("Invalid choice. Please enter a number between {} and {}.", range.start(), range.end());
            }
        }
    }
}

/// Prompt for a choice from a range with a default value
fn prompt_choice_with_default(prompt: &str, range: std::ops::RangeInclusive<usize>, default: usize) -> Result<usize> {
    loop {
        print!("{} [{}-{}] [default={}]: ", prompt, range.start(), range.end(), default);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim();

        if input.is_empty() {
            return Ok(default);
        }

        match input.parse::<usize>() {
            Ok(n) if range.contains(&n) => return Ok(n),
            _ => {
                println!("Invalid choice. Please enter a number between {} and {}, or press Enter for default ({}).",
                    range.start(), range.end(), default);
            }
        }
    }
}

/// Prompt for text input
fn prompt_input(prompt: &str, default: Option<String>) -> Result<String> {
    let default_text = default.as_deref().unwrap_or("");
    print!("{} [{}]: ", prompt, default_text);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim();
    if input.is_empty() && default.is_some() {
        Ok(default.unwrap())
    } else {
        Ok(input.to_string())
    }
}

/// Prompt for yes/no
fn prompt_yes_no(prompt: &str) -> Result<bool> {
    loop {
        print!("{} [y/N]: ", prompt);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" | "" => return Ok(false),
            _ => {
                println!("Please enter 'y' or 'n'.");
            }
        }
    }
}

/// Perform the actual installation
fn perform_installation(config: &InstallerConfig) -> Result<()> {
    println!();
    println!("Starting installation...");
    println!();

    // Step 1: Partition disk
    println!("[1/8] Partitioning disk...");
    partition_disk(config)?;

    // Step 2: Format partitions
    println!("[2/8] Formatting partitions...");
    format_partitions(config)?;

    // Step 3: Mount filesystems
    println!("[3/8] Mounting filesystems...");
    let mount_dir = mount_filesystems(config)?;

    // Step 4: Install base system
    println!("[4/8] Installing base system...");
    install_base_system(&mount_dir, config)?;

    // Step 5: Configure system
    println!("[5/8] Configuring system...");
    configure_system(&mount_dir, config)?;

    // Step 6: Install bootloader
    println!("[6/8] Installing bootloader...");
    install_bootloader(&mount_dir, config)?;

    // Step 7: Create user
    println!("[7/8] Creating user account...");
    create_user_account(&mount_dir, config)?;

    // Step 8: Configure network
    println!("[8/8] Configuring network...");
    configure_network(&mount_dir, config)?;

    // Unmount filesystems
    println!();
    println!("Unmounting filesystems...");
    unmount_filesystems(&mount_dir)?;

    Ok(())
}

/// Partition the disk
fn partition_disk(config: &InstallerConfig) -> Result<()> {
    // Use sfdisk or parted for partitioning
    // For simplicity, we'll use a basic partitioning scheme

    println!("  Creating partition table on {}...", config.device);

    // Create GPT partition table
    let status = Command::new("parted")
        .arg(&config.device)
        .arg("mklabel")
        .arg("gpt")
        .status()?;

    if !status.success() {
        // Fallback: wipe the device
        Command::new("wipefs")
            .arg("-a")
            .arg(&config.device)
            .status()?;
    }

    // EFI boot partition (512MB)
    println!("  Creating EFI boot partition (512MB)...");
    let efi_start = "1MiB";
    let efi_end = "513MiB";

    Command::new("parted")
        .arg(&config.device)
        .arg("mkpart")
        .arg("primary")
        .arg("fat32")
        .arg(efi_start)
        .arg(efi_end)
        .arg("set")
        .arg("1")
        .arg("esp")
        .arg("on")
        .status()
        .context("Failed to create EFI partition")?;

    let mut current_start = 513; // MB

    // Swap partition (optional)
    if config.use_swap {
        println!("  Creating swap partition (4GB)...");
        let swap_start = format!("{}MiB", current_start);
        let swap_end = format!("{}MiB", current_start + 4096);

        Command::new("parted")
            .arg(&config.device)
            .arg("mkpart")
            .arg("primary")
            .arg("linux-swap")
            .arg(&swap_start)
            .arg(&swap_end)
            .status()
            .context("Failed to create swap partition")?;

        current_start += 4096;
    }

    // Root partition
    println!("  Creating root partition...");
    let root_start = format!("{}MiB", current_start);

    if config.use_home_partition {
        // Root partition (50% of remaining space)
        Command::new("parted")
            .arg(&config.device)
            .arg("mkpart")
            .arg("primary")
            .arg("ext4")
            .arg(&root_start)
            .arg("50%")
            .status()
            .context("Failed to create root partition")?;

        // Home partition (remaining space)
        println!("  Creating home partition...");
        Command::new("parted")
            .arg(&config.device)
            .arg("mkpart")
            .arg("primary")
            .arg("ext4")
            .arg("50%")
            .arg("100%")
            .status()
            .context("Failed to create home partition")?;
    } else {
        // Root partition uses all remaining space
        Command::new("parted")
            .arg(&config.device)
            .arg("mkpart")
            .arg("primary")
            .arg("ext4")
            .arg(&root_start)
            .arg("100%")
            .status()
            .context("Failed to create root partition")?;
    }

    Ok(())
}

/// Format partitions
fn format_partitions(config: &InstallerConfig) -> Result<()> {
    // Format EFI partition as FAT32
    println!("  Formatting EFI partition (FAT32)...");
    Command::new("mkfs.fat")
        .arg("-F32")
        .arg(format!("{}1", config.device))
        .status()
        .context("Failed to format EFI partition")?;

    // Format swap partition
    if config.use_swap {
        println!("  Formatting swap partition...");
        Command::new("mkswap")
            .arg(format!("{}2", config.device))
            .status()
            .context("Failed to format swap partition")?;
    }

    // Determine root partition number
    let root_num = if config.use_swap { 3 } else { 2 };

    // Format root partition based on selected filesystem
    let fs_name = match config.filesystem {
        FilesystemType::Ext4 => "ext4",
        FilesystemType::F2FS => "f2fs",
        FilesystemType::Btrfs => "btrfs",
    };
    println!("  Formatting root partition ({})...", fs_name);

    match config.filesystem {
        FilesystemType::Ext4 => {
            Command::new("mkfs.ext4")
                .arg("-F")
                .arg(format!("{}{}", config.device, root_num))
                .status()
                .context("Failed to format root partition")?;
        }
        FilesystemType::F2FS => {
            Command::new("mkfs.f2fs")
                .arg(format!("{}{}", config.device, root_num))
                .status()
                .context("Failed to format root partition with F2FS")?;
        }
        FilesystemType::Btrfs => {
            Command::new("mkfs.btrfs")
                .arg("-f")
                .arg(format!("{}{}", config.device, root_num))
                .status()
                .context("Failed to format root partition with Btrfs")?;
        }
    }

    // Format home partition if it exists
    if config.use_home_partition {
        println!("  Formatting home partition ({})...", fs_name);
        let home_num = root_num + 1;

        match config.filesystem {
            FilesystemType::Ext4 => {
                Command::new("mkfs.ext4")
                    .arg("-F")
                    .arg(format!("{}{}", config.device, home_num))
                    .status()
                    .context("Failed to format home partition")?;
            }
            FilesystemType::F2FS => {
                Command::new("mkfs.f2fs")
                    .arg(format!("{}{}", config.device, home_num))
                    .status()
                    .context("Failed to format home partition with F2FS")?;
            }
            FilesystemType::Btrfs => {
                // For btrfs, use subvolume instead
                println!("    Note: Using btrfs subvolume for /home");
            }
        }
    }

    Ok(())
}

/// Mount filesystems
fn mount_filesystems(config: &InstallerConfig) -> Result<PathBuf> {
    let mount_dir = PathBuf::from("/mnt/rustux");
    fs::create_dir_all(&mount_dir)?;

    let root_num = if config.use_swap { 3 } else { 2 };
    let root_partition = format!("{}{}", config.device, root_num);

    println!("  Mounting root partition...");
    Command::new("mount")
        .arg(&root_partition)
        .arg(&mount_dir)
        .status()
        .context("Failed to mount root partition")?;

    // Mount EFI partition
    let efi_dir = mount_dir.join("boot/efi");
    fs::create_dir_all(&efi_dir)?;

    println!("  Mounting EFI partition...");
    Command::new("mount")
        .arg(format!("{}1", config.device))
        .arg(&efi_dir)
        .status()
        .context("Failed to mount EFI partition")?;

    // Mount home partition if it exists
    if config.use_home_partition {
        let home_dir = mount_dir.join("home");
        fs::create_dir_all(&home_dir)?;

        let home_num = root_num + 1;
        println!("  Mounting home partition...");
        Command::new("mount")
            .arg(format!("{}{}", config.device, home_num))
            .arg(&home_dir)
            .status()
            .context("Failed to mount home partition")?;
    }

    Ok(mount_dir)
}

/// Install base system
fn install_base_system(mount_dir: &Path, config: &InstallerConfig) -> Result<()> {
    // Create directory structure
    println!("  Creating directory structure...");

    for dir in &["bin", "boot", "dev", "etc", "home", "lib", "mnt", "opt", "proc", "root", "run", "sbin", "srv", "sys", "tmp", "usr", "var"] {
        fs::create_dir_all(mount_dir.join(dir))?;
    }

    // Create package manager directories
    let pkg_config = mount_dir.join("etc/rustica/pkg");
    fs::create_dir_all(&pkg_config)?;
    let pkg_cache = mount_dir.join("var/cache/rpg");
    fs::create_dir_all(&pkg_cache)?;
    let pkg_lib = mount_dir.join("var/lib/rpg");
    fs::create_dir_all(&pkg_lib)?;

    // Copy kernel to target
    println!("  Installing kernel...");
    let kernel_source = Path::new("/var/www/rustux.com/prod/kernel/target/x86_64-unknown-none/release/rustux");
    if kernel_source.exists() {
        let kernel_dest = mount_dir.join("boot/rustux");
        fs::copy(kernel_source, &kernel_dest)?;
    } else {
        println!("  Warning: Kernel not found. You'll need to build it first.");
    }

    // Copy essential CLI utilities to bootstrap package manager
    println!("  Installing bootstrap utilities...");
    let apps_target = Path::new("/var/www/rustux.com/prod/apps/target/release");

    for binary in &["rpg", "pkg-compat", "rustux-install"] {
        let src = apps_target.join(binary);
        if src.exists() {
            let dest = mount_dir.join("usr/bin").join(binary);
            fs::copy(&src, &dest)?;
            println!("    Bootstrap: {}", binary);
        }
    }

    // Create package repository configuration
    println!("  Configuring package repositories...");
    let sources_content = r#"# Rustica Package Repositories
# Format: name url [enabled|disabled]

kernel https://rustux.com/repo/kernel enabled
rustica https://rustux.com/repo/rustica enabled
apps https://rustux.com/repo/apps enabled
"#;
    fs::write(pkg_config.join("sources.list"), sources_content)?;

    // Install packages based on profile
    println!("  Installing profile packages...");
    let packages = get_packages_for_profile(config.profile);

    for package in &packages {
        println!("    Installing {}...", package);

        // Try to install from local target if available
        let src = apps_target.join(package);
        if src.exists() {
            let dest = mount_dir.join("usr/bin").join(package);
            fs::copy(&src, &dest)?;

            // Set executable permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&dest)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&dest, perms)?;
            }

            println!("      Installed: {}", package);
        } else {
            println!("      Skipped (not built): {}", package);
        }
    }

    println!("  Installed {} packages for {:?} profile", packages.len(), config.profile);

    Ok(())
}

/// Get package list for a profile
fn get_packages_for_profile(profile: Profile) -> Vec<&'static str> {
    // Essential packages installed on all profiles
    let mut packages = vec![
        "login",
        "svc",
    ];

    match profile {
        Profile::Desktop => {
            packages.extend(vec![
                "ip", "ping", "fwctl",
                "rustux-editor", "rustux-ssh",
                "rustux-dnslookup", "rustux-logview",
                "rustux-apt", "rustux-apt-get",
            ]);
        }
        Profile::Laptop => {
            packages.extend(vec![
                "ip", "ping", "fwctl",
                "rustux-ssh", "rustux-logview",
                "rustux-dnslookup",
            ]);
        }
        Profile::Mobile => {
            packages.extend(vec![
                "ip", "ping",
                "rustux-logview",
            ]);
        }
        Profile::Server => {
            packages.extend(vec![
                "ip", "ping", "fwctl",
                "rustux-ssh", "rustux-logview",
                "rustux-dnslookup",
            ]);
        }
        Profile::Minimal => {
            // Just the essentials
        }
    }

    packages
}

/// Configure system
fn configure_system(mount_dir: &Path, config: &InstallerConfig) -> Result<()> {
    // Create fstab
    println!("  Creating /etc/fstab...");
    let root_num = if config.use_swap { 3 } else { 2 };
    let root_uuid = get_partition_uuid(&format!("{}{}", config.device, root_num))?;
    let efi_uuid = get_partition_uuid(&format!("{}1", config.device))?;

    // Get filesystem type name
    let fs_type = match config.filesystem {
        FilesystemType::Ext4 => "ext4",
        FilesystemType::F2FS => "f2fs",
        FilesystemType::Btrfs => "btrfs",
    };

    // Get filesystem-specific options
    let fs_options = match config.filesystem {
        FilesystemType::Ext4 => "errors=remount-ro",
        FilesystemType::F2FS => "discard,noatime",
        FilesystemType::Btrfs => "defaults,noatime,subvolid=5,subvol=/",
    };

    let mut fstab_content = format!(
        "# /etc/fstab: static file system information\n\
         # <file system> <mount point>   <type>  <options>       <dump>  <pass>\n\
         UUID={}  /              {}    {}        0       1\n\
         UUID={}  /boot/efi      vfat    defaults        0       2\n",
        root_uuid, fs_type, fs_options, efi_uuid
    );

    if config.use_swap {
        let swap_uuid = get_partition_uuid(&format!("{}2", config.device))?;
        fstab_content.push_str(&format!("UUID={}  none           swap    sw              0       0\n", swap_uuid));
    }

    if config.use_home_partition {
        let home_num = root_num + 1;

        match config.filesystem {
            FilesystemType::Btrfs => {
                // For btrfs, use subvolume
                fstab_content.push_str(&format!(
                    "UUID={}  /home          btrfs    subvol=@home,defaults,noatime 0       2\n",
                    root_uuid
                ));
            }
            _ => {
                let home_uuid = get_partition_uuid(&format!("{}{}", config.device, home_num))?;
                let home_opts = match config.filesystem {
                    FilesystemType::F2FS => "discard,noatime",
                    _ => "defaults",
                };
                fstab_content.push_str(&format!("UUID={}  /home          {}    {}        0       2\n",
                    home_uuid, fs_type, home_opts));
            }
        }
    }

    fs::write(mount_dir.join("etc/fstab"), fstab_content)?;

    // Create hostname
    println!("  Setting hostname...");
    fs::write(mount_dir.join("etc/hostname"), &config.hostname)?;

    // Create hosts file
    println!("  Creating /etc/hosts...");
    let hosts_content = format!(
        "127.0.0.1   localhost {}\n\
         ::1         localhost {}\n",
        config.hostname, config.hostname
    );
    fs::write(mount_dir.join("etc/hosts"), hosts_content)?;

    // Set timezone
    println!("  Setting timezone...");
    fs::create_dir_all(mount_dir.join("usr/share/zoneinfo"))?;
    let _timezone_link = mount_dir.join("etc/localtime");
    let _timezone_target = format!("/usr/share/zoneinfo/{}", config.timezone);
    // We'll skip creating the actual symlink for now as the timezone files may not exist

    // Set keyboard layout
    println!("  Setting keyboard layout...");
    fs::write(
        mount_dir.join("etc/vconsole.conf"),
        format!("KEYMAP={}\n", config.keyboard),
    )?;

    // Set locale
    println!("  Setting locale...");
    fs::write(
        mount_dir.join("etc/locale.conf"),
        format!("LANG={}\n", config.language),
    )?;

    Ok(())
}

/// Install bootloader
fn install_bootloader(mount_dir: &Path, config: &InstallerConfig) -> Result<()> {
    println!("  Installing GRUB bootloader...");

    // Install GRUB for UEFI
    let status = Command::new("grub-install")
        .arg("--target=x86_64-efi")
        .arg("--efi-directory")
        .arg(mount_dir.join("boot/efi"))
        .arg("--bootloader-id=Rustica")
        .arg("--recheck")
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("  GRUB installed successfully.");
        }
        _ => {
            println!("  Warning: GRUB installation failed. You may need to install it manually.");
            println!("  Command: grub-install --target=x86_64-efi --efi-directory={}/boot/efi --bootloader-id=Rustica",
                     mount_dir.display());
        }
    }

    // Generate GRUB config
    println!("  Generating GRUB configuration...");
    let grub_cfg = mount_dir.join("boot/grub/grub.cfg");
    fs::create_dir_all(grub_cfg.parent().unwrap())?;

    let grub_content = format!(
        "set timeout=5\n\
         set default=0\n\
         \n\
         menuentry \"Rustica OS v.0.0.1\" {{\n\
             insmod gzio\n\
             insmod part_gpt\n\
             insmod ext2\n\
             set root='hd0,gpt{}'\n\
             echo 'Loading kernel...'\n\
             linux /boot/rustux quiet\n\
             echo 'Booting...'\n\
         }}\n",
        if config.use_swap { 3 } else { 2 }
    );

    fs::write(grub_cfg, grub_content)?;

    Ok(())
}

/// Create user account
fn create_user_account(mount_dir: &Path, config: &InstallerConfig) -> Result<()> {
    println!("  Creating user: {}", config.username);

    // Create home directory
    let user_home = mount_dir.join("home").join(&config.username);
    fs::create_dir_all(&user_home)?;

    // Add user to passwd file
    let passwd_path = mount_dir.join("etc/passwd");
    let mut passwd_content = if passwd_path.exists() {
        fs::read_to_string(&passwd_path)?
    } else {
        String::from("root:x:0:0:root:/root:/bin/sh\n")
    };

    // Get next available UID
    let uid = 1000;
    let gid = 1000;

    passwd_content.push_str(&format!(
        "{}:x:{}:{}:{}:/home/{}:/bin/sh\n",
        config.username, uid, gid, config.username, config.username
    ));

    fs::write(&passwd_path, passwd_content)?;

    // Create group file
    let group_path = mount_dir.join("etc/group");
    let mut group_content = if group_path.exists() {
        fs::read_to_string(&group_path)?
    } else {
        String::from("root:x:0:\n")
    };

    group_content.push_str(&format!(
        "{}:x:{}:{}\n",
        config.username, gid, config.username
    ));

    fs::write(&group_path, group_content)?;

    // Set permissions on home directory
    Command::new("chown")
        .arg("-R")
        .arg(format!("{}:{}", uid, gid))
        .arg(&user_home)
        .status()?;

    Ok(())
}

/// Configure network
fn configure_network(mount_dir: &Path, config: &InstallerConfig) -> Result<()> {
    println!("  Configuring network...");

    match &config.network_config {
        NetworkConfig::Dhcp => {
            // Create network configuration for DHCP
            let network_content = r#"[Match]
Name=en*

[Network]
DHCP=yes
"#;
            fs::write(
                mount_dir.join("etc/systemd/network/20-wired.network"),
                network_content,
            )?;
        }
        NetworkConfig::Static { ip, gateway, dns } => {
            // Create network configuration for static IP
            let network_content = format!(
                r#"[Match]
Name=en*

[Network]
Address={}
Gateway={}
DNS={}
"#,
                ip, gateway, dns
            );
            fs::write(
                mount_dir.join("etc/systemd/network/20-wired.network"),
                network_content,
            )?;
        }
    }

    Ok(())
}

/// Unmount filesystems
fn unmount_filesystems(mount_dir: &Path) -> Result<()> {
    // Unmount home if mounted
    let home_mount = mount_dir.join("home");
    if home_mount.exists() {
        let _ = Command::new("umount")
            .arg(&home_mount)
            .status();
    }

    // Unmount EFI
    let efi_mount = mount_dir.join("boot/efi");
    if efi_mount.exists() {
        let _ = Command::new("umount")
            .arg(&efi_mount)
            .status();
    }

    // Unmount root
    let _ = Command::new("umount")
        .arg(mount_dir)
        .status();

    Ok(())
}

/// Get partition UUID
fn get_partition_uuid(partition: &str) -> Result<String> {
    let output = Command::new("blkid")
        .arg("-s")
        .arg("UUID")
        .arg("-o")
        .arg("value")
        .arg(partition)
        .output()
        .context("Failed to get partition UUID")?;

    let uuid = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if uuid.is_empty() {
        anyhow::bail!("No UUID found for partition {}", partition);
    }

    Ok(uuid)
}
