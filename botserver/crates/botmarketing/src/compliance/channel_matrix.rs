//! Explicit delivery channel matrix for campaign sends (#2536).
//!
//! The worker has a real provider only for email and WhatsApp. Instagram is a
//! broadcast to the brand account performed by the publish path, and SMS has
//! no provider at all. Treating either as email used to create a fake success
//! (an email addressed to a phone number). This module makes the matrix
//! explicit and gives callers a typed error instead.

use serde_json::{json, Map, Value};
use uuid::Uuid;

use super::{evaluate_send_gate, MarketingChannel};

/// Whether the durable per-recipient worker has a real provider for `channel`.
///
/// Keep this in sync with `campaign::send::attempt_send`: every `true` channel
/// must have a provider branch there; every `false` channel must be handled by
/// the publish path or reported as unsupported.
pub fn supports_recipient_send(channel: MarketingChannel) -> bool {
    matches!(channel, MarketingChannel::Email | MarketingChannel::Whatsapp)
}

/// Typed reason a publish channel could not deliver (#2536).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelError {
    /// The channel has no provider integration (SMS today).
    UnsupportedChannel { channel: String },
    /// A broadcast channel had no audience that passed the compliance gate.
    NoConsentedRecipients { channel: String },
}

impl ChannelError {
    /// Stable machine-readable code returned in the API response.
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedChannel { .. } => "unsupported_channel",
            Self::NoConsentedRecipients { .. } => "no_consented_recipients",
        }
    }

    pub fn channel(&self) -> &str {
        match self {
            Self::UnsupportedChannel { channel } | Self::NoConsentedRecipients { channel } => {
                channel.as_str()
            }
        }
    }

    /// JSON form embedded in the publish response, always with an explicit
    /// `code` so clients never have to guess which variant they received.
    pub fn to_value(&self) -> Value {
        json!({ "code": self.code(), "channel": self.channel() })
    }
}

/// HTTP error body for a publish request whose channels all failed the
/// compliance gate or have no provider. Always machine-readable, never an
/// empty or implicit success.
pub fn all_channels_failed_body(channel_errors: &Map<String, Value>) -> String {
    let code = channel_errors
        .values()
        .next()
        .and_then(|value| value.get("code"))
        .and_then(|value| value.as_str())
        .unwrap_or("channel_error");
    serde_json::to_string(&json!({
        "success": false,
        "error": code,
        "channel_errors": channel_errors,
    }))
    .unwrap_or_else(|_| "{\"success\":false,\"error\":\"channel_error\"}".to_string())
}

/// Stable address used for a contact's suppression/consent key. Email wins;
/// phone is the fallback for WhatsApp/SMS-only contacts.
pub fn contact_gate_address<'a>(email: &'a str, phone: Option<&'a str>) -> &'a str {
    let email = email.trim();
    if !email.is_empty() {
        return email;
    }
    phone.unwrap_or("").trim()
}

/// True when at least one contact passes the suppression and consent gate for
/// `channel`. Broadcast channels (Instagram) have no per-recipient provider
/// call, so this audience gate is their only compliance check.
pub fn audience_has_consent<'a, I>(branch_id: Uuid, channel: MarketingChannel, contacts: I) -> bool
where
    I: IntoIterator<Item = (&'a str, Option<&'a str>, Option<bool>, Option<&'a Value>)>,
{
    contacts.into_iter().any(|(email, phone, subscribed, metadata)| {
        let address = contact_gate_address(email, phone);
        !address.is_empty()
            && evaluate_send_gate(
                branch_id,
                channel,
                address,
                None,
                subscribed,
                metadata,
            )
            .is_allowed()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::suppression::{SuppressionKey, SuppressionStore};
    use serde_json::json;
    use uuid::Uuid;

    fn address(prefix: &str) -> String {
        format!("{prefix}-{}@example.test", Uuid::new_v4())
    }

    #[test]
    fn recipient_send_matrix_is_explicit() {
        assert!(supports_recipient_send(MarketingChannel::Email));
        assert!(supports_recipient_send(MarketingChannel::Whatsapp));
        assert!(!supports_recipient_send(MarketingChannel::Sms));
        assert!(!supports_recipient_send(MarketingChannel::Instagram));
    }

    #[test]
    fn channel_errors_are_typed() {
        let unsupported = ChannelError::UnsupportedChannel {
            channel: "sms".to_string(),
        };
        assert_eq!(unsupported.code(), "unsupported_channel");
        assert_eq!(unsupported.channel(), "sms");
        assert_eq!(
            unsupported.to_value(),
            json!({ "code": "unsupported_channel", "channel": "sms" })
        );

        let no_audience = ChannelError::NoConsentedRecipients {
            channel: "instagram".to_string(),
        };
        assert_eq!(no_audience.code(), "no_consented_recipients");
        assert_ne!(unsupported, no_audience);
    }

    #[test]
    fn all_channels_failed_body_is_machine_readable() {
        let mut errors = Map::new();
        errors.insert(
            "sms".to_string(),
            ChannelError::UnsupportedChannel {
                channel: "sms".to_string(),
            }
            .to_value(),
        );
        let body = all_channels_failed_body(&errors);
        let parsed: Value = serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
        assert_eq!(parsed["success"], false);
        assert_eq!(parsed["error"], "unsupported_channel");
        assert_eq!(parsed["channel_errors"]["sms"]["channel"], "sms");
    }

    #[test]
    fn contact_gate_address_prefers_email_then_phone() {
        assert_eq!(contact_gate_address(" User@Example.Test ", Some("+55119999")), "User@Example.Test");
        assert_eq!(contact_gate_address("  ", Some(" +55119999 ")), "+55119999");
        assert_eq!(contact_gate_address("", None), "");
    }

    #[test]
    fn instagram_audience_gate_requires_consent_and_blocks_suppressed() {
        let branch_id = Uuid::new_v4();
        let email = address("ig-audience");
        let contacts = [(
            email.as_str(),
            None::<&str>,
            None::<bool>,
            None::<&Value>,
        )];

        assert!(!audience_has_consent(
            branch_id,
            MarketingChannel::Instagram,
            contacts.iter().copied()
        ));

        let consented = [(email.as_str(), None::<&str>, Some(true), None::<&Value>)];
        assert!(audience_has_consent(
            branch_id,
            MarketingChannel::Instagram,
            consented.iter().copied()
        ));

        let key = SuppressionKey::new(branch_id, MarketingChannel::Instagram, &email);
        let added = SuppressionStore::global().add(key);
        assert!(added.is_ok());
        assert!(!audience_has_consent(
            branch_id,
            MarketingChannel::Instagram,
            consented.iter().copied()
        ));
    }
}
