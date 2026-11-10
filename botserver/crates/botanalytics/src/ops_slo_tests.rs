use super::*;

const SAMPLE: &str = r#"
# comment line
version: 2
window_days: 30
owner: platform-oncall
objective.api_p95.metric: latency_histogram
objective.api_p95.threshold_ms: 500
objective.api_p95.target: 0.95
objective.api_p95.runbook: botbook/src/10-configuration-deployment/operations/runbook-disk-and-queue.md
objective.api_p95.panel: ops-endpoints
objective.error_rate.metric: error_rate
objective.error_rate.target: 0.99
objective.error_rate.runbook: botbook/src/10-configuration-deployment/operations/runbook-deploy-rollback.md
alert.fast_burn: 14.4
alert.medium_burn: 6
alert.slow_burn: 1
"#;

#[test]
fn parse_reads_objectives_and_alert_policy() {
    let config = SloConfig::parse(SAMPLE).expect("sample must parse");
    assert_eq!(config.version, 2);
    assert_eq!(config.owner, "platform-oncall");
    assert_eq!(config.objectives.len(), 2);
    assert!((config.alert.fast_burn - 14.4).abs() < f64::EPSILON);

    let api = config
        .objectives
        .iter()
        .find(|objective| objective.id == "api_p95")
        .expect("api_p95 present");
    assert_eq!(api.metric, MetricKind::LatencyHistogram);
    assert_eq!(api.threshold_ms, Some(500.0));
    assert!((api.budget_ratio() - 0.05).abs() < 1e-9);
}

#[test]
fn parse_skips_objectives_without_a_target() {
    let text = "objective.broken.metric: error_rate\nobjective.ok.metric: error_rate\nobjective.ok.target: 0.99\n";
    let config = SloConfig::parse(text).expect("one objective survives");
    assert_eq!(config.objectives.len(), 1);
    assert_eq!(config.objectives[0].id, "ok");
}

#[test]
fn parse_rejects_a_file_without_objectives() {
    assert!(SloConfig::parse("version: 1\nowner: nobody\n").is_none());
}

#[test]
fn defaults_cover_the_operational_slos() {
    let config = SloConfig::defaults();
    let ids: Vec<&str> = config.objectives.iter().map(|o| o.id.as_str()).collect();
    assert!(ids.contains(&"api_p95"));
    assert!(ids.contains(&"error_rate"));
    assert!(ids.contains(&"first_token_p95"));
    assert!(ids.contains(&"answer_p95"));
    assert!(ids.contains(&"ws_handshake"));
    assert!(config.source.contains("built-in"));
}

#[test]
fn burn_rate_uses_the_error_budget() {
    let config = SloConfig::defaults();
    let objective = config
        .objectives
        .iter()
        .find(|o| o.id == "error_rate")
        .expect("error_rate objective");
    // 1 % target budget, 2 % observed → burn rate 2.0.
    let burn = burn_rate(Some(0.02), objective).unwrap_or(-1.0);
    assert!((burn - 2.0).abs() < 1e-9);
    assert_eq!(burn_rate(None, objective), None);
}

#[test]
fn severity_follows_the_three_windows() {
    let alert = AlertPolicy::default();
    assert_eq!(severity(None, alert), BurnSeverity::NoData);
    assert_eq!(severity(Some(0.5), alert), BurnSeverity::Ok);
    assert_eq!(severity(Some(1.5), alert), BurnSeverity::Slow);
    assert_eq!(severity(Some(7.0), alert), BurnSeverity::Medium);
    assert_eq!(severity(Some(20.0), alert), BurnSeverity::Fast);
}

#[test]
fn burn_windows_are_the_documented_ones() {
    let labels: Vec<&str> = BURN_WINDOWS.iter().map(|(label, _)| *label).collect();
    assert_eq!(labels, vec!["1h", "6h", "3d"]);
}
