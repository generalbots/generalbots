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
