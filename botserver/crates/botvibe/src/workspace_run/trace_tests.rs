//! `workspace_run::trace` tests — the task/time table contract.

use super::*;

fn recorder() -> TraceRecorder {
    TraceRecorder::new("run-1", WorkspaceBudget::new(60_000, 10_000))
}

#[test]
fn a_started_marker_opens_a_row_and_ok_closes_it() {
    let mut trace = recorder();
    let opened = trace
        .ingest_line(
            r#"@@GB_TRACE@@ {"task": "install deps", "detail": "npm ci", "status": "started"}"#,
        )
        .expect("started row");
    assert_eq!(opened.step, "install-deps");
    assert_eq!(opened.status, TraceStatus::Running);
    assert!(opened.duration_ms.is_none());
    assert_eq!(trace.in_flight().as_deref(), Some("install deps"));

    let closed = trace
        .ingest_line(r#"@@GB_TRACE@@ {"task": "install deps", "status": "ok"}"#)
        .expect("finished row");
    assert_eq!(closed.status, TraceStatus::Ok);
    assert!(closed.duration_ms.is_some());
    assert!(trace.in_flight().is_none());

    let table = trace.table();
    assert_eq!(table.rows.len(), 1, "start/finish must share one row");
    assert_eq!(table.rows[0].status, TraceStatus::Ok);
    assert_eq!(table.budget_ms, 70_000);
    assert!(!table.timed_out);
}

#[test]
fn ordinary_output_is_not_a_trace_row() {
    let mut trace = recorder();
    assert!(trace.ingest_line("added 42 packages").is_none());
    assert!(trace.ingest_line("@@GB_TRACE@@").is_none());
    assert!(trace.table().rows.is_empty());
}

#[test]
fn a_bare_marker_is_a_running_task() {
    let mut trace = recorder();
    let row = trace.ingest_line("@@GB_TRACE@@ building").expect("row");
    assert_eq!(row.task, "building");
    assert_eq!(row.status, TraceStatus::Running);
    assert_eq!(trace.in_flight().as_deref(), Some("building"));
}

#[test]
fn the_table_keeps_partial_progress_when_the_budget_cuts_the_run_off() {
    let mut trace = recorder();
    trace.begin("install deps", "npm ci");
    let _ = trace.finish("install deps", TraceStatus::Ok, "142 packages");
    trace.begin("build", "vite build");
    trace.artifact("dist/app.js");
    trace.artifact("dist/app.js");
    trace.artifact("logs/build.log");
    trace.mark_timed_out();
    trace.close_open(TraceStatus::TimedOut, "interrupted by the run budget");

    let table = trace.table();
    assert!(table.timed_out);
    assert_eq!(table.rows.len(), 2, "the completed step is kept");
    assert_eq!(table.rows[0].status, TraceStatus::Ok);
    assert_eq!(table.rows[1].status, TraceStatus::TimedOut);
    assert_eq!(table.in_flight, None, "no row may stay open");
    assert_eq!(table.artifacts, vec!["dist/app.js", "logs/build.log"]);

    let message = trace.budget().timeout_message("build", &table.artifacts);
    assert!(message.contains("build"));
    assert!(message.contains("dist/app.js"));
}

#[test]
fn the_emitted_frame_carries_task_time_status_and_budget() {
    let mut trace = recorder();
    trace.begin("build deck", "27 slides");
    let row = trace.table().rows[0].clone();
    let frame = trace.frame_json(&row);
    assert_eq!(frame["tool"], "workspace_run");
    assert_eq!(frame["step"], "build-deck");
    assert_eq!(frame["label"], "build deck");
    assert_eq!(frame["detail"], "27 slides");
    assert_eq!(frame["phase"], "running");
    assert_eq!(frame["budget_ms"], 70_000);
    assert!(frame["elapsed_ms"].as_u64().is_some());
    assert!(frame["remaining_ms"].as_u64().is_some());
}

#[test]
fn the_progress_event_counts_finished_rows() {
    let mut trace = recorder();
    trace.begin("one", "");
    let _ = trace.finish("one", TraceStatus::Ok, "");
    trace.begin("two", "");
    let event = trace.progress_event("build", "running 'two'");
    assert_eq!(event.event_type, "vibe_progress");
    assert_eq!(event.run_id, "run-1");
    assert_eq!(event.step, "build");
    assert_eq!(event.total_steps, 2);
    assert_eq!(event.current_step, 1);
    assert_eq!(event.progress, 50);
}

#[test]
fn step_ids_are_stable_and_readable() {
    assert_eq!(step_id_for("Build deck"), "build-deck");
    assert_eq!(step_id_for("install deps!"), "install-deps");
    assert_eq!(step_id_for("   "), "task");
    assert_eq!(step_id_for("Build deck"), step_id_for("build   deck"));
}
