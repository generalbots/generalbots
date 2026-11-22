//! Suppression / do-not-contact store for campaign delivery (#2536).
//!
//! Every send path must call [`SuppressionStore::is_suppressed`] before it
//! hands a message to a channel provider. The store is shared process-wide via
//! [`SuppressionStore::global`] so a recipient who opts out through the public
//! unsubscribe endpoint is blocked even from a send started by another task.
//!
//! Persistence (#2536): the authoritative record is the branch-scoped
//! `marketing_suppressions` table (unique on `(branch_id, channel, email)`).
//! [`persist_suppression`] writes to it and [`hydrate_suppressions`] reloads it
//! into this process-wide fast gate, so an opt-out or a hard bounce survives a
//! restart. The durable opt-out signals in `marketing_contacts`
//! (`subscribed = false`) and `marketing_recipients` (`status =
//! 'unsubscribed'`) are kept as a second line of defence.

use chrono::Utc;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{OnceLock, RwLock};
use uuid::Uuid;

use crate::schema::marketing_suppressions;

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

/// Converts a durable row into a typed, normalized suppression key. Rows with
/// an unknown channel are ignored rather than becoming a wildcard suppression.
pub fn suppression_key_from_record(
    branch_id: Uuid,
    channel: &str,
    address: &str,
) -> Option<SuppressionKey> {
    MarketingChannel::parse(channel)
        .map(|channel| SuppressionKey::new(branch_id, channel, address))
}

/// Writes a suppression to the durable table. The unique index makes repeated
/// opt-outs idempotent, which is what the one-click endpoint relies on.
pub fn persist_suppression(
    conn: &mut PgConnection,
    key: &SuppressionKey,
    reason: &str,
) -> Result<SuppressionOutcome, String> {
    diesel::insert_into(marketing_suppressions::table)
        .values((
            marketing_suppressions::id.eq(Uuid::new_v4()),
            marketing_suppressions::branch_id.eq(key.branch_id),
            marketing_suppressions::email.eq(key.address.as_str()),
            marketing_suppressions::channel.eq(key.channel.as_str()),
            marketing_suppressions::reason.eq(reason),
            marketing_suppressions::created_at.eq(Utc::now()),
        ))
        .on_conflict_do_nothing()
        .execute(conn)
        .map(|rows| SuppressionOutcome {
            newly_added: rows > 0,
        })
        .map_err(|error| format!("Suppression insert failed: {error}"))
}

/// Reloads durable suppressions into the in-process gate. Called at the start
/// of each send pass so a restart cannot resurrect an opted-out recipient.
pub fn hydrate_suppressions(conn: &mut PgConnection) -> Result<usize, String> {
    let rows = marketing_suppressions::table
        .select((
            marketing_suppressions::branch_id,
            marketing_suppressions::channel,
            marketing_suppressions::email,
        ))
        .load::<(Uuid, String, String)>(conn)
        .map_err(|error| format!("Suppression hydrate failed: {error}"))?;

    let store = SuppressionStore::global();
    let mut loaded = 0usize;
    for (branch_id, channel, address) in rows {
        let Some(key) = suppression_key_from_record(branch_id, &channel, &address) else {
            log::warn!("ignoring suppression row with unknown channel {channel}");
            continue;
        };
        match store.add(key) {
            Ok(outcome) => {
                if outcome.newly_added {
                    loaded += 1;
                }
            }
            Err(error) => {
                log::warn!("suppression hydrate could not cache entry: {}", error.reason())
            }
        }
    }
    Ok(loaded)
}
