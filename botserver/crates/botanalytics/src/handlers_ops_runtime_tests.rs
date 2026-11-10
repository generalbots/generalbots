use super::*;

#[test]
fn endpoints_panel_without_counters_explains_why() {
    let html = render_endpoints(&[], 60);
    assert!(html.contains("No request counters recorded"));
    assert!(!html.contains("ops-share-fill"));
}

#[test]
fn endpoints_panel_computes_rate_error_share_and_p95() {
    let rows = vec![
        EndpointRow {
            endpoint_class: "ui".to_string(),
            requests: 300,
            errors: 3,
            b1: 100,
            b2: 100,
            b3: 90,
            b4: 10,
            b5: 0,
            b6: 0,
            b7: 0,
            b8: 0,
        },
        EndpointRow {
            endpoint_class: "api".to_string(),
            requests: 100,
            errors: 0,
            b1: 100,
            b2: 0,
            b3: 0,
            b4: 0,
            b5: 0,
            b6: 0,
            b7: 0,
            b8: 0,
        },
    ];
    let html = render_endpoints(&rows, 60);
    assert!(html.contains("5.0/min"));
    assert!(html.contains("1.00%"));
    assert!(html.contains("ops-warn"));
    assert!(html.contains("75%"));
}

#[test]
fn queue_panel_without_tables_reports_the_failure() {
    let html = render_queue(&[], 60);
    assert!(html.contains("No queue table answered"));
}

#[test]
fn queue_panel_escalates_depth_and_formats_wait() {
    let rows = vec![
        QueueRow {
            name: "agent runs",
            queued: 250,
            running: Some(3),
            failed: Some(1),
            wait_p95: Some(120.0),
        },
        QueueRow {
            name: "handoff",
            queued: 2,
            running: None,
            failed: None,
            wait_p95: Some(4.0),
        },
    ];
    let html = render_queue(&rows, 60);
    assert!(html.contains("2.0 min"));
    assert!(html.contains("4 s"));
    assert!(html.contains("ops-page"));
    assert!(html.contains("runbook-disk-and-queue.md"));
}

#[test]
fn cost_panel_never_invents_a_per_conversation_price() {
    let data = CostData {
        totals: CostTotalsRow {
            requests: 0,
            input_tokens: 0,
            output_tokens: 0,
            conversations: 0,
            failures: 0,
            cost: 0.0,
        },
        tenants: Vec::new(),
        quotas: Vec::new(),
    };
    let html = render_cost(&data, 60);
    assert!(html.contains("Cost / conversation"));
    assert!(html.contains("—"));
    assert!(html.contains("No tenant spend recorded"));
    assert!(html.contains("No LLM budget rows configured"));
}

#[test]
fn cost_panel_divides_cost_by_conversations() {
    let data = CostData {
        totals: CostTotalsRow {
            requests: 10,
            input_tokens: 1_000,
            output_tokens: 500,
            conversations: 4,
            failures: 1,
            cost: 0.4,
        },
        tenants: vec![TenantCostRow {
            tenant: "acme".to_string(),
            tokens: 1_500,
            cost: 0.4,
        }],
        quotas: vec![QuotaRow {
            bot: "support".to_string(),
            daily_spend: 9.0,
            daily_limit: 10.0,
        }],
    };
    let html = render_cost(&data, 60);
    assert!(html.contains("$0.1000"));
    assert!(html.contains("acme"));
    assert!(html.contains("90%"));
    assert!(html.contains("ops-warn"));
}
