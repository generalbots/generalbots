//! Keyed suggestion labels (#2095).
//!
//! `ADD SUGGESTION` / `ADD SUGGESTION_TOOL` / `ADD SUGGESTION_TEXT` accept a
//! catalogue key instead of an authored label:
//!
//! ```basic
//! ADD SUGGESTION "check-balance" AS "bot-suggest-check-my-balance"
//! ```
//!
//! The runtime resolves the key through `botlib/locales/<locale>/*.ftl`, so one
//! template serves every locale. A string the catalogue does not define is
//! kept exactly as authored — custom bots keep their own wording, and a label
//! never degrades to a raw identifier.

use botlib::i18n::{self, Locale};

use super::UserSession;

/// Locale used for a suggestion label.
///
/// The session wins (a client that announced its language in `context_data`),
/// then the bot/org `locale` configuration, then the catalogue default (`en`).
#[must_use]
pub fn pick_locale(session_locale: Option<&str>, configured: Option<&str>) -> Locale {
    [session_locale, configured]
        .into_iter()
        .flatten()
        .map(str::trim)
        .find_map(Locale::new)
        .unwrap_or_default()
}

/// Locale of the running session.
#[must_use]
pub fn session_locale(session: &UserSession, configured: Option<String>) -> Locale {
    let announced = session
        .context_data
        .get("locale")
        .and_then(serde_json::Value::as_str);
    pick_locale(announced, configured.as_deref())
}

/// Resolves a keyed label for `locale`; unknown keys return `raw` unchanged.
#[must_use]
pub fn resolve_label(raw: &str, locale: &Locale) -> String {
    resolve_with(raw, |key| {
        if i18n::has_message(key) {
            Some(i18n::translate(locale, key, &[]))
        } else {
            None
        }
    })
}

/// Lookup-injected core of [`resolve_label`], testable without a catalogue.
fn resolve_with(raw: &str, lookup: impl Fn(&str) -> Option<String>) -> String {
    let key = raw.trim();
    if key.is_empty() {
        return raw.to_string();
    }
    lookup(key).unwrap_or_else(|| raw.to_string())
}

#[cfg(test)]
mod tests {
    use super::{pick_locale, resolve_with};

    fn catalogue(key: &str) -> Option<String> {
        match key {
            "bot-suggest-check-my-balance" => Some("Consultar saldo".to_string()),
            _ => None,
        }
    }

    #[test]
    fn keyed_label_is_translated() {
        assert_eq!(
            resolve_with("bot-suggest-check-my-balance", catalogue),
            "Consultar saldo"
        );
        assert_eq!(
            resolve_with("  bot-suggest-check-my-balance  ", catalogue),
            "Consultar saldo"
        );
    }

    #[test]
    fn authored_label_is_kept() {
        assert_eq!(
            resolve_with("Marcar consulta", catalogue),
            "Marcar consulta"
        );
        assert_eq!(
            resolve_with("Check my balance", catalogue),
            "Check my balance"
        );
    }

    #[test]
    fn blank_label_is_kept() {
        assert_eq!(resolve_with("   ", catalogue), "   ");
    }

    #[test]
    fn session_locale_wins_over_the_bot_configuration() {
        assert_eq!(pick_locale(Some("pt-BR"), Some("en")).to_string(), "pt-BR");
        assert_eq!(pick_locale(None, Some("es")).to_string(), "es");
        assert_eq!(pick_locale(None, None).to_string(), "en");
        // A malformed code falls back to the next candidate instead of
        // pinning a locale with no translations.
        assert_eq!(pick_locale(Some("x"), Some("ja")).to_string(), "ja");
    }
}
