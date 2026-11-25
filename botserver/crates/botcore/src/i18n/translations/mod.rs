//! Every translation key exposed by the i18n API.

use super::{t, Locale};
use std::collections::HashSet;

mod translation_keys_actions;
mod translation_keys_common;
mod translation_keys_tours;

/// Every translation key exposed by the API: the explicit Rust catalogue in
/// order, followed by any additional key already authored in the `.ftl`
/// bundles. The union matters because the UI (`data-i18n` attributes and
/// generated chrome) evolves faster than the hand-maintained Rust list — a key
/// present only in an FTL file used to reach the browser as its raw identifier
/// (#1853).
fn translation_keys() -> Vec<String> {
    let mut keys: Vec<String> = translation_keys_actions::translation_keys()
        .chain(translation_keys_common::TRANSLATION_KEYS.iter().copied())
        .chain(translation_keys_tours::TRANSLATION_KEYS.iter().copied())
        .map(str::to_string)
        .collect();
    let mut seen: HashSet<String> = keys.iter().cloned().collect();
    for key in botlib::i18n::message_ids() {
        if seen.insert(key.clone()) {
            keys.push(key);
        }
    }
    keys
}

pub fn get_translations_json(locale: &Locale) -> serde_json::Value {
    let mut translations = serde_json::Map::new();

    for key in translation_keys() {
        let value = t(locale, &key);
        if value.contains('[') && value == format!("[{key}]") {
            log::warn!(
                "i18n: Missing translation for key: {} in locale: {:?}",
                key,
                locale
            );
        }
        translations.insert(key, serde_json::Value::String(value));
    }

    log::info!(
        "i18n: Returning {} translations for locale: {}",
        translations.len(),
        locale
    );
    serde_json::Value::Object(translations)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #2921 — the API catalogue is the union of the hand-maintained lists and
    /// every key embedded in the `.ftl` bundles. A key authored in a catalogue
    /// must reach `GET /api/i18n/{locale}` without being added to a Rust list;
    /// this pins that union so a registration regression fails the build.
    #[test]
    fn api_catalogue_serves_every_embedded_key() {
        let keys = translation_keys();
        assert!(!keys.is_empty(), "the i18n API catalogue must not be empty");
        let embedded = botlib::i18n::message_ids();
        assert!(
            !embedded.is_empty(),
            "the embedded English catalogue must be loaded"
        );
        for key in embedded {
            assert!(
                keys.contains(&key),
                "API catalogue must serve embedded key {key}"
            );
        }
        for key in [
            "chat-copy",
            "sidebar-apps-manage",
            "toast-open",
            "billing-export",
        ] {
            assert!(
                keys.iter().any(|served| served == key),
                "API catalogue must serve #2921 key {key}"
            );
        }
    }
}
