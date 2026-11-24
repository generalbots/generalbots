//! Offline edit queue with rebase-on-reconnect (#2447).
//!
//! A client that loses its WebSocket currently keeps an offline draft, and on
//! reconnect replays queued ops verbatim on top of whatever the server did in
//! the meantime — with no rebasing, an edit made while offline is discarded if
//! a remote op happened to carry a higher sequence number. `OfflineQueue`
//! buffers local edits while disconnected and `reconcile` folds them back with
//! the missed remote ops using the LWW-Register merge from [`super::merge`].
//!
//! Reconciliation policy (documented so every client behaves the same):
//! offline intent wins on cells that a *different* actor also touched while we
//! were away. Surviving local ops are re-stamped above the highest observed
//! remote clock, then pushed through the same deterministic merge. Because the
//! re-stamp derives only from remote clocks plus actor id, two clients that
//! reconnect with the same op set still converge to one value.

use super::merge::{CellKey, CellOp, CellState, LamportClock};
use std::collections::HashMap;

/// Local edits made while the client had no connection, in application order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineQueue {
    actor: String,
    clock: LamportClock,
    pending: Vec<CellOp>,
}

impl OfflineQueue {
    pub fn new(actor: &str) -> Self {
        Self {
            actor: actor.to_string(),
            clock: LamportClock::new(),
            pending: Vec::new(),
        }
    }

    pub fn actor(&self) -> &str {
        &self.actor
    }

    pub fn pending(&self) -> &[CellOp] {
        &self.pending
    }

    /// Fold in the highest remote clock observed before disconnecting so local
    /// edits are causally newer than server edits made before the disconnect.
    pub fn observe(&mut self, remote: u64) {
        self.clock.observe(remote);
    }

    /// Buffer a local cell edit. Returns the op so callers can render/echo it
    /// optimistically in their own grid while offline.
    pub fn edit(&mut self, cell: CellKey, value: Option<String>) -> CellOp {
        let op = CellOp::new(&self.actor, self.clock.tick(), cell, value);
        self.pending.push(op.clone());
        op
    }
}

/// Outcome of folding remote ops and buffered local intent together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileResult {
    /// The converged state after applying remote ops and replayed local ops.
    pub state: CellState,
    /// Local ops the client must push to the server on reconnect.
    pub replay: Vec<CellOp>,
    /// Superseded local edits that were not replayed (older edits to a cell).
    pub dropped: usize,
}

/// Reconcile a snapshot `base` (state at disconnect) with ops the server
/// accepted while offline and the local [`OfflineQueue`].
pub fn reconcile(base: &CellState, queue: &OfflineQueue, remote: &[CellOp]) -> ReconcileResult {
    let mut state = base.clone();
    for op in remote {
        state.apply(op.clone());
    }

    // Only the last local edit per cell carries surviving intent; earlier
    // buffered edits to the same cell are superseded and must not be replayed.
    let mut local_winner: HashMap<CellKey, CellOp> = HashMap::new();
    for op in &queue.pending {
        local_winner.insert(op.cell, op.clone());
    }

    let mut cells: Vec<CellKey> = local_winner.keys().copied().collect();
    cells.sort();

    let mut clock = queue.clock.clone();
    // Seed from the whole converged view (snapshot + missed remote ops), not
    // just the missed ops: a missed op can carry a lower clock than an op that
    // was already in the snapshot, and rebasing must never lower local clocks.
    clock.observe(state.max_clock());

    let mut replay = Vec::with_capacity(cells.len());
    for cell in cells {
        let Some(mut op) = local_winner.get(&cell).cloned() else {
            continue;
        };
        // A different actor won this cell while we were away: rebase our last
        // edit on top of their clock so the offline edit is not silently lost.
        if state
            .op(&cell)
            .is_some_and(|winner| winner.actor != queue.actor)
        {
            op.clock = clock.tick();
            let actor = queue.actor.as_str();
            let clock = op.clock;
            op.id = format!("{actor}:{clock}");
        }
        state.apply(op.clone());
        replay.push(op);
    }

    let dropped = queue.pending.len().saturating_sub(replay.len());
    ReconcileResult {
        state,
        replay,
        dropped,
    }
}

#[cfg(test)]
mod tests {
    use super::{CellState, OfflineQueue};
    use crate::collaboration::merge::{CellKey, CellOp};

