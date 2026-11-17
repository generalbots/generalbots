use super::*;

use super::*;

#[test]
fn activity_serialises_with_snake_case_kind() {
    let a = Activity::started(
        ActivityKind::AttachmentRead,
        serde_json::json!({"name": "test.pdf"}),
    );
    let json = serde_json::to_value(&a).expect("serialise");
    assert_eq!(json["kind"], "attachment_read");
    assert_eq!(json["status"], "started");
    assert!(json["id"].as_str().is_some());
}

#[test]
fn completed_keeps_the_id() {
    let a = Activity::started(ActivityKind::KbSearch, serde_json::json!({}));
    let b = a.completed(serde_json::json!({"results": []}));
    assert_eq!(a.id, b.id);
    assert_eq!(b.status, ActivityStatus::Completed);
}

#[test]
fn failed_carries_a_truncated_error() {
    let a = Activity::started(ActivityKind::ToolRun, serde_json::json!({}));
    let long = "x".repeat(500);
    let b = a.failed(&long);
    assert_eq!(b.status, ActivityStatus::Failed);
    let err = b.error.as_deref().unwrap_or("");
    assert!(err.len() <= MAX_EXCERPT_CHARS + 4); // UTF-8 ellipsis
}

#[test]
fn truncate_respects_limit() {
    assert_eq!(truncate("short", 10), "short");
    assert_eq!(truncate("hello world", 6), "hello\u{2026}");
}

#[test]
fn attachment_summary_caps_fields() {
    let long_name = "a".repeat(500);
    let summary = attachment_summary(
        &long_name,
        "application/pdf",
        1024,
        500,
        false,
        "/inbox/test.pdf",
    );
    let name = summary["name"].as_str().unwrap_or("");
    assert!(name.chars().count() <= MAX_PATH_CHARS + 1);
}

#[test]
fn kb_search_summary_caps_list() {
    let sources: Vec<KbSource> = (0..50)
        .map(|i| KbSource {
            document: format!("doc_{i}.md"),
            excerpt: format!("excerpt {i}"),
            score: 0.9,
        })
        .collect();
    let summary = kb_search_summary("test query", &sources);
    let results = summary["results"].as_array().unwrap();
    assert_eq!(results.len(), MAX_LIST_ENTRIES);
    assert_eq!(summary["total"], 50);
}

#[test]
fn drive_list_summary_caps_entries() {
    let entries: Vec<DriveEntry> = (0..30)
        .map(|i| DriveEntry {
            name: format!("file_{i}.txt"),
            size: 1024,
            kind: "file".into(),
        })
        .collect();
    let summary = drive_list_summary("/documents", &entries);
    let listed = summary["entries"].as_array().unwrap();
    assert_eq!(listed.len(), MAX_LIST_ENTRIES);
    assert_eq!(summary["total"], 30);
}

#[test]
fn merge_collapses_start_end_into_one_object() {
    let a = Activity::started(
        ActivityKind::AttachmentRead,
        serde_json::json!({"name": "f.pdf"}),
    );
    let b = a.completed(serde_json::json!({"name": "f.pdf", "chars_read": 500}));
    let events = vec![
        (a, "2026-11-15 14:30:00.000000".to_string()),
        (b, "2026-11-15 14:30:01.500000".to_string()),
    ];
    let merged = merge_activity_pairs(events);
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0]["status"], "completed");
    assert_eq!(merged[0]["summary"]["chars_read"], 500);
    assert_eq!(merged[0]["duration_ms"], 1500);
}

#[test]
fn merge_preserves_order_and_keeps_failed() {
    let a = Activity::started(ActivityKind::KbSearch, serde_json::json!({}));
    let b = Activity::started(ActivityKind::ToolRun, serde_json::json!({}));
    let a_fail = a.failed("timeout");
    let events = vec![
        (a, "2026-11-15 14:30:00.000000".to_string()),
        (b.clone(), "2026-11-15 14:30:00.100000".to_string()),
        (a_fail, "2026-11-15 14:30:02.000000".to_string()),
    ];
    let merged = merge_activity_pairs(events);
    assert_eq!(merged.len(), 2);
    assert_eq!(merged[0]["kind"], "kb_search");
    assert_eq!(merged[0]["status"], "failed");
    assert_eq!(merged[1]["kind"], "tool_run");
    assert_eq!(merged[1]["status"], "started"); // no end event
}

#[test]
fn needs_input_status_serialises() {
    let a = Activity::started(ActivityKind::Permission, serde_json::json!({}));
    let b = a.needs_input(serde_json::json!({"request_id": "abc"}));
    let json = serde_json::to_value(&b).expect("serialise");
    assert_eq!(json["status"], "needs_input");
}

#[test]
fn with_run_id_sets_the_field() {
    let a =
        Activity::started(ActivityKind::ApiCommand, serde_json::json!({})).with_run_id("run-123");
    assert_eq!(a.run_id.as_deref(), Some("run-123"));
}
