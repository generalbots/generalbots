//! #2541 — bounded, TTL-aware, truthful cache for generated sandbox previews.
//!
//! A preview stored here never answers with a silent miss: a lookup returns the
//! bytes together with a [`PreviewStatus`] that says whether they are still
//! usable, expired, or orphaned by a server restart. This is the in-process
//! half of the sandbox resource contract; the durable copy lives in Drive.
//!
//! The module is deliberately free of I/O and wall-clock reads so every policy
//! decision is a pure function of `(entry, now_ms, boot_id)` and can be tested
//! inline without a server or a database.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Default ceiling for cached previews. Keeps the process-global cache bounded.
pub const DEFAULT_CAPACITY: usize = 128;

/// Default lifetime of a preview (one hour, the signer's maximum TTL).
pub const DEFAULT_TTL_MS: u64 = 60 * 60 * 1000;

/// Why a cached preview has the state it has. `Eq` + `Copy` on purpose: these
/// values travel through comparisons and are cheap to hand around.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewStatus {
    /// Published by the current process and inside its TTL.
    Fresh,
    /// Published by the current process but older than its TTL.
    Expired,
    /// Published by a previous process: the publisher no longer exists.
    InterruptedByRestart,
}

impl PreviewStatus {
    /// Stable machine-readable reason, suitable for an API/UI status field.
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Expired => "expired",
            Self::InterruptedByRestart => "interrupted_by_restart",
        }
    }

    /// Whether a preview with this status may be rendered as live.
    #[must_use]
    pub const fn is_live(self) -> bool {
        matches!(self, Self::Fresh)
    }
}

/// A cached preview plus the metadata required to judge its truthfulness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewEntry {
    pub html: String,
    /// Epoch milliseconds at insertion time (supplied by the caller).
    pub created_at_ms: u64,
    /// Lifetime in milliseconds; `0` means already expired at insertion.
    pub ttl_ms: u64,
    /// Process generation that published the entry.
    pub boot_id: u64,
}

/// Pure policy: classify an entry at `now_ms` for process `boot_id`.
///
/// A foreign boot wins over TTL: a preview whose publisher is gone is always
/// reported as interrupted, even if it would still be fresh by age.
#[must_use]
pub const fn classify(entry: &PreviewEntry, now_ms: u64, boot_id: u64) -> PreviewStatus {
    if entry.boot_id != boot_id {
        return PreviewStatus::InterruptedByRestart;
    }
    if now_ms.saturating_sub(entry.created_at_ms) >= entry.ttl_ms {
        return PreviewStatus::Expired;
    }
    PreviewStatus::Fresh
}

struct Inner {
    entries: HashMap<String, PreviewEntry>,
    order: HashMap<String, u64>,
    sequence: u64,
    capacity: usize,
    boot_id: u64,
}

/// In-process cache with an explicit capacity bound and a boot generation.
pub struct PreviewStore {
    inner: Mutex<Inner>,
}

impl std::fmt::Debug for PreviewStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreviewStore").finish_non_exhaustive()
    }
}

impl Default for PreviewStore {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY, 0)
    }
}

