//! Component registrars: one file per component family, each inserting its
//! entries into `PackageManager::all_components()`.
//!
//! #1732 — the former flat duplicate registrar file was folded into this
//! directory (`register_workspace`, `register_databases`, `register_secrets`,
//! `register_observability`). Every `register_*` function kept its name and is
//! re-exported below, so the registry assembled in
//! `installer::manager_construction` is unchanged.

pub(crate) use crate::component::ComponentConfig;
pub(crate) use crate::installer::{
    get_component_url, get_llama_cpp_url, get_model_url, LLAMA_CPP_VERSION,
};
pub(crate) use botlib::security::get_stack_path;
pub(crate) use log::{info, warn};
pub(crate) use std::collections::HashMap;

mod register_botmodels;
mod register_databases;
mod register_directory;
mod register_drive;
mod register_observability;
mod register_secrets;
mod register_workspace;

pub use register_botmodels::*;
pub use register_databases::*;
pub use register_directory::*;
pub use register_drive::*;
pub use register_observability::*;
pub use register_secrets::*;
pub use register_workspace::*;
