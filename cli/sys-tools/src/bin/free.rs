// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! free - Display amount of free and used memory
//!
//! Replacement for the Linux free command.

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = sys_tools::FreeArgs::parse();
    sys_tools::cmd_free(args)
}
