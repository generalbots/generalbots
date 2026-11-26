//! Keyspace of the built-in cache: values, expiry and the memory bound.
//!
//! Nothing is persisted. Past the memory bound, expired keys go first, then
//! the keys closest to expiry, then any key: some keys (the suggestion sets,
//! the chat controls) never expire, so the last step is needed to stay bounded.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Approximate payload bound for the whole keyspace.
pub(super) const MEMORY_CAP: usize = 64 * 1024 * 1024;
const ENTRY_OVERHEAD: usize = 64;
const ELEMENT_OVERHEAD: usize = 32;
const SWEEP_EVERY: Duration = Duration::from_secs(1);

pub(super) enum Value {
    Str(Vec<u8>),
    Set(HashSet<Vec<u8>>),
    Hash(HashMap<Vec<u8>, Vec<u8>>),
}

pub(super) struct Entry {
    pub(super) value: Value,
    pub(super) expires_at: Option<Instant>,
}

impl Entry {
    pub(super) fn new(value: Value, expires_at: Option<Instant>) -> Self {
        Self { value, expires_at }
    }

    fn expired(&self, now: Instant) -> bool {
        self.expires_at.is_some_and(|at| at <= now)
    }

    fn size(&self, key: &[u8]) -> usize {
        let payload = match &self.value {
            Value::Str(bytes) => bytes.len(),
            Value::Set(members) => members.iter().map(|m| m.len() + ELEMENT_OVERHEAD).sum(),
            Value::Hash(fields) => fields
                .iter()
                .map(|(f, v)| f.len() + v.len() + ELEMENT_OVERHEAD)
                .sum(),
        };
        ENTRY_OVERHEAD + key.len() + payload
    }
}

pub(super) struct Store {
    entries: HashMap<Vec<u8>, Entry>,
    used: usize,
    cap: usize,
    last_sweep: Instant,
    eviction_warned: bool,
}

impl Store {
    pub(super) fn new(cap: usize) -> Self {
        Self {
            entries: HashMap::new(),
            used: 0,
            cap,
            last_sweep: Instant::now(),
            eviction_warned: false,
        }
    }

    /// Drops expired keys at most once per second; called before each command.
    pub(super) fn tick(&mut self) {
        if self.last_sweep.elapsed() >= SWEEP_EVERY {
            self.purge_expired();
        }
    }

    pub(super) fn get(&mut self, key: &[u8]) -> Option<&Entry> {
        self.drop_if_expired(key);
        self.entries.get(key)
    }

    pub(super) fn take(&mut self, key: &[u8]) -> Option<Entry> {
        self.drop_if_expired(key);
        self.take_raw(key)
    }

    pub(super) fn put(&mut self, key: Vec<u8>, entry: Entry) {
        self.take_raw(&key);
        self.used += entry.size(&key);
        self.entries.insert(key.clone(), entry);
        self.enforce_cap(&key);
    }

    /// Sets or clears the expiry of a live key; false when the key is missing.
    pub(super) fn set_expiry(&mut self, key: &[u8], expires_at: Option<Instant>) -> bool {
        self.drop_if_expired(key);
        match self.entries.get_mut(key) {
            Some(entry) => {
                entry.expires_at = expires_at;
                true
            }
            None => false,
        }
    }

    pub(super) fn len(&mut self) -> usize {
        self.purge_expired();
        self.entries.len()
    }

    pub(super) fn used(&self) -> usize {
        self.used
    }

    pub(super) fn clear(&mut self) {
        self.entries.clear();
        self.used = 0;
    }

    pub(super) fn matching_keys(&mut self, pattern: &[u8]) -> Vec<Vec<u8>> {
        self.purge_expired();
        self.entries
            .keys()
            .filter(|key| glob_match(pattern, key))
            .cloned()
            .collect()
    }

    /// One SCAN step. The cursor is a position in a fixed hash order, so keys
    /// deleted between steps do not make the walk skip the others. Cursors are
    /// always in `1..=i64::MAX` (positive i64) because the call sites parse
    /// them as `i64`; the sentinel "iteration complete" cursor is 0.
    pub(super) fn scan(
        &mut self,
        cursor: u64,
        pattern: Option<&[u8]>,
        count: usize,
    ) -> (u64, Vec<Vec<u8>>) {
        self.purge_expired();
        let mut ordered: Vec<(u64, &Vec<u8>)> = self
            .entries
            .keys()
            .map(|key| (scan_cursor(key), key))
            .filter(|(hash, _)| *hash >= cursor)
            .collect();
        ordered.sort_unstable();
        let next = ordered.get(count).map_or(0, |(hash, _)| *hash);
        let keys = ordered
            .iter()
            .take(count)
            .filter(|(_, key)| pattern.is_none_or(|p| glob_match(p, key)))
            .map(|(_, key)| (*key).clone())
            .collect();
        (next, keys)
    }

