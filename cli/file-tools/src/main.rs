// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! File Tools for Rustica OS
//!
//! Provides ls, cat, echo, and pwd commands.

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

#[derive(Parser, Debug)]
#[command(name = "file-tools")]
#[command(about = "File tools for Rustica OS", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List directory contents
    #[command(name = "ls")]
    Ls(LsArgs),
    /// Concatenate files to stdout
    #[command(name = "cat")]
    Cat(CatArgs),
    /// Display text
    #[command(name = "echo")]
    Echo(EchoArgs),
    /// Print working directory
    #[command(name = "pwd")]
    Pwd(PwdArgs),
}

// ============================================================================
// ls arguments
// ============================================================================

#[derive(Parser, Debug)]
pub struct LsArgs {
    /// List all files (including dotfiles)
    #[arg(short = 'a', long = "all")]
    pub all: bool,

    /// List in long format
    #[arg(short = 'l', long = "long")]
    pub long: bool,

    /// List entries by columns
    #[arg(short = 'C', long = "columns")]
    pub columns: bool,

    /// List one entry per line
    #[arg(short = '1', long = "single-line")]
    pub single_line: bool,

    /// Human-readable sizes
    #[arg(short = 'H', long = "human-readable")]
    pub human: bool,

    /// List directory entries themselves
    #[arg(short = 'd', long = "directory")]
    pub directory: bool,

    /// Reverse order
    #[arg(short = 'r', long = "reverse")]
    pub reverse: bool,

    /// Sort by time
    #[arg(short = 't', long = "time")]
    pub time: bool,

    /// Sort by size
    #[arg(short = 'S', long = "size")]
    pub size: bool,

    /// List inode numbers
    #[arg(short = 'i', long = "inode")]
    pub inode: bool,

    /// Output in JSON format
    #[arg(short = 'j', long = "json")]
    pub json: bool,

    /// Paths to list
    #[arg(value_name = "PATH", default_value = ".")]
    pub paths: Vec<String>,
}

// ============================================================================
// cat arguments
// ============================================================================

#[derive(Parser, Debug)]
pub struct CatArgs {
    /// Number all output lines
    #[arg(short = 'n', long = "number")]
    pub number: bool,

    /// Number non-empty output lines
    #[arg(short = 'b', long = "number-nonblank")]
    pub number_nonblank: bool,

    /// Show end of lines
    #[arg(short = 'E', long = "show-ends")]
    pub show_ends: bool,

    /// Show tabs
    #[arg(short = 'T', long = "show-tabs")]
    pub show_tabs: bool,

    /// Show all non-printing characters
    #[arg(short = 'A', long = "show-all")]
    pub show_all: bool,

    /// Squeeze blank lines
    #[arg(short = 's', long = "squeeze-blank")]
    pub squeeze_blank: bool,

    /// Output in JSON format
    #[arg(short = 'j', long = "json")]
    pub json: bool,

    /// Files to concatenate
    #[arg(value_name = "FILE")]
    pub files: Vec<String>,
}

// ============================================================================
// echo arguments
// ============================================================================

#[derive(Parser, Debug)]
pub struct EchoArgs {
    /// Do not output trailing newline
    #[arg(short = 'n', long = "no-newline")]
    pub no_newline: bool,

    /// Enable interpretation of backslash escapes
    #[arg(short = 'e', long = "enable-escapes")]
    pub enable_escapes: bool,

    /// Disable interpretation of backslash escapes (default)
    #[arg(short = 'E', long = "disable-escapes")]
    pub disable_escapes: bool,

    /// Text to display
    #[arg(value_name = "TEXT", required = true)]
    pub text: Vec<String>,
}

// ============================================================================
// pwd arguments
// ============================================================================

#[derive(Parser, Debug)]
pub struct PwdArgs {
    /// Output physical path without symlinks
    #[arg(short = 'P', long = "physical")]
    pub physical: bool,

    /// Output logical path (may include symlinks)
    #[arg(short = 'L', long = "logical")]
    pub logical: bool,

    /// Output in JSON format
    #[arg(short = 'j', long = "json")]
    pub json: bool,
}

// ============================================================================
// Data structures
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub is_executable: bool,
    pub size: u64,
    pub modified: String,
    pub permissions: String,
    pub owner: String,
    pub group: String,
    pub inode: Option<u64>,
    pub nlinks: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LsOutput {
    pub entries: Vec<FileInfo>,
    pub total: u64,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatOutput {
    pub lines: Vec<String>,
    pub file_count: usize,
    pub line_count: usize,
    pub byte_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct PwdOutput {
    pub path: String,
    pub is_symlink: bool,
}

// ============================================================================
// Main command handlers
// ============================================================================

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Ls(args) => cmd_ls(args),
        Commands::Cat(args) => cmd_cat(args),
        Commands::Echo(args) => cmd_echo(args),
        Commands::Pwd(args) => cmd_pwd(args),
    }
}

