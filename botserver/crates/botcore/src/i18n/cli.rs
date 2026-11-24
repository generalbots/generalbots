//! Locale of the process CLI (#2094).
//!
//! Human-facing CLI text (help, shutdown banner, `security` reports, the
//! wizard) is translated; log lines are not. The language comes from
//! `--lang <code>` / `--lang=<code>`, then `LC_ALL`, `LC_MESSAGES` and `LANG`,
//! and falls back to `en`. Machine-readable output (JSON, the `GBWIZARD:`
//! progress frames) never goes through here.

use std::sync::OnceLock;

use super::{self as i18n, Locale};

static CLI_LOCALE: OnceLock<Locale> = OnceLock::new();

/// Resolves the CLI locale from the arguments and the environment.
#[must_use]
pub fn resolve_locale(args: &[String]) -> Locale {
    match explicit_lang(args).or_else(environment_lang) {
        Some(locale) => locale,
        // No --lang flag and no locale environment variable: the documented
        // English fallback, not an error (a fresh shell has none of these).
        None => Locale::default(),
    }
}

/// Records the CLI locale once, at the start of `main`.
pub fn init_from_args(args: &[String]) -> Locale {
    let locale = resolve_locale(args);
    if let Err(previous) = CLI_LOCALE.set(locale.clone()) {
        log::warn!("CLI locale already initialized to {previous}; ignoring later init");
    }
    locale
}

/// The locale recorded by [`init_from_args`], or the default.
#[must_use]
pub fn locale() -> Locale {
    match CLI_LOCALE.get() {
        Some(locale) => locale.clone(),
        // init_from_args has not run yet (e.g. a library caller); use the
        // documented fallback instead of assuming CLI startup completed.
        None => Locale::default(),
    }
}

/// Translates one CLI message.
#[must_use]
pub fn t(key: &str) -> String {
    i18n::ensure_initialized();
    i18n::t(&locale(), key)
}

/// Translates one CLI message with `(name, value)` arguments.
#[must_use]
pub fn t_args(key: &str, args: &[(&str, &str)]) -> String {
    i18n::ensure_initialized();
    i18n::t_args(&locale(), key, args)
}

fn explicit_lang(args: &[String]) -> Option<Locale> {
    for (index, arg) in args.iter().enumerate() {
        let value = match arg.strip_prefix("--lang=") {
            Some(inline) => Some(inline),
            None if arg == "--lang" => args.get(index + 1).map(String::as_str),
            None => None,
        };

        if let Some(locale) = value.and_then(parse_locale) {
            return Some(locale);
        }
    }

    None
}

fn environment_lang() -> Option<Locale> {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .find_map(|value| parse_locale(&value))
}

/// Parses `pt_BR.UTF-8`, `pt-BR@euro`, `es` … into a catalogue locale. `C`
/// and `POSIX` mean "no preference", not a language.
fn parse_locale(raw: &str) -> Option<Locale> {
    // split() always yields at least one item, but handle the empty case
    // explicitly instead of leaning on a default.
    let code = match raw.trim().split(['.', '@']).next() {
        Some(code) => code.trim(),
        None => "",
    };

    if code.is_empty() || code.eq_ignore_ascii_case("c") || code.eq_ignore_ascii_case("posix") {
        return None;
    }

    Locale::new(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn explicit_lang_wins_over_the_environment() {
        assert_eq!(
            resolve_locale(&args(&["botserver", "--lang", "pt-BR"])).to_string(),
            "pt-BR"
        );
    }

    #[test]
    fn inline_lang_form_is_accepted() {
        assert_eq!(
            resolve_locale(&args(&["botserver", "--lang=es"])).to_string(),
            "es"
        );
    }

    #[test]
    fn posix_locales_have_no_preference() {
        assert!(parse_locale("C").is_none());
        assert!(parse_locale("POSIX").is_none());
    }

    #[test]
    fn posix_codes_are_stripped_from_the_environment_value() {
        assert_eq!(
            parse_locale("pt_BR.UTF-8").map(|locale| locale.to_string()),
            Some("pt-BR".to_string())
        );
    }
}
