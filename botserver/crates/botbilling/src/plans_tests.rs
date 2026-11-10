//! Offline unit tests for the plan manager (#2075).
//!
//! `PlanManager` is the whole billing decision surface that does not touch a
//! database: limit checks, upgrade/downgrade ordering, proration and plan
//! comparison. A wrong answer here charges a customer the wrong amount, so
//! every branch is pinned.

use super::*;
use crate::{BillingError, BillingPeriod, LimitValue, PlanConfig, PlanLimits, PlanPrice, UsageMetric};
use std::collections::HashMap;
use uuid::Uuid;

fn limits(bots: LimitValue, users: LimitValue, storage_mb: LimitValue, apps: LimitValue) -> PlanLimits {
    PlanLimits {
        messages_per_day: LimitValue::Limited(100),
        storage_mb,
        bots,
        users,
        api_calls_per_day: LimitValue::Limited(1000),
        signups_per_day: None,
        kb_documents: LimitValue::Limited(10),
        apps,
        emails_per_day: LimitValue::Limited(50),
        builds_per_day: LimitValue::Limited(5),
        uploads_per_hour: LimitValue::Limited(20),
    }
}

fn plan(name: &str, price: PlanPrice, limits: PlanLimits, features: &[&str]) -> PlanConfig {
    PlanConfig {
        name: name.to_string(),
        description: None,
        price,
        limits,
        features: features.iter().map(|f| f.to_string()).collect(),
        stripe_price_id: None,
        trial_days: None,
    }
}

fn free_limits() -> PlanLimits {
    limits(
        LimitValue::Limited(1),
        LimitValue::Limited(2),
        LimitValue::Limited(100),
        LimitValue::Limited(1),
    )
}

fn shared_limits() -> PlanLimits {
    limits(
        LimitValue::Limited(5),
        LimitValue::Limited(20),
        LimitValue::Limited(5000),
        LimitValue::Limited(10),
    )
}

fn private_limits() -> PlanLimits {
    limits(
        LimitValue::Unlimited,
        LimitValue::Unlimited,
        LimitValue::Unlimited,
        LimitValue::Unlimited,
    )
}

fn monthly(amount: u64) -> PlanPrice {
    PlanPrice::Fixed {
        amount,
        currency: "USD".to_string(),
        period: BillingPeriod::Monthly,
    }
}

fn manager() -> PlanManager {
    let mut plans = HashMap::new();
    plans.insert(
        "free".to_string(),
        plan("Free", PlanPrice::Free, free_limits(), &["chat"]),
    );
    plans.insert(
        "shared".to_string(),
        plan("Shared", monthly(399), shared_limits(), &["chat", "drive"]),
    );
    plans.insert(
        "private-cloud".to_string(),
        plan("Private", monthly(9999), private_limits(), &["chat", "drive", "sso"]),
    );
    plans.insert(
        "enterprise".to_string(),
        plan("Enterprise", PlanPrice::Custom, private_limits(), &[]),
    );
    PlanManager::new(plans)
}

fn usage(bots: u64, users: u64, storage_bytes: u64, apps: u64) -> OrganizationUsage {
    OrganizationUsage {
        organization_id: Uuid::nil(),
        bots,
        users,
        storage_bytes,
        apps,
        kb_documents: 0,
        messages_today: 0,
        api_calls_today: 0,
    }
}

#[test]
fn storage_limits_are_reported_in_bytes() {
    let manager = manager();
    assert_eq!(
        manager.get_limit("shared", UsageMetric::StorageBytes),
        Some(LimitValue::Limited(5000 * 1024 * 1024))
    );
    assert_eq!(
        manager.get_limit("private-cloud", UsageMetric::StorageBytes),
        Some(LimitValue::Unlimited)
    );
    assert_eq!(manager.get_limit("missing", UsageMetric::StorageBytes), None);
}

#[test]
fn unlimited_metrics_are_always_allowed() {
    let result = manager()
        .check_limit("private-cloud", UsageMetric::Bots, u64::MAX)
        .expect("known plan");
    assert!(result.is_allowed());
    assert!(!result.is_warning_threshold(0.0));
    match result {
        LimitCheckResult::Allowed {
            remaining,
            percentage_used,
        } => {
            assert_eq!(remaining, None);
            assert_eq!(percentage_used, 0.0);
        }
        LimitCheckResult::Exceeded { .. } => panic!("unlimited must not be exceeded"),
    }
}

