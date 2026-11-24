//! Deterministic merge model for concurrent sheet cell edits (#2447).
//!
//! The sheet WebSocket currently applies edits with transport-level
//! last-write-wins: whichever message happens to arrive last overwrites the
//! cell, so two clients that disconnect and reconnect can silently lose one
//! author's edit, and the survivor depends on delivery order. This module
//! models every cell as an independent **LWW-Register CRDT**: each edit is
//! tagged with a Lamport clock and a stable actor id, and the winner is chosen
//! by the total order `(clock, actor, op_id)`.
//!
//! That order makes `CellState::apply` commutative, associative and
//! idempotent — a join-semilattice — so replicating the same set of ops in any
//! order (including duplicates and offline replay) converges byte-identically.
//! Cell *values* are the unit of convergence here; structural conflicts (row
//! deletion, merged ranges) and text runs belong to the wider epic and are not
//! modelled by this slice.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Stable identity of a cell in a worksheet. Using an explicit key instead of
/// the `"row,col"` strings used by the transport avoids parsing and makes the
/// merge state independent of serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CellKey {
    pub worksheet: usize,
    pub row: u32,
    pub col: u32,
}

impl CellKey {
    pub fn new(worksheet: usize, row: u32, col: u32) -> Self {
        Self {
            worksheet,
            row,
            col,
        }
    }
}

/// A single cell edit. `id` is unique per op (`actor:clock`), `actor` breaks
/// clock ties deterministically and `value: None` represents clearing a cell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellOp {
    pub id: String,
    pub actor: String,
    pub clock: u64,
    pub cell: CellKey,
    pub value: Option<String>,
}

impl CellOp {
    pub fn new(actor: &str, clock: u64, cell: CellKey, value: Option<String>) -> Self {
        Self {
            id: format!("{actor}:{clock}"),
            actor: actor.to_string(),
            clock,
            cell,
            value,
        }
    }
}

/// Total order of the register. `&str` tuples compare lexicographically, so
/// the result is deterministic on every client without wall-clock timestamps.
fn rank(op: &CellOp) -> (u64, &str, &str) {
    (op.clock, op.actor.as_str(), op.id.as_str())
}

fn dominates(a: &CellOp, b: &CellOp) -> bool {
    rank(a) > rank(b)
}

/// Monotonic logical clock. `observe` folds in a remote clock and `tick`
/// produces the next local value; together they preserve causality without a
/// shared time source (which cannot be assumed for offline clients).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LamportClock {
    time: u64,
}

impl LamportClock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn now(&self) -> u64 {
        self.time
    }

    pub fn observe(&mut self, remote: u64) {
        self.time = self.time.max(remote);
    }

    pub fn tick(&mut self) -> u64 {
        self.time += 1;
        self.time
    }
}

/// Converged per-cell state: the winning op for each cell seen so far.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CellState {
    winners: HashMap<CellKey, CellOp>,
}

impl CellState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge `op`. Returns `true` when it became the new winner (a state
    /// change worth broadcasting); `false` for a stale or duplicate op.
    pub fn apply(&mut self, op: CellOp) -> bool {
        match self.winners.get(&op.cell) {
            Some(current) if !dominates(&op, current) => false,
            _ => {
                self.winners.insert(op.cell, op);
                true
            }
        }
    }

    pub fn op(&self, cell: &CellKey) -> Option<&CellOp> {
        self.winners.get(cell)
    }

    pub fn value(&self, cell: &CellKey) -> Option<&str> {
        self.winners.get(cell).and_then(|op| op.value.as_deref())
    }

    /// Highest clock in the state, used to seed a reconnecting client's clock.
    pub fn max_clock(&self) -> u64 {
        self.winners.values().map(|op| op.clock).max().unwrap_or(0)
    }

    pub fn len(&self) -> usize {
        self.winners.len()
    }

    pub fn is_empty(&self) -> bool {
        self.winners.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{CellKey, CellOp, CellState};

    fn op(actor: &str, clock: u64, cell: CellKey, value: &str) -> CellOp {
        CellOp::new(actor, clock, cell, Some(value.to_string()))
    }

    /// Regression: two authors edit the same cell at the same Lamport time
    /// (a genuine concurrency race). Delivery order must not decide the value.
    #[test]
    fn two_concurrent_edits_converge_regardless_of_delivery_order() {
        let cell = CellKey::new(0, 4, 2);
        let alice = op("alice", 7, cell, "alice's paragraph");
        let bob = op("bob", 7, cell, "bob's paragraph");

        let mut first = CellState::new();
        first.apply(alice.clone());
        first.apply(bob.clone());

        let mut second = CellState::new();
        second.apply(bob);
        second.apply(alice);

        assert_eq!(first, second);
        // Tie on clock is broken by actor id, so "bob" wins on both clients.
        assert_eq!(first.value(&cell), Some("bob's paragraph"));
    }

    #[test]
    fn newer_clock_wins_even_when_delivered_last() {
        let cell = CellKey::new(0, 0, 0);
        let mut state = CellState::new();
        assert!(state.apply(op("alice", 9, cell, "new")));
        assert!(!state.apply(op("bob", 3, cell, "stale")));
        assert_eq!(state.value(&cell), Some("new"));
    }

    /// Small deterministic PRNG (xorshift64) so convergence can be exercised
    /// without pulling a rand dependency into the pure merge module.
    fn next_rand(state: &mut u64) -> u64 {
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        *state = x;
        x
    }

    /// 4k randomized concurrent ops across 8 cells: two replicas apply the
    /// same multiset in different orders (one with duplicates) and must end in
    /// byte-identical states. This is the offline slice of the 10k-op harness.
    #[test]
    fn randomized_concurrent_ops_converge() {
        let actors = ["alice", "bob", "carol", "dave"];
        let mut seed = 0x9E37_79B9_7F4A_7C15u64;
        let mut ops = Vec::with_capacity(4096);
        for _ in 0..4096 {
            let a = actors[(next_rand(&mut seed) as usize) % actors.len()];
            let clock = next_rand(&mut seed) % 200;
            let cell = CellKey::new(0, (next_rand(&mut seed) % 8) as u32, 0);
            let v = format!("v{}", next_rand(&mut seed));
            ops.push(op(a, clock, cell, &v));
        }

        let mut forward = CellState::new();
        for edit in &ops {
            forward.apply(edit.clone());
        }

        let mut reverse = CellState::new();
        for edit in ops.iter().rev() {
            reverse.apply(edit.clone());
            // Duplicate delivery must be a no-op.
            reverse.apply(edit.clone());
        }

        assert_eq!(forward, reverse);
        assert!(forward.len() <= 8);
    }
}
