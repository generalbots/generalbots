//! Long-horizon goal runner arithmetic (#1705).
//!
//! A goal run advances one key result by a fixed step at every checkpoint,
//! clamping at the target, and stops once the target is reached or the
//! checkpoint budget is exhausted. The arithmetic is dependency-free so the
//! rounding/clamping rules are covered by executed tests.

/// Minimum/maximum automatic checkpoint interval (seconds).
pub const MIN_INTERVAL_SECS: u64 = 5;
pub const MAX_INTERVAL_SECS: u64 = 24 * 60 * 60;
/// Hard bound so a goal can never schedule an unbounded number of checkpoints.
pub const MAX_CHECKPOINTS: u32 = 200;

/// Advances `current` toward `target` by `step`, never overshooting.
#[must_use]
pub fn next_value(current: f64, target: f64, step: f64) -> f64 {
    if !current.is_finite() || !target.is_finite() || !step.is_finite() || step <= 0.0 {
        return current;
    }
    if current >= target {
        return target;
    }
    let next = current + step;
    if next >= target {
        target
    } else {
        next
    }
}

/// True when the value reached (or passed) its target.
#[must_use]
pub fn reached(current: f64, target: f64) -> bool {
    current >= target
}

/// Completion percentage of the key result, clamped to 0..=100.
#[must_use]
pub fn progress_percent(start: f64, target: f64, current: f64) -> u8 {
    if !start.is_finite() || !target.is_finite() || !current.is_finite() || target == start {
        return 100;
    }
    let ratio = (current - start) / (target - start);
    let percent = (ratio * 100.0).round();
    if percent <= 0.0 {
        return 0;
    }
    if percent >= 100.0 {
        return 100;
    }
    u8::try_from(percent as i64).unwrap_or(100)
}

/// Clamps a requested interval into the supported range.
#[must_use]
pub fn clamp_interval(seconds: u64) -> u64 {
    seconds.clamp(MIN_INTERVAL_SECS, MAX_INTERVAL_SECS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_value_clamps_at_the_target() {
        assert_eq!(next_value(0.0, 10.0, 3.0), 3.0);
        assert_eq!(next_value(9.0, 10.0, 3.0), 10.0);
        assert_eq!(next_value(10.0, 10.0, 3.0), 10.0);
    }

    #[test]
    fn invalid_or_zero_steps_do_not_advance() {
        assert_eq!(next_value(2.0, 10.0, 0.0), 2.0);
        assert_eq!(next_value(2.0, 10.0, -5.0), 2.0);
        assert!(next_value(f64::NAN, 10.0, 1.0).is_nan());
    }

    #[test]
    fn completion_and_percent_are_clamped() {
        assert!(!reached(9.0, 10.0));
        assert!(reached(10.0, 10.0));
        assert_eq!(progress_percent(0.0, 10.0, 0.0), 0);
        assert_eq!(progress_percent(0.0, 10.0, 5.0), 50);
        assert_eq!(progress_percent(0.0, 10.0, 12.0), 100);
        assert_eq!(progress_percent(5.0, 5.0, 5.0), 100);
    }

    #[test]
    fn intervals_are_bounded() {
        assert_eq!(clamp_interval(0), MIN_INTERVAL_SECS);
        assert_eq!(clamp_interval(60), 60);
        assert_eq!(clamp_interval(u64::MAX), MAX_INTERVAL_SECS);
        assert!(MAX_CHECKPOINTS >= 10);
    }
}
