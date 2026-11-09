//! Package manager: component registry, allowed shell helpers and installer URLs.

pub(crate) use crate::component::ComponentConfig;
pub(crate) use crate::os::detect_os;
pub(crate) use crate::{InstallMode, OsType};
pub(crate) use crate::installer_regs;
pub(crate) use crate::installer_vault;
pub(crate) use botlib::security::SafeCommand;
pub(crate) use anyhow::Result;
pub(crate) use log::{info, trace, warn};
pub(crate) use serde::Deserialize;
pub(crate) use std::collections::HashMap;
pub(crate) use std::path::PathBuf;

#[derive(Deserialize, Debug)]
struct ComponentEntry {
    url: String,
    #[serde(default)]
    url_win: Option<String>,
    #[serde(default)]
    url_arm64: Option<String>,
    // #2001 — the pinned digests travel in the same table; the container
    // installer needs them where there is no DownloadCache instance.
    #[serde(default)]
    sha256: String,
    #[serde(default)]
    sha256_win: String,
    #[serde(default)]
    sha256_arm64: String,
}

#[derive(Deserialize, Debug)]
struct ThirdPartyConfig {
    components: HashMap<String, ComponentEntry>,
    #[serde(default)]
    models: HashMap<String, ComponentEntry>,
}

static THIRDPARTY_CONFIG: std::sync::OnceLock<ThirdPartyConfig> = std::sync::OnceLock::new();

#[derive(Debug)]
pub struct PackageManager {
    pub mode: InstallMode,
    pub os_type: OsType,
    pub base_path: PathBuf,
    pub tenant: String,
    pub components: HashMap<String, ComponentConfig>,
}

mod component_entry;
mod urls;
mod shell;
mod llama;
mod manager_construction;
mod manager_run;

pub use urls::*;
pub use shell::*;
pub use llama::*;
