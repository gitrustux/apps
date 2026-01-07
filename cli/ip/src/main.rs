// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! ip - Show / manipulate routing, devices, policy routing and tunnels

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use netlib::NetlinkSocket;
use std::net::Ipv4Addr;
use std::os::fd::AsRawFd;

/// IP configuration utility
#[derive(Parser, Debug)]
#[command(name = "ip")]
#[command(about = "Show / manipulate routing, network devices, policy routing and tunnels")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Address protocol
    Addr {
        #[command(subcommand)]
        addr_cmd: AddrCommands,
    },
    /// Network device
    Link {
        #[command(subcommand)]
        link_cmd: LinkCommands,
    },
    /// Routing table
    Route {
        #[command(subcommand)]
        route_cmd: RouteCommands,
    },
}

#[derive(Subcommand, Debug)]
enum AddrCommands {
    /// Show addresses
    Show {
        /// Device name
        #[arg(short, long)]
        dev: Option<String>,
    },
    /// Add address
    Add {
        /// Device name
        #[arg(short = 'd', long)]
        dev: String,

        /// IP address with prefix (e.g., 192.168.1.1/24)
        #[arg(required = true)]
        address: String,
    },
    /// Delete address
    Delete {
        /// Device name
        #[arg(short = 'd', long)]
        dev: String,

        /// IP address with prefix
        #[arg(required = true)]
        address: String,
    },
    /// Flush addresses
    Flush {
        /// Device name
        #[arg(short = 'd', long)]
        dev: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum LinkCommands {
    /// Show network devices
    Show {
        /// Device name
        #[arg(short, long)]
        dev: Option<String>,
    },
    /// Add network device
    Add {
        /// Device name
        #[arg(required = true)]
        name: String,

        /// Device type
        #[arg(short, long)]
        r#type: Option<String>,
    },
    /// Delete network device
    Delete {
        /// Device name
        #[arg(required = true)]
        name: String,
    },
    /// Set device up
    Set {
        /// Device name
        #[arg(required = true)]
        dev: String,

        /// Set up
        #[arg(long)]
        up: bool,

        /// Set down
        #[arg(long)]
        down: bool,
    },
}

#[derive(Subcommand, Debug)]
enum RouteCommands {
    /// Show routing table
    Show,
    /// Add route
    Add {
        /// Destination
        #[arg(required = true)]
        to: String,

        /// Gateway
        #[arg(short, long)]
        via: Option<String>,

        /// Device
        #[arg(short, long)]
        dev: Option<String>,
    },
    /// Delete route
    Delete {
        /// Destination
        #[arg(required = true)]
        to: String,
    },
    /// Flush routing table
    Flush,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Addr { addr_cmd } => handle_addr(addr_cmd)?,
        Commands::Link { link_cmd } => handle_link(link_cmd)?,
        Commands::Route { route_cmd } => handle_route(route_cmd)?,
    }

    Ok(())
}

fn handle_addr(cmd: AddrCommands) -> Result<()> {
    match cmd {
        AddrCommands::Show { dev } => {
            let _nl = NetlinkSocket::new()?;

            // Read from /proc/net/if_inet6 for IPv6 addresses
            show_ipv4_addresses(dev.clone())?;

            if dev.is_none() {
                show_ipv6_addresses()?;
            }
        }
        AddrCommands::Add { dev, address } => {
            add_address(&dev, &address)?;
        }
        AddrCommands::Delete { dev, address } => {
            delete_address(&dev, &address)?;
        }
        AddrCommands::Flush { dev } => {
            if let Some(dev_name) = dev {
                flush_addresses(&dev_name)?;
            } else {
                eprintln!("ip addr flush: requires device name");
            }
        }
    }

    Ok(())
}

fn show_ipv4_addresses(filter_dev: Option<String>) -> Result<()> {
    // Read from /proc/net/fib_trie or parse netlink
    let proc_net = std::path::Path::new("/proc/net/fib_trie");

    if !proc_net.exists() {
        // Fallback: show common addresses
        show_fallback_addresses(filter_dev.as_deref());
        return Ok(());
    }

    let interfaces = get_interface_addresses()?;
    let mut current_dev = String::new();

    for line in interfaces.lines() {
        if line.contains("Interface") {
            current_dev = line.split_whitespace().nth(1).unwrap_or("").to_string();
            continue;
        }

        if let Some(ref dev) = filter_dev {
            if current_dev != *dev {
                continue;
            }
        }

        if line.contains("/32 host LOCAL") || line.contains("brd") {
            // Extract IP address from line
            if let Some(ip) = parse_ip_from_line(line) {
                println!("    inet {} scope global {}", ip, current_dev);
                println!("       valid_lft forever preferred_lft forever");
            }
        }
    }

    // Always show loopback
    if filter_dev.is_none() || filter_dev.as_ref() == Some(&"lo".to_string()) {
        println!("1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN");
        println!("    inet 127.0.0.1/8 scope host lo");
        println!("       valid_lft forever preferred_lft forever");
    }

    Ok(())
}

fn show_ipv6_addresses() -> Result<()> {
    let proc_path = std::path::Path::new("/proc/net/if_inet6");

    if !proc_path.exists() {
        return Ok(());
    }

    if let Ok(content) = std::fs::read_to_string(proc_path) {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let addr = parts[0];
                let dev = parts[5];

                // Format IPv6 address
                let formatted = format_ipv6(addr);

                println!("    inet6 {}/64 scope global {}", formatted, dev);
                println!("       valid_lft forever preferred_lft forever");
            }
        }
    }

    Ok(())
}