    fn op(actor: &str, clock: u64, cell: CellKey, value: &str) -> CellOp {
        CellOp::new(actor, clock, cell, Some(value.to_string()))
    }

    /// Offline-reconnect case: Alice edits a cell while disconnected, Bob edits
    /// the same cell on the server with a higher clock. Rebase must preserve
    /// Alice's offline intent instead of replaying it verbatim and losing.
    #[test]
    fn reconnect_rebases_local_intent_over_remote_edit() {
        let cell = CellKey::new(0, 1, 1);
        let mut base = CellState::new();
        base.apply(op("server", 2, cell, "original"));

        let mut queue = OfflineQueue::new("alice");
        queue.observe(2);
        let local = queue.edit(cell, Some("alice offline".to_string()));
        assert_eq!(local.clock, 3);

        let remote = vec![op("bob", 9, cell, "bob online")];
        let result = super::reconcile(&base, &queue, &remote);

        assert_eq!(result.state.value(&cell), Some("alice offline"));
        assert_eq!(result.replay.len(), 1);
        assert_eq!(result.replay[0].value.as_deref(), Some("alice offline"));
        assert!(
            result.replay[0].clock > 9,
            "replayed op must outrank remote"
        );
        assert_eq!(result.dropped, 0);
    }

    #[test]
    fn reconnect_drops_superseded_pending_edits() {
        let cell = CellKey::new(0, 0, 0);
        let mut queue = OfflineQueue::new("alice");
        queue.edit(cell, Some("draft 1".to_string()));
        queue.edit(cell, Some("draft 2".to_string()));
        queue.edit(cell, Some("final".to_string()));

        let result = super::reconcile(&CellState::new(), &queue, &[]);

        assert_eq!(result.replay.len(), 1);
        assert_eq!(result.replay[0].value.as_deref(), Some("final"));
        assert_eq!(result.dropped, 2);
        assert_eq!(result.state.value(&cell), Some("final"));
    }

    /// Two clients offline at the same time, editing the same cell, both
    /// reconnect against the same remote op set and exchange their replays.
    /// They must end byte-identical regardless of which replay arrives first.
    #[test]
    fn concurrent_reconnects_converge() {
        let cell = CellKey::new(0, 2, 3);
        let mut base = CellState::new();
        base.apply(op("server", 5, cell, "base"));
        let remote = vec![op("server", 5, cell, "base")];

        let mut alice_q = OfflineQueue::new("alice");
        alice_q.observe(5);
        alice_q.edit(cell, Some("alice".to_string()));
        let mut bob_q = OfflineQueue::new("bob");
        bob_q.observe(5);
        bob_q.edit(cell, Some("bob".to_string()));

        let alice = super::reconcile(&base, &alice_q, &remote);
        let bob = super::reconcile(&base, &bob_q, &remote);

        let mut final_alice = alice.state.clone();
        for replay in &bob.replay {
            final_alice.apply(replay.clone());
        }
        let mut final_bob = bob.state.clone();
        for replay in &alice.replay {
            final_bob.apply(replay.clone());
        }

        assert_eq!(final_alice, final_bob);
        assert_eq!(final_alice.value(&cell), Some("bob"));
    }

    /// Rebase must also outrank high-clock ops already present in the snapshot
    /// (not just ops missed during the disconnect), otherwise an offline edit
    /// silently loses when the client went offline with a stale clock.
    #[test]
    fn rebase_outranks_high_clock_in_snapshot() {
        let cell = CellKey::new(0, 0, 0);
        let mut base = CellState::new();
        base.apply(op("server", 50, cell, "server value"));

        let mut queue = OfflineQueue::new("alice");
        queue.edit(cell, Some("offline intent".to_string()));

        let result = super::reconcile(&base, &queue, &[]);
        assert_eq!(result.state.value(&cell), Some("offline intent"));
        assert!(result.replay[0].clock > 50);
    }

    #[test]
    fn replay_is_idempotent() {
        let cell = CellKey::new(0, 0, 0);
        let mut queue = OfflineQueue::new("alice");
        queue.edit(cell, Some("value".to_string()));
        let result = super::reconcile(&CellState::new(), &queue, &[]);

        let mut applied_twice = result.state.clone();
        for replay in &result.replay {
            applied_twice.apply(replay.clone());
        }

        assert_eq!(applied_twice, result.state);
    }
}
