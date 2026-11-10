//! Observability configuration and budget arithmetic: the config map is parsed
//! once at boot (a bad value must fall back, never abort), and the budget
//! checks decide whether a request is still allowed.
use super::*;

fn config_map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
        .collect()
}

fn budget(daily_limit: f64, daily_spend: f64, monthly_limit: f64, monthly_spend: f64) -> Budget {
    Budget {
        daily_limit,
        monthly_limit,
        alert_threshold: 0.8,
        daily_spend,
        monthly_spend,
        ..Default::default()
    }
}

#[test]
fn an_empty_map_yields_the_documented_defaults() {
    let manager = ObservabilityManager::from_config(&HashMap::new());

    assert!(manager.config.enabled);
    assert_eq!(manager.config.metrics_interval, 60);
    assert!(manager.config.cost_tracking);
    assert!(manager.config.trace_enabled);
    assert_eq!(manager.config.trace_sample_rate, 1.0);
    assert_eq!(manager.config.budget_daily, 100.0);
    assert_eq!(manager.config.budget_monthly, 2000.0);
    assert_eq!(manager.config.alert_threshold, 0.8);
}

#[test]
fn explicit_values_override_the_defaults() {
    let manager = ObservabilityManager::from_config(&config_map(&[
        ("observability-enabled", "false"),
        ("observability-metrics-interval", "30"),
        ("observability-cost-tracking", "false"),
        ("observability-trace-enabled", "false"),
        ("observability-trace-sample-rate", "0.25"),
        ("observability-budget-daily", "12.5"),
        ("observability-budget-monthly", "500"),
        ("observability-alert-threshold", "0.5"),
    ]));

    assert!(!manager.config.enabled);
    assert_eq!(manager.config.metrics_interval, 30);
    assert!(!manager.config.cost_tracking);
    assert!(!manager.config.trace_enabled);
    assert_eq!(manager.config.trace_sample_rate, 0.25);
    assert_eq!(manager.config.budget_daily, 12.5);
    assert_eq!(manager.config.budget_monthly, 500.0);
    assert_eq!(manager.config.alert_threshold, 0.5);
}

#[test]
fn an_unparsable_value_falls_back_instead_of_failing() {
    let manager = ObservabilityManager::from_config(&config_map(&[
        ("observability-metrics-interval", "often"),
        ("observability-budget-daily", "cheap"),
    ]));

    assert_eq!(manager.config.metrics_interval, 60);
    assert_eq!(manager.config.budget_daily, 100.0);
}

#[test]
fn cost_is_computed_per_thousand_tokens() {
    let manager = ObservabilityManager::from_config(&HashMap::new());

    // gpt-4: 0.03 per 1k input, 0.06 per 1k output.
    let cost = manager.calculate_cost("gpt-4", 1000, 1000);
    assert!((cost - 0.09).abs() < 1e-9, "cost was {cost}");
}

#[test]
fn a_local_model_is_free() {
    let manager = ObservabilityManager::from_config(&HashMap::new());

    assert_eq!(manager.calculate_cost("local", 1_000_000, 1_000_000), 0.0);
}

#[test]
fn an_unpriced_model_costs_nothing() {
    let manager = ObservabilityManager::from_config(&HashMap::new());

    assert_eq!(manager.calculate_cost("unknown-model", 1000, 1000), 0.0);
}

#[test]
fn budget_status_reports_percentages_and_remaining_amounts() {
    let status = ObservabilityManager::calculate_budget_status(&budget(100.0, 85.0, 2000.0, 100.0));

    assert_eq!(status.daily_remaining, 15.0);
    assert_eq!(status.daily_percentage, 85.0);
    assert_eq!(status.monthly_remaining, 1900.0);
    assert_eq!(status.monthly_percentage, 5.0);
    assert!(!status.daily_exceeded);
    assert!(!status.monthly_exceeded);
    assert!(status.near_daily_limit, "85% of the daily limit is near");
    assert!(!status.near_monthly_limit);
}

#[test]
fn budget_status_marks_a_spend_past_the_limit_as_exceeded() {
    let status =
        ObservabilityManager::calculate_budget_status(&budget(100.0, 101.0, 2000.0, 2001.0));

    assert!(status.daily_exceeded);
    assert!(status.monthly_exceeded);
    assert!(status.daily_remaining < 0.0);
}

#[test]
fn a_request_below_the_threshold_is_allowed() {
    let result = ObservabilityManager::check_budget_internal(&budget(100.0, 10.0, 2000.0, 100.0), 5.0);

    assert_eq!(result, BudgetCheckResult::Ok);
}

#[test]
fn a_request_that_would_cross_the_daily_limit_is_rejected() {
    let result =
        ObservabilityManager::check_budget_internal(&budget(100.0, 99.0, 2000.0, 100.0), 2.0);

    assert_eq!(result, BudgetCheckResult::DailyExceeded);
}

#[test]
fn a_request_that_would_cross_the_monthly_limit_is_rejected() {
    let result =
        ObservabilityManager::check_budget_internal(&budget(100.0, 0.0, 2000.0, 1999.0), 2.0);

    assert_eq!(result, BudgetCheckResult::MonthlyExceeded);
}

#[test]
fn a_request_near_the_daily_limit_warns_before_it_is_blocked() {
    let result =
        ObservabilityManager::check_budget_internal(&budget(100.0, 75.0, 2000.0, 100.0), 6.0);

    assert_eq!(result, BudgetCheckResult::NearDailyLimit);
}

#[test]
fn a_request_near_the_monthly_limit_warns_when_the_day_is_fine() {
    let result =
        ObservabilityManager::check_budget_internal(&budget(100.0, 10.0, 2000.0, 1590.0), 10.0);

    assert_eq!(result, BudgetCheckResult::NearMonthlyLimit);
}
