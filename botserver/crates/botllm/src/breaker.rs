//! Minimal circuit breaker protecting each routed LLM profile (issue #1173).
//!
//! The breaker counts consecutive failures; once the threshold is reached it
//! opens for a fixed interval, during which `allow()` returns `false` so the
//! router skips the profile. Any success closes the circuit again.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

const FAILURE_THRESHOLD_DEFAULT: u32 = 5;
const OPEN_SECS_DEFAULT: u64 = 60;

/// Failure counter plus open-until timestamp guarded by a mutex.
pub struct Breaker {
    failures: AtomicU32,
    open_until: Mutex<Option<Instant>>,
}

impl Default for Breaker {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Breaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Breaker")
            .field("failures", &self.failures.load(Ordering::Relaxed))
            .finish()
    }
}

impl Breaker {
    /// Creates a closed breaker with zero recorded failures.
    pub const fn new() -> Self {
        Self {
            failures: AtomicU32::new(0),
            open_until: Mutex::new(None),
        }
    }

    fn lock_guard(
        &self,
    ) -> std::sync::MutexGuard<'_, Option<Instant>> {
        match self.open_until.lock() {
            Ok(guard) => guard,
            Err(poisoned) => PoisonError::into_inner(poisoned),
        }
    }

    /// Reports whether a request may be attempted right now.
    pub fn allow(&self) -> bool {
        let guard = self.lock_guard();
        match *guard {
            Some(until) => Instant::now() >= until,
            None => true,
        }
    }

    /// Resets the failure count and closes the circuit after a success.
    pub fn record_success(&self) {
        self.failures.store(0, Ordering::Relaxed);
        let mut guard = self.lock_guard();
        *guard = None;
    }

    /// Records one failure; opens the circuit when `threshold` consecutive
    /// failures accumulate, keeping it open for `open_secs`.
    pub fn record_failure(&self, threshold: u32, open_secs: u64) {
        let total = self.failures.fetch_add(1, Ordering::Relaxed) + 1;
        if total >= threshold {
            let mut guard = self.lock_guard();
            *guard = Some(Instant::now() + Duration::from_secs(open_secs));
            self.failures.store(0, Ordering::Relaxed);
        }
    }

    /// Convenience wrapper using the platform defaults (threshold 5, 60 seconds).
    pub fn record_failure_default(&self) {
        self.record_failure(FAILURE_THRESHOLD_DEFAULT, OPEN_SECS_DEFAULT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_closed_and_stays_closed_under_threshold() {
        let breaker = Breaker::new();
        assert!(breaker.allow());
        for _ in 0..4 {
            breaker.record_failure(5, 60);
        }
        assert!(breaker.allow());
    }

    #[test]
    fn opens_after_threshold_failures() {
        let breaker = Breaker::new();
        for _ in 0..5 {
            breaker.record_failure(5, 60);
        }
        assert!(!breaker.allow());
    }

    #[test]
    fn success_resets_failure_count() {
        let breaker = Breaker::new();
        breaker.record_failure(5, 60);
        breaker.record_failure(5, 60);
        breaker.record_success();
        for _ in 0..4 {
            breaker.record_failure(5, 60);
        }
        assert!(breaker.allow());
    }

    #[test]
    fn success_closes_open_circuit_immediately() {
        let breaker = Breaker::new();
        for _ in 0..5 {
            breaker.record_failure(5, 60);
        }
        assert!(!breaker.allow());
        breaker.record_success();
        assert!(breaker.allow());
    }

    #[test]
    fn circuit_recovers_after_open_window() {
        let breaker = Breaker::new();
        for _ in 0..5 {
            breaker.record_failure(5, 0);
        }
        std::thread::sleep(Duration::from_millis(10));
        assert!(breaker.allow());
    }
}
