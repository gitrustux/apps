// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! ls - List directory contents
//!
//! Replacement for the Linux ls command.

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = file_tools::LsArgs::parse();
    file_tools::cmd_ls(args)
}
