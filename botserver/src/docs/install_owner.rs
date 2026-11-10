//! Per-install Docs owner for the single-user desktop profile (#2135).
//!
//! The owner id itself is generated and persisted by
//! [`botdocs::local_owner`]; this module only tells it where the install
//! configuration lives, so the id survives restarts and differs between
//! installs.

use std::path::PathBuf;

/// Resolves the Docs namespace owner of this install.
///
/// Reads (or creates) `docs_local_owner` in the install's
/// `conf/system/directory_config.json` — the same per-install file that holds
/// the SaaS JWT secret. Called once when the Docs router is built.
pub fn docs_local_owner_id() -> String {
    let stack_path = botcore::shared::utils::get_stack_path();
    let config_path = PathBuf::from(stack_path).join("conf/system/directory_config.json");
    botdocs::local_owner::resolve_from_config(&config_path)
}
