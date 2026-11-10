//! Contract tests for the shared BASIC types (#2064).

use crate::schema::tables;
use crate::types::{Attachment, TriggerKind, WorkflowEvent, WorkflowExecution};
use chrono::{TimeZone, Utc};
use uuid::Uuid;

fn table_sql<T>(table: &T) -> String
where
    T: diesel::query_builder::QueryFragment<diesel::pg::Pg>
        + diesel::query_builder::QueryId,
{
    diesel::debug_query::<diesel::pg::Pg, _>(table)
        .to_string()
        .split(" -- binds")
        .next()
        .unwrap_or_default()
        .replace('"', "")
        .trim()
        .to_string()
}

#[test]
fn trigger_kind_constructors_are_stable() {
    assert_eq!(TriggerKind::webhook().0, "webhook");
    assert_eq!(TriggerKind::email_received().0, "email_received");
    assert_eq!(TriggerKind::schedule().0, "schedule");
    assert_eq!(TriggerKind::database_change().0, "database_change");
    assert_eq!(TriggerKind::manual().0, "manual");
}

#[test]
fn trigger_kind_serde_round_trip() {
    let kind = TriggerKind::schedule();
    let json = serde_json::to_string(&kind).expect("serialize");
    let back: TriggerKind = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.0, "schedule");
}

#[test]
fn workflow_execution_serde_round_trip() {
    let created = Utc.with_ymd_and_hms(2026, 11, 9, 22, 0, 0).unwrap();
    let execution = WorkflowExecution {
        id: Uuid::nil(),
        bot_id: Uuid::nil(),
        workflow_name: "invoice-approval".to_string(),
        current_step: 3,
        state_json: "{\"step\":3}".to_string(),
        status: "running".to_string(),
        created_at: created,
        updated_at: created,
    };
    let json = serde_json::to_string(&execution).expect("serialize");
    let back: WorkflowExecution = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.workflow_name, "invoice-approval");
    assert_eq!(back.current_step, 3);
    assert_eq!(back.status, "running");
}

#[test]
fn workflow_event_keeps_optional_fields() {
    let created = Utc.with_ymd_and_hms(2026, 11, 9, 22, 0, 0).unwrap();
    let event = WorkflowEvent {
        id: Uuid::nil(),
        workflow_id: None,
        event_name: "invoice.created".to_string(),
        event_data_json: None,
        processed: false,
        created_at: created,
    };
    let back: WorkflowEvent =
        serde_json::from_str(&serde_json::to_string(&event).expect("serialize"))
            .expect("deserialize");
    assert_eq!(back.event_name, "invoice.created");
    assert!(back.workflow_id.is_none());
    assert!(back.event_data_json.is_none());
    assert!(!back.processed);
}

#[test]
fn attachment_serde_round_trip() {
    let attachment = Attachment {
        attachment_type: "image".to_string(),
        url: "/api/files/abc".to_string(),
        mime_type: Some("image/png".to_string()),
        filename: Some("robot.png".to_string()),
        size: Some(1024),
    };
    let back: Attachment =
        serde_json::from_str(&serde_json::to_string(&attachment).expect("serialize"))
            .expect("deserialize");
    assert_eq!(back.attachment_type, "image");
    assert_eq!(back.filename.as_deref(), Some("robot.png"));
    assert_eq!(back.size, Some(1024));
}

/// #1497/#1513 — the local `tables` module must re-export the canonical
/// `botschema` tables, never a private copy.
#[test]
fn schema_reexports_are_the_canonical_tables() {
    assert_eq!(table_sql(&tables::bots::table), "bots");
    assert_eq!(table_sql(&tables::user_sessions::table), "user_sessions");
    assert_eq!(table_sql(&tables::bot_memories::table), "bot_memories");
    assert_eq!(
        table_sql(&tables::session_tool_associations::table),
        "session_tool_associations"
    );
}
