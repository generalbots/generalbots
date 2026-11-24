//! Bounded, TTL-aware process-local caches (issue #2541).
//!
//! Several process-global maps in this crate hold user-visible state. Without
//! a cap and a lifetime, an abandoned run or a forgotten conversation grows
//! them without limit until the process is OOM-killed. This module provides
//! one small, dependency-free container shared by the run trace buffer
//! (`runs/observability.rs`), the publish ledger (`pipeline/output_publish/`),
//! the sandbox preview cache (`sandbox/store.rs`) and the run thread bus
//! (`runs/bus.rs`).
//!
//! Contract:
//! - every entry carries a monotonic touch sequence (insertion order, refreshed
//!   on access) and a wall-clock touch timestamp (used for TTL);
//! - [`BoundedMap::insert`] and [`BoundedMap::get_or_insert_default`] first drop
//!   expired entries, then the least recently touched ones, so the map never
//!   exceeds its cap (entries marked by [`BoundedMap::protecting`] are exempt —
//!   in-flight work must survive eviction);
//! - caps and TTLs are configurable via environment variables with
//!   conservative defaults;
//! - no daemon thread: reclamation happens synchronously on the next write.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Wall clock in milliseconds; `0` when the system clock predates the epoch.
#[must_use]
pub fn now_ms() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => u64::try_from(duration.as_millis()).unwrap_or(0),
        Err(_) => 0,
    }
}

