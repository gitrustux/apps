// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! logview - Log Viewer Utility
//!
//! View and monitor system logs and crash reports.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use chrono::{DateTime, Local};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal,
};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

/// Log Viewer
#[derive(Parser, Debug)]
#[command(name = "logview")]
#[command(author = "The Rustux Authors")]
#[command(version = "0.1.0")]
#[command(about = "System log viewer", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List available log files
    List {
        /// Log directory to scan
        #[arg(short, long, default_value = "/var/log")]
        dir: PathBuf,
    },

    /// View a log file
    View {
        /// Log file to view
        #[arg(required = true)]
        file: PathBuf,

        /// Number of lines to show
        #[arg(short, long, default_value = "100")]
        lines: usize,

        /// Filter by pattern
        #[arg(short, long)]
        pattern: Option<String>,

        /// Follow log output (like tail -f)
        #[arg(short, long)]
        follow: bool,

        /// Show timestamps
        #[arg(long)]
        timestamps: bool,
    },

    /// Search logs
    Search {
        /// Search pattern
        pattern: String,

        /// Log directory to search
        #[arg(short, long, default_value = "/var/log")]
        dir: PathBuf,

        /// Case insensitive search
        #[arg(short, long)]
        ignore_case: bool,
    },

    /// Show crash reports
    CrashList {
        /// Crash report directory
        #[arg(short, long, default_value = "/var/crash")]
        dir: PathBuf,
    },

    /// View crash report
    CrashView {
        /// Crash report file
        #[arg(required = true)]
        file: PathBuf,
    },

    /// Clear old logs
    Clear {
        /// Log directory
        #[arg(short, long, default_value = "/var/log")]
        dir: PathBuf,

        /// Days to keep
        #[arg(short, long, default_value = "30")]
        days: usize,

        /// Don't ask for confirmation
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug)]
struct LogEntry {
    timestamp: Option<String>,
    level: String,
    message: String,
    line_number: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::List { dir } => {
            list_logs(&dir)?;
        }
        Commands::View { file, lines, pattern, follow, timestamps } => {
            if follow {
                follow_log(&file, lines, pattern.as_deref(), timestamps)?;
            } else {
                view_log(&file, lines, pattern.as_deref(), timestamps)?;
            }
        }
        Commands::Search { pattern, dir, ignore_case } => {
            search_logs(&pattern, &dir, ignore_case)?;
        }
        Commands::CrashList { dir } => {
            list_crashes(&dir)?;
        }
        Commands::CrashView { file } => {
            view_crash(&file)?;
        }
        Commands::Clear { dir, days, force } => {
            clear_old_logs(&dir, days, force)?;
        }
    }

    Ok(())
}

fn list_logs(dir: &Path) -> Result<()> {
    println!("Available log files in {}:", dir.display());
    println!();

    if !dir.exists() {
        println!("  (No logs directory found)");
        return Ok(());
    }

    let entries = fs::read_dir(dir)
        .context("Failed to read log directory")?;

    let mut logs = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let metadata = fs::metadata(&path)?;
            let size = metadata.len();
            let modified = metadata.modified()?;

            let datetime: String = DateTime::<Local>::from(modified)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();

            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?");

            logs.push((name.to_string(), size, datetime));
        }
    }

    logs.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by size

    for (name, size, modified) in &logs {
        println!("  {} - {} ({})", name, modified, format_size(*size));
    }

    println!("\nTotal: {} log file(s)", logs.len());

    Ok(())
}

fn view_log(file: &Path, lines: usize, pattern: Option<&str>, show_timestamps: bool) -> Result<()> {
    if !file.exists() {
        return Err(anyhow::anyhow!("Log file not found: {}", file.display()));
    }

    let f = File::open(file)?;
    let reader = BufReader::new(f);
    let file_lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    let total_lines = file_lines.len();
    let start_line = if lines >= total_lines { 0 } else { total_lines - lines };

    let end_line = total_lines;
    let matching_lines: Vec<(usize, String)> = file_lines
        .iter()
        .enumerate()
        .skip(start_line)
        .filter(|(_, line)| {
            if let Some(pat) = pattern {
                line.to_lowercase().contains(&pat.to_lowercase())
            } else {
                true
            }
        })
        .map(|(i, line)| (i + 1, line.clone()))
        .collect();

    println!("Log file: {}", file.display());
    println!("Showing lines {}-{} (of {} total)", start_line + 1, end_line, total_lines);
    if let Some(pat) = pattern {
        println!("Filtered by: {}", pat);
    }
    println!();

    for (line_num, line) in &matching_lines {
        if show_timestamps {
            println!("{:6}: {}", line_num, line);
        } else {
            println!("{}", line);
        }
    }

    println!("\nShowing {} of {} lines", matching_lines.len(), total_lines);

    Ok(())
}

