//! Offline tests for the smart-router model selection (issue #2075, AI wave).
//!
//! `SmartLLMRouter::select_optimal_model` holds a cache behind an `RwLock` and
//! needs an `AppState`; the decision itself is the pure [`select_model`] below,
//! which the router delegates to. Every case is table-driven over published
//! model performance figures.

use super::*;

fn performance(name: &str, latency_ms: u64, cost: f64, success_rate: f64) -> ModelPerformance {
    ModelPerformance {
        model_name: name.to_string(),
        avg_latency_ms: latency_ms,
        avg_cost_per_token: cost,
        success_rate,
        total_requests: 10,
        last_updated: chrono::Utc::now(),
    }
}

fn sample_models() -> Vec<ModelPerformance> {
    vec![
        performance("fast-expensive", 100, 0.02, 0.99),
        performance("slow-cheap", 2_000, 0.0001, 0.95),
        performance("balanced", 500, 0.002, 0.98),
    ]
}

#[test]
fn goal_names_are_parsed_case_insensitively_with_a_balanced_default() {
    assert!(matches!(
        OptimizationGoal::from_str_name("speed"),
        OptimizationGoal::Speed
    ));
    assert!(matches!(
        OptimizationGoal::from_str_name("SPEED"),
        OptimizationGoal::Speed
    ));
    assert!(matches!(
        OptimizationGoal::from_str_name("Cost"),
        OptimizationGoal::Cost
    ));
    assert!(matches!(
        OptimizationGoal::from_str_name("quality"),
        OptimizationGoal::Quality
    ));
    for unknown in ["", "balanced", "cheapest", "unknown"] {
        assert!(
            matches!(
                OptimizationGoal::from_str_name(unknown),
                OptimizationGoal::Balanced
            ),
            "{unknown}"
        );
    }
}

#[test]
fn speed_goal_picks_the_lowest_latency() {
    assert_eq!(
        select_model(&sample_models(), OptimizationGoal::Speed, None, None).as_deref(),
        Some("fast-expensive")
    );
}

#[test]
fn cost_goal_picks_the_cheapest_model() {
    assert_eq!(
        select_model(&sample_models(), OptimizationGoal::Cost, None, None).as_deref(),
        Some("slow-cheap")
    );
}

#[test]
fn quality_goal_picks_the_highest_success_rate() {
    assert_eq!(
        select_model(&sample_models(), OptimizationGoal::Quality, None, None).as_deref(),
        Some("fast-expensive")
    );
}

#[test]
fn constraints_are_applied_before_the_goal() {
    let models = vec![
        performance("slow-reliable", 2_000, 0.0001, 0.99),
        performance("mid", 500, 0.001, 0.97),
        performance("fast-pricy", 100, 0.02, 0.90),
    ];

    assert_eq!(
        select_model(&models, OptimizationGoal::Speed, Some(0.01), None).as_deref(),
        Some("mid"),
        "a cost ceiling excludes the fastest model"
    );
    assert_eq!(
        select_model(&models, OptimizationGoal::Quality, None, Some(1_000)).as_deref(),
        Some("mid"),
        "a latency ceiling excludes the most reliable model"
    );
    assert_eq!(
        select_model(&models, OptimizationGoal::Speed, Some(0.01), Some(1_000)).as_deref(),
        Some("mid"),
        "both ceilings apply together"
    );
}

#[test]
fn an_empty_or_fully_filtered_candidate_set_falls_back_to_the_default_model() {
    assert_eq!(
        select_model(&[], OptimizationGoal::Speed, None, None).as_deref(),
        Some(DEFAULT_MODEL)
    );
    assert_eq!(
        select_model(&sample_models(), OptimizationGoal::Speed, Some(0.0), None).as_deref(),
        Some(DEFAULT_MODEL)
    );
    assert_eq!(
        select_model(&sample_models(), OptimizationGoal::Speed, None, Some(0)).as_deref(),
        Some(DEFAULT_MODEL)
    );
}

#[test]
fn balanced_goal_prefers_a_model_that_dominates_on_every_axis() {
    let models = vec![
        performance("dominated", 2_000, 0.02, 0.90),
        performance("dominant", 500, 0.001, 0.95),
    ];

    assert_eq!(
        select_model(&models, OptimizationGoal::Balanced, None, None).as_deref(),
        Some("dominant")
    );
}

#[test]
fn a_zero_latency_sample_produces_a_finite_balanced_score() {
    // `track_performance` can record a 0 ms sample (an instant cache hit);
    // `1000.0 / 0.0` would be +inf and pin the model as the permanent winner.
    let unmeasured = performance("unmeasured", 0, 0.02, 0.50);
    let score = balanced_score(&unmeasured);

    assert!(score.is_finite(), "balanced score must stay finite: {score}");
    assert!(score > 0.0);
}

#[test]
fn balanced_goal_is_order_independent_for_ties() {
    let models = vec![
        performance("a", 500, 0.001, 0.95),
        performance("b", 500, 0.001, 0.95),
    ];

    let forward = select_model(&models, OptimizationGoal::Balanced, None, None);
    let mut reversed = models;
    reversed.reverse();
    let backward = select_model(&reversed, OptimizationGoal::Balanced, None, None);

    assert!(forward.is_some());
    assert!(backward.is_some());
    assert!(
        matches!(forward.as_deref(), Some("a") | Some("b")),
        "a tie must still resolve to one of the tied models: {forward:?}"
    );
}
