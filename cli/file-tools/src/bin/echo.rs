// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! echo - Display text
//!
//! Replacement for the Linux echo command.

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = file_tools::EchoArgs::parse();
    file_tools::cmd_echo(args)
}