fn follow_log(file: &Path, lines: usize, pattern: Option<&str>, show_timestamps: bool) -> Result<()> {
    if !file.exists() {
        return Err(anyhow::anyhow!("Log file not found: {}", file.display()));
    }

    println!("Following log file: {} (Ctrl+C to exit)", file.display());
    println!();

    terminal::enable_raw_mode()?;

    // Spawn thread to read file
    let file_path = file.to_path_buf();
    let pattern_clone = pattern.map(|p| p.to_string());

    let rx = thread::spawn(move || {
        let mut last_size = 0;
        let mut line_count = 0;

        loop {
            if let Ok(metadata) = fs::metadata(&file_path) {
                let new_size = metadata.len();

                if new_size > last_size {
                    if let Ok(f) = File::open(&file_path) {
                        let mut reader = BufReader::new(f);
                        reader.seek(SeekFrom::Start(last_size)).ok();

                        loop {
                            let mut line = String::new();
                            let bytes_read = reader.read_line(&mut line).unwrap_or(0);

                            if bytes_read == 0 {
                                break;
                            }

                            if let Some(ref pat) = pattern_clone {
                                if line.to_lowercase().contains(&pat.to_lowercase()) {
                                    println!("{}", line);
                                }
                            } else {
                                println!("{}", line);
                            }

                            line_count += 1;
                        }

                        last_size = new_size;
                    }
                }
            }

            thread::sleep(Duration::from_millis(100));
        }
    });

    // Wait for Ctrl+C
    loop {
        if event::poll(Duration::from_millis(100)).unwrap_or(false) {
            if let Ok(Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                ..
            })) = event::read()
            {
                println!();
                println!("Stopped following log.");
                break;
            }
        }
    }

    terminal::disable_raw_mode()?;

    rx.thread().unpark();

    Ok(())
}

fn search_logs(pattern: &str, dir: &Path, ignore_case: bool) -> Result<()> {
    println!("Searching for '{}' in {}...", pattern, dir.display());
    println!();

    if !dir.exists() {
        println!("  (No logs directory found)");
        return Ok(());
    }

    let search_pattern = if ignore_case {
        pattern.to_lowercase()
    } else {
        pattern.to_string()
    };

    let mut total_matches = 0;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let f = File::open(&path)?;
            let reader = BufReader::new(f);

            let mut matches = 0;
            for (i, line) in reader.lines().enumerate() {
                let line = line?;
                let search_line = if ignore_case {
                    line.to_lowercase()
                } else {
                    line.clone()
                };

                if search_line.contains(&search_pattern) {
                    if matches == 0 {
                        println!("  File: {}", path.display());
                    }
                    println!("    Line {}: {}", i + 1, line.trim());
                    matches += 1;
                }
            }

            if matches > 0 {
                println!("    Found {} match(es)", matches);
                println!();
                total_matches += matches;
            }
        }
    }

    println!("Total matches: {}", total_matches);

    Ok(())
}

fn list_crashes(dir: &Path) -> Result<()> {
    println!("Crash reports in {}:", dir.display());
    println!();

    if !dir.exists() {
        println!("  (No crash directory found)");
        return Ok(());
    }

    let entries = fs::read_dir(dir)
        .context("Failed to read crash directory")?;

    let mut crashes = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "crash" || ext == "dmp" || ext == "core" {
                    let metadata = fs::metadata(&path)?;
                    let size = metadata.len();
                    let modified = metadata.modified()?;

                    let datetime: String = DateTime::<Local>::from(modified)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string();

                    let name = path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("?");

                    crashes.push((name.to_string(), size, datetime));
                }
            }
        }
    }

    crashes.sort_by(|a: &(String, u64, String), b| b.2.cmp(&a.2)); // Sort by date (newest first)

    for (name, size, datetime) in &crashes {
        println!("  {} - {} ({})", name, datetime, format_size(*size));
    }

    println!("\nTotal: {} crash report(s)", crashes.len());

    Ok(())
}

fn view_crash(file: &Path) -> Result<()> {
    if !file.exists() {
        return Err(anyhow::anyhow!("Crash file not found: {}", file.display()));
    }

    println!("Crash Report: {}", file.display());
    println!("{}", "=".repeat(60));
    println!();

    let content = fs::read_to_string(file)?;
    print!("{}", content);

    Ok(())
}

fn clear_old_logs(dir: &Path, days: usize, force: bool) -> Result<()> {
    if !dir.exists() {
        println!("No logs directory found at {}", dir.display());
        return Ok(());
    }

    let now = std::time::SystemTime::now();
    let duration = std::time::Duration::from_secs(days as u64 * 24 * 60 * 60);
    let cutoff = now - duration;

    println!("Clearing logs older than {} days from {}...", days, dir.display());

    if !force {
        print!("Continue? [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "y" {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let mut cleared = 0;
    let mut freed = 0;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if modified < cutoff {
                        let size = metadata.len();
                        fs::remove_file(&path)?;
                        cleared += 1;
                        freed += size;
                    }
                }
            }
        }
    }

    println!("Cleared {} log file(s), freed {}", cleared, format_size(freed));

    Ok(())
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}