/// Positive capacity from `env`, or `default`. Non-numeric values use default.
#[must_use]
pub fn cap_from_env(env: &str, default: usize) -> usize {
    std::env::var(env)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

/// Positive millisecond TTL from `env`, or `default`.
#[must_use]
pub fn ttl_from_env(env: &str, default: u64) -> u64 {
    std::env::var(env)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

/// Pure TTL predicate: `true` when an entry last touched at `touched_ms` is
/// stale at `now`. A zero `ttl_ms` disables expiry.
#[must_use]
pub fn is_expired(touched_ms: u64, now: u64, ttl_ms: u64) -> bool {
    ttl_ms > 0 && now.saturating_sub(touched_ms) >= ttl_ms
}

/// Metadata of one evictable entry, passed to the pure [`eviction_plan`].
#[derive(Debug, Clone)]
pub struct EvictCandidate<K> {
    pub key: K,
    /// Monotonic insert/touch order; lower means older.
    pub seq: u64,
    pub touched_ms: u64,
}

/// Pure eviction planner: expired entries first, then the oldest by `seq`
/// until at most `cap` evictable entries remain. `protected_count` accounts for
/// entries that must never be evicted (in-flight runs); the effective cap for
/// the evictable set is `cap - protected_count`.
#[must_use]
pub fn eviction_plan<K: Clone>(
    evictable: &[EvictCandidate<K>],
    protected_count: usize,
    now: u64,
    ttl_ms: u64,
    cap: usize,
) -> Vec<K> {
    let target = cap.saturating_sub(protected_count);
    let mut plan = Vec::new();
    let mut live: Vec<&EvictCandidate<K>> = Vec::new();
    for candidate in evictable {
        if is_expired(candidate.touched_ms, now, ttl_ms) {
            plan.push(candidate.key.clone());
        } else {
            live.push(candidate);
        }
    }
    let overflow = live.len().saturating_sub(target);
    if overflow > 0 {
        live.sort_by_key(|candidate| candidate.seq);
        for candidate in live.into_iter().take(overflow) {
            plan.push(candidate.key.clone());
        }
    }
    plan
}

#[derive(Debug, Clone)]
struct Entry<V> {
    value: V,
    seq: u64,
    touched_ms: u64,
}

type Protect<K, V> = dyn Fn(&K, &V) -> bool + Send + Sync;

/// Small bounded LRU + TTL map. Reads refresh recency; writes run reclamation.
/// All operations are synchronous and lock-free from the caller's perspective
/// (the caller supplies the outer mutex when sharing across threads).
pub struct BoundedMap<K: Eq + Hash + Clone, V> {
    entries: HashMap<K, Entry<V>>,
    cap: usize,
    ttl_ms: u64,
    next_seq: u64,
    protect: Option<Arc<Protect<K, V>>>,
}

impl<K: Eq + Hash + Clone, V> BoundedMap<K, V> {
    #[must_use]
    pub fn new(cap: usize, ttl_ms: u64) -> Self {
        Self {
            entries: HashMap::new(),
            cap: cap.max(1),
            ttl_ms,
            next_seq: 0,
            protect: None,
        }
    }

    /// Marks entries that must survive eviction, e.g. in-flight run threads.
    #[must_use]
    pub fn protecting(mut self, predicate: impl Fn(&K, &V) -> bool + Send + Sync + 'static) -> Self {
        self.protect = Some(Arc::new(predicate));
        self
    }

    /// Reads a value and refreshes its recency.
    #[must_use]
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let (seq, now) = (self.bump(), now_ms());
        match self.entries.get_mut(key) {
            Some(entry) => {
                entry.seq = seq;
                entry.touched_ms = now;
                Some(&entry.value)
            }
            None => None,
        }
    }

    /// Mutable read with the same recency refresh as [`Self::get`].
    #[must_use]
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let (seq, now) = (self.bump(), now_ms());
        match self.entries.get_mut(key) {
            Some(entry) => {
                entry.seq = seq;
                entry.touched_ms = now;
                Some(&mut entry.value)
            }
            None => None,
        }
    }

    /// Inserts a value, evicting first so the map never exceeds its cap.
    pub fn insert(&mut self, key: K, value: V) {
        let reserve = usize::from(!self.entries.contains_key(&key));
        let now = now_ms();
        self.evict(now, reserve);
        let seq = self.bump();
        self.entries.insert(key, Entry { value, seq, touched_ms: now });
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.entries.remove(key).map(|entry| entry.value)
    }

    #[must_use]
    pub fn contains_key(&self, key: &K) -> bool {
        self.entries.contains_key(key)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn bump(&mut self) -> u64 {
        self.next_seq = self.next_seq.wrapping_add(1);
        self.next_seq
    }

    fn evict(&mut self, now: u64, reserve: usize) {
        let mut evictable = Vec::new();
        let mut protected_count = 0_usize;
        for (key, entry) in &self.entries {
            let pinned = match &self.protect {
                Some(predicate) => predicate(key, &entry.value),
                None => false,
            };
            if pinned {
                protected_count += 1;
            } else {
                evictable.push(EvictCandidate {
                    key: key.clone(),
                    seq: entry.seq,
                    touched_ms: entry.touched_ms,
                });
            }
        }
        let target = self.cap.saturating_sub(reserve);
        for key in eviction_plan(&evictable, protected_count, now, self.ttl_ms, target) {
            self.entries.remove(&key);
        }
    }
}

impl<K: Eq + Hash + Clone, V: Default> BoundedMap<K, V> {
    /// Returns the existing value or inserts a default, refreshing recency in
    /// both cases. Reclamation runs first, reserving room for a new key.
    pub fn get_or_insert_default(&mut self, key: K) -> &mut V {
        let is_new = !self.entries.contains_key(&key);
        let now = now_ms();
        self.evict(now, usize::from(is_new));
        let seq = self.bump();
        let entry = self.entries.entry(key).or_insert_with(|| Entry {
            value: V::default(),
            seq,
            touched_ms: now,
        });
        entry.seq = seq;
        entry.touched_ms = now;
        &mut entry.value
    }
}

#[cfg(test)]
mod tests {
    use super::{eviction_plan, is_expired, BoundedMap, EvictCandidate};

    fn candidate(key: &str, seq: u64, touched_ms: u64) -> EvictCandidate<String> {
        EvictCandidate {
            key: key.to_string(),
            seq,
            touched_ms,
        }
    }

    #[test]
    fn ttl_expiry_predicate_handles_boundary_and_clock_skew() {
        assert!(!is_expired(1_000, 1_999, 1_000));
        assert!(is_expired(1_000, 2_000, 1_000));
        assert!(!is_expired(500, 100, 1_000));
        assert!(!is_expired(0, u64::MAX, 0));
    }

    #[test]
    fn cap_reached_evicts_oldest_insertion() {
        let entries = vec![candidate("a", 1, 100), candidate("b", 2, 100), candidate("c", 3, 100)];
        assert_eq!(eviction_plan(&entries, 0, 100, 0, 2), vec!["a"]);
    }

    #[test]
    fn ttl_expiry_beats_lru_order() {
        let entries = vec![candidate("fresh", 1, 180), candidate("stale", 2, 10)];
        assert_eq!(eviction_plan(&entries, 0, 200, 100, 2), vec!["stale"]);
    }

    #[test]
    fn insertion_order_follows_sequence_not_slice_position() {
        let entries = vec![
            candidate("newest", 9, 100),
            candidate("oldest", 2, 100),
            candidate("middle", 5, 100),
        ];
        assert_eq!(eviction_plan(&entries, 0, 100, 0, 2), vec!["oldest"]);
    }

    #[test]
    fn protected_entries_shrink_the_evictable_budget() {
        let entries = vec![candidate("a", 1, 100), candidate("b", 2, 100)];
        assert_eq!(eviction_plan(&entries, 2, 100, 0, 2), vec!["a", "b"]);
    }

    #[test]
    fn bounded_map_never_exceeds_cap() {
        let mut map = BoundedMap::new(3, 0);
        for index in 0..10 {
            map.insert(format!("k{index}"), index);
        }
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn reading_refreshes_recency() {
        let mut map = BoundedMap::new(2, 0);
        let (a, b, c) = ("a".to_string(), "b".to_string(), "c".to_string());
        map.insert(a.clone(), 1);
        map.insert(b.clone(), 2);
        assert_eq!(map.get(&a), Some(&1));
        map.insert(c.clone(), 3);
        assert!(map.contains_key(&a));
        assert!(!map.contains_key(&b));
    }

    #[test]
    fn protected_entries_survive_cap_eviction() {
        let mut map = BoundedMap::new(3, 0)
            .protecting(|key: &String, value: &i32| key.as_str() == "keep" || *value > 10);
        let (keep, old, live, fresh) = (
            "keep".to_string(),
            "old".to_string(),
            "live".to_string(),
            "fresh".to_string(),
        );
        map.insert(keep.clone(), 1);
        map.insert(old.clone(), 2);
        map.insert(live.clone(), 20);
        map.insert(fresh.clone(), 3);
        assert!(map.contains_key(&keep));
        assert!(map.contains_key(&live));
        assert!(map.contains_key(&fresh));
        assert!(!map.contains_key(&old));
    }
}
