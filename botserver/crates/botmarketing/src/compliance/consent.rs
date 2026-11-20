//! Self-contained marketing consent predicate (#2536).
//!
//! The `botconsent` crate is an *agent action* permission/audit system
//! (`app_permissions`, action classes such as `publish`) and is not a
//! dependency of `botmarketing`; it carries no per-channel marketing opt-in
//! record. This module is therefore the marketing-specific consent gate.
//! It reads the documented typed flag from `marketing_contacts.metadata` and
//! falls back to the legacy `subscribed` flag.
//!
//! A missing flag is `Unknown`, never `OptedIn`: no consent record means no
//! send. Callers must log a graceful skip event instead of silently dropping
//! the recipient.

use serde_json::Value;

use super::suppression::MarketingChannel;

/// Effective state of a recipient's consent for one channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentState {
    OptedIn,
    OptedOut,
    Unknown,
}

impl ConsentState {
    /// Short, caller-safe reason used in skip events.
    pub fn reason(self) -> &'static str {
        match self {
            Self::OptedIn => "consent present",
            Self::OptedOut => "recipient opted out",
            Self::Unknown => "no consent record for channel",
        }
    }

    pub fn is_opted_in(self) -> bool {
        matches!(self, Self::OptedIn)
    }
}

fn parse_bool(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(flag) => Some(*flag),
        Value::String(raw) => match raw.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "y" | "opt_in" | "opted_in" => Some(true),
            "false" | "0" | "no" | "n" | "opt_out" | "opted_out" => Some(false),
            _ => None,
        },
        Value::Number(number) => match number.as_i64() {
            Some(1) => Some(true),
            Some(0) => Some(false),
            _ => None,
        },
        _ => None,
    }
}

/// Reads the explicit flag for one channel. The spellings mirror
/// `segment::consent_flags`: flat (`consent_email`), dotted (`consent.email`),
/// suffixed (`email_consent`) and `*_opt_in` are all accepted.
fn explicit_flag(metadata: Option<&Value>, channel: MarketingChannel) -> Option<bool> {
    let root = metadata?;
    let name = channel.as_str();
    let keys = [
        format!("consent_{name}"),
        format!("{name}_consent"),
        format!("{name}_opt_in"),
    ];
    let pointer = format!("/consent/{name}");
    keys.iter()
        .find_map(|key| root.get(key))
        .or_else(|| root.pointer(pointer.as_str()))
        .and_then(parse_bool)
}

/// Decides whether a recipient may be contacted on `channel`.
///
/// An explicit per-channel flag wins. When absent, `subscribed = true` is the
/// legacy consent record; `subscribed = false` and a missing flag both block
/// the send.
pub fn channel_consent(
    subscribed: Option<bool>,
    metadata: Option<&Value>,
    channel: MarketingChannel,
) -> ConsentState {
    match explicit_flag(metadata, channel) {
        Some(true) => ConsentState::OptedIn,
        Some(false) => ConsentState::OptedOut,
        None => match subscribed {
            Some(true) => ConsentState::OptedIn,
            Some(false) => ConsentState::OptedOut,
            None => ConsentState::Unknown,
        },
    }
}
