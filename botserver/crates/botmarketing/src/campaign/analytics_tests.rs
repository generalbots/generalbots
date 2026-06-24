use super::*;
use crate::campaign::models::{Campaign, CampaignMetrics, ChannelMetrics, Schedule};

fn create_test_campaign() -> Campaign {
    let schedule = Schedule {
        start_date: None,
        end_date: None,
        frequency: "daily".to_string(),
        time_slots: vec!["09:00".to_string()],
    };

    let mut campaign = Campaign::new("test-1".to_string(), "Test Campaign".to_string(), schedule);

    let mut metrics = CampaignMetrics::default();
    metrics.update_channel_metrics(
        "instagram".to_string(),
        ChannelMetrics {
            impressions: 10000,
            clicks: 500,
            conversions: 50,
            spend: 1000.0,
            revenue: Some(5000.0),
        },
    );
    metrics.update_channel_metrics(
        "email".to_string(),
        ChannelMetrics {
            impressions: 5000,
            clicks: 300,
            conversions: 30,
            spend: 500.0,
            revenue: Some(2000.0),
        },
    );

    campaign.metrics = metrics;
    campaign
}

#[test]
fn test_generate_report() {
    let campaign = create_test_campaign();
    let report = generate_report(&campaign, "2025-01-01", "2025-01-31");
    assert_eq!(report.campaign_id, "test-1");
    assert_eq!(report.summary.total_impressions, 15000);
    assert_eq!(report.summary.total_clicks, 800);
    assert_eq!(report.summary.total_conversions, 80);
    assert!(report.summary.total_spend > 0.0);
    assert!(report.roi_analysis.is_profitable);
    assert!(!report.recommendations.is_empty());
}

#[test]
fn test_compare_channels() {
    let campaign = create_test_campaign();
    let comparison = compare_channels(&campaign.metrics);
    assert!(comparison.is_some());
    let comp = comparison.unwrap();
    assert_eq!(comp.best_performing, "instagram");
    assert!(!comp.channel_rankings.is_empty());
}

#[test]
fn test_calculate_roi_profitable() {
    let mut metrics = CampaignMetrics::default();
    metrics.update_channel_metrics(
        "instagram".to_string(),
        ChannelMetrics {
            impressions: 1000,
            clicks: 100,
            conversions: 10,
            spend: 500.0,
            revenue: Some(2000.0),
        },
    );
    let roi = calculate_roi(&metrics);
    assert!(roi.is_profitable);
    assert!(roi.roi_pct > 0.0);
    assert!((roi.roi_pct - 300.0).abs() < 0.01);
}

#[test]
fn test_calculate_roi_unprofitable() {
    let mut metrics = CampaignMetrics::default();
    metrics.update_channel_metrics(
        "instagram".to_string(),
        ChannelMetrics {
            impressions: 1000,
            clicks: 100,
            conversions: 10,
            spend: 500.0,
            revenue: Some(100.0),
        },
    );
    let roi = calculate_roi(&metrics);
    assert!(!roi.is_profitable);
    assert!(roi.roi_pct < 0.0);
}

#[test]
fn test_merge_metrics() {
    let mut base = CampaignMetrics::default();
    let incoming = CampaignMetrics::default();
    merge_campaign_metrics(&mut base, &incoming);
    assert_eq!(base.impressions, 0);
}

#[test]
fn test_daily_metrics_count() {
    let campaign = create_test_campaign();
    let report = generate_report(&campaign, "2025-01-01", "2025-01-07");
    assert_eq!(report.daily_metrics.len(), 7);
}

#[test]
fn test_csv_export() {
    let campaign = create_test_campaign();
    let csv = export_csv(&[campaign]);
    assert!(csv.starts_with("campaign_id,name,"));
    assert!(csv.contains("test-1"));
}

#[test]
fn test_trend_analysis_empty() {
    let trend = calculate_trend(&[]);
    assert!(trend.impression_trend.is_empty());
    assert_eq!(trend.growth_rate, 0.0);
}

#[test]
fn test_aggregate_cross_channel() {
    let campaign = create_test_campaign();
    let agg = aggregate_cross_channel(&campaign);
    assert!(agg.contains_key("total_impressions"));
    assert!(agg.contains_key("channels"));
}

#[test]
fn test_compare_channels_empty() {
    let metrics = CampaignMetrics::default();
    let comparison = compare_channels(&metrics);
    assert!(comparison.is_none());
}

#[test]
fn test_summary_conversion_rate() {
    let campaign = create_test_campaign();
    let report = generate_report(&campaign, "2025-01-01", "2025-01-31");
    let expected_rate = (80.0 / 15000.0) * 100.0;
    assert!((report.summary.conversion_rate - expected_rate).abs() < 0.01);
}

#[test]
fn test_csv_escape() {
    assert_eq!(escape_csv("hello"), "hello");
    assert_eq!(escape_csv("hello,world"), "\"hello,world\"");
    assert_eq!(escape_csv("hello\"world"), "\"hello\"\"world\"");
}

#[test]
fn test_recommendations_generated() {
    let campaign = create_test_campaign();
    let report = generate_report(&campaign, "2025-01-01", "2025-01-31");
    assert!(!report.recommendations.is_empty());
}
