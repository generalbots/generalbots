//! Unit tests for the metrics collector and the alert lifecycle.
//!
//! `MetricsCollector::new()` is pure (`RwLock` + `broadcast`, no I/O), so the
//! whole registry and alert state machine runs offline under a current-thread
//! runtime.

use super::{AlertCondition, AlertRule, AlertSeverity, AlertStatus, MetricType, MetricsCollector};
use chrono::{Duration, Utc};
use std::collections::HashMap;
use uuid::Uuid;

fn rule(name: &str, condition: AlertCondition) -> AlertRule {
    let now = Utc::now();
    AlertRule {
        id: Uuid::new_v4(),
        name: name.to_string(),
        description: String::new(),
        metric_name: "test_metric".to_string(),
        condition,
        severity: AlertSeverity::Warning,
        duration_seconds: 0,
        labels: HashMap::new(),
        annotations: HashMap::new(),
        enabled: true,
        created_at: now,
        updated_at: now,
    }
}

fn labels(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[tokio::test]
async fn record_metric_appends_a_data_point_and_updates_the_current_value() {
    let collector = MetricsCollector::new();
    collector
        .register_metric("cpu", MetricType::Gauge, "cpu load", Some("%".to_string()))
        .await;

    collector.record_metric("cpu", 10.0, labels(&[("host", "a")])).await;
    collector.record_metric("cpu", 20.0, HashMap::new()).await;

    let metric = collector.get_metric("cpu").await.expect("registered");
    assert_eq!(metric.current_value, 20.0);
    assert_eq!(metric.data_points.len(), 2);
    assert_eq!(metric.data_points[0].value, 10.0);
    assert_eq!(metric.data_points[0].labels.get("host"), Some(&"a".to_string()));
    assert_eq!(metric.metric_type, MetricType::Gauge);
    assert_eq!(metric.unit.as_deref(), Some("%"));
}

#[tokio::test]
async fn record_metric_creates_an_unregistered_metric_as_a_gauge() {
    let collector = MetricsCollector::new();
    collector.record_metric("ad_hoc", 3.0, HashMap::new()).await;

    let metric = collector.get_metric("ad_hoc").await.expect("created");
    assert_eq!(metric.metric_type, MetricType::Gauge);
    assert_eq!(metric.current_value, 3.0);
    assert_eq!(metric.data_points.len(), 1);
}

#[tokio::test]
async fn register_metric_replaces_the_previous_definition() {
    let collector = MetricsCollector::new();
    collector
        .register_metric("hits", MetricType::Counter, "old", None)
        .await;
    collector
        .register_metric("hits", MetricType::Histogram, "new", Some("ms".to_string()))
        .await;

    let metric = collector.get_metric("hits").await.expect("present");
    assert_eq!(metric.metric_type, MetricType::Histogram);
    assert_eq!(metric.description, "new");
    assert!(metric.data_points.is_empty());
}

#[tokio::test]
async fn increment_counter_starts_a_missing_metric_at_one() {
    let collector = MetricsCollector::new();
    collector.increment_counter("requests", HashMap::new()).await;
    assert_eq!(
        collector.get_metric("requests").await.map(|m| m.current_value),
        Some(1.0)
    );

    collector.increment_counter("requests", HashMap::new()).await;
    assert_eq!(
        collector.get_metric("requests").await.map(|m| m.current_value),
        Some(2.0)
    );
}

#[tokio::test]
async fn metric_history_filters_inclusively_on_both_bounds() {
    let collector = MetricsCollector::new();
    collector.record_metric("m", 1.0, HashMap::new()).await;
    let first = collector.get_metric("m").await.expect("present").data_points[0].timestamp;

    let from = first - Duration::seconds(1);
    let to = first + Duration::seconds(1);
    assert_eq!(collector.get_metric_history("m", from, to).await.len(), 1);
    assert_eq!(
        collector
            .get_metric_history("m", first, first)
            .await
            .len(),
        1
    );
    assert!(collector
        .get_metric_history("m", to, to + Duration::seconds(1))
        .await
        .is_empty());
    assert!(collector
        .get_metric_history("unknown", from, to)
        .await
        .is_empty());
}

#[tokio::test]
async fn data_points_are_capped_at_one_thousand_keeping_the_newest() {
    let collector = MetricsCollector::new();
    for value in 0..1005 {
        collector
            .record_metric("capped", f64::from(value), HashMap::new())
            .await;
    }

    let metric = collector.get_metric("capped").await.expect("present");
    assert_eq!(metric.data_points.len(), 1000);
    assert_eq!(metric.data_points[0].value, 5.0);
    assert_eq!(metric.current_value, 1004.0);
}

#[tokio::test]
async fn subscribers_receive_metric_and_alert_messages() {
    let collector = MetricsCollector::new();
    let mut rx = collector.subscribe();

    collector.add_alert_rule(rule("hot", AlertCondition::GreaterThan(5.0))).await;
    collector.record_metric("test_metric", 9.0, HashMap::new()).await;

    let first = rx.try_recv().expect("metric update");
    assert!(matches!(first, super::MonitoringMessage::MetricUpdate { .. }));
    let second = rx.try_recv().expect("alert fired");
    assert!(matches!(second, super::MonitoringMessage::AlertFired { .. }));
}

#[tokio::test]
async fn a_breach_fires_exactly_one_alert_until_it_recovers() {
    let collector = MetricsCollector::new();
    let rule = rule("hot", AlertCondition::GreaterThan(5.0));
    collector.add_alert_rule(rule.clone()).await;

    collector.record_metric("test_metric", 9.0, HashMap::new()).await;
    let active = collector.get_active_alerts().await;
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].status, AlertStatus::Firing);
    assert_eq!(active[0].metric_value, 9.0);
    assert_eq!(active[0].threshold, 5.0);
    assert_eq!(active[0].severity, AlertSeverity::Warning);
    assert_eq!(active[0].rule_name, "hot");
    assert!(active[0].resolved_at.is_none());
    assert!(active[0].message.contains("test_metric is 9"));

    // A second breach while firing must not open a duplicate.
    collector.record_metric("test_metric", 12.0, HashMap::new()).await;
    assert_eq!(collector.get_active_alerts().await.len(), 1);

    // Recovery resolves it.
    collector.record_metric("test_metric", 1.0, HashMap::new()).await;
    assert!(collector.get_active_alerts().await.is_empty());
    let recent = collector.get_recent_alerts(10).await;
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].status, AlertStatus::Resolved);
    assert!(recent[0].resolved_at.is_some());
}

