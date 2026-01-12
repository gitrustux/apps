// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! File Tools Library for Rustica OS
//!
//! Public API for ls, cat, echo, and pwd commands.

pub mod main;

pub use main::{
    LsArgs, CatArgs, EchoArgs, PwdArgs,
    FileInfo, LsOutput, CatOutput, PwdOutput,
    cmd_ls, cmd_cat, cmd_echo, cmd_pwd,
};
