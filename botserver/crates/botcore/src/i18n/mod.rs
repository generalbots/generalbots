//! Locale negotiation, extractors and the public i18n surface.
//!
//! The translation catalogue lives in [`translations`].
//!
//! #2087 — there is exactly one `Locale` type: the one in `botlib::i18n`.
//! This module used to re-declare it (with its own parser) and to hard-code
//! `AVAILABLE_LOCALES`, so the server could negotiate a locale the catalogue
//! does not ship. The available set now comes from the loaded bundles.

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::ACCEPT_LANGUAGE, request::Parts},
};
use botlib::i18n::{self, MessageArgs as BotlibMessageArgs};

mod error;
mod routes;
mod translations;
#[cfg(test)]
mod tests;

pub use botlib::i18n::Locale;
pub use error::LocalizedError;
pub use routes::configure_i18n_routes;
pub use translations::get_translations_json;

/// Locales the catalogue is guaranteed to ship. Used only before the bundles
/// are loaded (e.g. during early boot or in unit tests without initialisation).
const BUNDLED_LOCALES: &[&str] = &["en", "pt-BR", "es", "zh-CN", "fr", "de", "ja", "ko"];

pub struct RequestLocale(pub Locale);

impl RequestLocale {
    #[must_use]
    pub fn locale(&self) -> &Locale {
        &self.0
    }

    #[must_use]
    pub fn language(&self) -> &str {
        self.0.language()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RequestLocale
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let locale = parts
            .headers
            .get(ACCEPT_LANGUAGE)
            .and_then(|h| h.to_str().ok())
            .map(parse_accept_language)
            .and_then(|langs| negotiate_locale(&langs))
            .unwrap_or_default();

        Ok(Self(locale))
    }
}

fn parse_accept_language(header: &str) -> Vec<(String, f32)> {
    let mut langs: Vec<(String, f32)> = header
        .split(',')
        .filter_map(|part| {
            let mut iter = part.trim().split(';');
            let lang = iter.next()?.trim().to_string();

            if lang.is_empty() || lang == "*" {
                return None;
            }

            let quality = iter
                .next()
                .and_then(|q| q.trim().strip_prefix("q="))
                .and_then(|q| q.parse().ok())
                .unwrap_or(1.0);

            Some((lang, quality))
        })
        .collect();

    langs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    langs
}

/// Negotiates the best catalogue locale for an `Accept-Language` list.
///
/// Exact language+region wins, then language-only, then the default locale.
/// Only locales the catalogue actually ships are ever returned.
fn negotiate_locale(requested: &[(String, f32)]) -> Option<Locale> {
    let available: Vec<Locale> = available_locales()
        .iter()
        .filter_map(|code| Locale::new(code))
        .collect();

    let requested_locales: Vec<Locale> = requested
        .iter()
        .filter_map(|(lang, _)| Locale::new(lang))
        .collect();

    if available.is_empty() {
        return Some(Locale::default());
    }

    let requested_refs: Vec<&Locale> = requested_locales.iter().collect();
    let fallback = Locale::default();

    Some(
        Locale::negotiate(&requested_refs, &available, &fallback).clone(),
    )
}

pub type MessageArgs = botlib::i18n::MessageArgs;

pub fn init_i18n(locales_path: &str) -> Result<(), String> {
    i18n::init(locales_path).map_err(|e| format!("Failed to initialize i18n: {e}"))
}

pub fn is_i18n_initialized() -> bool {
    i18n::is_initialized()
}

pub fn t(locale: &Locale, key: &str) -> String {
    t_with_args(locale, key, None)
}

pub fn t_with_args(locale: &Locale, key: &str, args: Option<&MessageArgs>) -> String {
    let botlib_args: Option<BotlibMessageArgs> = args.map(|a| {
        a.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    });
    i18n::get_with_args(locale, key, botlib_args.as_ref())
}

pub fn available_locales() -> Vec<String> {
    if is_i18n_initialized() {
        let loaded = i18n::available_locales();
        if !loaded.is_empty() {
            return loaded;
        }
    }

    BUNDLED_LOCALES.iter().map(|s| (*s).to_string()).collect()
}
