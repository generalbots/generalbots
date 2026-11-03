//! Thread-scoped sequence numbers.
//!
//! One sequencer per thread owns the counter, so two runs never mint the same
//! `seq`. The registry is process local: the durable ordering guarantee comes
//! from the `UNIQUE (thread_id, seq)` constraint on `run_events`, and a run
//! that resumes after a restart is seeded from the last persisted `seq`.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct ThreadSequencer {
    next: AtomicU64,
}

impl ThreadSequencer {
    /// Starts a sequencer whose first minted value is `start` (inclusive).
    #[must_use]
    pub fn starting_at(start: u64) -> Self {
        Self {
            next: AtomicU64::new(start.max(1)),
        }
    }

    pub fn next_seq(&self) -> u64 {
        self.next.fetch_add(1, Ordering::SeqCst)
    }

    /// Highest value handed out so far (0 when nothing was minted).
    #[must_use]
    pub fn last_seq(&self) -> u64 {
        self.next.load(Ordering::SeqCst).saturating_sub(1)
    }

    /// Raises the counter so the next minted value is greater than
    /// `last_seen`. Used when a run resumes from the persisted log.
    pub fn advance_to(&self, last_seen: u64) {
        self.next
            .fetch_max(last_seen.saturating_add(1), Ordering::SeqCst);
    }
}

/// Map of thread id to its sequencer. Cheap to clone behind an `Arc`.
#[derive(Debug, Default)]
pub struct SequencerRegistry {
    threads: Mutex<HashMap<String, Arc<ThreadSequencer>>>,
}

impl SequencerRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            threads: Mutex::new(HashMap::new()),
        }
    }

    /// Returns the sequencer for `thread_id`, creating one at `start` when it
    /// does not exist yet.
    pub fn sequencer(&self, thread_id: &str, start: u64) -> Arc<ThreadSequencer> {
        let mut guard = match self.threads.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard
            .entry(thread_id.to_string())
            .or_insert_with(|| Arc::new(ThreadSequencer::starting_at(start)))
            .clone()
    }

    /// Drops the in-memory counter for a thread (used when a thread is
    /// deleted). The counter is rebuilt from the log on the next run.
    pub fn forget(&self, thread_id: &str) {
        let mut guard = match self.threads.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard.remove(thread_id);
    }
}

#[cfg(test)]
mod tests {
    use super::{SequencerRegistry, ThreadSequencer};

    #[test]
    fn mints_dense_values() {
        let sequencer = ThreadSequencer::starting_at(1);
        assert_eq!(sequencer.next_seq(), 1);
        assert_eq!(sequencer.next_seq(), 2);
        assert_eq!(sequencer.last_seq(), 2);
    }

    #[test]
    fn one_counter_per_thread() {
        let registry = SequencerRegistry::new();
        let first = registry.sequencer("a", 1);
        let second = registry.sequencer("a", 99);
        assert_eq!(first.next_seq(), 1);
        assert_eq!(second.next_seq(), 2);
        let other = registry.sequencer("b", 1);
        assert_eq!(other.next_seq(), 1);
    }
}
