//! Per-user undo/redo stacks for sheet cells (#2447).
//!
//! There is no undo in the collaboration path today: an edit is broadcast and
//! applied to every session, so a global "undo" would have to guess whose op
//! to revert and would clobber other authors' text. This module keeps a
//! separate undo/redo stack per actor. Undo reverts only the caller's most
//! recent edit, and only while that edit is still the current value of the
//! cell — if another author has since overwritten it, undo is a no-op and
//! returns `false`. That is the "undo in one client never changes another
//! client's text" acceptance criterion for the cell model.

use super::merge::{CellKey, CellOp, CellState, LamportClock};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
struct EditRecord {
    cell: CellKey,
    undo_value: Option<String>,
    redo_value: Option<String>,
    /// Id of the op that currently produced the live value for this record.
    /// Guards against reverting a cell another author has since changed.
    marker_op_id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct UserStacks {
    undo: Vec<EditRecord>,
    redo: Vec<EditRecord>,
}

/// Independent undo/redo history for every actor, sharing one logical clock so
/// compensating ops are causally newer than the op they correct.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UndoManager {
    clock: LamportClock,
    stacks: HashMap<String, UserStacks>,
}

impl UndoManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a local edit and remember the inverse on the actor's undo stack.
    /// Clears that actor's redo stack, as a new branch of history invalidates
    /// previously undone ops for this actor only.
    pub fn apply_local_edit(
        &mut self,
        actor: &str,
        cell: CellKey,
        value: Option<String>,
        state: &mut CellState,
    ) -> CellOp {
        let before = state.value(&cell).map(str::to_string);
        let op = make_op(&mut self.clock, actor, cell, value, state);
        state.apply(op.clone());
        let record = EditRecord {
            cell,
            undo_value: before,
            redo_value: op.value.clone(),
            marker_op_id: op.id.clone(),
        };
        let stacks = self.stacks.entry(actor.to_string()).or_default();
        stacks.undo.push(record);
        stacks.redo.clear();
        op
    }

    /// Revert the caller's latest edit. Returns `false` when the caller has no
    /// history or another actor has overwritten the cell since that edit.
    pub fn undo(&mut self, actor: &str, state: &mut CellState) -> bool {
        let clock = &mut self.clock;
        let Some(stacks) = self.stacks.get_mut(actor) else {
            return false;
        };
        let Some(mut record) = stacks.undo.pop() else {
            return false;
        };
        if !can_apply(state, &record) {
            return false;
        }
        let op = make_op(clock, actor, record.cell, record.undo_value.clone(), state);
        state.apply(op.clone());
        record.marker_op_id = op.id;
        stacks.redo.push(record);
        true
    }

    /// Re-apply the caller's most recently undone edit, with the same
    /// overwrite guard as `undo`.
    pub fn redo(&mut self, actor: &str, state: &mut CellState) -> bool {
        let clock = &mut self.clock;
        let Some(stacks) = self.stacks.get_mut(actor) else {
            return false;
        };
        let Some(mut record) = stacks.redo.pop() else {
            return false;
        };
        if !can_apply(state, &record) {
            return false;
        }
        let op = make_op(clock, actor, record.cell, record.redo_value.clone(), state);
        state.apply(op.clone());
        record.marker_op_id = op.id;
        stacks.undo.push(record);
        true
    }
}

/// Only revert when our own op is still the winner on that cell.
fn can_apply(state: &CellState, record: &EditRecord) -> bool {
    state
        .op(&record.cell)
        .is_some_and(|winner| winner.id == record.marker_op_id)
}

/// Build a fresh compensating op whose clock outranks the cell's current
/// winner, so undo/redo are themselves ordinary mergeable cell ops.
fn make_op(
    clock: &mut LamportClock,
    actor: &str,
    cell: CellKey,
    value: Option<String>,
    state: &CellState,
) -> CellOp {
    if let Some(current) = state.op(&cell) {
        clock.observe(current.clock);
    }
    CellOp::new(actor, clock.tick(), cell, value)
}

#[cfg(test)]
mod tests {
    use super::UndoManager;
    use crate::collaboration::merge::{CellKey, CellOp, CellState};

    fn cell(row: u32) -> CellKey {
        CellKey::new(0, row, 0)
    }

    fn value(state: &CellState, c: CellKey) -> Option<&str> {
        state.value(&c)
    }

    #[test]
    fn undo_only_reverts_the_callers_edit() {
        let mut state = CellState::new();
        let mut manager = UndoManager::new();
        let alice_cell = cell(0);
        let bob_cell = cell(1);

        manager.apply_local_edit("alice", alice_cell, Some("alice".into()), &mut state);
        manager.apply_local_edit("bob", bob_cell, Some("bob".into()), &mut state);
        assert!(manager.undo("alice", &mut state));

        assert_eq!(value(&state, alice_cell), None);
        assert_eq!(value(&state, bob_cell), Some("bob"));
    }

    #[test]
    fn undo_never_clobbers_a_later_remote_edit() {
        let mut state = CellState::new();
        let mut manager = UndoManager::new();
        let target = cell(0);

        manager.apply_local_edit("alice", target, Some("alice".into()), &mut state);
        let remote = CellOp::new("bob", 100, target, Some("bob".into()));
        state.apply(remote);

        assert!(!manager.undo("alice", &mut state));
        assert_eq!(value(&state, target), Some("bob"));
    }

    #[test]
    fn redo_restores_after_undo() {
        let mut state = CellState::new();
        let mut manager = UndoManager::new();
        let target = cell(0);

        manager.apply_local_edit("alice", target, Some("alice".into()), &mut state);
        assert!(manager.undo("alice", &mut state));
        assert_eq!(value(&state, target), None);
        assert!(manager.redo("alice", &mut state));
        assert_eq!(value(&state, target), Some("alice"));
    }

    #[test]
    fn stacks_are_isolated_per_user() {
        let mut state = CellState::new();
        let mut manager = UndoManager::new();
        let alice_a = cell(0);
        let alice_b = cell(1);
        let bob_cell = cell(2);

        manager.apply_local_edit("alice", alice_a, Some("a1".into()), &mut state);
        manager.apply_local_edit("alice", alice_b, Some("a2".into()), &mut state);
        manager.apply_local_edit("bob", bob_cell, Some("b1".into()), &mut state);

        assert!(manager.undo("alice", &mut state));
        assert!(manager.undo("alice", &mut state));
        assert!(!manager.undo("alice", &mut state));
        assert_eq!(value(&state, alice_a), None);
        assert_eq!(value(&state, alice_b), None);
        assert_eq!(value(&state, bob_cell), Some("b1"));

        assert!(manager.undo("bob", &mut state));
        assert_eq!(value(&state, bob_cell), None);
    }
}
