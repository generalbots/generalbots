//! Offline tests for the sales forecast arithmetic (issue #2075, CRM wave).
//!
//! `ForecastService` is a pure calculator: it takes already-loaded deal rows and
//! returns forecast structs. `build_monthly_forecast` stamps a fresh `id` and
//! `created_at`, so the assertions cover the deterministic fields only.

use super::*;

fn monthly(year: i32, month: u32, predicted_value: f64, deal_count: i32) -> MonthlyInput {
    MonthlyInput {
        year,
        month,
        predicted_value,
        deal_count,
    }
}

fn deal(value: f64, stage_probability: f64) -> DealInput {
    DealInput {
        id: Uuid::new_v4(),
        title: "Deal".to_string(),
        value,
        stage: "proposal".to_string(),
        stage_probability,
        expected_close: None,
    }
}

#[test]
fn historical_trends_average_the_published_win_rates() {
    let history = vec![
        HistoricalTrend {
            period: "2026-Q1".to_string(),
            total_value: 100.0,
            won_value: 40.0,
            lost_value: 60.0,
            win_rate: 0.4,
            deal_count: 10,
        },
        HistoricalTrend {
            period: "2026-Q2".to_string(),
            total_value: 200.0,
            won_value: 120.0,
            lost_value: 80.0,
            win_rate: 0.6,
            deal_count: 12,
        },
    ];

    let rate = ForecastService::analyze_historical_trends(&history);
    assert!((rate - 0.5).abs() < 1e-9, "rate {rate}");
}

#[test]
fn an_empty_history_assumes_a_coin_flip_win_rate() {
    assert_eq!(ForecastService::analyze_historical_trends(&[]), 0.5);
}

#[test]
fn a_single_period_history_is_returned_unchanged() {
    let history = vec![HistoricalTrend {
        period: "2026-Q3".to_string(),
        total_value: 0.0,
        won_value: 0.0,
        lost_value: 0.0,
        win_rate: 0.75,
        deal_count: 4,
    }];

    assert!((ForecastService::analyze_historical_trends(&history) - 0.75).abs() < 1e-9);
}

#[test]
fn weighted_pipeline_is_the_probability_adjusted_total() {
    let stages = [(1_000.0, 0.5), (2_000.0, 0.25), (500.0, 1.0)];

    let (predicted, weighted) = ForecastService::weighted_pipeline_calculation(&stages);
    assert!((predicted - 3_500.0).abs() < 1e-9, "predicted {predicted}");
    assert!((weighted - 1_500.0).abs() < 1e-9, "weighted {weighted}");
}

#[test]
fn an_empty_pipeline_forecasts_nothing() {
    assert_eq!(
        ForecastService::weighted_pipeline_calculation(&[]),
        (0.0, 0.0)
    );
}

#[test]
fn monthly_forecast_weights_revenue_by_the_historical_win_rate() {
    let forecast = ForecastService::build_monthly_forecast(
        Uuid::nil(),
        &[monthly(2026, 1, 10_000.0, 6)],
        0.4,
    );

    assert_eq!(forecast.monthly_breakdown.len(), 1);
    let january = &forecast.monthly_breakdown[0];
    assert_eq!(january.year, 2026);
    assert_eq!(january.month, 1);
    assert_eq!(january.predicted_revenue, 10_000.0);
    assert!((january.weighted_revenue - 4_000.0).abs() < 1e-9);
    assert_eq!(january.deal_count, 6);
    assert!((forecast.overall_predicted - 10_000.0).abs() < 1e-9);
    assert!((forecast.overall_weighted - 4_000.0).abs() < 1e-9);
}

#[test]
fn monthly_forecast_is_sorted_by_period() {
    let forecast = ForecastService::build_monthly_forecast(
        Uuid::nil(),
        &[
            monthly(2026, 3, 300.0, 1),
            monthly(2025, 12, 100.0, 1),
            monthly(2026, 1, 200.0, 1),
        ],
        0.5,
    );

    let periods: Vec<(i32, u32)> = forecast
        .monthly_breakdown
        .iter()
        .map(|m| (m.year, m.month))
        .collect();
    assert_eq!(periods, vec![(2025, 12), (2026, 1), (2026, 3)]);
}

