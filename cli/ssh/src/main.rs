// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! ssh - SSH Client Utility
//!
//! Secure Shell client for remote connections.

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

/// SSH Client
#[derive(Parser, Debug)]
#[command(name = "ssh")]
#[command(author = "The Rustux Authors")]
#[command(version = "0.1.0")]
#[command(about = "Rustica SSH Client", long_about = None)]
struct Args {
    /// Remote host to connect to (user@hostname or hostname)
    #[arg(value_name = "DESTINATION")]
    destination: String,

    /// Command to execute on remote host
    #[arg(last = true)]
    command: Vec<String>,

    /// Port to connect to
    #[arg(short, long, default_value = "22")]
    port: u16,

    /// Identity file (private key)
    #[arg(short, long)]
    identity: Option<PathBuf>,

    /// Verbose mode
    #[arg(short, long)]
    verbose: bool,

    /// Connect timeout in seconds
    #[arg(short, long, default_value = "30")]
    connect_timeout: u64,

    /// Batch mode (no interactive prompt)
    #[arg(long)]
    batch: bool,

    /// Disable strict host key checking
    #[arg(long = "strict-host-key-checking")]
    strict_host_key_checking: Option<String>,

    /// User
    #[arg(short, long)]
    user: Option<String>,

    /// Forward X11
    #[arg(short = 'X', long)]
    forward_x11: bool,

    /// Forward agent
    #[arg(short = 'A', long)]
    forward_agent: bool,

    /// Remote port forwarding
    #[arg(short = 'R', long)]
    remote_forward: Option<String>,

    /// Local port forwarding
    #[arg(short = 'L', long)]
    local_forward: Option<String>,

    /// Dynamic port forwarding
    #[arg(short = 'D', long)]
    dynamic_forward: Option<String>,

    /// Force pseudo-terminal allocation
    #[arg(short = 't', long)]
    force_pty: bool,

    /// Request a specific tty
    #[arg(short = 'T', long, action = clap::ArgAction::SetFalse)]
    allocate_pty: Option<bool>,

    /// Compression
    #[arg(short = 'C', long)]
    compression: bool,

    /// Connection attempts
    #[arg(short, long, default_value = "1")]
    connection_attempts: u32,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Build ssh command
    let mut cmd = Command::new("ssh");

    // Add port
    cmd.arg("-p").arg(args.port.to_string());

    // Add identity file if specified
    if let Some(ref identity) = args.identity {
        cmd.arg("-i").arg(identity);
    }

    // Add verbose flag
    if args.verbose {
        cmd.arg("-v");
    }

    // Add connect timeout
    cmd.arg("-o")
        .arg(format!("ConnectTimeout={}", args.connect_timeout));

    // Add batch mode
    if args.batch {
        cmd.arg("-o").arg("BatchMode=yes");
    }

    // Add strict host key checking
    if let Some(ref value) = args.strict_host_key_checking {
        cmd.arg("-o").arg(format!("StrictHostKeyChecking={}", value));
    }

    // Add user if specified
    if let Some(ref user) = args.user {
        cmd.arg("-o").arg(format!("User={}", user));
    }

    // Add X11 forwarding
    if args.forward_x11 {
        cmd.arg("-X");
    }

    // Add agent forwarding
    if args.forward_agent {
        cmd.arg("-A");
    }

    // Add remote port forwarding
    if let Some(ref forward) = args.remote_forward {
        cmd.arg("-R").arg(forward);
    }

    // Add local port forwarding
    if let Some(ref forward) = args.local_forward {
        cmd.arg("-L").arg(forward);
    }

    // Add dynamic port forwarding
    if let Some(ref forward) = args.dynamic_forward {
        cmd.arg("-D").arg(forward);
    }

    // Add force pty
    if args.force_pty {
        cmd.arg("-t");
    }

    // Add no pty
    if args.allocate_pty == Some(false) {
        cmd.arg("-T");
    }

    // Add compression
    if args.compression {
        cmd.arg("-C");
    }

    // Add connection attempts
    cmd.arg("-o")
        .arg(format!("ConnectionAttempts={}", args.connection_attempts));

    // Add destination
    cmd.arg(&args.destination);

    // Add command if specified
    if !args.command.is_empty() {
        cmd.args(&args.command);
    }

    // Execute ssh command
    let status = cmd.status().with_context(|| "Failed to execute ssh command")?;

    // Exit with same code as ssh
    std::process::exit(status.code().unwrap_or(1));
}
