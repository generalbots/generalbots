// ProtectionInstaller/VerifyResult are in botsecurity which depends on botcore
// To break the cycle, security protection functions stay in botserver layer

pub(crate) use crate::package_manager::certs_utils;
pub(crate) use botcoresecrets::{SecretPaths, SecretsManager};
pub(crate) use crate::package_manager::{get_all_components, InstallMode, PackageManager};

const VERSION: &str = env!("CARGO_PKG_VERSION");

use botlib::security::command_guard::SafeCommand;
use anyhow::Result;
use rand::Rng;
use std::collections::HashMap;
use std::env;

mod safe_pkill;
mod cli_run;
mod print_usage;
mod vault_put;
mod rotate_secret;
mod rotate_all_secrets;

use safe_pkill::*;
pub use cli_run::*;
use print_usage::*;
use vault_put::*;
use rotate_secret::*;
use rotate_all_secrets::*;
