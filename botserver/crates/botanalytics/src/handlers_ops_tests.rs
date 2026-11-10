use super::*;

fn objective(metric: MetricKind, target: f64, threshold_ms: Option<f64>) -> Objective {
    Objective {
        id: "test".to_string(),
        metric,
        target,
        threshold_ms,
        runbook: String::new(),
        panel: String::new(),
    }
}

#[test]
fn latency_ratio_uses_the_histogram_boundary() {
    let stats = WindowStats {
        requests: 100,
        buckets: [50, 0, 0, 40, 10, 0, 0, 0],
        ..Default::default()
    };
    // 10 of 100 requests are above the 500 ms boundary.
    let ratio = bad_ratio(
        &objective(MetricKind::LatencyHistogram, 0.95, Some(500.0)),
        &stats,
    )
    .unwrap_or(-1.0);
    assert!((ratio - 0.10).abs() < 1e-9);
}

#[test]
fn error_rate_ratio_needs_requests() {
    let empty = WindowStats::default();
    assert!(bad_ratio(&objective(MetricKind::ErrorRate, 0.99, None), &empty).is_none());

    let stats = WindowStats {
        requests: 200,
        errors: 2,
        ..Default::default()
    };
    let ratio = bad_ratio(&objective(MetricKind::ErrorRate, 0.99, None), &stats).unwrap_or(-1.0);
    assert!((ratio - 0.01).abs() < 1e-9);
}

#[test]
fn ws_and_llm_ratios_are_independent() {
    let stats = WindowStats {
        ws_requests: 10,
        ws_failures: 1,
        llm_requests: 4,
        llm_ttft_samples: 4,
        llm_ttft_slow: 1,
        llm_answer_slow: 2,
        ..Default::default()
    };
    assert!(
        (bad_ratio(&objective(MetricKind::WsHandshake, 0.99, None), &stats).unwrap_or(-1.0) - 0.1)
            .abs()
            < 1e-9
    );
    assert!(
        (bad_ratio(
            &objective(MetricKind::LlmFirstToken, 0.95, Some(2000.0)),
            &stats
        )
        .unwrap_or(-1.0)
            - 0.25)
            .abs()
            < 1e-9
    );
    assert!(
        (bad_ratio(
            &objective(MetricKind::LlmAnswer, 0.95, Some(15000.0)),
            &stats
        )
        .unwrap_or(-1.0)
            - 0.5)
            .abs()
            < 1e-9
    );
}

#[test]
fn observed_label_never_fabricates_a_value() {
    let empty = WindowStats::default();
    for metric in [
        MetricKind::LatencyHistogram,
        MetricKind::ErrorRate,
        MetricKind::WsHandshake,
        MetricKind::LlmFirstToken,
        MetricKind::LlmAnswer,
    ] {
        assert_eq!(
            observed_label(&objective(metric, 0.99, Some(500.0)), &empty),
            "no data"
        );
    }
}

#[test]
fn render_links_every_objective_to_its_runbook() {
    let config = SloConfig::defaults();
    let windows = vec![
        ("1h".to_string(), WindowStats::default()),
        ("6h".to_string(), WindowStats::default()),
        ("3d".to_string(), WindowStats::default()),
    ];
    let html = render(&config, &windows, &WindowStats::default());
    assert!(html.contains("ops-slo"));
    assert!(html.contains("runbook-deploy-rollback.md"));
    assert!(html.contains("runbook-llm-outage.md"));
    assert!(html.contains("no data"));
    assert!(!html.contains("NaN"));
}

#[test]
fn gate_only_objectives_are_listed_without_a_burn_rate() {
    let mut config = SloConfig::defaults();
    config.objectives.push(Objective {
        id: "cache_p95".to_string(),
        metric: MetricKind::LatencyHistogram,
        target: 0.95,
        threshold_ms: Some(50.0),
        runbook: "botbook/src/10-configuration-deployment/operations/runbook-disk-and-queue.md"
            .to_string(),
        panel: "ops-endpoints".to_string(),
    });
    let windows = vec![
        ("1h".to_string(), WindowStats::default()),
        ("6h".to_string(), WindowStats::default()),
        ("3d".to_string(), WindowStats::default()),
    ];
    let html = render(&config, &windows, &WindowStats::default());
    assert!(html.contains("measured by the CI SLO gate"));
    assert!(html.contains("cache_p95"));
    assert!(html.contains("scripts/check_slo.py"));
}
