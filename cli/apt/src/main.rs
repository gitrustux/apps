// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! apt - APT Compatibility Wrapper
//!
//! Provides apt/apt-get compatibility by translating commands to pkg

use anyhow::Result;
use clap::Parser;
use std::env;
use std::io::{self, Write};
use std::process::Command;

/// APT Compatibility Wrapper for Rustica Package Manager
#[derive(Parser, Debug)]
#[command(name = "apt")]
#[command(author = "The Rustux Authors")]
#[command(version = "0.1.0")]
#[command(about = "Package manager compatibility wrapper (translates to rpg)", long_about = None)]
#[command(disable_version_flag = true)]
struct AptArgs {
    /// Verbosity level (-q, -qq, etc.)
    #[arg(short = 'q', action = clap::ArgAction::Count)]
    quiet: u8,

    /// Assume yes for all prompts
    #[arg(short = 'y', long)]
    yes: bool,

    /// Show version
    #[arg(long)]
    version: bool,

    /// Subcommand
    #[arg(required = true)]
    command: String,

    /// Package names or other arguments
    #[arg(trailing_var_arg = true)]
    args: Vec<String>,
}

fn main() -> Result<()> {
    let args = AptArgs::parse();

    if args.version {
        println!("apt {}", env!("CARGO_PKG_VERSION"));
        println!("Rustica Package Manager (rpg) compatibility wrapper");
        println!();
        println!("This is a compatibility wrapper that translates apt commands to rpg commands.");
        println!("Supported commands: install, remove, update, upgrade, list, search");
        return Ok(());
    }

    let rpg_args = translate_apt_to_rpg(&args.command, &args.args, args.yes)?;

    // Run rpg command
    let status = Command::new("rpg")
        .args(&rpg_args)
        .status()?;

    std::process::exit(status.code().unwrap_or(1));
}

fn translate_apt_to_rpg(command: &str, args: &[String], yes: bool) -> Result<Vec<String>> {
    let mut rpg_args = Vec::new();

    match command {
        // apt install <package> -> rpg install <package>
        "install" | "reinstall" => {
            rpg_args.push("install".to_string());
            if yes {
                rpg_args.push("--yes".to_string());
            }
            rpg_args.extend(args.iter().cloned());
        }

        // apt remove <package> -> rpg remove <package>
        "remove" | "purge" => {
            rpg_args.push("remove".to_string());
            if yes {
                rpg_args.push("--yes".to_string());
            }
            rpg_args.extend(args.iter().cloned());
        }

        // apt update -> rpg update
        "update" => {
            rpg_args.push("update".to_string());
        }

        // apt upgrade -> rpg upgrade
        "upgrade" | "full-upgrade" | "dist-upgrade" => {
            rpg_args.push("upgrade".to_string());
            if yes {
                rpg_args.push("--yes".to_string());
            }
        }

        // apt list --installed -> rpg list
        "list" => {
            rpg_args.push("list".to_string());
            // Handle --installed flag
            if args.iter().any(|a| a == "--installed" || a == "-i") {
                // List all (default behavior)
            }
        }

        // apt search <query> -> rpg search <query>
        "search" => {
            rpg_args.push("search".to_string());
            if !args.is_empty() {
                rpg_args.extend(args.iter().cloned());
            } else {
                // Show usage if no query provided
                eprintln!("Usage: apt search <pattern>");
                std::process::exit(1);
            }
        }

        // apt show <package> -> rpg info <package>
        "show" | "info" => {
            rpg_args.push("info".to_string());
            rpg_args.extend(args.iter().cloned());
        }

        // apt autoremove -> not directly supported, warn user
        "autoremove" => {
            eprintln!("Note: 'apt autoremove' is not directly supported.");
            eprintln!("In Rustica, unused dependencies are tracked automatically.");
            eprintln!("Use 'rpg remove <package>' to uninstall packages.");
            std::process::exit(0);
        }

        // apt clean / autoclean -> not applicable
        "clean" | "autoclean" => {
            eprintln!("Note: 'apt {}' is not needed in Rustica.", command);
            eprintln!("Package cache is managed automatically by rpg.");
            std::process::exit(0);
        }

        // apt edit-sources -> suggest editing sources.list
        "edit-sources" => {
            let editor = env::var("EDITOR").unwrap_or_else(|_| "editor".to_string());
            eprintln!("Opening package sources in {}...", editor);
            let status = Command::new(&editor)
                .arg("/etc/rustica/rpg/sources.list")
                .status()?;
            std::process::exit(status.code().unwrap_or(1));
        }

        // Unsupported commands
        _ => {
            eprintln!("apt: command '{}' not supported by compatibility wrapper", command);
            eprintln!();
            eprintln!("Supported commands:");
            eprintln!("  apt install <package>    - Install packages");
            eprintln!("  apt remove <package>     - Remove packages");
            eprintln!("  apt update                - Update package lists");
            eprintln!("  apt upgrade               - Upgrade packages");
            eprintln!("  apt list [--installed]   - List packages");
            eprintln!("  apt search <query>        - Search packages");
            eprintln!("  apt show <package>        - Show package info");
            eprintln!();
            eprintln!("For full functionality, use 'rpg' directly:");
            eprintln!("  rpg update, rpg install, rpg remove, rpg search, rpg list, rpg info");
            std::process::exit(1);
        }
    }

    Ok(rpg_args)
}
