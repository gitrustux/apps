// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! System Tools for Rustica OS
//!
//! Provides uname, hostname, lsb_release, and free commands.

use anyhow::Result;
use clap::{Parser, Subcommand};
use rutils::{SystemInfo, DistributionInfo};
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(name = "sys-tools")]
#[command(about = "System information tools for Rustica OS", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Print system information (uname replacement)
    #[command(name = "uname")]
    Uname(UnameArgs),
    /// Show or set system hostname
    #[command(name = "hostname")]
    Hostname(HostnameArgs),
    /// Print distribution information (lsb_release replacement)
    #[command(name = "lsb_release")]
    LsbRelease(LsbReleaseArgs),
    /// Display amount of free and used memory
    #[command(name = "free")]
    Free(FreeArgs),
    /// Report file system disk space usage (df replacement)
    #[command(name = "df")]
    Df(DfArgs),
    /// Display system information
    #[command(name = "sysinfo")]
    SysInfo(SysInfoArgs),
}

/// uname arguments
#[derive(Parser, Debug)]
pub struct UnameArgs {
    /// Print the kernel name
    #[arg(short = 's', long = "kernel-name")]
    pub kernel_name: bool,

    /// Print the network node hostname
    #[arg(short = 'n', long = "nodename")]
    pub nodename: bool,

    /// Print the kernel release
    #[arg(short = 'r', long = "kernel-release")]
    pub kernel_release: bool,

    /// Print the kernel version
    #[arg(short = 'v', long = "kernel-version")]
    pub kernel_version: bool,

    /// Print the machine hardware name
    #[arg(short = 'm', long = "machine")]
    pub machine: bool,

    /// Print the processor type
    #[arg(short = 'p', long = "processor")]
    pub processor: bool,

    /// Print the hardware platform
    #[arg(short = 'i', long = "hardware-platform")]
    pub hardware_platform: bool,

    /// Print the operating system name
    #[arg(short = 'o', long = "operating-system")]
    pub operating_system: bool,

    /// Print all information (equivalent to -snrvmpio)
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// Output in JSON format
    #[arg(long = "json")]
    pub json: bool,
}

/// hostname arguments
#[derive(Parser, Debug)]
pub struct HostnameArgs {
    /// Set the hostname
    #[arg(short = 's', long = "set")]
    pub set: Option<String>,

    /// Print the FQDN (Fully Qualified Domain Name)
    #[arg(short = 'f', long = "fqdn")]
    pub fqdn: bool,

    /// Print the IP address
    #[arg(short = 'i', long = "ip-address")]
    pub ip_address: bool,

    /// Print all addresses
    #[arg(short = 'I', long = "all-ip-addresses")]
    pub all_ip_addresses: bool,

    /// Output in JSON format
    #[arg(long = "json")]
    pub json: bool,
}

/// lsb_release arguments
#[derive(Parser, Debug)]
pub struct LsbReleaseArgs {
    /// Print all information
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// Print distributor ID
    #[arg(short = 'i', long = "id", long = "distributor-id")]
    pub distributor_id: bool,

    /// Print distribution description
    #[arg(short = 'd', long = "description")]
    pub description: bool,

    /// Print distribution release
    #[arg(short = 'r', long = "release")]
    pub release: bool,

    /// Print distribution codename
    #[arg(short = 'c', long = "codename")]
    pub codename: bool,

    /// Output in JSON format
    #[arg(long = "json")]
    pub json: bool,
}

/// free arguments
#[derive(Parser, Debug)]
pub struct FreeArgs {
    /// Show output in bytes
    #[arg(short = 'b', long = "bytes")]
    pub bytes: bool,

    /// Show output in kilobytes (default)
    #[arg(short = 'k', long = "kilo")]
    pub kilo: bool,

    /// Show output in megabytes
    #[arg(short = 'm', long = "mega")]
    pub mega: bool,

    /// Show output in gigabytes
    #[arg(short = 'g', long = "giga")]
    pub giga: bool,

    /// Show output in human-readable format
    #[arg(short = 'H', long = "human")]
    pub human: bool,

    /// Show output in powers of 1000 not 1024
    #[arg(short = 'B', long = "si")]
    pub si: bool,

    /// Display low/high memory information
    #[arg(short = 'l', long = "lowhigh")]
    pub lowhigh: bool,

    /// Output in JSON format (default)
    #[arg(short = 'j', long = "json")]
    pub json: bool,

    /// Repeat printing every N seconds
    #[arg(short = 's', long = "seconds")]
    pub seconds: Option<u64>,

    /// Count of times to display
    #[arg(short = 'c', long = "count")]
    pub count: Option<u32>,
}

/// sysinfo arguments
#[derive(Parser, Debug)]
pub struct SysInfoArgs {
    /// Output in JSON format (default)
    #[arg(short = 'j', long = "json")]
    pub json: bool,

