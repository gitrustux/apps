// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! fwctl - Firewall control utility
//!
//! Manages nftables-based firewall rules.

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Firewall control utility
#[derive(Parser, Debug)]
#[command(name = "fwctl")]
#[command(about = "Firewall control utility", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Load firewall rules from file
    Load {
        /// Rules file to load
        file: Option<String>,

        /// Apply immediately
        #[arg(short, long)]
        apply: bool,
    },

    /// Save current rules to file
    Save {
        /// Output file
        #[arg(short, long)]
        output: Option<String>,
    },

    /// List current rules
    List {
        /// Table name
        #[arg(short, long)]
        table: Option<String>,

        /// Show numeric output
        #[arg(short = 'n', long)]
        numeric: bool,
    },

    /// Add a rule
    Add {
        /// Chain (input/output/forward)
        #[arg(short, long)]
        chain: String,

        /// Source address
        #[arg(short, long)]
        source: Option<String>,

        /// Destination port
        #[arg(short = 'd', long)]
        dest_port: Option<u16>,

        /// Protocol (tcp/udp/icmp)
        #[arg(short, long)]
        protocol: Option<String>,

        /// Action (accept/drop/reject)
        #[arg(short, long)]
        action: String,
    },

    /// Delete a rule
    Delete {
        /// Chain name
        #[arg(short, long)]
        chain: String,

        /// Rule handle number
        #[arg(short, long)]
        handle: u32,
    },

    /// Flush all rules
    Flush {
        /// Table name
        #[arg(short, long)]
        table: Option<String>,

        /// Chain name
        #[arg(short, long)]
        chain: Option<String>,
    },

    /// Set default policy
    Policy {
        /// Chain name
        #[arg(short, long)]
        chain: String,

        /// Policy (accept/drop)
        #[arg(short, long)]
        policy: String,
    },

    /// Enable firewall
    Enable {
        /// Profile to use (strict/medium/permissive)
        #[arg(short, long, default_value = "medium")]
        profile: String,
    },

    /// Disable firewall
    Disable,

    /// Show status
    Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FirewallRule {
    handle: Option<u32>,
    chain: String,
    source: Option<String>,
    dest_port: Option<u16>,
    protocol: Option<String>,
    action: String,
    comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FirewallConfig {
    enabled: bool,
    default_policy: String,
    rules: Vec<FirewallRule>,
}

struct FirewallManager {
    config_dir: PathBuf,
    state_dir: PathBuf,
}

impl FirewallManager {
    fn new() -> Result<Self> {
        let config_dir = PathBuf::from("/etc/rustica/fw");
        let state_dir = PathBuf::from("/var/lib/rustica/fw");

        fs::create_dir_all(&config_dir)?;
        fs::create_dir_all(&state_dir)?;

        Ok(Self {
            config_dir,
            state_dir,
        })
    }

    fn load_rules(&self, file: Option<String>, apply: bool) -> Result<()> {
        let rules_file = if let Some(f) = file {
            PathBuf::from(f)
        } else {
            self.config_dir.join("rules.json")
        };

        if !rules_file.exists() {
            return Err(anyhow::anyhow!("Rules file not found: {}", rules_file.display()));
        }

        let content = fs::read_to_string(&rules_file)?;
        let config: FirewallConfig = serde_json::from_str(&content)?;

        if apply {
            println!("Applying firewall rules from {}...", rules_file.display());
            self.apply_config(&config)?;
        } else {
            println!("Rules loaded from {}", rules_file.display());
            println!("Use --apply to activate");
        }

        Ok(())
    }

    fn save_rules(&self, output: Option<String>) -> Result<()> {
        let config = self.get_current_config()?;

        let output_file = if let Some(f) = output {
            PathBuf::from(f)
        } else {
            self.config_dir.join("rules.json")
        };

        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&output_file, content)?;

        println!("Rules saved to {}", output_file.display());

        Ok(())
    }

    fn list_rules(&self, table: Option<String>, numeric: bool) -> Result<()> {
        println!("Current firewall rules:");
        println!();

        // Use nft list command
        let mut cmd = Command::new("nft");
        cmd.arg("list");
        cmd.arg("ruleset");

        if let Some(t) = table {
            cmd.arg("table");
            cmd.arg(&t);
        }

        let output = cmd.output()?;

        if output.status.success() {
            let rules = String::from_utf8_lossy(&output.stdout);
            println!("{}", rules);
        } else {
            // Fallback: show from config
            self.show_config_rules(&self.get_current_config()?, numeric)?;
        }

        Ok(())
    }

    fn add_rule(&self, chain: String, source: Option<String>, dest_port: Option<u16>,
                protocol: Option<String>, action: String) -> Result<()> {
        let rule = FirewallRule {
            handle: None,
            chain,
            source,
            dest_port,
            protocol,
            action,
            comment: None,
        };

        println!("Adding rule: {:?}", rule);

        // Build nft command
        let mut nft_cmd = Command::new("nft");
        nft_cmd.arg("add");
        nft_cmd.arg("rule");
        nft_cmd.arg("inet");
        nft_cmd.arg("fw");
        nft_cmd.arg(&rule.chain);

        // Add rule conditions
        if let Some(ref proto) = rule.protocol {
            nft_cmd.arg("meta");
            nft_cmd.arg("l4proto");
            nft_cmd.arg(proto);
        }

        if let Some(ref src) = rule.source {
            nft_cmd.arg("ip");
            nft_cmd.arg("saddr");
            nft_cmd.arg(src);
        }

        if let Some(port) = rule.dest_port {
            nft_cmd.arg("tcp");
            nft_cmd.arg("dport");
            nft_cmd.arg(&port.to_string());
        }

        nft_cmd.arg(&rule.action);

        let output = nft_cmd.output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to add rule: {}", error));
        }

        println!("Rule added successfully");

        Ok(())
    }

    fn delete_rule(&self, chain: String, handle: u32) -> Result<()> {
        println!("Deleting rule {} from chain {}", handle, chain);

        let output = Command::new("nft")
            .args(&["delete", "rule", "inet", "fw", &chain, "handle", &handle.to_string()])
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to delete rule: {}", error));
        }

        println!("Rule deleted successfully");

        Ok(())
    }

    fn flush_rules(&self, table: Option<String>, chain: Option<String>) -> Result<()> {
        println!("Flushing firewall rules...");

        let mut cmd_args = vec!["flush", "ruleset"];

        if let Some(t) = &table {
            cmd_args = vec!["flush", "table", "inet", t.as_str()];
        }

        let output = Command::new("nft")
            .args(&cmd_args)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            eprintln!("Warning: {}", error);
        }

        println!("Rules flushed successfully");

        Ok(())
    }

    fn set_policy(&self, chain: String, policy: String) -> Result<()> {
        println!("Setting {} chain policy to {}", chain, policy);

        let output = Command::new("nft")
            .args(&["chain", "inet", "fw", &chain, "{", "policy", &policy, ";", "}"])
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to set policy: {}", error));
        }

        println!("Policy set successfully");

        Ok(())
    }

    fn enable(&self, profile: String) -> Result<()> {
        println!("Enabling firewall with {} profile...", profile);

        // Create basic nftables ruleset
        let ruleset = self.build_profile_ruleset(&profile)?;

        // Apply ruleset
        let output = Command::new("nft")
            .arg("-f")
            .stdin(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = output.stdin.as_ref() {
            use std::io::Write;
            stdin.write_all(ruleset.as_bytes())?;
        }

        Ok(())
    }

    fn disable(&self) -> Result<()> {
        println!("Disabling firewall...");

        let output = Command::new("nft")
            .args(&["flush", "ruleset"])
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to disable firewall: {}", error));
        }

        println!("Firewall disabled successfully");

        Ok(())
    }

    fn status(&self) -> Result<()> {
        println!("Firewall Status:");
        println!();

        let output = Command::new("nft")
            .args(&["list", "ruleset"])
            .output()?;

        if output.status.success() {
            let rules = String::from_utf8_lossy(&output.stdout);

            if rules.trim().is_empty() {
                println!("Firewall is disabled (no rules loaded)");
            } else {
                println!("{}", rules);
            }
        } else {
            println!("Firewall: unknown");
        }

        Ok(())
    }

    fn build_profile_ruleset(&self, profile: &str) -> Result<String> {
        let ruleset = match profile {
            "strict" => r#"
#!/usr/sbin/nft -f
# Strict firewall profile

table inet fw {
    chain input {
        type filter hook input priority 0; policy drop;

        # Allow established/related connections
        ct state established,related accept

        # Allow loopback
        iif "lo" accept

        # Allow ICMP
        ip protocol icmp accept
        ip6 nexthdr icmpv6 accept

        # Allow SSH (customize port as needed)
        tcp dport 22 accept
    }

    chain forward {
        type filter hook forward priority 0; policy drop;
    }

    chain output {
        type filter hook output priority 0; policy accept;
    }
}
"#.to_string(),

            "medium" => r#"
#!/usr/sbin/nft -f
# Medium security firewall profile

table inet fw {
    chain input {
        type filter hook input priority 0; policy drop;

        # Allow established/related connections
        ct state established,related accept

        # Allow loopback
        iif "lo" accept

        # Allow ICMP
        ip protocol icmp accept
        ip6 nexthdr icmpv6 accept

        # Allow common services
        tcp dport {22, 80, 443} accept
        udp dport 53 accept
    }

    chain forward {
        type filter hook forward priority 0; policy drop;
    }

    chain output {
        type filter hook output priority 0; policy accept;
    }
}
"#.to_string(),

            "permissive" => r#"
#!/usr/sbin/nft -f
# Permissive firewall profile

table inet fw {
    chain input {
        type filter hook input priority 0; policy accept;

        # Drop invalid packets
        ct state invalid drop
    }

    chain forward {
        type filter hook forward priority 0; policy accept;
    }

    chain output {
        type filter hook output priority 0; policy accept;
    }
}
"#.to_string(),

            _ => return Err(anyhow::anyhow!("Unknown profile: {}", profile)),
        };

        Ok(ruleset)
    }

    fn get_current_config(&self) -> Result<FirewallConfig> {
        let config_file = self.config_dir.join("rules.json");

        if config_file.exists() {
            let content = fs::read_to_string(&config_file)?;
            return Ok(serde_json::from_str(&content)?);
        }

        // Return default config
        Ok(FirewallConfig {
            enabled: false,
            default_policy: "accept".to_string(),
            rules: Vec::new(),
        })
    }

    fn apply_config(&self, config: &FirewallConfig) -> Result<()> {
        if config.enabled {
            self.enable("medium".to_string())?;
        } else {
            self.disable()?;
        }

        for rule in &config.rules {
            self.add_rule(
                rule.chain.clone(),
                rule.source.clone(),
                rule.dest_port,
                rule.protocol.clone(),
                rule.action.clone(),
            )?;
        }

        Ok(())
    }

    fn show_config_rules(&self, config: &FirewallConfig, _numeric: bool) -> Result<()> {
        println!("Enabled: {}", config.enabled);
        println!("Default Policy: {}", config.default_policy);
        println!();
        println!("Rules:");

        for rule in &config.rules {
            let handle = rule.handle.map_or("?".to_string(), |h| h.to_string());
            println!("  {} {}: {}", handle, rule.chain, rule.action);

            if let Some(ref src) = rule.source {
                println!("      from: {}", src);
            }

            if let Some(port) = rule.dest_port {
                println!("      port: {}", port);
            }

            if let Some(ref proto) = rule.protocol {
                println!("      proto: {}", proto);
            }

            println!();
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let fw = FirewallManager::new()?;

    match args.command {
        Commands::Load { file, apply } => {
            fw.load_rules(file, apply)?;
        }
        Commands::Save { output } => {
            fw.save_rules(output)?;
        }
        Commands::List { table, numeric } => {
            fw.list_rules(table, numeric)?;
        }
        Commands::Add { chain, source, dest_port, protocol, action } => {
            fw.add_rule(chain, source, dest_port, protocol, action)?;
        }
        Commands::Delete { chain, handle } => {
            fw.delete_rule(chain, handle)?;
        }
        Commands::Flush { table, chain } => {
            if chain.is_some() {
                // Flush specific chain
                println!("Note: nftables flushes by table, not chain");
            }
            fw.flush_rules(table, chain)?;
        }
        Commands::Policy { chain, policy } => {
            fw.set_policy(chain, policy)?;
        }
        Commands::Enable { profile } => {
            fw.enable(profile)?;
        }
        Commands::Disable => {
            fw.disable()?;
        }
        Commands::Status => {
            fw.status()?;
        }
    }

    Ok(())
}
