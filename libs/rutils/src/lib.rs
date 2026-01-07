// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! rutils - Rustica Utilities Library
//!
//! Shared utilities for Rustica OS applications.

pub mod process;
pub mod file;
pub mod auth;

pub use process::{Process, ProcessInfo};
pub use file::{ensure_dir, safe_write};

#[cfg(feature = "sha2")]
pub use file::compute_checksum;

pub use auth::{authenticate_user, get_username, get_home_dir, switch_user};

#[cfg(feature = "serde")]
pub mod config;

#[cfg(feature = "serde")]
pub use config::{Config, ConfigFile};
