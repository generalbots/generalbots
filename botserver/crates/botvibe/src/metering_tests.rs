//! #2069 VIB-2 — the metering window parse and the cap decision. Both used to
//! live inside functions that need a PostgreSQL pool, so neither was asserted.
use super::*;

#[test]
fn vib2_a_missing_window_falls_back_to_thirty_days() {
    assert_eq!(window_seconds_from(None), DEFAULT_WINDOW_SECONDS);
    assert_eq!(DEFAULT_WINDOW_SECONDS, 30 * 24 * 3600);
}

#[test]
fn vib2_a_positive_window_is_honoured() {
    assert_eq!(window_seconds_from(Some("60")), 60);
    assert_eq!(window_seconds_from(Some(" 90 ")), 90);
    assert_eq!(window_seconds_from(Some("2592000")), 2_592_000);
}

#[test]
fn vib2_a_misconfigured_window_falls_back_instead_of_disabling_metering() {
    // A zero or negative window puts `period_start` in the future, which made
    // every usage query return nothing and every cap pass silently.
    for raw in ["", " ", "abc", "0", "-1", "-2592000", "1.5", "1e9", "9999999999999999999999"] {
        assert_eq!(
            window_seconds_from(Some(raw)),
            DEFAULT_WINDOW_SECONDS,
            "{raw:?} must fall back"
        );
    }
}

#[test]
fn vib2_a_missing_or_non_positive_cap_allows_everything() {
    let allowed = |cap: f64| cap_decision(MeterPlan::Free, MeterKind::VmHours, 1_000.0, cap);

    for cap in [0.0, -1.0, -1e9] {
        let result = allowed(cap).expect("no cap means allowed");
        assert!(result.allowed);
        assert_eq!(result.metered, "vm_hours");
    }
}

#[test]
fn vib2_a_nan_cap_is_treated_as_no_cap() {
    // Documented: NaN compares false against everything, so it behaves like
    // "no limit". Only a corrupted limit row can produce it.
    assert!(cap_decision(MeterPlan::Shared, MeterKind::EgressGb, 10.0, f64::NAN).is_ok());
}

#[test]
fn vib2_usage_below_the_cap_is_allowed() {
    let result = cap_decision(MeterPlan::Shared, MeterKind::BuildMinutes, 9.99, 10.0)
        .expect("under the cap");
    assert!(result.allowed);
    assert_eq!(result.metered, "build_minutes");
}

#[test]
fn vib2_usage_at_or_above_the_cap_is_refused() {
    for used in [10.0, 10.01, 1e9] {
        let err = cap_decision(MeterPlan::Shared, MeterKind::StorageGbHours, used, 10.0)
            .expect_err("at or over the cap");
        assert!(err.contains("storage_gb_hours"), "{err}");
        assert!(err.contains("shared plan"), "{err}");
        assert!(err.contains("10.00"), "{err}");
        assert!(err.contains(&format!("{used:.2}")), "{err}");
    }
}

#[test]
fn vib2_the_refusal_names_the_meter_and_the_plan_of_every_plan() {
    for (plan, label) in [
        (MeterPlan::Free, "free plan"),
        (MeterPlan::Shared, "shared plan"),
        (MeterPlan::PrivateCloud, "private-cloud plan"),
    ] {
        let err = cap_decision(plan, MeterKind::SnapshotCount, 5.0, 5.0).expect_err("refused");
        assert!(err.contains(label), "{err}");
        assert!(err.contains("snapshot_count"), "{err}");
    }
}