impl PreviewStore {
    /// Creates a store that holds at most `capacity` previews (minimum 1).
    #[must_use]
    pub fn new(capacity: usize, boot_id: u64) -> Self {
        Self {
            inner: Mutex::new(Inner {
                entries: HashMap::new(),
                order: HashMap::new(),
                sequence: 0,
                capacity: capacity.max(1),
                boot_id,
            }),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        match self.inner.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// The process generation this store currently belongs to.
    #[must_use]
    pub fn boot_id(&self) -> u64 {
        self.lock().boot_id
    }

    /// Inserts or replaces a preview, evicting the oldest entry on overflow.
    pub fn insert(&self, hash: &str, html: &str, ttl_ms: u64, now_ms: u64) {
        let mut inner = self.lock();
        inner.sequence = inner.sequence.wrapping_add(1);
        let seq = inner.sequence;
        let boot_id = inner.boot_id;
        let entry = PreviewEntry {
            html: html.to_string(),
            created_at_ms: now_ms,
            ttl_ms,
            boot_id,
        };
        let was_present = inner.entries.insert(hash.to_string(), entry).is_some();
        inner.order.insert(hash.to_string(), seq);
        if was_present {
            return;
        }
        while inner.entries.len() > inner.capacity {
            let oldest = inner
                .order
                .iter()
                .min_by_key(|(_, seq)| **seq)
                .map(|(key, _)| key.clone());
            match oldest {
                Some(key) => {
                    inner.order.remove(&key);
                    inner.entries.remove(&key);
                }
                None => break,
            }
        }
    }

    /// Returns the preview bytes and their honest status, never a bare miss
    /// for an entry that is still cached (no silent 404 for stale content).
    #[must_use]
    pub fn get(&self, hash: &str, now_ms: u64) -> Option<(String, PreviewStatus)> {
        let inner = self.lock();
        let entry = inner.entries.get(hash)?;
        let status = classify(entry, now_ms, inner.boot_id);
        Some((entry.html.clone(), status))
    }

    /// Returns only the status for a cached hash.
    #[must_use]
    pub fn status(&self, hash: &str, now_ms: u64) -> Option<PreviewStatus> {
        let inner = self.lock();
        let entry = inner.entries.get(hash)?;
        Some(classify(entry, now_ms, inner.boot_id))
    }

    /// Drops entries whose TTL elapsed. Foreign-boot entries are kept until a
    /// reader sees their honest status or they are forgotten/replaced.
    pub fn prune_expired(&self, now_ms: u64) {
        let mut inner = self.lock();
        let boot_id = inner.boot_id;
        let expired: Vec<String> = inner
            .entries
            .iter()
            .filter(|(_, entry)| {
                entry.boot_id == boot_id
                    && now_ms.saturating_sub(entry.created_at_ms) >= entry.ttl_ms
            })
            .map(|(key, _)| key.clone())
            .collect();
        for key in expired {
            inner.order.remove(&key);
            inner.entries.remove(&key);
        }
    }

    /// Reconciles the cache with a new process generation. Entries published by
    /// an older boot now classify as [`PreviewStatus::InterruptedByRestart`].
    /// Expired entries of the outgoing boot are pruned to bound memory.
    pub fn reconcile_on_boot(&self, new_boot_id: u64, now_ms: u64) {
        {
            let mut inner = self.lock();
            inner.boot_id = new_boot_id;
        }
        self.prune_expired(now_ms);
    }

    /// Forgets one hash (used when a preview is revoked).
    pub fn forget(&self, hash: &str) {
        let mut inner = self.lock();
        inner.order.remove(hash);
        inner.entries.remove(hash);
    }

    /// Number of cached previews, including stale ones awaiting reconciliation.
    #[must_use]
    pub fn len(&self) -> usize {
        self.lock().entries.len()
    }

    /// Whether the cache holds no previews.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lock().entries.is_empty()
    }
}

static SHARED: OnceLock<PreviewStore> = OnceLock::new();

/// Process-wide cache. `OnceLock` is initialised with the current boot id;
/// call [`PreviewStore::reconcile_on_boot`] again after a logical restart.
#[must_use]
pub fn shared() -> &'static PreviewStore {
    SHARED.get_or_init(|| PreviewStore::new(DEFAULT_CAPACITY, 0))
}

#[cfg(test)]
mod tests {
    use super::{classify, shared, PreviewEntry, PreviewStatus, PreviewStore, DEFAULT_TTL_MS};

    #[test]
    fn fresh_preview_is_fresh() {
        let entry = PreviewEntry {
            html: "<h1>ok</h1>".to_string(),
            created_at_ms: 1_000,
            ttl_ms: DEFAULT_TTL_MS,
            boot_id: 7,
        };
        assert_eq!(classify(&entry, 1_000, 7), PreviewStatus::Fresh);
        assert_eq!(
            classify(&entry, 1_000 + DEFAULT_TTL_MS - 1, 7),
            PreviewStatus::Fresh
        );
        assert!(classify(&entry, 1_000, 7).is_live());
    }

