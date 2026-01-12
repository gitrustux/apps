// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! System Tools Library for Rustica OS
//!
//! Public API for uname, hostname, lsb_release, and free commands.

pub mod main;

pub use main::{UnameArgs, HostnameArgs, LsbReleaseArgs, FreeArgs, DfArgs, SysInfoArgs};
pub use main::{cmd_uname, cmd_hostname, cmd_lsb_release, cmd_free, cmd_df, cmd_sysinfo};
