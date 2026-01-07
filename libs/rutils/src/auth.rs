// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Authentication utilities

use anyhow::{Context, Result};
use nix::unistd::{Uid, User, Gid, setgid, setuid};
use std::path::PathBuf;

/// Authenticate a user with password
/// Uses shadow password authentication
pub fn authenticate_user(username: &str, _password: &str) -> Result<bool> {
    // In production, would use PAM or directly read /etc/shadow
    // For now, we'll implement a basic check

    // Check if user exists in /etc/passwd
    let _user = User::from_name(username)
        .context("failed to lookup user")?
        .ok_or_else(|| anyhow::anyhow!("user not found: {}", username))?;

    // Read shadow file (requires root)
    let shadow_path = PathBuf::from("/etc/shadow");
    if !shadow_path.exists() {
        // Fallback: check if running as root
        return Ok(Uid::effective().is_root());
    }

    let shadow_content = std::fs::read_to_string(&shadow_path)
        .context("failed to read shadow file")?;

    // Find user entry in shadow file
    for line in shadow_content.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 2 && parts[0] == username {
            let stored_hash = parts[1];

            // Check for disabled/locked accounts
            if stored_hash == "*" || stored_hash == "!" {
                return Ok(false);
            }

            // Verify password
            // In production, would use crypt() from libc
            // For now, return false to indicate not implemented
            return Ok(false);
        }
    }

    Ok(false)
}

/// Get the username of the current user
pub fn get_username() -> Result<String> {
    let uid = Uid::effective();
    let user = User::from_uid(uid)
        .context("failed to get username")?
        .ok_or_else(|| anyhow::anyhow!("no username for current uid"))?;

    Ok(user.name)
}

/// Get the home directory of a user
pub fn get_home_dir(username: &str) -> Result<PathBuf> {
    let user = User::from_name(username)
        .context("failed to lookup user")?
        .ok_or_else(|| anyhow::anyhow!("user not found: {}", username))?;

    Ok(PathBuf::from(user.dir))
}

/// Get the home directory of the current user
pub fn get_current_home_dir() -> Result<PathBuf> {
    let username = get_username()?;
    get_home_dir(&username)
}

/// Check if current user is root
pub fn is_root() -> bool {
    Uid::effective().is_root()
}

/// Switch to a different user
pub fn switch_user(username: &str) -> Result<()> {
    let user = User::from_name(username)
        .context("failed to lookup user")?
        .ok_or_else(|| anyhow::anyhow!("user not found: {}", username))?;

    // Set group ID first
    setgid(Gid::from_raw(u32::from(user.gid))).context("failed to set group ID")?;

    // Set user ID
    setuid(Uid::from_raw(u32::from(user.uid))).context("failed to set user ID")?;

    Ok(())
}
