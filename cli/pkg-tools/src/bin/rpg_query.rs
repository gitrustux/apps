// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! rpg-query - Query installed packages
//!
//! Replacement for the Linux dpkg-query command.

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = pkg_tools::RpgQueryArgs::parse();
    pkg_tools::cmd_rpg_query(args)
}
