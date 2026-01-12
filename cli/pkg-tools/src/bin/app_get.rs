// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! app-get - Low-level package management
//!
//! Replacement for the Linux apt-get command.

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = pkg_tools::AppGetArgs::parse();
    pkg_tools::cmd_app_get(args)
}
