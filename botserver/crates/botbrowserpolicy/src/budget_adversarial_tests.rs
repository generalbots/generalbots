//! Budget-guard arithmetic tests (#2066).
//!
//! The tracker is the last line of defence against a runaway browsing task.
//! These tests cover the boundary (`cap`, `cap + 1`) and the arithmetic that
//! used to wrap: a saturated counter must fail closed, not reset.

use super::*;
use crate::policy::PolicyConfig;

fn caps(max_steps: u32, max_cost_milli: u64) -> BudgetCaps {
    BudgetCaps {
        max_steps,
        max_cost_milli,
    }
}

#[test]
fn a_charge_exactly_at_the_cap_is_accepted_and_the_next_is_not() {
    let t = BudgetTracker::new(caps(2, 100));
    assert_eq!(t.charge_step(), Ok(1));
    assert_eq!(t.charge_step(), Ok(2));
    assert_eq!(
        t.charge_step(),
        Err(BudgetExceeded::Steps { used: 3, max: 2 })
    );

    assert_eq!(t.charge_cost(60), Ok(60));
    assert_eq!(t.charge_cost(40), Ok(100));
    assert_eq!(
        t.charge_cost(1),
        Err(BudgetExceeded::Cost {
            used_milli: 101,
            max_milli: 100
        })
    );
}

#[test]
fn a_zero_cost_charge_is_accepted_and_changes_nothing() {
    let t = BudgetTracker::new(caps(1, 0));
    assert_eq!(t.charge_cost(0), Ok(0));
    assert_eq!(t.charge_cost(0), Ok(0));
    assert_eq!(t.snapshot().cost_milli_used, 0);
    assert_eq!(t.charge_cost(1), Err(BudgetExceeded::Cost { used_milli: 1, max_milli: 0 }));
}

#[test]
fn a_huge_cost_delta_cannot_wrap_the_counter_known_bypass() {
    // Regression: `fetch_add` wrapped at u64::MAX, so charging u64::MAX
    // reported an error but reset the counter to zero, and the next charge
    // was accepted. The tracker must stay over budget instead.
    let t = BudgetTracker::new(caps(10, 500));
    assert!(t.charge_cost(u64::MAX).is_err());
    assert!(
        t.charge_cost(1).is_err(),
        "the budget must not be reset by an overflowing delta"
    );
    assert!(t.charge_cost(0).is_err(), "saturated usage stays over cap");
    assert_eq!(t.snapshot().cost_milli_used, u64::MAX);
    assert_eq!(t.snapshot().cost_milli_remaining, 0);
}

#[test]
fn a_huge_step_count_cannot_wrap_the_counter() {
    let t = BudgetTracker::with_usage(caps(3, 0), u32::MAX - 1, 0);
    assert!(t.charge_step().is_err());
    assert!(t.charge_step().is_err());
    assert_eq!(t.snapshot().steps_used, u32::MAX);
    assert_eq!(t.snapshot().steps_remaining, 0);
}

#[test]
fn rehydrated_usage_above_the_cap_is_already_exhausted() {
    let t = BudgetTracker::with_usage(caps(2, 100), 9, 999);
    let snapshot = t.snapshot();
    assert_eq!(snapshot.steps_used, 9);
    assert_eq!(snapshot.cost_milli_used, 999);
    assert_eq!(snapshot.steps_remaining, 0);
    assert_eq!(snapshot.cost_milli_remaining, 0);
    assert!(t.charge_step().is_err());
    assert!(t.charge_cost(1).is_err());
}

#[test]
fn a_zero_cap_denies_the_first_charge() {
    let t = BudgetTracker::new(caps(0, 0));
    assert_eq!(
        t.charge_step(),
        Err(BudgetExceeded::Steps { used: 1, max: 0 })
    );
}

#[test]
fn the_snapshot_reports_remaining_after_partial_use() {
    let t = BudgetTracker::new(caps(10, 1000));
    t.charge_step().expect("step");
    t.charge_cost(250).expect("cost");
    let snapshot = t.snapshot();
    assert_eq!(snapshot.steps_used, 1);
    assert_eq!(snapshot.steps_remaining, 9);
    assert_eq!(snapshot.cost_milli_used, 250);
    assert_eq!(snapshot.cost_milli_remaining, 750);
}

#[test]
fn the_exceeded_errors_describe_the_overrun() {
    assert_eq!(
        BudgetExceeded::Steps { used: 4, max: 3 }.to_string(),
        "step budget exceeded: 4/3"
    );
    assert_eq!(
        BudgetExceeded::Cost {
            used_milli: 600,
            max_milli: 500
        }
        .to_string(),
        "cost budget exceeded: 600/500 milli-units"
    );
}

#[test]
fn caps_are_derived_verbatim_from_a_custom_policy() {
    let policy = PolicyConfig {
        max_steps: 12,
        max_cost_units: 345,
        ..PolicyConfig::default()
    };
    let derived = BudgetCaps::from_policy(&policy);
    assert_eq!(derived, caps(12, 345));
    let t = BudgetTracker::new(derived);
    assert_eq!(t.caps(), caps(12, 345));
}
