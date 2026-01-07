// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! pkg - Compatibility Wrapper
//!
//! This is a compatibility wrapper that forwards all `pkg` commands to `rpg`.
//! The package manager was renamed from `pkg` to `rpg` (Rustica Package Manager).

use anyhow::Result;
use std::env;
use std::process::Command;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Forward all arguments to rpg
    let status = Command::new("rpg")
        .args(&args[1..])
        .status()?;

    std::process::exit(status.code().unwrap_or(1));
}
