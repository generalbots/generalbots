// Core modules (always included)
pub mod basic;
pub mod core;

// Re-export shared from core
pub use core::shared;

// Re-exports from core (always included)
pub use core::automation;
pub use core::bootstrap;
pub use core::bot;
pub use core::config;
pub use core::package_manager;
pub use core::session;
pub use core::web_server;

// Feature-gated modules
#[cfg(feature = "attendance")]
pub mod attendance;

#[cfg(feature = "calendar")]
pub mod calendar;

#[cfg(feature = "compliance")]
pub mod compliance;

#[cfg(feature = "console")]
pub mod console;

#[cfg(feature = "desktop")]
pub mod desktop;

#[cfg(feature = "directory")]
pub mod directory;

#[cfg(feature = "drive")]
pub mod drive;

#[cfg(feature = "email")]
pub mod email;

#[cfg(feature = "instagram")]
pub mod instagram;

#[cfg(feature = "llm")]
pub mod llm;

#[cfg(feature = "meet")]
pub mod meet;

#[cfg(feature = "msteams")]
pub mod msteams;

#[cfg(feature = "nvidia")]
pub mod nvidia;

#[cfg(feature = "tasks")]
pub mod tasks;

#[cfg(feature = "vectordb")]
pub mod vector_db;

#[cfg(feature = "weba")]
pub mod weba;

#[cfg(feature = "whatsapp")]
pub mod whatsapp;

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
