// Bootstrap type definitions
use crate::core::package_manager::InstallMode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug)]
pub struct ComponentInfo {
    pub name: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapManager {
    pub install_mode: InstallMode,
    pub tenant: Option<String>,
    pub stack_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BootstrapProgress {
    StartingComponent(String),
    InstallingComponent(String),
    UploadingTemplates,
    BootstrapComplete,
    BootstrapError(String),
}

impl std::fmt::Display for BootstrapProgress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StartingComponent(name) => write!(f, "Installing: {}", name),
            Self::InstallingComponent(name) => write!(f, "Installing: {}", name),
            Self::UploadingTemplates => write!(f, "Uploading templates"),
            Self::BootstrapComplete => write!(f, "Complete"),
            Self::BootstrapError(err) => write!(f, "Error: {}", err),
        }
    }
}
