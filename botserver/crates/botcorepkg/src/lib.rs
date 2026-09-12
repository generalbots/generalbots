pub mod plugin;
pub mod db_utils;
pub mod alm_setup;
pub mod cache;
pub mod cli;
pub mod cli_display;
pub mod cli_ops;
pub mod cli_secrets;
pub mod component;
pub mod container;
pub mod facade;
pub mod facade_connection;
pub mod facade_container;
pub mod facade_download;
pub mod installer;
pub mod installer_regs;
pub mod installer_regs2;
pub mod installer_vault;
pub mod installer_vault2;
pub mod os;
pub mod setup;

// #1347 — package_manager duplication note: the module that is actually
// invoked at boot is `botcore::package_manager` (botserver main.rs CLI and
// init.rs). The files in this crate that also exist there (setup, cache,
// alm_setup, cli, facade, container, os, component, installer) are the
// drift-prone copies documented in issue #1347 section 3; only `plugin`,
// `component`, `container` and the installer_regs*/installer_vault* component
// tables are shared from here via re-exports and direct calls. Do not edit a
// drifted copy expecting a behavior change at runtime — check which side the
// caller imports first.

use rand::Rng;
use serde::{Deserialize, Serialize};

pub fn generate_random_string(length: usize) -> String {
    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..charset.len());
            charset[idx] as char
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallMode {
    Local,
    Container,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OsType {
    Linux,
    MacOS,
    Windows,
}

pub use installer::PackageManager;
