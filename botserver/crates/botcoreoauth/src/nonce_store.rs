//! Single-use registry for OAuth `state` nonces (#2130).
//!
//! A signed state proves we minted it, but not that it has only been used
//! once: whoever observes the callback URL (browser history, proxy log,
//! referrer) could replay it. Every issued nonce is therefore recorded here and
//! removed on first use, so a replayed callback is rejected.
//!
//! The registry is in-process because botserver runs one process per
//! deployment; a horizontally scaled deployment would need to move this map to
//! Valkey. Entries are pruned on every call and expire with the state itself,
//! so a flood of abandoned flows cannot grow the map without bound.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::state::STATE_TTL_SECS;

fn registry() -> &'static Mutex<HashMap<String, i64>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, i64>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Records a freshly issued nonce with its expiry.
pub fn issue(nonce: &str, now: i64) {
    if nonce.is_empty() {
        return;
    }
    let Ok(mut map) = registry().lock() else {
        log::error!("OAuth nonce registry is poisoned — refusing to record the nonce");
        return;
    };
    map.retain(|_, expires_at| *expires_at >= now);
    map.insert(nonce.to_string(), now + STATE_TTL_SECS);
}

/// Marks a nonce as used. Returns `true` only for the first call; a replayed
/// nonce (or one we never issued) returns `false`.
pub fn consume(nonce: &str, now: i64) -> bool {
    if nonce.is_empty() {
        return false;
    }
    let Ok(mut map) = registry().lock() else {
        log::error!("OAuth nonce registry is poisoned — refusing the flow");
        return false;
    };
    map.retain(|_, expires_at| *expires_at >= now);
    match map.remove(nonce) {
        Some(expires_at) => expires_at >= now,
        None => false,
    }
}

/// Whether a nonce is still pending (unused and unexpired).
pub fn is_pending(nonce: &str, now: i64) -> bool {
    if nonce.is_empty() {
        return false;
    }
    let Ok(map) = registry().lock() else {
        return false;
    };
    map.get(nonce).is_some_and(|expires_at| *expires_at >= now)
}

/// Number of pending nonces — used by tests and diagnostics only.
pub fn pending_count() -> usize {
    match registry().lock() {
        Ok(map) => map.len(),
        Err(_) => 0,
    }
}