    #[test]
    fn expired_preview_is_expired() {
        let entry = PreviewEntry {
            html: "<h1>old</h1>".to_string(),
            created_at_ms: 1_000,
            ttl_ms: 500,
            boot_id: 1,
        };
        assert_eq!(classify(&entry, 1_500, 1), PreviewStatus::Expired);
        assert_eq!(classify(&entry, 9_999, 1), PreviewStatus::Expired);
        assert!(!classify(&entry, 9_999, 1).is_live());
        assert_eq!(PreviewStatus::Expired.reason_code(), "expired");
    }

    #[test]
    fn orphaned_preview_reports_restart_even_when_fresh() {
        let entry = PreviewEntry {
            html: "<h1>orphan</h1>".to_string(),
            created_at_ms: 5_000,
            ttl_ms: DEFAULT_TTL_MS,
            boot_id: 1,
        };
        assert_eq!(
            classify(&entry, 5_001, 2),
            PreviewStatus::InterruptedByRestart
        );
        assert_eq!(
            PreviewStatus::InterruptedByRestart.reason_code(),
            "interrupted_by_restart"
        );
    }

    #[test]
    fn lookup_returns_bytes_with_status_instead_of_a_silent_miss() {
        let store = PreviewStore::new(4, 3);
        store.insert("abc", "<p>hi</p>", 1_000, 100);
        assert_eq!(
            store.get("abc", 200),
            Some(("<p>hi</p>".to_string(), PreviewStatus::Fresh))
        );
        assert!(store.get("missing", 200).is_none());
    }

    #[test]
    fn capacity_is_bounded_and_evicts_oldest() {
        let store = PreviewStore::new(2, 0);
        store.insert("a", "a", 10_000, 0);
        store.insert("b", "b", 10_000, 1);
        store.insert("c", "c", 10_000, 2);
        assert_eq!(store.len(), 2);
        assert!(store.get("a", 3).is_none());
        assert!(store.get("b", 3).is_some());
        assert!(store.get("c", 3).is_some());
    }

    #[test]
    fn insert_replaces_without_growing() {
        let store = PreviewStore::new(1, 0);
        store.insert("a", "first", 10_000, 0);
        store.insert("a", "second", 10_000, 1);
        assert_eq!(store.len(), 1);
        assert_eq!(
            store.get("a", 2).map(|(html, _)| html),
            Some("second".to_string())
        );
    }

    #[test]
    fn boot_reconciliation_reclassifies_cached_previews() {
        let store = PreviewStore::new(4, 1);
        store.insert("p", "<p>p</p>", DEFAULT_TTL_MS, 100);
        assert_eq!(store.status("p", 200), Some(PreviewStatus::Fresh));
        store.reconcile_on_boot(2, 200);
        assert_eq!(store.boot_id(), 2);
        assert_eq!(
            store.status("p", 200),
            Some(PreviewStatus::InterruptedByRestart)
        );
    }

    #[test]
    fn prune_removes_only_expired_current_boot_entries() {
        let store = PreviewStore::new(4, 1);
        store.insert("fresh", "f", 1_000, 900);
        store.insert("old", "o", 100, 100);
        store.prune_expired(1_000);
        assert_eq!(store.len(), 1);
        assert!(store.get("fresh", 1_000).is_some());
        assert!(store.get("old", 1_000).is_none());
    }

    #[test]
    fn forget_removes_one_hash() {
        let store = PreviewStore::new(4, 1);
        store.insert("a", "a", 1_000, 0);
        store.insert("b", "b", 1_000, 0);
        store.forget("a");
        assert!(!store.is_empty());
        assert!(store.get("a", 1).is_none());
        assert!(store.get("b", 1).is_some());
    }

    #[test]
    fn shared_singleton_is_stable() {
        let first = shared() as *const PreviewStore;
        let second = shared() as *const PreviewStore;
        assert!(std::ptr::eq(first, second));
    }
}
