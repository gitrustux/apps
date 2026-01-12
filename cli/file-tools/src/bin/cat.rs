// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! cat - Concatenate files to stdout
//!
//! Replacement for the Linux cat command.

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = file_tools::CatArgs::parse();
    file_tools::cmd_cat(args)
}
