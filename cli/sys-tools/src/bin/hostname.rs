// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! hostname - Show or set system hostname
//!
//! Replacement for the Linux hostname command.

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = sys_tools::HostnameArgs::parse();
    sys_tools::cmd_hostname(args)
}
