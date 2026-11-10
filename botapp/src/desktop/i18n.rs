//! Native strings for the Tauri shell (#2095).
//!
//! The tray menu, the splash status line and the native dialogs live outside the
//! webview, so they cannot use `window.gbT` / the Fluent catalogue in
//! `botlib/locales`. They read a small JSON catalogue instead, one file per
//! locale under `botapp/i18n/`, embedded at compile time with `include_str!` —
//! no i18n crate, no runtime file lookup, no network.
//!
//! ## Translating
//!
//! 1. Copy `botapp/i18n/en.json` to `botapp/i18n/<locale>.json` (BCP-47, e.g.
//!    `ja.json`, `zh-CN.json`).
//! 2. Translate the values only; the keys are the contract.
//! 3. Add the file to `CATALOGUES` below and to `botapp/README.md`.
//!
//! ## Locale resolution
//!
//! `GB_LOCALE` (the app/`gb-locale` override) → `LC_ALL` → `LANG` → `en`. An
//! unknown or partially translated locale falls back to English per key, so a
//! missing entry never renders an empty label or a raw key.

use std::collections::HashMap;
use std::sync::OnceLock;

const CATALOGUES: &[(&str, &str)] = &[
    ("en", include_str!("../../i18n/en.json")),
    ("pt-BR", include_str!("../../i18n/pt-BR.json")),
    ("es", include_str!("../../i18n/es.json")),
];

const FALLBACK_LOCALE: &str = "en";

type Catalogue = HashMap<String, String>;

static ACTIVE: OnceLock<Catalogue> = OnceLock::new();
static ENGLISH: OnceLock<Catalogue> = OnceLock::new();

fn parse(raw: &str) -> Catalogue {
    match serde_json::from_str::<Catalogue>(raw) {
        Ok(map) => map,
        Err(e) => {
            log::error!("botapp i18n: could not parse an embedded catalogue: {e}");
            Catalogue::new()
        }
    }
}

fn english() -> &'static Catalogue {
    ENGLISH.get_or_init(|| {
        CATALOGUES
            .iter()
            .find(|(code, _)| *code == FALLBACK_LOCALE)
            .map_or_else(Catalogue::new, |(_, raw)| parse(raw))
    })
}

/// The locale this process renders native strings in, normalised to the codes
/// used by the catalogue (`pt_BR.UTF-8` → `pt-BR`).
#[must_use]
pub fn locale() -> String {
    let requested = std::env::var("GB_LOCALE")
        .ok()
        .or_else(|| std::env::var("LC_ALL").ok())
        .or_else(|| std::env::var("LANG").ok())
        .unwrap_or_default();
    resolve(&requested)
}

fn resolve(requested: &str) -> String {
    let normalised = normalise(requested);
    if normalised.is_empty() {
        return FALLBACK_LOCALE.to_string();
    }
    if let Some((code, _)) = CATALOGUES.iter().find(|(code, _)| *code == normalised) {
        return (*code).to_string();
    }
    // A bare language tag picks the first catalogue of that language
    // (`pt` → `pt-BR`, `es-MX` → `es`).
    let language = normalised.split('-').next().unwrap_or_default();
    CATALOGUES
        .iter()
        .find(|(code, _)| code.split('-').next() == Some(language))
        .map_or_else(|| FALLBACK_LOCALE.to_string(), |(code, _)| (*code).to_string())
}

fn normalise(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // `pt_BR.UTF-8` / `pt_BR@euro` → `pt-BR`; `C` / `POSIX` mean "unset".
    let base = trimmed
        .split('.')
        .next()
        .unwrap_or_default()
        .split('@')
        .next()
        .unwrap_or_default();
    if base.eq_ignore_ascii_case("c") || base.eq_ignore_ascii_case("posix") {
        return String::new();
    }
    let mut parts = base.split(['_', '-']);
    let language = parts.next().unwrap_or_default().to_ascii_lowercase();
    if language.is_empty() {
        return String::new();
    }
    match parts.next() {
        Some(region) if !region.is_empty() => format!("{language}-{}", region.to_ascii_uppercase()),
        _ => language,
    }
}

fn active() -> &'static Catalogue {
    ACTIVE.get_or_init(|| {
        let code = locale();
        if code == FALLBACK_LOCALE {
            return english().clone();
        }
        CATALOGUES
            .iter()
            .find(|(candidate, _)| *candidate == code)
            .map_or_else(|| english().clone(), |(_, raw)| parse(raw))
    })
}

/// Native string for `key` in the process locale, falling back to English and
/// then to the key itself (never to an empty label).
#[must_use]
pub fn t(key: &str) -> String {
    t_args(key, &[])
}

/// [`t`] with `{name}` placeholders replaced by the supplied values.
#[must_use]
pub fn t_args(key: &str, args: &[(&str, &str)]) -> String {
    let template = active()
        .get(key)
        .or_else(|| english().get(key))
        .map_or(key, String::as_str);
    let mut text = template.to_string();
    for (name, value) in args {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_is_the_fallback_for_unknown_locales() {
        assert_eq!(resolve("ja_JP.UTF-8"), FALLBACK_LOCALE);
        assert_eq!(resolve("C"), FALLBACK_LOCALE);
        assert_eq!(resolve(""), FALLBACK_LOCALE);
    }

    #[test]
    fn regional_tags_resolve_to_a_catalogue() {
        assert_eq!(resolve("pt_BR.UTF-8"), "pt-BR");
        assert_eq!(resolve("pt"), "pt-BR");
        assert_eq!(resolve("es-MX"), "es");
        assert_eq!(resolve("en_US.UTF-8"), "en");
    }

    #[test]
    fn every_key_exists_in_every_catalogue() {
        for (code, raw) in CATALOGUES {
            let map = parse(raw);
            for key in english().keys() {
                assert!(map.contains_key(key), "{code} is missing {key}");
            }
        }
    }

    #[test]
    fn placeholders_are_replaced() {
        let text = t_args("download-write-failed", &[("path", "/tmp/x"), ("error", "denied")]);
        assert!(text.contains("/tmp/x"), "{text}");
        assert!(text.contains("denied"), "{text}");
    }

    #[test]
    fn unknown_key_returns_the_key() {
        assert_eq!(t("no-such-key"), "no-such-key");
    }
}