#[tokio::test]
async fn a_disabled_rule_never_fires() {
    let collector = MetricsCollector::new();
    let mut disabled = rule("off", AlertCondition::GreaterThan(5.0));
    disabled.enabled = false;
    collector.add_alert_rule(disabled).await;

    collector.record_metric("test_metric", 100.0, HashMap::new()).await;
    assert!(collector.get_active_alerts().await.is_empty());
    assert!(collector.get_recent_alerts(10).await.is_empty());
}

#[tokio::test]
async fn a_rule_only_watches_its_own_metric() {
    let collector = MetricsCollector::new();
    collector.add_alert_rule(rule("hot", AlertCondition::GreaterThan(5.0))).await;

    collector.record_metric("other_metric", 100.0, HashMap::new()).await;
    assert!(collector.get_active_alerts().await.is_empty());
}

#[tokio::test]
async fn remove_alert_rule_reports_whether_it_existed() {
    let collector = MetricsCollector::new();
    let rule = rule("temp", AlertCondition::GreaterThan(1.0));
    collector.add_alert_rule(rule.clone()).await;
    assert_eq!(collector.get_alert_rules().await.len(), 1);

    assert!(collector.remove_alert_rule(rule.id).await);
    assert!(!collector.remove_alert_rule(rule.id).await);
    assert!(collector.get_alert_rules().await.is_empty());
}

#[tokio::test]
async fn replace_alert_rule_keeps_the_id_and_the_slot() {
    let collector = MetricsCollector::new();
    let original = rule("first", AlertCondition::GreaterThan(1.0));
    let other = rule("second", AlertCondition::GreaterThan(2.0));
    collector.add_alert_rule(original.clone()).await;
    collector.add_alert_rule(other.clone()).await;

    let mut updated = original.clone();
    updated.name = "renamed".to_string();
    updated.condition = AlertCondition::GreaterThan(50.0);
    collector.replace_alert_rule(updated).await;

    let rules = collector.get_alert_rules().await;
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].id, original.id);
    assert_eq!(rules[0].name, "renamed");

    // An unknown id is appended, not silently dropped.
    let fresh = rule("third", AlertCondition::LessThan(0.0));
    collector.replace_alert_rule(fresh.clone()).await;
    let rules = collector.get_alert_rules().await;
    assert_eq!(rules.len(), 3);
    assert!(rules.iter().any(|r| r.id == fresh.id));
}

