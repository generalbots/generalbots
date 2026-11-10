//! Tests for the wire spelling of a fraud decision.

use super::*;

#[test]
fn each_action_serializes_to_its_published_wire_value() {
    let cases = [
        (FraudAction::Allow, "allow"),
        (FraudAction::Flag, "flag"),
        (FraudAction::Review, "review"),
        (FraudAction::Block, "block"),
    ];
    for (action, wire) in cases {
        assert_eq!(action.as_str(), wire);
    }
}

#[test]
fn classify_outputs_match_the_action_wire_values() {
    // `scoring::classify` and the SQL filters in `handlers/stats.rs` compare
    // against these literals; a rename on either side must fail here.
    let cases = [
        (0, FraudAction::Allow),
        (30, FraudAction::Flag),
        (60, FraudAction::Review),
        (80, FraudAction::Block),
    ];
    for (score, action) in cases {
        let (_, wire) = crate::scoring::classify(score, false);
        assert_eq!(wire, action.as_str(), "score {score}");
    }
}
