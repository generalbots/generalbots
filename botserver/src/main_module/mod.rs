#[cfg(feature = "automation")]
pub mod auto_service;
pub mod background;
pub mod cache;
pub mod channel_delivery;
pub mod constants;
pub mod directory_setup;
#[cfg(feature = "drive")]
pub mod drive_monitors;
mod drive_utils;
#[cfg(feature = "drive")]
pub mod event_dispatcher;
#[cfg(all(feature = "drive", feature = "vibe"))]
pub mod git_bot_monitor;
mod health;
mod init;
pub mod init_tasks;
pub mod listen_impl;
pub mod llm_setup;
pub mod media_capabilities;
pub mod migrations;
pub mod routes;
pub mod security_cmd;
mod server;
#[cfg(feature = "digest")]
pub mod settings_digest;
mod shutdown;
pub mod state_builder;
mod types;
pub mod ui_plan;
pub mod wizard_provision;

pub use background::*;
pub use cache::*;
pub use init::*;

/// CLI text locale resolution (`--lang`/`LANG`), owned by `botcore::i18n` so
/// the package-manager help pages use the same setting (#2094).
pub use botcore::i18n::cli_locale;

#[cfg(feature = "automation")]
pub use auto_service::start_automation_service;
pub use constants::get_noise_filters;
#[cfg(feature = "drive")]
pub use drive_utils::*;
pub use health::*;
pub use init_tasks::init_task_scheduler;
pub use migrations::*;
pub use routes::org_handlers::*;
pub use security_cmd::handle_security_command;
pub use server::*;
pub use shutdown::*;
pub use state_builder::*;
pub use types::*;
