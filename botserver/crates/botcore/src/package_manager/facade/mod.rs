pub(crate) use crate::package_manager::cache::{CacheResult, DownloadCache};
pub(crate) use crate::package_manager::component::{ComponentConfig, InstallResult};
use crate::shared::utils::{self, get_database_url_sync, parse_database_url};
pub(crate) use anyhow::{Context, Result};
pub(crate) use log::{error, info, trace, warn};
pub(crate) use std::io::Write;

use crate::package_manager::installer::PackageManager;
use crate::package_manager::InstallMode;
use crate::package_manager::OsType;
use botlib::security::command_guard::SafeCommand;
use reqwest::Client;
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::path::PathBuf;

/// Desktop-local builds are offline by default: the downloadable bundle carries
/// every component next to the binary, so provisioning never touches the
/// network. A slim download ships no bundled installers; its launcher sets
/// `GB_DESKTOP_ALLOW_DOWNLOAD=1` and the provisioner resolves what it needs from
/// `3rdparty.toml` instead of failing the component.
pub(crate) fn desktop_download_allowed() -> bool {
    matches!(
        std::env::var("GB_DESKTOP_ALLOW_DOWNLOAD").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

mod safe_lxc;
mod install;
mod install_container;
mod initialize_vault;
mod generate_connection_info;
mod install_system_packages;
mod install_binary;
mod setup_port_forwarding;
mod collapse_single_subdirectory;

use safe_lxc::*;
use collapse_single_subdirectory::*;
