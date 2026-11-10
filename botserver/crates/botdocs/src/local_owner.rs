//! Stable per-install owner for the desktop-local Docs namespace (#2135).
//!
//! A single-user desktop install has no directory account, so the Docs app used
//! to fall back to the global legacy namespace. That id is identical on
//! every install, which means two installs pointed at the same Drive would share
//! documents — and a leaked bucket would expose all of them.
//!
//! The owner here is a random UUID generated once per install and persisted in
//! the install's `directory_config.json` (the same file that holds the
//! per-install SaaS JWT secret). It is stable across restarts, distinct between
//! installs, and prefixed with `local-` so it can never be mistaken for a
//! directory user id.

use std::path::Path;

use serde_json::Value;
use uuid::Uuid;

use crate::legacy::is_legacy_owner;
use crate::storage_core::sanitize_owner;

/// Key holding the install owner in the directory configuration file.
pub const CONFIG_KEY: &str = "docs_local_owner";

/// Prefix of a per-install owner id.
pub const LOCAL_OWNER_PREFIX: &str = "local-";

/// A fresh per-install owner id.
pub fn new_owner_id() -> String {
    format!("{LOCAL_OWNER_PREFIX}{}", Uuid::new_v4())
}

/// Reads the per-install owner from `config_path`, creating it when absent.
///
/// A stored value that cannot own a namespace (empty, the legacy shared
/// owner, or one that sanitises away) is replaced by a fresh id. A
/// write failure is logged and the fresh id is still returned: the install then
/// gets a new namespace per boot instead of sharing the legacy one, which is the
/// safe direction to fail in.
pub fn resolve_from_config(config_path: &Path) -> String {
    if let Some(owner) = read_persisted(config_path) {
        return owner;
    }

    let owner = new_owner_id();
    if let Err(error) = persist(config_path, &owner) {
        log::warn!(
            "docs: could not persist the desktop-local docs owner in {}: {error}",
            config_path.display()
        );
    } else {
        log::info!(
            "docs: desktop-local docs namespace owner {} stored in {}",
            owner,
            config_path.display()
        );
    }
    owner
}

/// Whether `value` may be used as a Docs owner segment.
pub fn is_usable_owner(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && !is_legacy_owner(trimmed)
        && sanitize_owner(trimmed).chars().any(char::is_alphanumeric)
}

fn read_persisted(config_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(config_path).ok()?;
    let config: Value = serde_json::from_str(&content).ok()?;
    let owner = config.get(CONFIG_KEY)?.as_str()?;
    if is_usable_owner(owner) {
        Some(owner.trim().to_string())
    } else {
        None
    }
}

fn persist(config_path: &Path, owner: &str) -> std::io::Result<()> {
    let mut config: Value = std::fs::read_to_string(config_path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()));
    if !config.is_object() {
        config = Value::Object(serde_json::Map::new());
    }
    config[CONFIG_KEY] = Value::String(owner.to_string());

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let serialized = serde_json::to_string_pretty(&config)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    std::fs::write(config_path, serialized)
}

#[cfg(test)]
#[path = "local_owner_tests.rs"]
mod tests;
