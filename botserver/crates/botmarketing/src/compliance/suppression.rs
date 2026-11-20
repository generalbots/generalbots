//! Suppression / do-not-contact store for campaign delivery (#2536).
//!
//! Every send path must call [`SuppressionStore::is_suppressed`] before it
//! hands a message to a channel provider. The store is shared process-wide via
//! [`SuppressionStore::global`] so a recipient who opts out through the public
//! unsubscribe endpoint is blocked even from a send started by another task.
//!
//! PERSISTENCE GAP: this is an in-memory store. A `marketing_suppressions`
//! table (branch-scoped, unique on `(branch_id, channel, address)`) still has
//! to be added via a migration and exposed in `botschema` before suppression
//! survives a process restart. To limit the gap, the unsubscribe and bounce
//! handlers also persist durable opt-out signals in `marketing_contacts`
//! (`subscribed = false`) and `marketing_recipients` (`status =
//! 'unsubscribed'`), which the send paths consult as well. A restart therefore
//! keeps blocking those recipients, but hard-bounce-only entries that have no
//! matching contact row are lost until the table exists.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{OnceLock, RwLock};
use uuid::Uuid;

/// Channel a marketing message travels on. Kept as a typed value so a channel
/// string typo can never silently bypass the suppression key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketingChannel {
    Email,
    Sms,
    Whatsapp,
    Instagram,
}

impl MarketingChannel {
    /// Stable storage/API spelling of the channel.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Sms => "sms",
            Self::Whatsapp => "whatsapp",
            Self::Instagram => "instagram",
        }
    }

    /// Parses the channel names used by campaign rows, publish requests and
    /// webhook payloads. Unknown channels return `None` instead of being
    /// treated as a wildcard.
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "email" | "e-mail" | "mail" => Some(Self::Email),
            "sms" | "text" => Some(Self::Sms),
            "whatsapp" | "wa" => Some(Self::Whatsapp),
            "instagram" | "ig" => Some(Self::Instagram),
            _ => None,
        }
    }
}

/// Normalizes an address for comparison: surrounding whitespace and case are
/// never allowed to create a second, unsuppressed spelling of the same mailbox
/// or phone number.
fn normalize_address(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

/// A single `(branch_id, channel, address)` suppression entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SuppressionKey {
    pub branch_id: Uuid,
    pub channel: MarketingChannel,
    pub address: String,
}

impl SuppressionKey {
    pub fn new(branch_id: Uuid, channel: MarketingChannel, address: &str) -> Self {
        Self {
            branch_id,
            channel,
            address: normalize_address(address),
        }
    }
}

/// Failure to acquire the in-memory store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuppressionError {
    StoreUnavailable,
}

impl SuppressionError {
    pub fn reason(self) -> &'static str {
        match self {
            Self::StoreUnavailable => "suppression store unavailable",
        }
    }
}

/// Result of recording an opt-out. `newly_added` is false on a repeat call,
/// which is what makes the unsubscribe endpoint idempotent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuppressionOutcome {
    pub newly_added: bool,
}

#[derive(Default)]
struct SuppressionInner {
    keys: HashSet<SuppressionKey>,
}

/// Branch-scoped, channel-scoped do-not-contact store.
#[derive(Default)]
pub struct SuppressionStore {
    inner: RwLock<SuppressionInner>,
}

impl SuppressionStore {
    /// Process-wide store. `OnceLock` initialization cannot fail.
    pub fn global() -> &'static Self {
        static STORE: OnceLock<SuppressionStore> = OnceLock::new();
        STORE.get_or_init(SuppressionStore::default)
    }

    /// Typed check used by callers that need to surface an error.
    pub fn try_is_suppressed(&self, key: &SuppressionKey) -> Result<bool, SuppressionError> {
        let guard = self
            .inner
            .read()
            .map_err(|_| SuppressionError::StoreUnavailable)?;
        Ok(guard.keys.contains(key))
    }

    /// Fails closed: an unavailable store must never turn into permission to
    /// send marketing mail.
    pub fn is_suppressed(&self, key: &SuppressionKey) -> bool {
        match self.try_is_suppressed(key) {
            Ok(suppressed) => suppressed,
            Err(error) => {
                log::error!("suppression check failed, skipping send: {}", error.reason());
                true
            }
        }
    }

    /// Records an opt-out. Inserting the same key twice is a no-op.
    pub fn add(&self, key: SuppressionKey) -> Result<SuppressionOutcome, SuppressionError> {
        let mut guard = self
            .inner
            .write()
            .map_err(|_| SuppressionError::StoreUnavailable)?;
        Ok(SuppressionOutcome {
            newly_added: guard.keys.insert(key),
        })
    }
}
