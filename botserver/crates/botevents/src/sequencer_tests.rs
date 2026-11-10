//! Sequencer edge cases: the first minted value, resumption and forgetting.

use super::{SequencerRegistry, ThreadSequencer};

#[test]
fn the_first_minted_value_is_one_even_when_asked_for_zero() {
    let sequencer = ThreadSequencer::starting_at(0);
    assert_eq!(sequencer.last_seq(), 0);
    assert_eq!(sequencer.next_seq(), 1);
    assert_eq!(sequencer.last_seq(), 1);
}

#[test]
fn a_sequencer_can_start_above_one() {
    let sequencer = ThreadSequencer::starting_at(10);
    assert_eq!(sequencer.last_seq(), 9);
    assert_eq!(sequencer.next_seq(), 10);
    assert_eq!(sequencer.next_seq(), 11);
}

#[test]
fn advance_to_never_rewinds() {
    let sequencer = ThreadSequencer::starting_at(1);
    assert_eq!(sequencer.next_seq(), 1);
    sequencer.advance_to(5);
    assert_eq!(sequencer.next_seq(), 6);
    sequencer.advance_to(2);
    assert_eq!(sequencer.next_seq(), 7);
    sequencer.advance_to(u64::MAX);
    assert_eq!(sequencer.last_seq(), u64::MAX - 1);
    assert_eq!(sequencer.next_seq(), u64::MAX);
}

#[test]
fn the_registry_reuses_one_sequencer_per_thread() {
    let registry = SequencerRegistry::default();
    let first = registry.sequencer("a", 1);
    let second = registry.sequencer("a", 50);
    assert_eq!(first.next_seq(), 1);
    assert_eq!(second.next_seq(), 2);
    assert_eq!(registry.sequencer("b", 1).next_seq(), 1);
}

#[test]
fn forget_drops_the_counter_so_the_log_rebuilds_it() {
    let registry = SequencerRegistry::new();
    assert_eq!(registry.sequencer("a", 1).next_seq(), 1);
    assert_eq!(registry.sequencer("a", 1).next_seq(), 2);
    registry.forget("a");
    assert_eq!(registry.sequencer("a", 1).next_seq(), 1);
    // Forgetting an unknown thread is a no-op.
    registry.forget("missing");
}

#[test]
fn a_poisoned_registry_lock_still_serves_sequences() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::Arc;

    let registry = Arc::new(SequencerRegistry::new());
    let poison = Arc::clone(&registry);
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let _guard = poison.threads.lock().expect("lock");
        panic!("poison the registry lock");
    }));
    assert_eq!(registry.sequencer("a", 1).next_seq(), 1);
    registry.forget("a");
    assert_eq!(registry.sequencer("a", 1).next_seq(), 1);
}
