//! Inbound Telegram authenticity (#2142).
//!
//! `/webhook/telegram` and `/webhook/telegram/:bot_name` are anonymous routes:
//! Telegram posts from its own infrastructure and carries no user session, so
//! the only proof a delivery is genuine is the
//! `X-Telegram-Bot-Api-Secret-Token` header registered through Bot API
//! `setWebhook`.
//!
//! The gate used to accept every delivery when no secret was configured — the
//! default for a fresh bot — which let any caller create sessions and drive the
//! model under a victim identity. It now fails closed: an unconfigured secret
//! refuses the delivery and logs the operator action once per bot.

use botsecurity_core::constant_time;
use std::collections::HashSet;
use std::sync::{LazyLock, Mutex};
use uuid::Uuid;

/// Header Telegram sends on every delivery once the webhook is registered with
/// a `secret_token` (Bot API `setWebhook`).
pub const SECRET_HEADER: &str = "x-telegram-bot-api-secret-token";

/// Bot config key holding the expected secret token. It is mandatory: without
/// it no delivery can be proven authentic, so every POST is refused.
pub const SECRET_CONFIG_KEY: &str = "telegram-webhook-secret";

/// Outcome of the inbound authenticity gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryAuth {
    /// The delivered header matches the configured secret.
    Verified,
    /// The header is absent or does not match the configured secret.
    Mismatch,
    /// No secret is configured, so no delivery can be proven authentic.
    Unconfigured,
}

/// Decides whether a delivery is authentic.
///
/// The comparison is byte-wise constant time (`constant_time::eq_str`): `==`
/// on `&str` returns at the first differing byte, which lets a caller recover a
/// valid secret byte by byte from response timing. Surrounding horizontal
/// whitespace is ignored because the Bot API forwards the value verbatim and an
/// operator may paste it with surrounding spaces; any other character has to
/// match exactly.
pub fn authorize_delivery(expected_secret: &str, provided: Option<&str>) -> DeliveryAuth {
    let expected = expected_secret.trim();
    if expected.is_empty() {
        return DeliveryAuth::Unconfigured;
    }

    // Only horizontal whitespace is padding: a header may be pasted with
    // spaces or a tab, but a newline or any other control character means the
    // value is not the secret, so it must not be trimmed away (#2142).
    let provided = provided.unwrap_or_default().trim_matches([' ', '\t']);
    if provided.is_empty() {
        return DeliveryAuth::Mismatch;
    }

    if constant_time::eq_str(expected, provided) {
        DeliveryAuth::Verified
    } else {
        DeliveryAuth::Mismatch
    }
}

/// Bots already warned about the missing secret. A configuration error is a
/// deployment mistake, not a per-request event: logging it on every forged POST
/// would let a stranger flood the log with one line per delivery.
static WARNED_BOTS: LazyLock<Mutex<HashSet<Uuid>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

/// Emits the configuration error for `bot_id` once per process and reports
/// whether this call was the one that logged it.
pub fn warn_unconfigured_once(bot_id: Uuid) -> bool {
    let mut warned = match WARNED_BOTS.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    if !warned.insert(bot_id) {
        return false;
    }

    log::error!(
        "Telegram webhook for bot {bot_id} has no {SECRET_CONFIG_KEY} configured: \
         every delivery is refused until the secret is set (Bot API setWebhook secret_token)"
    );
    true
}

#[cfg(test)]
#[path = "webhook_gate_tests.rs"]
mod webhook_gate_tests;