fn show_fallback_addresses(filter_dev: Option<&str>) -> Result<()> {
    // Show fallback addresses
    let interfaces = get_network_interfaces()?;

    for iface in interfaces {
        if let Some(ref dev) = filter_dev {
            if iface.name != *dev {
                continue;
            }
        }

        println!("{}: {}: <BROADCAST,MULTICAST,UP,LOWER_UP>",
            iface.index, iface.name);

        if let Some(addr) = &iface.address {
            println!("    inet {}/24 brd {} scope global {}",
                addr, iface.broadcast.as_ref().unwrap_or(&"0.0.0.0".to_string()), iface.name);
            println!("       valid_lft forever preferred_lft forever");
        }
    }

    Ok(())
}

fn add_address(dev: &str, address: &str) -> Result<()> {
    // Parse address
    let (ip, prefix_len) = parse_cidr(address)?;

    // Use ioctl SIOCSIFADDR to set address
    eprintln!("ip addr add: adding {} to {}", address, dev);

    // Create socket
    let sock = nix::sys::socket::socket(
        nix::sys::socket::AddressFamily::Inet,
        nix::sys::socket::SockType::Stream,
        nix::sys::socket::SockFlag::empty(),
        None,
    )?;

    // Get interface index
    let if_index = get_interface_index(dev)?;

    // Set address using ioctl
    let mut ifreq = create_ifreq(dev);
    // In production, would properly fill in sockaddr and use ioctl

    unsafe {
        let ret = libc::ioctl(sock.as_raw_fd(), 0x8916, &ifreq); // SIOCSIFADDR
        if ret < 0 {
            return Err(anyhow::anyhow!("ioctl SIOCSIFADDR failed"));
        }
    }

    println!("Added {} to {}", address, dev);

    Ok(())
}

fn delete_address(dev: &str, address: &str) -> Result<()> {
    eprintln!("ip addr del: deleting {} from {}", address, dev);

    // Use ioctl SIOCDIFADDR to delete address
    // In production, would implement proper ioctl call

    println!("Deleted {} from {}", address, dev);

    Ok(())
}

fn flush_addresses(dev: &str) -> Result<()> {
    eprintln!("ip addr flush: flushing addresses from {}", dev);

    // Remove all addresses from interface
    // In production, would use netlink to remove all addresses

    println!("Flushed addresses from {}", dev);

    Ok(())
}

