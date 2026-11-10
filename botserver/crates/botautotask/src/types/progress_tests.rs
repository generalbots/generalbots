//! Offline tests for the progress-event builders (issue #2075). These structs
//! feed the AutoTask UI, so the percentage maths must stay in 0..=100.

use super::*;

#[test]
fn a_new_event_starts_at_zero_progress() {
    let event = TaskProgressEvent::new("task-1", "init", "starting");

    assert_eq!(event.event_type, "task_progress");
    assert_eq!(event.task_id, "task-1");
    assert_eq!(event.step, "init");
    assert_eq!(event.message, "starting");
    assert_eq!(event.progress, 0);
    assert_eq!(event.total_steps, 0);
    assert_eq!(event.current_step, 0);
    assert!(event.details.is_none());
    assert!(event.error.is_none());
    assert!(!event.timestamp.is_empty());
}

#[test]
fn progress_is_a_truncated_percentage_of_the_total() {
    let cases = [
        (1u8, 3u8, 33u8),
        (2, 3, 66),
        (3, 3, 100),
        (1, 4, 25),
        (0, 5, 0),
    ];
    for (current, total, expected) in cases {
        let event = TaskProgressEvent::new("t", "s", "m").with_progress(current, total);
        assert_eq!(event.progress, expected, "{current}/{total}");
    }
}

#[test]
fn progress_never_exceeds_one_hundred_percent() {
    let event = TaskProgressEvent::new("t", "s", "m").with_progress(9, 3);
    assert_eq!(event.progress, 100);

    let single = TaskProgressEvent::new("t", "s", "m").with_progress(255, 1);
    assert_eq!(single.progress, 100);
}

#[test]
fn a_zero_total_keeps_progress_at_zero() {
    let event = TaskProgressEvent::new("t", "s", "m").with_progress(3, 0);
    assert_eq!(event.progress, 0);
    assert_eq!(event.total_steps, 0);
}

#[test]
fn builders_chain_without_losing_earlier_values() {
    let event = TaskProgressEvent::new("t", "s", "m")
        .with_progress(2, 4)
        .with_details("step detail")
        .with_event_type("task_step")
        .with_activity(AgentActivity::new("writing"));

    assert_eq!(event.progress, 50);
    assert_eq!(event.details.as_deref(), Some("step detail"));
    assert_eq!(event.event_type, "task_step");
    assert_eq!(
        event.activity.as_ref().map(|a| a.phase.as_str()),
        Some("writing")
    );
}

#[test]
fn an_error_event_switches_the_type_and_records_the_message() {
    let event = TaskProgressEvent::new("t", "s", "m").with_error("boom");

    assert_eq!(event.event_type, "task_error");
    assert_eq!(event.error.as_deref(), Some("boom"));
}

#[test]
fn completion_forces_one_hundred_percent() {
    let event = TaskProgressEvent::new("t", "s", "m")
        .with_progress(1, 4)
        .completed();

    assert_eq!(event.event_type, "task_completed");
    assert_eq!(event.progress, 100);
}

#[test]
fn a_started_event_carries_the_total_step_count() {
    let event = TaskProgressEvent::started("task-9", "beginning", 7);

    assert_eq!(event.event_type, "task_started");
    assert_eq!(event.step, "init");
    assert_eq!(event.total_steps, 7);
    assert_eq!(event.current_step, 0);
    assert_eq!(event.progress, 0);
}

#[test]
fn a_new_activity_has_no_measurements_yet() {
    let activity = AgentActivity::new("planning");

    assert_eq!(activity.phase, "planning");
    assert_eq!(activity.items_processed, 0);
    assert!(activity.items_total.is_none());
    assert!(activity.speed_per_min.is_none());
    assert!(activity.eta_seconds.is_none());
    assert!(activity.current_item.is_none());
    assert!(activity.bytes_processed.is_none());
    assert!(activity.tokens_used.is_none());
    assert!(activity.files_created.is_none());
    assert!(activity.tables_created.is_none());
    assert!(activity.log_lines.is_none());
}

#[test]
fn activity_builders_record_each_measurement() {
    let activity = AgentActivity::new("writing")
        .with_progress(3, Some(10))
        .with_speed(12.5, Some(30))
        .with_bytes(2048)
        .with_current_item("orders.bas")
        .with_files(vec!["orders.bas".to_string()])
        .with_tables(vec!["orders".to_string()])
        .with_log_lines(vec!["line".to_string()])
        .with_tokens(512);

    assert_eq!(activity.items_processed, 3);
    assert_eq!(activity.items_total, Some(10));
    assert_eq!(activity.speed_per_min, Some(12.5));
    assert_eq!(activity.eta_seconds, Some(30));
    assert_eq!(activity.bytes_processed, Some(2048));
    assert_eq!(activity.current_item.as_deref(), Some("orders.bas"));
    assert_eq!(activity.files_created.as_ref().map(Vec::len), Some(1));
    assert_eq!(activity.tables_created.as_ref().map(Vec::len), Some(1));
    assert_eq!(activity.log_lines.as_ref().map(Vec::len), Some(1));
    assert_eq!(activity.tokens_used, Some(512));
}