#[test]
fn confidence_scales_with_the_deal_count_and_caps_at_published_ceilings() {
    // > 5 deals: the historical win rate, capped at 95 %.
    let busy = ForecastService::build_monthly_forecast(
        Uuid::nil(),
        &[monthly(2026, 1, 0.0, 6)],
        1.0,
    );
    assert!((busy.monthly_breakdown[0].confidence - 95.0).abs() < 1e-9);

    // 1..=5 deals: the capped win rate reduced by 30 %, i.e. at most 56 %.
    let quiet = ForecastService::build_monthly_forecast(
        Uuid::nil(),
        &[monthly(2026, 1, 0.0, 2)],
        1.0,
    );
    assert!((quiet.monthly_breakdown[0].confidence - 56.0).abs() < 1e-9);

    // No deals: a fixed low confidence.
    let empty = ForecastService::build_monthly_forecast(
        Uuid::nil(),
        &[monthly(2026, 1, 0.0, 0)],
        1.0,
    );
    assert!((empty.monthly_breakdown[0].confidence - 10.0).abs() < 1e-9);
}

#[test]
fn overall_confidence_of_an_empty_pipeline_is_zero() {
    let forecast = ForecastService::build_monthly_forecast(Uuid::nil(), &[], 0.5);

    assert!(forecast.monthly_breakdown.is_empty());
    assert_eq!(forecast.overall_predicted, 0.0);
    assert_eq!(forecast.overall_weighted, 0.0);
    assert_eq!(forecast.overall_confidence, 0.0);
}

#[test]
fn ai_prediction_requires_at_least_one_deal() {
    let error = ForecastService::ai_prediction_stub(&[]).expect_err("empty input is rejected");
    assert_eq!(error, "No deals provided for AI prediction");
}

#[test]
fn ai_prediction_reports_the_share_of_high_probability_deals() {
    let deals = vec![deal(1_000.0, 0.9), deal(3_000.0, 0.8), deal(2_000.0, 0.1)];

    let prediction = ForecastService::ai_prediction_stub(&deals).expect("prediction");
    assert!((prediction.predicted_close_rate - 2.0 / 3.0).abs() < 1e-9);
    assert!((prediction.confidence_score - (2.0 / 3.0) * 100.0).abs() < 1e-9);
    assert!((prediction.estimated_total - 4_000.0).abs() < 1e-6);
    assert_eq!(
        prediction.suggested_focus,
        "Pipeline health appears strong; focus on accelerating existing deals"
    );
}

#[test]
fn a_minority_of_hot_deals_suggests_nurturing() {
    let cases: Vec<Vec<DealInput>> = vec![
        // 1 of 1: every deal is cold.
        vec![deal(1_000.0, 0.1)],
        // 1 of 3: a third is close to closing.
        vec![deal(1_000.0, 0.9), deal(1_000.0, 0.1), deal(1_000.0, 0.1)],
        // 0 of 4.
        vec![
            deal(1_000.0, 0.1),
            deal(1_000.0, 0.1),
            deal(1_000.0, 0.1),
            deal(1_000.0, 0.1),
        ],
    ];

    for deals in cases {
        let prediction = ForecastService::ai_prediction_stub(&deals).expect("prediction");
        assert_eq!(
            prediction.suggested_focus,
            "Nurture high-probability deals to improve close rate",
            "{} deal(s)",
            deals.len()
        );
    }
}

#[test]
fn half_of_the_pipeline_at_high_probability_is_reported_as_strong() {
    let deals = vec![deal(1_000.0, 0.9), deal(1_000.0, 0.1)];

    let prediction = ForecastService::ai_prediction_stub(&deals).expect("prediction");
    assert_eq!(
        prediction.suggested_focus,
        "Pipeline health appears strong; focus on accelerating existing deals"
    );
}