    fn drop_if_expired(&mut self, key: &[u8]) {
        let now = Instant::now();
        if self.entries.get(key).is_some_and(|entry| entry.expired(now)) {
            self.take_raw(key);
        }
    }

    fn take_raw(&mut self, key: &[u8]) -> Option<Entry> {
        let entry = self.entries.remove(key)?;
        self.used = self.used.saturating_sub(entry.size(key));
        Some(entry)
    }

    fn purge_expired(&mut self) {
        let now = Instant::now();
        let expired: Vec<Vec<u8>> = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.expired(now))
            .map(|(key, _)| key.clone())
            .collect();
        for key in expired {
            self.take_raw(&key);
        }
        self.last_sweep = now;
    }

    fn enforce_cap(&mut self, keep: &[u8]) {
        if self.used <= self.cap {
            return;
        }
        self.purge_expired();
        while self.used > self.cap {
            let victim = self
                .entries
                .iter()
                .filter(|(key, _)| key.as_slice() != keep)
                .min_by_key(|(_, entry)| match entry.expires_at {
                    Some(at) => (0_u8, Some(at)),
                    None => (1_u8, None),
                })
                .map(|(key, _)| key.clone());
            let Some(victim) = victim else {
                break;
            };
            self.take_raw(&victim);
            if !self.eviction_warned {
                self.eviction_warned = true;
                log::warn!(
                    "Built-in cache reached its {} MiB bound; evicting keys",
                    self.cap / (1024 * 1024)
                );
            }
        }
    }
}

/// Redis glob: `*`, `?`, `[abc]`, `[^a]`, `[a-z]` and `\` escapes.
pub(super) fn glob_match(pattern: &[u8], text: &[u8]) -> bool {
    match pattern.split_first() {
        None => text.is_empty(),
        Some((b'*', rest)) => {
            let rest = trim_stars(rest);
            rest.is_empty() || (0..=text.len()).any(|i| glob_match(rest, text.get(i..).unwrap_or_default()))
        }
        Some((b'?', rest)) => text
            .split_first()
            .is_some_and(|(_, tail)| glob_match(rest, tail)),
        Some((b'[', rest)) => {
            let Some((&c, tail)) = text.split_first() else {
                return false;
            };
            match class_match(rest, c) {
                Some((matched, after)) => matched && glob_match(after, tail),
                None => c == b'[' && glob_match(rest, tail),
            }
        }
        Some((b'\\', [escaped, rest @ ..])) => literal(*escaped, rest, text),
        Some((&p, rest)) => literal(p, rest, text),
    }
}

fn literal(expected: u8, rest: &[u8], text: &[u8]) -> bool {
    text.split_first()
        .is_some_and(|(&c, tail)| c == expected && glob_match(rest, tail))
}

fn trim_stars(mut pattern: &[u8]) -> &[u8] {
    while let Some((b'*', rest)) = pattern.split_first() {
        pattern = rest;
    }
    pattern
}

/// Matches one byte against a `[...]` class; None when the class never closes.
fn class_match(class: &[u8], c: u8) -> Option<(bool, &[u8])> {
    let (negate, mut rest) = match class.split_first() {
        Some((b'^', tail)) => (true, tail),
        _ => (false, class),
    };
    let mut matched = false;
    loop {
        match rest {
            [] => return None,
            [b']', tail @ ..] => return Some((matched != negate, tail)),
            [b'\\', escaped, tail @ ..] => {
                matched |= *escaped == c;
                rest = tail;
            }
            [lo, b'-', hi, tail @ ..] if *hi != b']' => {
                let (lo, hi) = if lo <= hi { (*lo, *hi) } else { (*hi, *lo) };
                matched |= (lo..=hi).contains(&c);
                rest = tail;
            }
            [ch, tail @ ..] => {
                matched |= *ch == c;
                rest = tail;
            }
        }
    }
}

/// A SCAN cursor for a key: always in `1..=i64::MAX` so the call sites that
/// parse cursors as `i64` never see a negative number, and 0 is reserved for
/// the "iteration complete" sentinel.
fn scan_cursor(key: &[u8]) -> u64 {
    let hash = key.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    // Clear the sign bit so it fits in a positive i64, then map 0 to 1.
    match hash & (i64::MAX as u64) {
        0 => 1,
        n => n,
    }
}
