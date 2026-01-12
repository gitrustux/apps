// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! rpg - Package installer
//!
//! Replacement for the Linux dpkg command.

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = pkg_tools::RpgArgs::parse();
    pkg_tools::cmd_rpg(args)
}
