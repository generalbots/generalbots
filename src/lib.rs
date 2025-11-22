pub mod auth;
pub mod automation;
pub mod basic;
pub mod bootstrap;
pub mod bot;
pub mod channels;
pub mod config;
pub mod context;
pub mod drive;
pub mod drive_monitor;
#[cfg(feature = "email")]
pub mod email;
pub mod file;
pub mod llm;
pub mod llm_models;
pub mod meet;
pub mod nvidia;
pub mod package_manager;
pub mod session;
pub mod shared;
pub mod tests;
pub mod ui_tree;
pub mod web_server;

// Bootstrap progress enum used by UI
#[derive(Debug, Clone)]
pub enum BootstrapProgress {
    StartingBootstrap,
    InstallingComponent(String),
    StartingComponent(String),
    UploadingTemplates,
    ConnectingDatabase,
    StartingLLM,
    BootstrapComplete,
    BootstrapError(String),
}
