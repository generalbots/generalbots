use super::*;

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
