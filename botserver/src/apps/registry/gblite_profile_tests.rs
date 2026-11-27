//! Guard for the government desktop edition's product file (#2560).
//!
//! The edition ships Chat and Drive with the agent workspace (agent-vm).
//! Every other registry app has to be listed as hidden in `.product.gblite`: an
//! app that is neither always visible nor hidden is treated as a preview app,
//! and the edition has no preview tier. A new registry app therefore fails here
//! until the file names it.

use super::all_apps;
use botcore::product::ProductConfig;
use std::collections::HashSet;

/// Core surfaces are always available and are never listed in a product file.
const CORE_APPS: &[&str] = &["about", "settings", "auth"];
const EDITION_APPS: &[&str] = &["chat", "drive"];

fn edition_profile() -> ProductConfig {
    match ProductConfig::parse(include_str!("../../../.product.gblite")) {
        Ok(config) => config,
        Err(error) => panic!(".product.gblite does not parse: {error}"),
    }
}

#[test]
fn edition_shows_chat_and_drive_only() {
    let config = edition_profile();
    let enabled: HashSet<String> = config.get_enabled_apps().into_iter().collect();
    let expected: HashSet<String> = EDITION_APPS.iter().map(|id| (*id).to_string()).collect();
    assert_eq!(enabled, expected, ".product.gblite apps= must be chat,drive");
    assert!(
        config.get_preview_apps().is_empty(),
        ".product.gblite must not declare preview apps"
    );
    assert_eq!(config.suite_default_app.as_deref(), Some("chat"));
    assert_eq!(config.edition.as_deref(), Some("desktop"));
    assert_eq!(config.shell.as_deref(), Some("assistant"));
    assert!(!config.no_security, "the edition requires sign-in");
}

#[test]
fn every_other_registry_app_is_hidden() {
    let config = edition_profile();
    let mut unclassified: Vec<String> = all_apps()
        .into_iter()
        .map(|app| app.id)
        .filter(|id| !CORE_APPS.contains(&id.as_str()))
        .filter(|id| !EDITION_APPS.contains(&id.as_str()))
        .filter(|id| !config.is_app_hidden(id))
        .collect();
    unclassified.sort();
    assert!(
        unclassified.is_empty(),
        "registry apps missing from hidden_apps= in .product.gblite: {unclassified:?}"
    );
}
