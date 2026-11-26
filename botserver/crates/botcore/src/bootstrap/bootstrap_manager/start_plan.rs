//! Start-up plan for the local stack: which services may start together, and
//! how to wait for a service by polling its health check instead of sleeping
//! a fixed time.

use std::time::{Duration, Instant};

/// Services that need another service up before their own start command runs.
/// Zitadel (`directory`) and Forgejo (`alm`) open their PostgreSQL connection
/// while starting and exit when it is missing.
pub(super) fn depends_on(name: &str) -> &'static [&'static str] {
    match name {
        "directory" | "alm" => &["tables"],
        _ => &[],
    }
}

/// Groups the services into waves: every service of a wave starts at the same
/// time, and a wave begins once the services it depends on were started by an
/// earlier wave. A dependency that is not in `names` (not installed, skipped)
/// does not hold a service back.
pub(super) fn start_waves(names: &[&'static str]) -> Vec<Vec<&'static str>> {
    let mut waves: Vec<Vec<&'static str>> = Vec::new();
    let mut placed: Vec<&'static str> = Vec::new();
    let mut pending: Vec<&'static str> = names.to_vec();
    while !pending.is_empty() {
        let (ready, blocked): (Vec<_>, Vec<_>) = pending.iter().copied().partition(|name| {
            depends_on(name)
                .iter()
                .all(|dep| placed.contains(dep) || !names.contains(dep))
        });
        if ready.is_empty() {
            // A cycle cannot happen with the table above; start the rest
            // together rather than never.
            waves.push(blocked);
            break;
        }
        placed.extend(ready.iter().copied());
        waves.push(ready);
        pending = blocked;
    }
    waves
}

/// Polls `check` every `interval` until it answers or `max` has passed.
/// Returns how long the service took, or `None` when it never answered.
pub(super) fn wait_ready(
    check: impl Fn() -> bool,
    max: Duration,
    interval: Duration,
) -> Option<Duration> {
    let start = Instant::now();
    loop {
        if check() {
            return Some(start.elapsed());
        }
        if start.elapsed() >= max {
            return None;
        }
        std::thread::sleep(interval);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn independent_services_start_in_one_wave() {
        let waves = start_waves(&["vector_db", "tables", "cache", "drive", "llm"]);
        assert_eq!(waves, vec![vec!["vector_db", "tables", "cache", "drive", "llm"]]);
    }

    #[test]
    fn directory_waits_for_tables() {
        let waves = start_waves(&["vector_db", "tables", "cache", "drive", "directory", "llm"]);
        assert_eq!(
            waves,
            vec![vec!["vector_db", "tables", "cache", "drive", "llm"], vec!["directory"]]
        );
    }

    #[test]
    fn missing_dependency_does_not_hold_a_service_back() {
        let waves = start_waves(&["cache", "directory"]);
        assert_eq!(waves, vec![vec!["cache", "directory"]]);
    }

    #[test]
    fn empty_plan_has_no_waves() {
        assert!(start_waves(&[]).is_empty());
    }

    #[test]
    fn wait_ready_returns_as_soon_as_the_check_answers() {
        let calls = Cell::new(0u32);
        let took = wait_ready(
            || {
                calls.set(calls.get() + 1);
                calls.get() >= 3
            },
            Duration::from_secs(5),
            Duration::from_millis(1),
        );
        assert!(took.is_some());
        assert_eq!(calls.get(), 3);
    }

    #[test]
    fn wait_ready_gives_up_after_the_limit() {
        let calls = Cell::new(0u32);
        let took = wait_ready(
            || {
                calls.set(calls.get() + 1);
                false
            },
            Duration::from_millis(20),
            Duration::from_millis(5),
        );
        assert!(took.is_none());
        assert!(calls.get() >= 2);
    }

    #[test]
    fn wait_ready_checks_once_even_with_no_time() {
        let took = wait_ready(|| true, Duration::ZERO, Duration::from_millis(1));
        assert_eq!(took.map(|d| d < Duration::from_secs(1)), Some(true));
    }
}