    /// Show CPU information
    #[arg(short = 'c', long = "cpu")]
    pub cpu: bool,

    /// Show memory information
    #[arg(short = 'm', long = "memory")]
    pub memory: bool,

    /// Show disk information
    #[arg(short = 'd', long = "disk")]
    pub disk: bool,

    /// Show network information
    #[arg(short = 'n', long = "network")]
    pub network: bool,

    /// Show all information
    #[arg(short = 'a', long = "all")]
    pub all: bool,
}

/// df arguments
#[derive(Parser, Debug)]
pub struct DfArgs {
    /// Show file systems in human-readable format
    #[arg(short = 'H', long = "human-readable")]
    pub human: bool,

    /// Show output in bytes
    #[arg(long = "bytes")]
    pub bytes: bool,

    /// Show output in kilobytes (default)
    #[arg(short = 'k', long = "kilo")]
    pub kilo: bool,

    /// Show output in megabytes
    #[arg(short = 'm', long = "mega")]
    pub mega: bool,

    /// Show output in gigabytes
    #[arg(short = 'g', long = "giga")]
    pub giga: bool,

    /// Show inode information instead of block usage
    #[arg(short = 'i', long = "inodes")]
    pub inodes: bool,

    /// Show all file systems
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// Limit listing to local file systems
    #[arg(short = 'l', long = "local")]
    pub local: bool,

    /// Specify file system type
    #[arg(short = 't', long = "type")]
    pub fs_type: Option<String>,

    /// Exclude file system type
    #[arg(short = 'x', long = "exclude-type")]
    pub exclude_type: Option<String>,

    /// Output in JSON format
    #[arg(short = 'j', long = "json")]
    pub json: bool,
}

#[derive(Serialize)]
struct UnameOutput {
    kernel_name: String,
    nodename: String,
    kernel_release: String,
    kernel_version: String,
    machine: String,
    processor: String,
    hardware_platform: String,
    operating_system: String,
}

#[derive(Serialize)]
struct HostnameOutput {
    hostname: String,
    fqdn: Option<String>,
    ip_address: Option<String>,
    all_ip_addresses: Vec<String>,
}

#[derive(Serialize)]
struct LsbReleaseOutput {
    distributor_id: String,
    description: String,
    release: String,
    codename: String,
}

#[derive(Serialize)]
struct MemoryInfo {
    total: u64,
    used: u64,
    free: u64,
    available: u64,
    swap_total: u64,
    swap_used: u64,
    swap_free: u64,
}

#[derive(Serialize)]
struct FreeOutput {
    mem: MemoryInfo,
    unit: String,
}

#[derive(Serialize)]
struct DiskEntry {
    filesystem: String,
    total: u64,
    used: u64,
    available: u64,
    use_percent: u8,
    mountpoint: String,
    fs_type: String,
}

#[derive(Serialize)]
struct DfOutput {
    entries: Vec<DiskEntry>,
    unit: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Uname(args) => cmd_uname(args),
        Commands::Hostname(args) => cmd_hostname(args),
        Commands::LsbRelease(args) => cmd_lsb_release(args),
        Commands::Free(args) => cmd_free(args),
        Commands::Df(args) => cmd_df(args),
        Commands::SysInfo(args) => cmd_sysinfo(args),
    }
}

pub fn cmd_uname(args: UnameArgs) -> Result<()> {
    let info = SystemInfo::gather()?;

    let output = UnameOutput {
        kernel_name: info.kernel_name.clone(),
        nodename: info.hostname.clone(),
        kernel_release: info.kernel_release.clone(),
        kernel_version: info.kernel_version.clone(),
        machine: info.machine.clone(),
        processor: info.processor.clone(),
        hardware_platform: info.hardware_platform.clone(),
        operating_system: info.operating_system.clone(),
    };

    if args.json {
        let json = serde_json::to_string_pretty(&output)?;
        println!("{}", json);
    } else if !args.all && !args.kernel_name && !args.nodename && !args.kernel_release
        && !args.kernel_version && !args.machine && !args.processor
        && !args.hardware_platform && !args.operating_system {
        // Default: print kernel name only
        println!("{}", output.kernel_name);
    } else {
        let mut parts = Vec::new();
        if args.all || args.kernel_name {
            parts.push(output.kernel_name.as_str());
        }
        if args.all || args.nodename {
            parts.push(output.nodename.as_str());
        }
        if args.all || args.kernel_release {
            parts.push(output.kernel_release.as_str());
        }
        if args.all || args.kernel_version {
            parts.push(output.kernel_version.as_str());
        }
        if args.all || args.machine {
            parts.push(output.machine.as_str());
        }
        if args.all || args.processor {
            parts.push(output.processor.as_str());
        }
        if args.all || args.hardware_platform {
            parts.push(output.hardware_platform.as_str());
        }
        if args.all || args.operating_system {
            parts.push(output.operating_system.as_str());
        }
        println!("{}", parts.join(" "));
    }

    Ok(())
}