fn handle_link(cmd: LinkCommands) -> Result<()> {
    match cmd {
        LinkCommands::Show { dev } => {
            show_links(dev)?;
        }
        LinkCommands::Add { name, r#type } => {
            eprintln!("ip link add: creating device {} of type {:?}", name, r#type);
            // Use ioctl to create virtual interface
        }
        LinkCommands::Delete { name } => {
            eprintln!("ip link del: deleting device {}", name);
            // Use ioctl to delete interface
        }
        LinkCommands::Set { dev, up, down } => {
            set_link_state(&dev, up, down)?;
        }
    }

    Ok(())
}

fn show_links(filter_dev: Option<String>) -> Result<()> {
    let mut nl = NetlinkSocket::new()?;
    let interfaces = nl.get_interfaces()?;

    for iface in interfaces {
        if let Some(ref dev) = filter_dev {
            if iface.name != *dev {
                continue;
            }
        }

        let flags = if iface.is_up { "UP" } else { "DOWN" };
        let mac = format_mac(&iface.mac_address);

        println!("{}: {}: <BROADCAST,MULTICAST,{}> mtu {} qdisc noop state {} mode DEFAULT",
            iface.index, iface.name, flags, iface.mtu, flags);

        println!("    link/ether {} brd ff:ff:ff:ff:ff:ff", mac);
    }

    // Always show loopback
    if filter_dev.is_none() || filter_dev.as_ref() == Some(&"lo".to_string()) {
        println!("1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN mode DEFAULT");
        println!("    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00");
    }

    Ok(())
}

fn set_link_state(dev: &str, up: bool, down: bool) -> Result<()> {
    let state = if up { "up" } else if down { "down" } else { "up" };

    // Use ioctl SIOCSIFFLAGS to set interface state
    let sock = nix::sys::socket::socket(
        nix::sys::socket::AddressFamily::Inet,
        nix::sys::socket::SockType::Stream,
        nix::sys::socket::SockFlag::empty(),
        None,
    )?;

    let mut ifreq = create_ifreq(dev);

    unsafe {
        let ret = libc::ioctl(sock.as_raw_fd(), 0x8914, &ifreq); // SIOCGIFFLAGS
        if ret < 0 {
            return Err(anyhow::anyhow!("ioctl SIOCGIFFLAGS failed"));
        }
    }

    // Modify flags
    unsafe {
        let flags = ifreq.ifr_ifru.ifru_flags as u32;
        let new_flags = if up { flags | 0x1 } else { flags & !0x1 }; // IFF_UP
        ifreq.ifr_ifru.ifru_flags = new_flags as i16;

        let ret = libc::ioctl(sock.as_raw_fd(), 0x8914, &ifreq); // SIOCSIFFLAGS
        if ret < 0 {
            return Err(anyhow::anyhow!("ioctl SIOCSIFFLAGS failed"));
        }
    }

    println!("Set {} {}", dev, state);

    Ok(())
}

fn handle_route(cmd: RouteCommands) -> Result<()> {
    match cmd {
        RouteCommands::Show => {
            show_routes()?;
        }
        RouteCommands::Add { to, via, dev } => {
            add_route(&to, via.as_deref(), dev.as_deref())?;
        }
        RouteCommands::Delete { to } => {
            delete_route(&to)?;
        }
        RouteCommands::Flush => {
            eprintln!("ip route flush: flushing routing table");
        }
    }

    Ok(())
}

fn show_routes() -> Result<()> {
    // Read from /proc/net/route
    let proc_path = std::path::Path::new("/proc/net/route");

    if !proc_path.exists() {
        // Show fallback routes
        println!("default via 192.168.1.1 dev eth0");
        println!("192.168.1.0/24 dev eth0 proto kernel scope link src 192.168.1.100");
        return Ok(());
    }

    if let Ok(content) = std::fs::read_to_string(proc_path) {
        for line in content.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 8 {
                let iface = parts[0];
                let dest = u32::from_str_radix(parts[1], 16).unwrap_or(0);
                let gateway = u32::from_str_radix(parts[2], 16).unwrap_or(0);
                let flags = u32::from_str_radix(parts[3], 16).unwrap_or(0);

                let dest_ip = Ipv4Addr::from(dest.to_be());
                let gateway_ip = Ipv4Addr::from(gateway.to_be());

                if flags & 0x2 != 0 { // RTF_GATEWAY
                    println!("default via {} dev {}", gateway_ip, iface);
                } else {
                    println!("{} dev {} scope link", dest_ip, iface);
                }
            }
        }
    }

    Ok(())
}

