pub(crate) use crate::package_manager::component::ComponentConfig;
pub(crate) use crate::package_manager::os::detect_os;
pub(crate) use crate::package_manager::{InstallMode, OsType};
pub(crate) use anyhow::{Context, Result};
pub(crate) use log::{info, trace, warn};
pub(crate) use std::io::Write;

#[derive(Debug)]
pub struct PackageManager {
    pub mode: InstallMode,
    pub os_type: OsType,
    pub base_path: PathBuf,
    pub tenant: String,
    pub components: HashMap<String, ComponentConfig>,
}

use botlib::security::command_guard::SafeCommand;
use std::collections::HashMap;
use std::path::PathBuf;

mod safe_sh_command;
mod with_base_path;
mod fetch_vault_credentials;
mod initialize_vault_local;
mod seed_vault_defaults;
mod unseal_vault;

use safe_sh_command::*;
