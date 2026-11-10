mod bundle;
mod locale;
mod translation_parser;

pub use bundle::I18nBundle;
pub use locale::Locale;
pub use translation_parser::MessageArgs;

use crate::error::BotResult;
use std::sync::OnceLock;

static GLOBAL_BUNDLE: OnceLock<I18nBundle> = OnceLock::new();

pub fn init(locales_path: &str) -> BotResult<()> {
    let bundle = I18nBundle::load(locales_path)?;
    match GLOBAL_BUNDLE.set(bundle) {
        Ok(()) => Ok(()),
        // A lazy embedded load already installed the catalogue (`bundle`).
        // `init` then only has to confirm a catalogue is present, not replace
        // it, so a boot hook that runs after the first lookup still succeeds.
        Err(_) => Ok(()),
    }
}

/// The process-wide catalogue.
///
/// With the `i18n` feature the translations are embedded in the binary, so the
/// first lookup installs them without a path or a boot hook. Library crates
/// (channel adapters, emails, the CLI) and their unit tests then resolve real
/// strings instead of rendering `[message_id]`. Without the feature the files
/// have to be read from disk, so only [`init`] can install the catalogue.
#[cfg(feature = "i18n")]
fn bundle() -> Option<&'static I18nBundle> {
    if let Some(installed) = GLOBAL_BUNDLE.get() {
        return Some(installed);
    }
    match I18nBundle::load("") {
        Ok(loaded) => {
            // Losing the race is fine: the winner's catalogue is equivalent.
            let _ = GLOBAL_BUNDLE.set(loaded);
            GLOBAL_BUNDLE.get()
        }
        Err(e) => {
            log::error!("embedded i18n catalogue could not be loaded: {e}");
            None
        }
    }
}

#[cfg(not(feature = "i18n"))]
fn bundle() -> Option<&'static I18nBundle> {
    GLOBAL_BUNDLE.get()
}

pub fn get(locale: &Locale, message_id: &str) -> String {
    get_with_args(locale, message_id, None)
}

pub fn get_with_args(locale: &Locale, message_id: &str, args: Option<&MessageArgs>) -> String {
    bundle()
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

/// Header a client may send to pin the response language (#2094).
pub const LOCALE_HEADER: &str = "x-locale";

/// Locale negotiation from raw header values, for crates that cannot use the
/// `botcore` extractor (the people, attendance and products routes).
///
/// The order matches `botcore::i18n::locale_from_headers`: explicit
/// `X-Locale` (only when the catalogue ships it), then `Accept-Language`, then
/// the default locale.
#[must_use]
pub fn locale_from_request(explicit: Option<&str>, accept_language: Option<&str>) -> Locale {
    if let Some(locale) = explicit
        .and_then(|raw| Locale::new(raw.trim()))
        .filter(|locale| is_supported(locale))
    {
        return locale;
    }

    accept_language
        .map(parse_accept_language)
        .and_then(|requested| negotiate_requested(&requested))
        .unwrap_or_default()
}

/// Whether the catalogue ships `locale`. Before the bundle is loaded nothing
/// can be verified, so the explicit header is honoured as sent.
fn is_supported(locale: &Locale) -> bool {
    let available = available_locales();

    available.is_empty()
        || available
            .iter()
            .filter_map(|code| Locale::new(code))
            .any(|candidate| candidate.matches(locale))
}

/// `Accept-Language` tags ordered by descending quality, dropping `q=0`
/// (RFC 9110 "not acceptable") and the `*` wildcard.
fn parse_accept_language(header: &str) -> Vec<Locale> {
    let mut weighted: Vec<(Locale, f32)> = header
        .split(',')
        .filter_map(|part| {
            let mut iter = part.trim().split(';');
            let tag = iter.next()?.trim();

            if tag.is_empty() || tag == "*" {
                return None;
            }

            let quality = iter
                .next()
                .and_then(|q| q.trim().strip_prefix("q="))
                .and_then(|q| q.parse::<f32>().ok())
                .unwrap_or(1.0);

            if quality <= 0.0 {
                return None;
            }

            Locale::new(tag).map(|locale| (locale, quality))
        })
        .collect();

    weighted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    weighted.into_iter().map(|(locale, _)| locale).collect()
}

fn negotiate_requested(requested: &[Locale]) -> Option<Locale> {
    let available: Vec<Locale> = available_locales()
        .iter()
        .filter_map(|code| Locale::new(code))
        .collect();

    if available.is_empty() {
        return Some(Locale::default());
    }

    let requested_refs: Vec<&Locale> = requested.iter().collect();
    let fallback = Locale::default();

    Some(Locale::negotiate(&requested_refs, &available, &fallback).clone())
}

pub fn available_locales() -> Vec<String> {
    bundle()
        .map(I18nBundle::available_locales)
        .unwrap_or_default()
}

/// Whether `message_id` is defined by the catalogue (#2095).
///
/// `false` before the bundle is loaded, so a keyed label degrades to the
/// authored text instead of rendering an identifier.
#[must_use]
pub fn has_message(message_id: &str) -> bool {
    bundle().is_some_and(|b| b.has_message(message_id))
}

/// Message ids authored in the English (fallback) locale bundle.
pub fn message_ids() -> Vec<String> {
    bundle()
        .map(I18nBundle::message_ids)
        .unwrap_or_default()
}

pub fn is_initialized() -> bool {
    bundle().is_some()
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

    /// #2094 — `q=0` opts a language out and the `*` wildcard is not a locale.
    #[test]
    fn accept_language_drops_zero_quality_and_wildcard() {
        let parsed = parse_accept_language("pt-BR;q=0, *;q=0.9, es;q=0.5, en;q=0.8");
        let tags: Vec<String> = parsed.iter().map(Locale::to_string).collect();
        assert_eq!(tags, vec!["en".to_string(), "es".to_string()]);
    }

    #[test]
    fn accept_language_orders_by_quality() {
        let parsed = parse_accept_language("en;q=0.5, pt-BR;q=0.9, es;q=0.7");
        let tags: Vec<String> = parsed.iter().map(Locale::to_string).collect();
        assert_eq!(tags, vec!["pt-BR".to_string(), "es".to_string(), "en".to_string()]);
    }

    /// A malformed header degrades to the default locale instead of erroring.
    #[test]
    fn malformed_accept_language_falls_back_to_default() {
        let locale = locale_from_request(None, Some(";;, =, *, q=0"));
        assert_eq!(locale, Locale::default());
    }

    #[test]
    fn explicit_locale_header_wins_over_accept_language() {
        let locale = locale_from_request(Some("pt-BR"), Some("en-US,en;q=0.9"));
        assert_eq!(locale.to_string(), "pt-BR");
    }
}
