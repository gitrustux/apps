// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! ping - Send ICMP echo requests

use anyhow::{Context, Result, anyhow};
use clap::Parser;
use netlib::{IcmpPacket, IcmpSocket, PingStats, ICMP_ECHO_REPLY, ICMP_ECHO_REQUEST};
use std::env;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};
use std::process::exit;

/// Ping utility
#[derive(Parser, Debug)]
#[command(name = "ping")]
#[command(about = "Send ICMP ECHO_REQUEST to network hosts", long_about = None)]
struct Args {
    /// Host to ping
    #[arg(required = true)]
    host: String,

    /// Number of pings to send
    #[arg(short, long, default_value_t = 4)]
    count: u32,

    /// Interval between pings (seconds)
    #[arg(short = 'i', long, default_value_t = 1.0)]
    interval: f64,

    /// Timeout for each ping (seconds)
    #[arg(short = 'W', long, default_value_t = 5.0)]
    timeout: f64,

    /// Packet size (bytes)
    #[arg(short, long, default_value_t = 64)]
    size: usize,

    /// Flood mode (send packets as fast as possible)
    #[arg(short = 'f', long)]
    flood: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Resolve hostname
    let addr = resolve_hostname(&args.host)?;

    println!("PING {} ({}) {} bytes of data", args.host, addr, args.size);

    // Create ICMP socket
    let socket = match IcmpSocket::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ping: {} - error: {}", args.host, e);
            eprintln!("Note: ICMP sockets require CAP_NET_RAW capability or root privileges");
            exit(1);
        }
    };

    socket.set_timeout(Duration::from_secs_f64(args.timeout))?;

    // Ping loop
    let mut stats = PingStats::new();
    let mut sequence = 1u16;

    let start_time = Instant::now();

    for _ in 0..args.count {
        let ping_start = Instant::now();

        // Create ICMP packet
        let payload = create_payload(args.size, sequence);
        let icmp_packet = IcmpPacket::new_echo_request(
            (env::var("SHLVL").unwrap_or_default().len() as u16) ^ 0xffff,
            sequence,
            &payload,
        );

        let packet_bytes = icmp_packet.to_bytes();

        // Send packet
        if let Err(e) = socket.send_to(&packet_bytes, &addr) {
            eprintln!("ping: sendto: {}", e);
            continue;
        }

        stats.packets_sent += 1;

        // Receive reply
        let mut recv_buf = vec![0u8; 2048];

        match socket.recv_from(&mut recv_buf) {
            Ok((len, from_addr)) => {
                let rtt = ping_start.elapsed();

                // Parse IP header (20 bytes for IPv4)
                let ip_header_len = 20;
                if len > ip_header_len {
                    let icmp_data = &recv_buf[ip_header_len..len];

                    if let Some(reply) = IcmpPacket::from_bytes(icmp_data) {
                        if reply.is_echo_reply() && reply.header.icmp_seq == sequence {
                            stats.update(rtt.as_secs_f64() * 1000.0);

                            let from_ip = match from_addr {
                                SocketAddr::V4(a) => a.ip().to_string(),
                                _ => from_addr.to_string(),
                            };

                            println!("{} bytes from {}: icmp_seq={} ttl=64 time={:.2} ms",
                                len - ip_header_len, from_ip, sequence, rtt.as_secs_f64() * 1000.0);
                        }
                    }
                }
            }
            Err(e) => {
                if args.verbose {
                    eprintln!("ping: recvmsg: {}", e);
                }
                // Timeout
                println!("From {} icmp_seq={} Destination Host Unreachable", addr, sequence);
            }
        }

        sequence += 1;

        // Wait before next ping
        if !args.flood && u32::from(sequence) <= args.count {
            let elapsed = ping_start.elapsed();
            let wait_time = Duration::from_secs_f64(args.interval);

            if elapsed < wait_time {
                std::thread::sleep(wait_time - elapsed);
            }
        }
    }

    // Print statistics
    print_statistics(&stats, start_time.elapsed());

    // Exit with appropriate code
    if stats.packets_received == 0 {
        exit(1);
    }

    Ok(())
}

fn resolve_hostname(host: &str) -> Result<Ipv4Addr> {
    // Try parsing as IP address first
    if let Ok(addr) = host.parse::<Ipv4Addr>() {
        return Ok(addr);
    }

    // DNS resolution using standard library
    use std::net::ToSocketAddrs;

    // Try to resolve as hostname first
    if let Ok(addrs) = format!("{}:0", host).to_socket_addrs() {
        for addr in addrs {
            if let SocketAddr::V4(v4) = addr {
                return Ok(*v4.ip());
            }
        }
    }

    // If that fails, try parsing as IPv4 address directly
    if let Ok(addr) = host.parse::<Ipv4Addr>() {
        return Ok(addr);
    }

    Err(anyhow::anyhow!("hostname lookup failed: {}", host))
}

fn create_payload(size: usize, sequence: u16) -> Vec<u8> {
    let mut payload = Vec::with_capacity(size);

    // Add timestamp (8 bytes)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64;

    payload.extend_from_slice(&now.to_le_bytes());

    // Fill with pattern
    while payload.len() < size {
        payload.push((payload.len() % 256) as u8);
    }

    payload.truncate(size);
    payload
}

fn print_statistics(stats: &PingStats, total_time: Duration) {
    println!();
    println!("--- {} ping statistics ---", "target");

    let transmitted = stats.packets_sent;
    let received = stats.packets_received;
    let packet_loss = stats.packet_loss_percent();

    println!("{} packets transmitted, {} received, {:.0}% packet loss, time {:.0}ms",
        transmitted, received, packet_loss, total_time.as_secs_f64() * 1000.0);

    if received > 0 {
        println!("rtt min/avg/max/mdev = {:.3}/{:.3}/{:.3}/{:.3} ms",
            stats.min_rtt_ms, stats.avg_rtt_ms(), stats.max_rtt_ms,
            (stats.max_rtt_ms - stats.min_rtt_ms) / 2.0);
    }
}
