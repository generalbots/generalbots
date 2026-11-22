//! Progress of a chat turn's model call, for its outer time limit.
//!
//! `LLM_TURN_TIMEOUT_SECS` bounds quiet time, not total time: a model chunk,
//! an open permission card and an allowed workspace action still running all
//! count as progress, so a bridged agent turn that waits on the person or runs
//! a long command is not ended while it works. The agent bridges end their own
//! sessions after 30 minutes; [`TURN_CEILING`] stays above that.

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::{Duration, Instant};

/// Longest a turn may run in total, whatever its progress.
pub const TURN_CEILING: Duration = Duration::from_secs(45 * 60);
const POLL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy)]
struct Entry {
    last: Instant,
    busy: u32,
    depth: u32,
}

static TURNS: OnceLock<Mutex<HashMap<String, Entry>>> = OnceLock::new();

fn with<R>(change: impl FnOnce(&mut HashMap<String, Entry>) -> R) -> R {
    let mut turns = TURNS.get_or_init(Mutex::default).lock().unwrap_or_else(PoisonError::into_inner);
    change(&mut turns)
}

/// Something happened in the turn of conversation `thread`.
pub fn touch(thread: &str) {
    with(|turns| {
        if let Some(entry) = turns.get_mut(thread) {
            entry.last = Instant::now();
        }
    });
}

/// An open card or a running action in `thread` began (`true`) or ended.
pub fn busy(thread: &str, began: bool) {
    with(|turns| {
        if let Some(entry) = turns.get_mut(thread) {
            entry.busy = if began { entry.busy.saturating_add(1) } else { entry.busy.saturating_sub(1) };
            entry.last = Instant::now();
        }
    });
}

fn quiet(thread: &str, now: Instant) -> Duration {
    with(|turns| match turns.get(thread) {
        Some(entry) if entry.busy == 0 => now.saturating_duration_since(entry.last),
        _ => Duration::ZERO,
    })
}

struct Watched(String);

impl Drop for Watched {
    fn drop(&mut self) {
        with(|turns| {
            if let Some(entry) = turns.get_mut(&self.0) {
                entry.depth = entry.depth.saturating_sub(1);
                if entry.depth == 0 {
                    turns.remove(&self.0);
                }
            }
        });
    }
}

/// Runs `turn` until it ends; `None` when it made no progress for `limit`
/// or ran past [`TURN_CEILING`] (the turn is dropped).
pub async fn until_quiet<F: Future>(thread: &str, limit: Duration, turn: F) -> Option<F::Output> {
    let started = Instant::now();
    with(|turns| {
        let entry = turns.entry(thread.to_string()).or_insert(Entry { last: started, busy: 0, depth: 0 });
        entry.depth = entry.depth.saturating_add(1);
        entry.last = started;
    });
    let watched = Watched(thread.to_string());
    let mut turn = std::pin::pin!(turn);
    let mut tick = tokio::time::interval(POLL.min(limit / 4).max(Duration::from_millis(10)));
    let output = loop {
        tokio::select! {
            output = &mut turn => break Some(output),
            _ = tick.tick() => {
                let now = Instant::now();
                if quiet(thread, now) >= limit || now.saturating_duration_since(started) >= TURN_CEILING {
                    break None;
                }
            }
        }
    };
    drop(watched);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMIT: Duration = Duration::from_millis(200);

    #[tokio::test]
    async fn quiet_time_ends_the_turn_and_progress_keeps_it() {
        let stalled = until_quiet("t-quiet", LIMIT, tokio::time::sleep(Duration::from_secs(5))).await;
        assert!(stalled.is_none());
        let working = async {
            for _ in 0..12 {
                tokio::time::sleep(Duration::from_millis(50)).await;
                touch("t-work");
            }
            "done"
        };
        assert_eq!(until_quiet("t-work", LIMIT, working).await, Some("done"));
    }

    #[tokio::test]
    async fn an_open_card_counts_as_progress() {
        let waiting = async {
            busy("t-card", true);
            tokio::time::sleep(Duration::from_millis(700)).await;
            busy("t-card", false);
            "answered"
        };
        assert_eq!(until_quiet("t-card", LIMIT, waiting).await, Some("answered"));
        assert_eq!(quiet("t-card", Instant::now()), Duration::ZERO, "the entry is gone after the turn");
    }

    #[tokio::test]
    async fn nested_limits_share_one_entry() {
        let inner = until_quiet("t-nest", LIMIT, async { "inner" });
        let outer = async {
            let first = inner.await;
            tokio::time::sleep(Duration::from_millis(100)).await;
            first
        };
        assert_eq!(until_quiet("t-nest", LIMIT, outer).await, Some(Some("inner")));
        assert_eq!(quiet("t-nest", Instant::now()), Duration::ZERO);
    }
}
