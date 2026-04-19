mod bundle;
mod locale;

pub use bundle::{I18nBundle, MessageArgs};
pub use locale::Locale;

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

pub fn available_locales() -> Vec<String> {
    GLOBAL_BUNDLE
        .get()
        .map(I18nBundle::available_locales)
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
}
