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
