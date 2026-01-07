// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! dnslookup - DNS Lookup Utility
//!
//! Query DNS information for domains using system resolver.

use anyhow::{Context, Result};
use clap::Parser;
use std::net::{SocketAddr, ToSocketAddrs};

/// DNS Lookup Utility
#[derive(Parser, Debug)]
#[command(name = "dnslookup")]
#[command(author = "The Rustux Authors")]
#[command(version = "0.1.0")]
#[command(about = "DNS Lookup Utility", long_about = None)]
struct Args {
    /// Domain name to lookup
    domain: String,

    /// DNS record type (A, AAAA, MX, TXT, NS, ANY)
    #[arg(short = 't', long, default_value = "A")]
    query_type: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        println!("Looking up {} records for {}...\n", args.query_type, args.domain);
    }

    match args.query_type.to_uppercase().as_str() {
        "A" => lookup_a(&args.domain)?,
        "AAAA" => lookup_aaaa(&args.domain)?,
        "MX" | "NS" | "TXT" => {
            println!("Note: MX, NS, and TXT record lookups require a DNS server.");
            println!("Using system resolver for basic lookup...");
            lookup_any(&args.domain)?;
        }
        "ANY" => lookup_any(&args.domain)?,
        _ => {
            eprintln!("Unknown record type: {}", args.query_type);
            eprintln!("Supported types: A, AAAA, MX, NS, TXT, ANY");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Lookup A records (IPv4 addresses)
fn lookup_a(domain: &str) -> Result<()> {
    println!("A RECORDS:");
    println!("-----------");

    // Try to resolve using system resolver
    let addrs: Vec<SocketAddr> = format!("{}:0", domain)
        .to_socket_addrs()
        .with_context(|| format!("Failed to resolve domain: {}", domain))?
        .collect();

    let mut found = false;
    for addr in addrs {
        if let SocketAddr::V4(v4_addr) = addr {
            println!("  {}", v4_addr.ip());
            found = true;
        }
    }

    if !found {
        println!("  No A records found");
    }

    Ok(())
}

/// Lookup AAAA records (IPv6 addresses)
fn lookup_aaaa(domain: &str) -> Result<()> {
    println!("AAAA RECORDS:");
    println!("--------------");

    // Try to resolve using system resolver
    let addrs: Vec<SocketAddr> = format!("{}:0", domain)
        .to_socket_addrs()
        .with_context(|| format!("Failed to resolve domain: {}", domain))?
        .collect();

    let mut found = false;
    for addr in addrs {
        if let SocketAddr::V6(v6_addr) = addr {
            println!("  {}", v6_addr.ip());
            found = true;
        }
    }

    if !found {
        println!("  No AAAA records found");
    }

    Ok(())
}

/// Lookup any records using system resolver
fn lookup_any(domain: &str) -> Result<()> {
    println!("DNS LOOKUP RESULTS:");
    println!("-------------------");

    // Try to resolve using system resolver
    let addrs: Vec<SocketAddr> = format!("{}:0", domain)
        .to_socket_addrs()
        .with_context(|| format!("Failed to resolve domain: {}", domain))?
        .collect();

    if addrs.is_empty() {
        println!("  No records found");
        return Ok(());
    }

    println!("Domain: {}", domain);
    println!("Addresses:");
    for addr in addrs {
        match addr {
            SocketAddr::V4(v4) => println!("  A  - {}", v4.ip()),
            SocketAddr::V6(v6) => println!("  AAAA - {}", v6.ip()),
        }
    }

    Ok(())
}