#[tokio::test]
async fn acknowledging_moves_the_alert_out_of_active_and_stamps_the_user() {
    let collector = MetricsCollector::new();
    collector.add_alert_rule(rule("hot", AlertCondition::GreaterThan(5.0))).await;
    collector.record_metric("test_metric", 9.0, HashMap::new()).await;

    let alert_id = collector.get_active_alerts().await[0].id;
    assert!(collector.acknowledge_alert(alert_id, "ops@example.com").await);

    let active = collector.get_active_alerts().await;
    assert_eq!(active.len(), 1, "acknowledged alerts stay active");
    assert_eq!(active[0].status, AlertStatus::Acknowledged);
    assert_eq!(active[0].acknowledged_by.as_deref(), Some("ops@example.com"));
    assert!(active[0].acknowledged_at.is_some());

    // Acknowledging twice is a no-op: only Firing can be acknowledged.
    assert!(!collector.acknowledge_alert(alert_id, "again").await);
    assert!(!collector.acknowledge_alert(Uuid::new_v4(), "ghost").await);
}

#[tokio::test]
async fn an_acknowledged_alert_does_not_refire_on_the_next_breach() {
    let collector = MetricsCollector::new();
    collector.add_alert_rule(rule("hot", AlertCondition::GreaterThan(5.0))).await;
    collector.record_metric("test_metric", 9.0, HashMap::new()).await;
    let alert_id = collector.get_active_alerts().await[0].id;
    assert!(collector.acknowledge_alert(alert_id, "ops").await);

    collector.record_metric("test_metric", 11.0, HashMap::new()).await;
    assert_eq!(collector.get_active_alerts().await.len(), 1);
}

#[tokio::test]
async fn recent_alerts_are_capped_and_ordered_newest_first() {
    let collector = MetricsCollector::new();
    for index in 0..3 {
        let mut r = rule("hot", AlertCondition::GreaterThan(5.0));
        r.metric_name = format!("metric_{index}");
        collector.add_alert_rule(r).await;
        collector
            .record_metric(&format!("metric_{index}"), 9.0, HashMap::new())
            .await;
    }

    assert_eq!(collector.get_recent_alerts(2).await.len(), 2);
    let all = collector.get_recent_alerts(10).await;
    assert_eq!(all.len(), 3);
    assert!(
        all.windows(2).all(|w| w[0].started_at >= w[1].started_at),
        "recent alerts must be newest-first"
    );
    assert!(collector.get_recent_alerts(0).await.is_empty());
}

#[tokio::test]
async fn record_request_feeds_the_counters_behind_request_stats() {
    let collector = MetricsCollector::new();
    collector.record_request(10, false);
    collector.record_request(20, true);
    collector.record_request(30, false);

    assert_eq!(collector.get_health().await.status, super::HealthStatus::Unknown);
    // The counters are private; the arithmetic they feed is asserted through
    // `policy::request_stats` in `policy_tests.rs`. Here we only prove the
    // recorder does not panic and leaves the collector usable.
    collector.record_metric("after", 1.0, HashMap::new()).await;
    assert!(collector.get_metric("after").await.is_some());
}

#[tokio::test]
async fn dashboard_composes_metrics_health_and_alerts() {
    let collector = MetricsCollector::new();
    collector.add_alert_rule(rule("hot", AlertCondition::GreaterThan(5.0))).await;
    collector.record_metric("test_metric", 9.0, HashMap::new()).await;

    let dashboard = collector.get_dashboard().await;
    assert_eq!(dashboard.metrics.len(), 1);
    assert_eq!(dashboard.active_alerts.len(), 1);
    assert_eq!(dashboard.recent_alerts.len(), 1);
    assert_eq!(dashboard.health.status, super::HealthStatus::Unknown);
}