pub fn cmd_hostname(args: HostnameArgs) -> Result<()> {
    let info = SystemInfo::gather()?;

    if let Some(new_hostname) = args.set {
        // Set hostname (requires root)
        std::fs::write("/proc/sys/kernel/hostname", format!("{}\n", new_hostname))?;
        println!("Hostname set to: {}", new_hostname);
        return Ok(());
    }

    let output = HostnameOutput {
        hostname: info.hostname.clone(),
        fqdn: None,
        ip_address: None,
        all_ip_addresses: Vec::new(),
    };

    if args.json {
        let json = serde_json::to_string_pretty(&output)?;
        println!("{}", json);
    } else if args.fqdn {
        println!("{}", output.fqdn.as_ref().unwrap_or(&output.hostname));
    } else if args.ip_address {
        println!("{}", output.ip_address.as_ref().unwrap_or(&"127.0.0.1".to_string()));
    } else if args.all_ip_addresses {
        for ip in &output.all_ip_addresses {
            println!("{}", ip);
        }
    } else {
        println!("{}", output.hostname);
    }

    Ok(())
}

pub fn cmd_lsb_release(args: LsbReleaseArgs) -> Result<()> {
    let distro = DistributionInfo::gather()?;

    let output = LsbReleaseOutput {
        distributor_id: distro.distributor_id.clone(),
        description: distro.description.clone(),
        release: distro.release.clone(),
        codename: distro.codename.clone(),
    };

    if args.json {
        let json = serde_json::to_string_pretty(&output)?;
        println!("{}", json);
    } else if args.all || (!args.distributor_id && !args.description && !args.release && !args.codename) {
        println!("Distributor ID:\t{}", output.distributor_id);
        println!("Description:\t{}", output.description);
        println!("Release:\t{}", output.release);
        println!("Codename:\t{}", output.codename);
    } else {
        if args.distributor_id {
            println!("{}", output.distributor_id);
        }
        if args.description {
            println!("{}", output.description);
        }
        if args.release {
            println!("{}", output.release);
        }
        if args.codename {
            println!("{}", output.codename);
        }
    }

    Ok(())
}

pub fn cmd_free(args: FreeArgs) -> Result<()> {
    let info = SystemInfo::gather()?;

    let used = info.total_memory.saturating_sub(info.available_memory);
    let swap_used = info.total_swap.saturating_sub(info.free_swap);

    let memory = MemoryInfo {
        total: info.total_memory,
        used,
        free: info.free_memory,
        available: info.available_memory,
        swap_total: info.total_swap,
        swap_used,
        swap_free: info.free_swap,
    };

    if args.json {
        let unit = if args.giga { "GB" } else if args.mega { "MB" } else if args.bytes { "bytes" } else { "KB" };
        let divisor = if args.giga { 1024 * 1024 * 1024 } else if args.mega { 1024 * 1024 } else if args.bytes { 1 } else { 1024 };

        let scaled = MemoryInfo {
            total: memory.total / divisor,
            used: memory.used / divisor,
            free: memory.free / divisor,
            available: memory.available / divisor,
            swap_total: memory.swap_total / divisor,
            swap_used: memory.swap_used / divisor,
            swap_free: memory.swap_free / divisor,
        };

        let output = FreeOutput {
            mem: scaled,
            unit: unit.to_string(),
        };

        let json = serde_json::to_string_pretty(&output)?;
        println!("{}", json);
    } else {
        // Human-readable table format
        let divisor = if args.giga { 1024.0 * 1024.0 * 1024.0 }
                      else if args.mega { 1024.0 * 1024.0 }
                      else if args.bytes { 1.0 }
                      else if args.si { 1000.0 }
                      else { 1024.0 };

        println!("              total        used        free      available");
        println!("Mem:        {:8.1}  {:8.1}  {:8.1}  {:12.1}",
            memory.total as f64 / divisor,
            memory.used as f64 / divisor,
            memory.free as f64 / divisor,
            memory.available as f64 / divisor
        );
        println!("Swap:       {:8.1}  {:8.1}  {:8.1}",
            memory.swap_total as f64 / divisor,
            memory.swap_used as f64 / divisor,
            memory.swap_free as f64 / divisor
        );
    }

    Ok(())
}

