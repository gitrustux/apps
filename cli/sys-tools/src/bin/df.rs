// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! df - Report file system disk space usage
//!
//! Replacement for the Linux df command.

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = sys_tools::DfArgs::parse();
    sys_tools::cmd_df(args)
}
