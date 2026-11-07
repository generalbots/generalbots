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
    let mut keys: Vec<String> = translation_keys_actions::TRANSLATION_KEYS
        .iter()
        .chain(translation_keys_common::TRANSLATION_KEYS.iter())
        .chain(translation_keys_tours::TRANSLATION_KEYS.iter())
        .map(|key| (*key).to_string())
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