pub fn cmd_sysinfo(args: SysInfoArgs) -> Result<()> {
    let info = SystemInfo::gather()?;

    if args.json || (!args.cpu && !args.memory && !args.disk && !args.network && !args.all) {
        let json = serde_json::to_string_pretty(&info)?;
        println!("{}", json);
    } else {
        if args.all || args.cpu {
            println!("[CPU]");
            println!("  Cores: {}", info.cpu_cores);
            println!();
        }

        if args.all || args.memory {
            println!("[Memory]");
            println!("  Total: {} bytes", info.total_memory);
            println!("  Available: {} bytes", info.available_memory);
            println!("  Free: {} bytes", info.free_memory);
            println!();
        }

        if args.all || args.disk {
            println!("[Disk]");
            println!("  (Not yet implemented)");
            println!();
        }

        if args.all || args.network {
            println!("[Network]");
            println!("  (Not yet implemented)");
            println!();
        }
    }

    Ok(())
}

pub fn cmd_df(args: DfArgs) -> Result<()> {
    // Read mount information from /proc/mounts
    let mounts_content = std::fs::read_to_string("/proc/mounts")
        .map_err(|_| anyhow::anyhow!("Cannot read /proc/mounts"))?;

    let mut entries = Vec::new();

    for line in mounts_content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let device = parts[0];
        let mountpoint = parts[1];
        let fs_type = parts[2];

        // Skip pseudo filesystems unless --all is specified
        if !args.all {
            match fs_type {
                "proc" | "sysfs" | "devtmpfs" | "devpts" | "tmpfs" | "cgroup"
                | "cgroup2" | "pstore" | "configfs" | "debugfs" | "tracefs"
                | "securityfs" | "selinuxfs" | "binfmt_misc" | "fusectl"
                | "fuse.lxcfs" | "mqueue" | "hugetlbfs" | "rpc_pipefs"
                | "autofs" | "nfsd" => continue,
                _ => {}
            }
        }

        // Filter by fs type if specified
        if let Some(ref filter_type) = args.fs_type {
            if fs_type != filter_type {
                continue;
            }
        }

        // Exclude fs type if specified
        if let Some(ref exclude_type) = args.exclude_type {
            if fs_type == exclude_type {
                continue;
            }
        }

        // Skip remote filesystems if --local is specified
        if args.local {
            match fs_type {
                "nfs" | "nfs4" | "cifs" | "smb" | "fuse.sshfs" => continue,
                _ => {}
            }
        }

        // Get disk statistics
        let stat = match nix::sys::statvfs::statvfs(mountpoint) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let block_size = stat.block_size();
        let total_blocks = stat.blocks();
        let free_blocks = stat.blocks_free();

        let total = total_blocks * block_size;
        let available = free_blocks * block_size;
        let used = total.saturating_sub(available);

        let use_percent = if total > 0 {
            ((used as f64 / total as f64) * 100.0) as u8
        } else {
            0
        };

        entries.push(DiskEntry {
            filesystem: device.to_string(),
            total,
            used,
            available,
            use_percent,
            mountpoint: mountpoint.to_string(),
            fs_type: fs_type.to_string(),
        });
    }

    if args.json {
        let unit = if args.giga { "GB" } else if args.mega { "MB" } else if args.bytes { "bytes" } else { "KB" };
        let divisor = if args.giga { 1024 * 1024 * 1024 } else if args.mega { 1024 * 1024 } else if args.bytes { 1 } else { 1024 };

        let scaled_entries: Vec<DiskEntry> = entries
            .iter()
            .map(|e| DiskEntry {
                total: e.total / divisor,
                used: e.used / divisor,
                available: e.available / divisor,
                use_percent: e.use_percent,
                filesystem: e.filesystem.clone(),
                mountpoint: e.mountpoint.clone(),
                fs_type: e.fs_type.clone(),
            })
            .collect();

        let output = DfOutput {
            entries: scaled_entries,
            unit: unit.to_string(),
        };

        let json = serde_json::to_string_pretty(&output)?;
        println!("{}", json);
    } else {
        // Human-readable table format
        let divisor = if args.giga { 1024.0 * 1024.0 * 1024.0 }
                      else if args.mega { 1024.0 * 1024.0 }
                      else if args.bytes { 1.0 }
                      else if args.human { 1024.0 }
                      else { 1024.0 };

        println!("{:<20} {:>10} {:>10} {:>10} {:>5} {:<10}",
            "Filesystem", "Size", "Used", "Avail", "Use%", "Mounted on");

        for entry in &entries {
            println!("{:<20} {:>10.1} {:>10.1} {:>10.1} {:>4}% {:<10}",
                truncate_path(&entry.filesystem, 20),
                entry.total as f64 / divisor,
                entry.used as f64 / divisor,
                entry.available as f64 / divisor,
                entry.use_percent,
                truncate_path(&entry.mountpoint, 10)
            );
        }
    }

    Ok(())
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len() - max_len + 3..])
    }
}