#[test]
fn the_limit_is_exclusive_at_the_boundary() {
    let manager = manager();
    // One below the limit is allowed, exactly at the limit is exceeded.
    let allowed = manager
        .check_limit("shared", UsageMetric::Bots, 4)
        .expect("known plan");
    match allowed {
        LimitCheckResult::Allowed {
            remaining,
            percentage_used,
        } => {
            assert_eq!(remaining, Some(1));
            assert!((percentage_used - 80.0).abs() < 0.01);
        }
        LimitCheckResult::Exceeded { .. } => panic!("4/5 must be allowed"),
    }

    let exceeded = manager
        .check_limit("shared", UsageMetric::Bots, 5)
        .expect("known plan");
    assert!(matches!(exceeded, LimitCheckResult::Exceeded { limit: 5, current: 5 }));
    assert!(exceeded.is_warning_threshold(100.0));
    assert!(!exceeded.is_warning_threshold(101.0));
}

#[test]
fn an_unknown_plan_fails_the_check() {
    let err = manager()
        .check_limit("nope", UsageMetric::Messages, 1)
        .expect_err("unknown plan");
    assert!(matches!(err, BillingError::PlanNotFound(ref id) if id == "nope"));
}

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
fn proration_charges_the_difference_when_upgrading() {
    let manager = manager();
    // 399 -> 9999 over a 30 day period with 15 days left:
    // (9999/30*15) - (399/30*15) = 4999.5 - 199.5 = 4800 exactly.
    match manager
        .calculate_proration("shared", "private-cloud", 15, 30)
        .expect("both plans priced")
    {
        ProrationType::ChargeNow(amount) => assert_eq!(amount, 4800),
        other => panic!("expected a charge, got {other:?}"),
    }
}

#[test]
fn proration_rounds_up_a_fractional_charge_or_credit() {
    let manager = manager();
    // 399 -> 9999 with 1 day left: 333.3 - 13.3 = 320.0 exactly; use 7 days
    // over 30 to force a fraction: 2333.1 - 93.1 = 2240.0. 3 days: 999.9 - 39.9.
    // 1 day over a 7 day period: 1428.43 - 57.0 = 1371.43 -> ceil 1372.
    match manager
        .calculate_proration("shared", "private-cloud", 1, 7)
        .expect("both plans priced")
    {
        ProrationType::ChargeNow(amount) => assert_eq!(amount, 1372),
        other => panic!("expected a charge, got {other:?}"),
    }

    // Downgrading 9999 -> 399 for 1 of 7 days yields a credit.
    match manager
        .calculate_proration("private-cloud", "shared", 1, 7)
        .expect("both plans priced")
    {
        ProrationType::Credit(amount) => assert_eq!(amount, 1372),
        other => panic!("expected a credit, got {other:?}"),
    }
}

#[test]
fn proration_is_neutral_when_the_remaining_value_matches() {
    let manager = manager();
    assert!(matches!(
        manager.calculate_proration("shared", "shared", 10, 30),
        Some(ProrationType::NoChange)
    ));
    // Free -> free also costs nothing.
    assert!(matches!(
        manager.calculate_proration("free", "free", 10, 30),
        Some(ProrationType::NoChange)
    ));
}

#[test]
fn proration_is_undefined_for_custom_or_missing_plans() {
    let manager = manager();
    assert!(manager
        .calculate_proration("enterprise", "shared", 10, 30)
        .is_none());
    assert!(manager
        .calculate_proration("shared", "enterprise", 10, 30)
        .is_none());
    assert!(manager
        .calculate_proration("ghost", "shared", 10, 30)
        .is_none());
}

#[test]
fn downgrade_is_allowed_when_usage_fits() {
    let validation = manager().validate_downgrade("shared", &usage(3, 10, 1024, 2));
    assert!(validation.allowed);
    assert!(validation.blockers.is_empty());
}

