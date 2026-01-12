// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! uname - Print system information
//!
//! Replacement for the Linux uname command.

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = sys_tools::UnameArgs::parse();
    sys_tools::cmd_uname(args)
}
