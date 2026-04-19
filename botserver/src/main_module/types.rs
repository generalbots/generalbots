//! Type definitions for the main application

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
