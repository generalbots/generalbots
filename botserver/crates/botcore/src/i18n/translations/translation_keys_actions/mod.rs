//! Hand-maintained translation key catalogue for the action, chrome and
//! settings keys.
//!
//! #1730 — the catalogue outgrew the 450-line limit, so it is split by
//! responsibility into sibling modules. `translation_keys()` chains them
//! back in the original order.

mod ui_chrome_keys;
mod navigation_and_app_keys;
mod settings_keys;

/// Every key of this catalogue, in the order the locales were authored.
pub(super) fn translation_keys() -> impl Iterator<Item = &'static str> {
    ui_chrome_keys::KEYS
        .iter()
        .chain(navigation_and_app_keys::KEYS)
        .chain(settings_keys::KEYS)
        .copied()
}