pub fn cmd_ls(args: LsArgs) -> Result<()> {
    let mut entries = Vec::new();

    for path_str in &args.paths {
        let path = Path::new(path_str);

        // Handle directory vs file
        if path.is_dir() && !args.directory {
            // List directory contents
            if let Ok(read_dir) = fs::read_dir(path) {
                for entry in read_dir {
                    if let Ok(entry) = entry {
                        let file_info = get_file_info(&entry.path(), &args)?;
                        if args.all || !file_info.name.starts_with('.') {
                            entries.push(file_info);
                        }
                    }
                }
            }
        } else {
            // List the path itself
            entries.push(get_file_info(path, &args)?);
        }
    }

    // Sort entries
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    if args.reverse {
        entries.reverse();
    }

    if args.time {
        entries.sort_by(|a, b| b.modified.cmp(&a.modified));
    }

    if args.size {
        entries.sort_by(|a, b| b.size.cmp(&a.size));
    }

    if args.json {
        let output = LsOutput {
            entries: entries.clone(),
            total: entries.len() as u64,
            path: args.paths.join(" "),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else if args.long {
        let total_blocks: u64 = entries.iter().map(|e| (e.size + 1023) / 1024).sum();
        println!("total {}", total_blocks);
        for entry in &entries {
            let file_type = if entry.is_symlink {
                "l"
            } else if entry.is_dir {
                "d"
            } else {
                "-"
            };

            println!("{}{} {:>8} {:>8} {:>8} {} {} {}",
                file_type,
                entry.permissions,
                entry.nlinks.unwrap_or(1),
                entry.owner,
                entry.group,
                format_size(entry.size, args.human),
                entry.modified.split('T').next().unwrap_or(""),
                entry.name
            );
        }
    } else {
        for entry in &entries {
            if args.single_line {
                println!("{}", entry.name);
            } else if args.columns {
                print!("{:<30} ", entry.name);
            } else {
                println!("{}", entry.name);
            }
        }
        if args.columns && !entries.is_empty() {
            println!();
        }
    }

    Ok(())
}

pub fn cmd_cat(args: CatArgs) -> Result<()> {
    let mut all_lines = Vec::new();
    let mut total_bytes = 0;

    let files: Vec<String> = if args.files.is_empty() {
        vec!["-".to_string()] // stdin
    } else {
        args.files.clone()
    };

    for file_path in &files {
        if file_path == "-" {
            // Read from stdin
            use std::io::{self, Read};
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            all_lines.push(input.trim().to_string());
            total_bytes += input.len();
        } else {
            let content = fs::read_to_string(file_path)?;
            total_bytes += content.len();

            for line in content.lines() {
                let mut output = line.to_string();

                if args.show_ends || args.show_all {
                    output.push('$');
                }

                if args.show_tabs || args.show_all {
                    output = output.replace('\t', "^I");
                }

                all_lines.push(output);
            }
        }
    }

    if args.json {
        let output = CatOutput {
            lines: all_lines.clone(),
            file_count: files.len(),
            line_count: all_lines.len(),
            byte_count: total_bytes,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        for (i, line) in all_lines.iter().enumerate() {
            if args.number {
                println!("{:6}\t{}", i + 1, line);
            } else if args.number_nonblank {
                if !line.is_empty() {
                    println!("{:6}\t{}", i + 1, line);
                } else {
                    println!();
                }
            } else {
                println!("{}", line);
            }
        }
    }

    Ok(())
}

pub fn cmd_echo(args: EchoArgs) -> Result<()> {
    let mut output = args.text.join(" ");

    if args.enable_escapes {
        output = expand_escapes(&output);
    }

    if args.no_newline {
        print!("{}", output);
    } else {
        println!("{}", output);
    }

    Ok(())
}

pub fn cmd_pwd(args: PwdArgs) -> Result<()> {
    let current = if args.physical {
        std::env::current_dir()?
    } else {
        std::env::current_dir()?
    };

    let path_str = current.to_string_lossy().to_string();
    let is_symlink = current.is_symlink();

    if args.json {
        let output = PwdOutput {
            path: path_str,
            is_symlink,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{}", path_str);
    }

    Ok(())
}

// ============================================================================
// Helper functions
// ============================================================================

fn get_file_info(path: &Path, args: &LsArgs) -> Result<FileInfo> {
    let metadata = fs::metadata(path)?;
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(".")
        .to_string();

    let modified = metadata.modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .and_then(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0))
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string());

    let mode = metadata.permissions().mode();
    let permissions = format_permissions(mode);

    let is_executable = mode & 0o111 != 0;

    let inode = if args.inode {
        Some(metadata.ino())
    } else {
        None
    };

    let nlinks = if args.long {
        Some(metadata.nlink())
    } else {
        None
    };

    Ok(FileInfo {
        name: name.clone(),
        path: path.to_string_lossy().to_string(),
        is_dir: path.is_dir(),
        is_symlink: path.is_symlink(),
        is_executable,
        size: metadata.len(),
        modified,
        permissions,
        owner: "root".to_string(),
        group: "root".to_string(),
        inode,
        nlinks,
    })
}

fn format_permissions(mode: u32) -> String {
    let mut perms = String::new();

    // User permissions
    perms.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o100 != 0 { 'x' } else { '-' });

    // Group permissions
    perms.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o010 != 0 { 'x' } else { '-' });

    // Other permissions
    perms.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    perms.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    perms.push(if mode & 0o001 != 0 { 'x' } else { '-' });

    perms
}

fn format_size(size: u64, human: bool) -> String {
    if human {
        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;
        const GB: u64 = 1024 * MB;

        if size >= GB {
            format!("{:.1}G", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.1}M", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.1}K", size as f64 / KB as f64)
        } else {
            format!("{}B", size)
        }
    } else {
        size.to_string()
    }
}

fn expand_escapes(s: &str) -> String {
    s.replace("\\n", "\n")
     .replace("\\t", "\t")
     .replace("\\r", "\r")
     .replace("\\\\", "\\")
     .replace("\\0", "\0")
}
