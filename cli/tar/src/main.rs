// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! tar - Rustica Archive Utility
//!
//! Create, extract, and list tar archives with gzip compression support.

use anyhow::{Context, Result};
use clap::Parser;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Rustica Archive Utility
#[derive(Parser, Debug)]
#[command(name = "tar")]
#[command(author = "The Rustux Authors")]
#[command(version = "0.1.0")]
#[command(about = "Rustica Archive Utility", long_about = None)]
struct Args {
    /// Create a new archive
    #[arg(short, long)]
    create: bool,

    /// Extract files from an archive
    #[arg(short, long)]
    extract: bool,

    /// List files in an archive
    #[arg(short, long)]
    list: bool,

    /// File name of the archive
    #[arg(short, long)]
    file: String,

    /// Files to archive or extract
    #[arg(value_name = "FILE")]
    paths: Vec<PathBuf>,

    /// Use gzip compression
    #[arg(short, long)]
    gzip: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.create {
        create_archive(&args)?;
    } else if args.extract {
        extract_archive(&args)?;
    } else if args.list {
        list_archive(&args)?;
    } else {
        eprintln!("Error: Must specify one of: --create, --extract, or --list");
        std::process::exit(1);
    }

    Ok(())
}

/// Create a new archive
fn create_archive(args: &Args) -> Result<()> {
    println!("Creating archive: {}", args.file);

    let mut paths_to_add = Vec::new();
    for path in &args.paths {
        collect_paths(path, &mut paths_to_add)?;
    }

    if paths_to_add.is_empty() {
        anyhow::bail!("No files to archive");
    }

    let output = if args.gzip {
        let file = File::create(&args.file)
            .with_context(|| format!("Failed to create {}", args.file))?;
        Box::new(GzEncoder::new(file, Compression::default())) as Box<dyn std::io::Write>
    } else {
        let file = File::create(&args.file)
            .with_context(|| format!("Failed to create {}", args.file))?;
        Box::new(file) as Box<dyn std::io::Write>
    };

    let mut builder = tar::Builder::new(output);

    for path in &paths_to_add {
        if args.verbose {
            println!("  Adding: {}", path.display());
        }

        let name = path.to_string_lossy().to_string();

        if path.is_dir() {
            builder.append_dir_all(&name, path)
                .with_context(|| format!("Failed to add directory: {}", path.display()))?;
        } else {
            builder.append_path_with_name(path, &name)
                .with_context(|| format!("Failed to add file: {}", path.display()))?;
        }
    }

    builder.finish()
        .context("Failed to finalize archive")?;

    println!("Archive created successfully.");
    Ok(())
}

/// Extract files from an archive
fn extract_archive(args: &Args) -> Result<()> {
    println!("Extracting from: {}", args.file);

    let input = if args.gzip || args.file.ends_with(".gz") || args.file.ends_with(".tgz") {
        let file = File::open(&args.file)
            .with_context(|| format!("Failed to open {}", args.file))?;
        Box::new(GzDecoder::new(file)) as Box<dyn std::io::Read>
    } else {
        let file = File::open(&args.file)
            .with_context(|| format!("Failed to open {}", args.file))?;
        Box::new(file) as Box<dyn std::io::Read>
    };

    let mut archive = tar::Archive::new(input);

    let mut file_count = 0;
    let mut dir_count = 0;

    for mut entry in archive.entries()?.flatten() {
        let path = entry.path()?.to_path_buf();

        if args.verbose {
            println!("  Extracting: {}", path.display());
        }

        entry.unpack(&path)
            .with_context(|| format!("Failed to extract: {}", path.display()))?;

        if path.is_dir() {
            dir_count += 1;
        } else {
            file_count += 1;
        }
    }

    println!("Extracted {} files and {} directories.", file_count, dir_count);
    Ok(())
}

/// List files in an archive
fn list_archive(args: &Args) -> Result<()> {
    println!("Listing archive: {}", args.file);
    println!();

    let input = if args.gzip || args.file.ends_with(".gz") || args.file.ends_with(".tgz") {
        let file = File::open(&args.file)
            .with_context(|| format!("Failed to open {}", args.file))?;
        Box::new(GzDecoder::new(file)) as Box<dyn std::io::Read>
    } else {
        let file = File::open(&args.file)
            .with_context(|| format!("Failed to open {}", args.file))?;
        Box::new(file) as Box<dyn std::io::Read>
    };

    let mut archive = tar::Archive::new(input);

    println!("{:<48} {:>10} {:>12} {:>12}", "Name", "Mode", "Size", "Modified");
    println!("{:-<48} {:-<10} {:-<12} {:-<12}", "--------", "----", "----", "---------");

    for entry in archive.entries()?.flatten() {
        let header = entry.header();

        let name = entry.path()?.to_string_lossy().to_string();
        let mode = header.mode()?;
        let size = header.size()?;
        let mtime = header.mtime()?;

        // Format modification time (simplified - just show timestamp)
        let datetime = if mtime > 0 {
            use std::time::UNIX_EPOCH;
            match UNIX_EPOCH.checked_add(std::time::Duration::from_secs(mtime as u64)) {
                Some(dt) => format!("{:?}", dt),
                None => "Unknown".to_string(),
            }
        } else {
            "Unknown".to_string()
        };

        let size_str = if entry.header().entry_type().is_dir() {
            "-".to_string()
        } else {
            format!("{}", size)
        };

        println!("{:<48} {:>10o} {:>12} {:>12}", name, mode, size_str, datetime);
    }

    Ok(())
}

/// Collect all files and directories recursively
fn collect_paths(path: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    paths.push(path.to_path_buf());

    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            collect_paths(&entry_path, paths)?;
        }
    }

    Ok(())
}
