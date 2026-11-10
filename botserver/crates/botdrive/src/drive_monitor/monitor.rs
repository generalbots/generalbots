use std::sync::atomic::Ordering;
use std::time::Duration;

use super::types::DriveMonitor;

/// Intervalo de verificação do DriveMonitor e DriveCompiler (em segundos)
pub const CHECK_INTERVAL_SECS: u64 = 5;

/// Backoff for a given number of consecutive failures.
///
/// The first four failures double the interval; from there it stays at the
/// cap so a permanently broken bucket does not hammer the object store.
#[must_use]
pub fn backoff_for_failures(failures: u32) -> Duration {
    if failures == 0 {
        return Duration::from_secs(CHECK_INTERVAL_SECS);
    }
    let backoff_secs = CHECK_INTERVAL_SECS * (1u64 << failures.min(4));
    Duration::from_secs(backoff_secs.min(300))
}

impl DriveMonitor {
    pub fn calculate_backoff(&self) -> Duration {
        backoff_for_failures(self.consecutive_failures.load(Ordering::Relaxed))
    }
}

#[cfg(test)]
mod backoff_tests {
    use super::*;

    #[test]
    fn a_healthy_monitor_polls_at_the_check_interval() {
        assert_eq!(
            backoff_for_failures(0),
            Duration::from_secs(CHECK_INTERVAL_SECS)
        );
    }

    #[test]
    fn each_failure_doubles_the_interval() {
        assert_eq!(backoff_for_failures(1), Duration::from_secs(10));
        assert_eq!(backoff_for_failures(2), Duration::from_secs(20));
        assert_eq!(backoff_for_failures(3), Duration::from_secs(40));
        assert_eq!(backoff_for_failures(4), Duration::from_secs(80));
    }

    #[test]
    fn the_interval_is_capped() {
        // 5s * 2^4 = 80s; the shift is clamped, so every later value is 80s
        // and never exceeds the 300s ceiling.
        for failures in [5, 6, 10, 32, u32::MAX] {
            let backoff = backoff_for_failures(failures);
            assert_eq!(backoff, Duration::from_secs(80), "{failures} failures");
            assert!(backoff <= Duration::from_secs(300));
        }
    }

    #[test]
    fn the_interval_never_regresses_as_failures_grow() {
        let mut previous = Duration::ZERO;
        for failures in 0..12 {
            let current = backoff_for_failures(failures);
            assert!(
                current >= previous,
                "{failures} failures must not back off less than {}",
                failures.saturating_sub(1)
            );
            previous = current;
        }
    }
}
