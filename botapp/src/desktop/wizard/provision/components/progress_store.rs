//! Last known state of every wizard row, as `status` replays it to a window
//! that polls instead of listening.
//!
//! A failed row keeps its failure for the rest of the run: a later event for
//! that row (a "starting" from a step that was already under way) would erase
//! the error the person has to read. A retry is a new run, and `provision`
//! clears the store before it starts, so the row can move again then.

use super::super::super::ProgressEvent;

/// Records `event` as its row's last state. Returns `false` when the event is
/// dropped because the row already failed in this run.
pub(super) fn record(progress: &mut Vec<ProgressEvent>, event: &ProgressEvent) -> bool {
    match progress.iter_mut().find(|entry| entry.id == event.id) {
        Some(existing) if existing.state == "failed" => false,
        Some(existing) => {
            *existing = event.clone();
            true
        }
        None => {
            progress.push(event.clone());
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(id: &str, state: &str, message: &str) -> ProgressEvent {
        ProgressEvent {
            id: id.to_string(),
            state: state.to_string(),
            message: message.to_string(),
            elapsed_ms: 0,
        }
    }

    #[test]
    fn a_row_moves_forward_until_it_fails() {
        let mut store = Vec::new();
        assert!(record(&mut store, &event("botserver", "starting", "Starting")));
        assert!(record(&mut store, &event("botserver", "failed", "port in use")));
        assert_eq!(store.len(), 1);
        assert_eq!(store[0].state, "failed");
    }

    #[test]
    fn a_failed_row_is_not_overwritten_by_a_later_event() {
        let mut store = Vec::new();
        record(&mut store, &event("botserver", "failed", "port in use"));
        assert!(!record(&mut store, &event("botserver", "starting", "Starting")));
        assert!(!record(&mut store, &event("botserver", "ready", "ready")));
        assert_eq!(store[0].state, "failed");
        assert_eq!(store[0].message, "port in use");
    }

    #[test]
    fn other_rows_keep_updating_after_one_fails() {
        let mut store = Vec::new();
        record(&mut store, &event("botserver", "failed", "port in use"));
        assert!(record(&mut store, &event("__done__", "ready", "done")));
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn a_retry_starts_from_a_cleared_store() {
        let mut store = Vec::new();
        record(&mut store, &event("botserver", "failed", "port in use"));
        store.clear();
        assert!(record(&mut store, &event("botserver", "starting", "Starting")));
        assert_eq!(store[0].state, "starting");
    }
}
