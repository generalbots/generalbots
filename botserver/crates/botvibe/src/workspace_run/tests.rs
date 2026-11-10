//! `workspace_run::tests` — budget + trace behaviour shared by the pipeline.

use super::*;

fn fast_budget() -> WorkspaceBudget {
    // 1 ms of program time: any real await outlives it.
    WorkspaceBudget::new(1, 0)
}

#[tokio::test]
async fn a_step_inside_the_budget_completes_and_keeps_its_rows() {
    let mut trace = TraceRecorder::new("run-ok", WorkspaceBudget::new(5_000, 0));
    trace.begin("install deps", "npm ci");
    let _ = trace.finish("install deps", TraceStatus::Ok, "142 packages");

    let outcome = trace.run(async { Ok::<u32, String>(7) }).await;
    match outcome {
        BudgetOutcome::Completed { value, .. } => assert_eq!(value, 7),
        other => panic!("expected completion, got {other:?}"),
    }
    assert!(!outcome.timed_out());
    let table = trace.table();
    assert_eq!(table.rows.len(), 1);
    assert!(!table.timed_out);
}

#[tokio::test]
async fn an_error_keeps_the_partial_table() {
    let mut trace = TraceRecorder::new("run-err", WorkspaceBudget::new(5_000, 0));
    trace.begin("build", "vite build");
    let outcome = trace
        .run(async { Err::<(), String>("tsc failed".to_string()) })
        .await;
    match outcome {
        BudgetOutcome::Failed { error, .. } => assert_eq!(error, "tsc failed"),
        other => panic!("expected failure, got {other:?}"),
    }
    let table = trace.table();
    assert_eq!(table.rows.len(), 1, "the in-flight row is preserved");
    assert!(table.rows[0].status.is_open());
}

#[tokio::test]
async fn the_budget_cut_off_keeps_completed_rows_and_closes_the_in_flight_one() {
    let mut trace = TraceRecorder::new("run-timeout", fast_budget());
    trace.begin("install deps", "npm ci");
    let _ = trace.finish("install deps", TraceStatus::Ok, "142 packages");
    trace.begin("build", "vite build");
    trace.artifact("dist/app.js");

    let outcome = trace
        .run(async {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            Ok::<(), String>(())
        })
        .await;
    assert!(
        outcome.timed_out(),
        "the 1 ms budget must cut the sleep off"
    );

    let table = trace.table();
    assert!(table.timed_out);
    assert_eq!(table.rows.len(), 2);
    assert_eq!(table.rows[0].status, TraceStatus::Ok);
    assert_eq!(table.rows[1].status, TraceStatus::TimedOut);
    assert!(table.rows[1].duration_ms.is_some());
    assert_eq!(table.in_flight, None);
    assert_eq!(table.artifacts, vec!["dist/app.js".to_string()]);
    assert!(table.remaining_ms == 0);

    let message = trace.budget().timeout_message("build", &table.artifacts);
    assert!(message.contains("build"));
    assert!(message.contains("dist/app.js"));
}

#[test]
fn a_trace_line_round_trips_through_the_marker_contract() {
    let marker = parse_trace_line(r#"@@GB_TRACE@@ {"task":"build","detail":"x","status":"error"}"#)
        .expect("marker");
    assert_eq!(marker.status, TraceStatus::Error);
    assert_eq!(marker.status.as_str(), "error");
    assert_eq!(TRACE_PREFIX, "@@GB_TRACE@@");
}