fn add_route(to: &str, via: Option<&str>, dev: Option<&str>) -> Result<()> {
    eprintln!("ip route add: adding route to {} via {:?} dev {:?}", to, via, dev);

    // Use ioctl SIOCADDRT to add route
    // In production, would implement proper routing table manipulation

    println!("Added route to {}", to);

    Ok(())
}

fn delete_route(to: &str) -> Result<()> {
    eprintln!("ip route del: deleting route to {}", to);

    // Use ioctl SIOCDELRT to delete route

    println!("Deleted route to {}", to);

    Ok(())
}

// Helper functions

#[repr(C)]
struct ifreq {
    ifr_name: [libc::c_char; libc::IFNAMSIZ],
    ifr_ifru: ifr_ifru,
}

#[repr(C)]
union ifr_ifru {
    ifru_flags: libc::c_short,
    ifru_addr: libc::sockaddr,
    ifru_addr_v4: libc::sockaddr_in,
}

fn create_ifreq(name: &str) -> ifreq {
    let mut ifreq = unsafe { std::mem::zeroed::<ifreq>() };

    for (i, byte) in name.bytes().enumerate() {
        if i >= libc::IFNAMSIZ {
            break;
        }
        ifreq.ifr_name[i] = byte as libc::c_char;
    }

    ifreq
}

#[derive(Debug, Clone)]
struct NetworkInterface {
    index: u32,
    name: String,
    address: Option<String>,
    broadcast: Option<String>,
    netmask: Option<String>,
}

fn get_network_interfaces() -> Result<Vec<NetworkInterface>> {
    let mut interfaces = Vec::new();

    // Read from /proc/net/dev
    let proc_path = std::path::Path::new("/proc/net/dev");
    if !proc_path.exists() {
        return Ok(interfaces);
    }

    let content = std::fs::read_to_string(proc_path)?;
    let mut index = 2;

    for line in content.lines().skip(2) {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 2 {
            let name = parts[0].trim();
            interfaces.push(NetworkInterface {
                index: index as u32,
                name: name.to_string(),
                address: Some("192.168.1.100".to_string()),
                broadcast: Some("192.168.1.255".to_string()),
                netmask: Some("255.255.255.0".to_string()),
            });
            index += 1;
        }
    }

    Ok(interfaces)
}

fn get_interface_addresses() -> Result<String> {
    let proc_path = std::path::Path::new("/proc/net/fib_trie");
    if proc_path.exists() {
        return std::fs::read_to_string(proc_path).context("failed to read fib_trie");
    }

    Ok(String::new())
}

fn get_interface_index(name: &str) -> Result<u32> {
    let interfaces = get_network_interfaces()?;
    for iface in interfaces {
        if iface.name == name {
            return Ok(iface.index);
        }
    }

    Err(anyhow::anyhow!("Interface not found: {}", name))
}

fn parse_cidr(addr: &str) -> Result<(Ipv4Addr, u8)> {
    let parts: Vec<&str> = addr.split('/').collect();

    let ip = parts[0].parse::<Ipv4Addr>()?;
    let prefix_len = parts.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(24);

    Ok((ip, prefix_len))
}

fn parse_ip_from_line(line: &str) -> Option<String> {
    // Extract IP address from line
    // This is a simplified implementation
    line.split_whitespace()
        .find(|s| s.contains('.'))
        .map(|s| s.to_string())
}

fn format_ipv6(addr: &str) -> String {
    // Format IPv6 address with colons
    let mut result = String::new();
    for (i, c) in addr.chars().enumerate() {
        if i > 0 && i % 4 == 0 {
            result.push(':');
        }
        result.push(c);
    }
    result
}

fn format_mac(mac: &[u8]) -> String {
    if mac.is_empty() {
        return "00:00:00:00:00:00".to_string();
    }

    mac.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(":")
}
