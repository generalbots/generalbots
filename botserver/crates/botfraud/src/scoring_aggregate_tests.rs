//! Table-driven tests for the score accumulator that feeds the classifier.
//!
//! Weights: each triggered rule contributes `rule_score(severity)`, a
//! blocklist hit is absolute (the score becomes 100), a velocity hit adds 30,
//! and the result is clamped to 0..=100 before classification.

use super::*;

#[test]
fn rule_weights_are_summed() {
    let cases = [
        (vec![], 0),
        (vec![5], 5),
        (vec![40, 25], 65),
        (vec![15, 15, 15], 45),
        (vec![40, 40, 40], 100),
        (vec![40, 40, 40, 40], 100),
    ];
    for (scores, expected) in cases {
        assert_eq!(accumulate(&scores, false, false), expected, "scores {scores:?}");
    }
}

#[test]
fn a_velocity_hit_adds_thirty_and_is_clamped() {
    assert_eq!(accumulate(&[], false, true), VELOCITY_WEIGHT);
    assert_eq!(accumulate(&[25], false, true), 55);
    assert_eq!(accumulate(&[40, 25], false, true), 95);
    // 40 + 25 + 30 = 95; one more critical rule saturates instead of wrapping.
    assert_eq!(accumulate(&[40, 25, 40], false, true), 100);
}

#[test]
fn a_blocklist_hit_overrides_every_rule_weight() {
    assert_eq!(accumulate(&[], true, false), 100);
    assert_eq!(accumulate(&[5], true, false), 100);
    assert_eq!(accumulate(&[40, 25, 15], true, false), 100);
    // Velocity cannot push a blocked case past the ceiling either.
    assert_eq!(accumulate(&[], true, true), 100);
}

#[test]
fn the_accumulated_score_classifies_at_the_published_thresholds() {
    let cases = [
        (0, "low", "allow"),
        (29, "low", "allow"),
        (30, "medium", "flag"),
        (59, "medium", "flag"),
        (60, "high", "review"),
        (79, "high", "review"),
        (80, "critical", "block"),
        (100, "critical", "block"),
    ];
    for (score, level, action) in cases {
        assert_eq!(classify(score, false), (level.to_string(), action.to_string()));
    }
}

#[test]
fn a_single_critical_rule_alone_does_not_block() {
    // 40 points from a critical rule is "medium/flag"; blocking needs 80 or a
    // blocklist hit. This pins the difference between severity and action.
    let score = accumulate(&[rule_score("critical")], false, false);
    assert_eq!(classify(score, false), ("medium".to_string(), "flag".to_string()));
}

#[test]
fn two_critical_rules_reach_the_block_threshold() {
    let score = accumulate(&[rule_score("critical"), rule_score("critical")], false, false);
    assert_eq!(score, 80);
    assert_eq!(classify(score, false), ("critical".to_string(), "block".to_string()));
}
