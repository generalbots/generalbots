//! Offline tests for the risk-level labels of the AutoTask safety layer
//! (issue #2075).

use super::*;

#[test]
fn risk_levels_render_their_published_label() {
    let cases = [
        (RiskLevel::None, "None"),
        (RiskLevel::Low, "Low"),
        (RiskLevel::Medium, "Medium"),
        (RiskLevel::High, "High"),
        (RiskLevel::Critical, "Critical"),
    ];
    for (level, label) in cases {
        assert_eq!(level.to_string(), label);
    }
}

#[test]
fn risk_levels_are_ordered_by_severity() {
    assert!(RiskLevel::Low < RiskLevel::Medium);
    assert!(RiskLevel::Medium < RiskLevel::High);
    assert!(RiskLevel::High < RiskLevel::Critical);
    assert!(RiskLevel::None < RiskLevel::Low);
}

#[test]
fn the_default_risk_level_is_low() {
    assert_eq!(RiskLevel::default(), RiskLevel::Low);
}
