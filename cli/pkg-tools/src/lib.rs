// Copyright 2025 The Rustux Authors
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//! Package Management Library for Rustica OS
//!
//! Public API for app, app-get, app-cache, rpg, and rpg-query commands.

pub mod main;

pub use main::{
    AppArgs, AppGetArgs, AppCacheArgs, RpgArgs, RpgQueryArgs,
    PackageInfo, PackageStatus, PackageState, PackageListEntry, PackageCacheStats,
    cmd_app, cmd_app_get, cmd_app_cache, cmd_rpg, cmd_rpg_query,
};
