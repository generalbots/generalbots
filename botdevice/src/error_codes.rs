//! User-visible error keys for the mobile shell (#2095).
//!
//! The BotOS webview is the only surface this crate talks to, so every message
//! it authors is a rejected promise the UI shows to a person. Each one carries a
//! stable kebab-case key here and English text as the source of truth; a locale
//! catalogue can be added later without touching the call sites.
//!
//! Where these are rendered: `send_to_bot` returns `Err(message)`, Tauri rejects
//! the `invoke("send_to_bot")` promise with it, and the botui page displays the
//! message in its error toast. `get_device_info` only returns brand strings.

/// Network/transport failure while reaching the assistant.
pub const SEND_NETWORK: &str = "botdevice-send-network";
/// The assistant answered with a body the webview could not read.
pub const SEND_RESPONSE: &str = "botdevice-send-response";

/// English text for `key`, with `{name}` placeholders replaced by `args`.
///
/// An unknown key returns the key itself: a visible identifier beats an empty
/// error, and it names the missing entry in a bug report.
#[must_use]
pub fn text(key: &str, args: &[(&str, &str)]) -> String {
    let template = match key {
        SEND_NETWORK => "Could not reach the assistant: {error}",
        SEND_RESPONSE => "The assistant returned an unreadable response: {error}",
        other => return other.to_string(),
    };
    let mut message = template.to_string();
    for (name, value) in args {
        message = message.replace(&format!("{{{name}}}"), value);
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholders_are_replaced() {
        let message = text(SEND_NETWORK, &[("error", "connection refused")]);
        assert!(message.contains("connection refused"), "{message}");
        assert!(!message.contains("{error}"), "{message}");
    }

    #[test]
    fn unknown_key_returns_the_key() {
        assert_eq!(text("no-such-key", &[]), "no-such-key");
    }
}