#[test]
fn downgrade_is_blocked_by_every_exceeded_limit() {
    // shared: 5 bots, 20 users, 5000 MB, 10 apps.
    let validation = manager().validate_downgrade(
        "shared",
        &usage(6, 21, 5001 * 1024 * 1024, 11),
    );
    assert!(!validation.allowed);
    assert_eq!(validation.blockers.len(), 4);
    assert!(validation
        .blockers
        .iter()
        .any(|b| matches!(b, DowngradeBlocker::TooManyBots { current: 6, limit: 5 })));
    assert!(validation
        .blockers
        .iter()
        .any(|b| matches!(b, DowngradeBlocker::TooManyUsers { current: 21, limit: 20 })));
    assert!(validation.blockers.iter().any(|b| matches!(
        b,
        DowngradeBlocker::TooMuchStorage { current_mb: 5001, limit_mb: 5000 }
    )));
    assert!(validation
        .blockers
        .iter()
        .any(|b| matches!(b, DowngradeBlocker::TooManyApps { current: 11, limit: 10 })));
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

#[test]
fn downgrade_to_an_unlimited_plan_is_never_blocked() {
    let validation = manager().validate_downgrade(
        "private-cloud",
        &usage(u64::MAX, u64::MAX, u64::MAX, u64::MAX),
    );
    assert!(validation.allowed);
}

#[test]
fn comparison_reports_limit_direction_and_features() {
    let comparison = manager()
        .compare_plans("free", "shared")
        .expect("both plans exist");

    assert_eq!(comparison.from_plan, "free");
    assert_eq!(comparison.to_plan, "shared");
    assert_eq!(comparison.feature_changes.added, ["drive"]);
    assert!(comparison.feature_changes.removed.is_empty());
    assert!(matches!(
        comparison.price_change,
        PriceChange::Increase { from: 0, to: 399 }
    ));

    let bots = comparison
        .limit_changes
        .iter()
        .find(|change| change.metric == UsageMetric::Bots)
        .expect("bots are compared");
    assert_eq!(bots.direction, ChangeDirection::Increase);

    let storage = comparison
        .limit_changes
        .iter()
        .find(|change| change.metric == UsageMetric::StorageBytes)
        .expect("storage is compared");
    assert_eq!(storage.direction, ChangeDirection::Increase);
}

#[test]
fn comparison_detects_removed_features_and_unlimited_transitions() {
    let comparison = manager()
        .compare_plans("private-cloud", "shared")
        .expect("both plans exist");

    assert_eq!(comparison.feature_changes.removed, ["sso"]);
    assert!(comparison.feature_changes.added.is_empty());
    assert!(matches!(
        comparison.price_change,
        PriceChange::Decrease { from: 9999, to: 399 }
    ));

    let bots = comparison
        .limit_changes
        .iter()
        .find(|change| change.metric == UsageMetric::Bots)
        .expect("bots are compared");
    assert_eq!(bots.direction, ChangeDirection::Decrease);
    assert_eq!(bots.from, LimitValue::Unlimited);
    assert_eq!(bots.to, LimitValue::Limited(5));
}

#[test]
fn comparison_of_identical_plans_has_no_changes() {
    let comparison = manager()
        .compare_plans("shared", "shared")
        .expect("both plans exist");
    assert!(comparison
        .limit_changes
        .iter()
        .all(|change| change.direction == ChangeDirection::NoChange));
    assert!(matches!(comparison.price_change, PriceChange::NoChange));
    assert!(comparison.feature_changes.added.is_empty());
    assert!(comparison.feature_changes.removed.is_empty());
}

#[test]
fn comparison_to_or_from_a_custom_plan_is_flagged() {
    let to_custom = manager()
        .compare_plans("shared", "enterprise")
        .expect("both plans exist");
    assert!(matches!(to_custom.price_change, PriceChange::ToCustom));

    let from_custom = manager()
        .compare_plans("enterprise", "shared")
        .expect("both plans exist");
    assert!(matches!(from_custom.price_change, PriceChange::FromCustom));

    assert!(manager().compare_plans("shared", "ghost").is_none());
    assert!(manager().compare_plans("ghost", "shared").is_none());
}