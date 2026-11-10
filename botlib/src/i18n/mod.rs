mod bundle;
mod locale;
mod translation_parser;

pub use bundle::I18nBundle;
pub use locale::Locale;
pub use translation_parser::MessageArgs;

use crate::error::{BotError, BotResult};
use std::sync::OnceLock;

static GLOBAL_BUNDLE: OnceLock<I18nBundle> = OnceLock::new();

pub fn init(locales_path: &str) -> BotResult<()> {
    let bundle = I18nBundle::load(locales_path)?;
    GLOBAL_BUNDLE
        .set(bundle)
        .map_err(|_| BotError::config("i18n already initialized"))
}

pub fn get(locale: &Locale, message_id: &str) -> String {
    get_with_args(locale, message_id, None)
}

pub fn get_with_args(locale: &Locale, message_id: &str, args: Option<&MessageArgs>) -> String {
    GLOBAL_BUNDLE
        .get()
        .map(|b| b.get_message(locale, message_id, args))
        .unwrap_or_else(|| format!("[{message_id}]"))
}

/// Translates `message_id` for `locale` with a `(name, value)` argument list.
///
/// Library crates that cannot depend on `botcore` (emails, channel adapters,
/// the CLI) call this instead of rebuilding the argument map themselves: the
/// catalogue, the negotiation and the fallback chain stay in one place
/// (#2094). `botcore::i18n` re-exports it for server-side call sites.
pub fn translate(locale: &Locale, message_id: &str, args: &[(&str, &str)]) -> String {
    if args.is_empty() {
        return get(locale, message_id);
    }

    let map: MessageArgs = args
        .iter()
        .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
        .collect();

    get_with_args(locale, message_id, Some(&map))
}

/// Language of a bot/workspace, read from its `locale` configuration value.
///
/// Channel adapters have no request and no `users` row for the person writing
/// (WhatsApp and Telegram identify a phone or a chat id), so the bot/org
/// default is the only signal available. `read` is the caller's bot-config
/// accessor; anything missing or unknown falls back to English.
pub fn locale_from_config(read: impl FnOnce(&str) -> Option<String>) -> Locale {
    read("locale")
        .and_then(|raw| Locale::new(raw.trim()))
        .unwrap_or_default()
}

pub fn available_locales() -> Vec<String> {
    GLOBAL_BUNDLE
        .get()
        .map(I18nBundle::available_locales)
        .unwrap_or_default()
}

/// Whether `message_id` is defined by the catalogue (#2095).
///
/// `false` before the bundle is loaded, so a keyed label degrades to the
/// authored text instead of rendering an identifier.
#[must_use]
pub fn has_message(message_id: &str) -> bool {
    GLOBAL_BUNDLE
        .get()
        .is_some_and(|b| b.has_message(message_id))
}

/// Message ids authored in the English (fallback) locale bundle.
pub fn message_ids() -> Vec<String> {
    GLOBAL_BUNDLE
        .get()
        .map(I18nBundle::message_ids)
        .unwrap_or_default()
}

pub fn is_initialized() -> bool {
    GLOBAL_BUNDLE.get().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_message_returns_key() {
        let locale = Locale::default();
        let result = get(&locale, "nonexistent-key");
        assert_eq!(result, "[nonexistent-key]");
    }

    #[test]
    fn test_is_initialized_before_init() {
        assert!(!is_initialized() || is_initialized());
    }

    /// #2095 — keyed labels resolve only when the catalogue defines the key,
    /// so an authored label is never replaced by an identifier.
    #[test]
    fn has_message_reads_the_english_key_set() {
        let locale_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("locales");
        let loaded = I18nBundle::load(locale_dir.to_str().unwrap_or("locales"));
        assert!(loaded.is_ok(), "the shipped catalogue must load");
        let Ok(bundle) = loaded else { return };

        assert!(bundle.has_message("bot-greeting-default"));
        assert!(bundle.has_message("bot-suggest-check-my-balance"));
        assert!(!bundle.has_message("Marcar consulta"));
        assert!(!bundle.has_message("bot-suggest-not-a-real-key"));
    }

    #[test]
    fn has_message_is_false_without_a_loaded_catalogue() {
        // A process that never initialised i18n keeps the authored text.
        assert!(!has_message("bot-suggest-check-my-balance") || is_initialized());
    }
}
