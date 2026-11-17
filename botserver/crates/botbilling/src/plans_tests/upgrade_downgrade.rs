use super::*;

#[test]
fn upgrade_and_downgrade_follow_the_plan_order() {
    let manager = manager();
    assert!(manager.can_upgrade("free", "shared"));
    assert!(manager.can_upgrade("shared", "private-cloud"));
    assert!(manager.can_upgrade("free", "private-cloud"));
    assert!(!manager.can_upgrade("shared", "free"));
    assert!(!manager.can_upgrade("shared", "shared"));

    assert!(manager.can_downgrade("private-cloud", "free"));
    assert!(!manager.can_downgrade("free", "shared"));
    assert!(!manager.can_downgrade("free", "free"));

    // A plan outside the ordering is never an upgrade or a downgrade.
    assert!(!manager.can_upgrade("free", "enterprise"));
    assert!(!manager.can_upgrade("enterprise", "free"));
    assert!(!manager.can_downgrade("enterprise", "free"));
}

#[test]
fn upgrade_options_are_the_strictly_higher_plans() {
    let manager = manager();
    let mut from_free: Vec<&str> = manager
        .get_upgrade_options("free")
        .iter()
        .map(|(id, _)| id.as_str())
        .collect();
    from_free.sort_unstable();
    assert_eq!(from_free, ["private-cloud", "shared"]);

    assert!(manager.get_upgrade_options("private-cloud").is_empty());
    assert!(manager.get_upgrade_options("enterprise").is_empty());
}

#[test]
fn public_plans_are_the_three_self_service_tiers() {
    let manager = manager();
    let mut public: Vec<&str> = manager
        .list_public_plans()
        .iter()
        .map(|(id, _)| id.as_str())
        .collect();
    public.sort_unstable();
    assert_eq!(public, ["free", "private-cloud", "shared"]);
    assert_eq!(manager.list_plans().len(), 4);
}

#[test]
fn downgrade_is_allowed_when_usage_fits() {
    let validation = manager().validate_downgrade("shared", &usage(3, 10, 1024, 2));
    assert!(validation.allowed);
    assert!(validation.blockers.is_empty());
}

#[test]
fn downgrade_to_an_unknown_plan_is_blocked() {
    let validation = manager().validate_downgrade("ghost", &usage(0, 0, 0, 0));
    assert!(!validation.allowed);
    assert!(matches!(
        validation.blockers.as_slice(),
        [DowngradeBlocker::PlanNotFound]
    ));
    assert_eq!(
        DowngradeBlocker::PlanNotFound.to_string(),
        "Target plan not found"
    );
}
