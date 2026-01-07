// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! apt-get - APT-GET Compatibility Wrapper
//!
//! Provides apt-get compatibility by forwarding to apt wrapper

use anyhow::Result;
use std::env;
use std::process::Command;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Forward to apt wrapper, skipping the program name
    let status = Command::new("apt")
        .args(&args[1..])
        .status()?;

    std::process::exit(status.code().unwrap_or(1));
}
