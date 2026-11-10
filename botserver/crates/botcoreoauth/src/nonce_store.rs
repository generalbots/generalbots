//! Single-use registry for OAuth `state` nonces (#2130).
//!
//! A signed state proves we minted it, but not that it has only been used
//! once: whoever observes the callback URL (browser history, proxy log,
//! referrer) could replay it. Every issued nonce is therefore recorded here and
//! removed on first use, so a replayed callback is rejected.
//!
//! The entry also remembers the user the flow was started *for* when it is an
//! account link, so the callback can re-check that the signed `user_id` matches
//! the authenticated user who started the flow instead of trusting the state
//! payload alone.
//!
//! The registry is in-process because botserver runs one process per
//! deployment; a horizontally scaled deployment would need to move this map to
//! Valkey. Entries are pruned on every call and expire with the state itself,
//! so a flood of abandoned flows cannot grow the map without bound.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::state::STATE_TTL_SECS;

/// A nonce that was issued and not yet consumed.
#[derive(Debug, Clone)]
pub struct PendingNonce {
    user_id: Option<String>,
}

impl PendingNonce {
    /// The user id recorded when the flow started (account linking only).
    pub fn user_id(&self) -> Option<&str> {
        self.user_id.as_deref()
    }
}

#[derive(Debug, Clone)]
struct Entry {
    expires_at: i64,
    user_id: Option<String>,
}

fn registry() -> &'static Mutex<HashMap<String, Entry>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, Entry>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Records a freshly issued nonce with its expiry.
pub fn issue(nonce: &str, now: i64) {
    issue_for_user(nonce, None, now);
}

/// Records a freshly issued nonce together with the authenticated user the
/// flow belongs to (account linking).
pub fn issue_for_user(nonce: &str, user_id: Option<&str>, now: i64) {
    if nonce.is_empty() {
        return;
    }
    let Ok(mut map) = registry().lock() else {
        log::error!("OAuth nonce registry is poisoned — refusing to record the nonce");
        return;
    };
    map.retain(|_, entry| entry.expires_at >= now);
    map.insert(
        nonce.to_string(),
        Entry {
            expires_at: now + STATE_TTL_SECS,
            user_id: user_id.map(|id| id.to_string()),
        },
    );
}

/// Marks a nonce as used. Returns the pending entry only for the first call; a
/// replayed nonce (or one we never issued) returns `None`.
pub fn consume(nonce: &str, now: i64) -> Option<PendingNonce> {
    if nonce.is_empty() {
        return None;
    }
    let Ok(mut map) = registry().lock() else {
        log::error!("OAuth nonce registry is poisoned — refusing the flow");
        return None;
    };
    map.retain(|_, entry| entry.expires_at >= now);
    let entry = map.remove(nonce)?;
    Some(PendingNonce {
        user_id: entry.user_id,
    })
}

/// Whether a nonce is still pending (unused and unexpired).
pub fn is_pending(nonce: &str, now: i64) -> bool {
    if nonce.is_empty() {
        return false;
    }
    let Ok(map) = registry().lock() else {
        return false;
    };
    map.get(nonce)
        .is_some_and(|entry| entry.expires_at >= now)
}
