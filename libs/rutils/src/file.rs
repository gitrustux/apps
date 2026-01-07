// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! File utilities

use anyhow::{Context, Result};
use std::path::Path;

pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .with_context(|| format!("failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

pub fn safe_write(path: &Path, content: &[u8]) -> Result<()> {
    // Write to temporary file first
    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, content)
        .with_context(|| format!("failed to write temp file: {}", temp_path.display()))?;

    // Atomic rename
    std::fs::rename(&temp_path, path)
        .with_context(|| format!("failed to rename {} to {}", temp_path.display(), path.display()))?;

    Ok(())
}

pub fn read_file_string(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("failed to read file: {}", path.display()))
}

#[cfg(feature = "sha2")]
pub fn compute_checksum(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}
