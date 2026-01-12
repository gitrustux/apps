// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! app - Package management utility
//!
//! Replacement for the Linux apt command.

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = pkg_tools::AppArgs::parse();
    pkg_tools::cmd_app(args)
}
