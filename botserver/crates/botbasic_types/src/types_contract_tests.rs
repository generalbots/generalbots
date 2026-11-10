//! Contract tests for the shared BASIC types (#2064).
//!
//! `types.rs` is the boundary every BASIC keyword crate shares with `botlib`
//! and the database: a lost field in `From<botlib::models::UserSession>`, or a
//! `NOT NULL` column that serde lets through as `None`, is a silent data bug.
//! These tests pin both directions (Rust → JSON and botlib → botbasic_types).

use crate::types::{Attachment, BotMemory, UserSession, WorkflowEvent, WorkflowExecution};
use chrono::{TimeZone, Utc};
use uuid::Uuid;

fn timestamp() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 9, 22, 0, 0)
        .single()
        .expect("valid date")
}

fn botlib_session() -> botlib::models::UserSession {
    botlib::models::UserSession {
        id: Uuid::from_u128(1),
        user_id: Uuid::from_u128(2),
        branch_id: Uuid::from_u128(3),
        bot_id: Uuid::from_u128(4),
        title: "invoice review".to_string(),
        context_data: serde_json::json!({"step": 2}),
        current_tool: Some("classify_media".to_string()),
        created_at: timestamp(),
        updated_at: Utc
            .with_ymd_and_hms(2026, 11, 9, 23, 30, 0)
            .single()
            .expect("valid date"),
        locked: true,
        lock_pin_hash: Some("hash".to_string()),
    }
}

/// `From<botlib::models::UserSession>` is how every channel hands a session to
/// the BASIC runtime. The conversion is lossy on purpose (`answer_mode`,
/// counters), but the identity and the branch must survive.
#[test]
fn conversion_from_botlib_session_keeps_identity_and_branch() {
    let source = botlib_session();
    let converted = UserSession::from(source.clone());

    assert_eq!(converted.id, source.id);
    assert_eq!(converted.user_id, source.user_id.to_string());
    assert_eq!(converted.bot_id, source.bot_id);
    assert_eq!(converted.branch_id, source.branch_id);
    assert_eq!(converted.title, "invoice review");
    assert_eq!(converted.context_data, serde_json::json!({"step": 2}));
    assert_eq!(converted.current_tool.as_deref(), Some("classify_media"));
    assert_eq!(converted.created_at, source.created_at);
    assert_eq!(converted.updated_at, source.updated_at);
}

/// A brand-new session starts empty: the counters are zero, nothing is locked
/// and no expiry/tenant/email account is bound yet.
#[test]
fn conversion_resets_the_fields_the_database_owns() {
    let converted = UserSession::from(botlib_session());

    assert_eq!(converted.answer_mode, 0);
    assert_eq!(converted.message_count, 0);
    assert_eq!(converted.total_tokens, 0);
    assert_eq!(converted.last_activity, converted.updated_at);
    assert_eq!(converted.tenant_id, None);
    assert_eq!(converted.active_email_account_id, None);
    assert_eq!(converted.session_id, None);
    assert_eq!(converted.data, None);
    assert_eq!(converted.expires_at, None);
    // #1682 — the lock is per channel session, never inherited.
    assert!(!converted.locked);
    assert_eq!(converted.lock_pin_hash, None);
}

#[test]
fn user_session_serde_round_trip_is_lossless() {
    let session = UserSession {
        id: Uuid::from_u128(10),
        user_id: "zitadel:user".to_string(),
        bot_id: Uuid::from_u128(11),
        title: "support".to_string(),
        answer_mode: 2,
        context_data: serde_json::json!({"queue": "billing"}),
        current_tool: None,
        message_count: 42,
        total_tokens: 1234,
        created_at: timestamp(),
        updated_at: timestamp(),
        last_activity: timestamp(),
        tenant_id: Some(Uuid::from_u128(12)),
        active_email_account_id: None,
        branch_id: Uuid::from_u128(13),
        session_id: Some("ws-1".to_string()),
        data: Some(serde_json::json!({"pin": "1234"})),
        expires_at: Some(timestamp()),
        locked: true,
        lock_pin_hash: Some("$argon2".to_string()),
    };

    let json = serde_json::to_string(&session).expect("serialize");
    let back: UserSession = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(back.user_id, "zitadel:user");
    assert_eq!(back.message_count, 42);
    assert_eq!(back.total_tokens, 1234);
    assert_eq!(back.tenant_id, session.tenant_id);
    assert_eq!(back.data, session.data);
    assert_eq!(back.expires_at, session.expires_at);
    // #1682 — a locked conversation must stay locked after a round-trip.
    assert!(back.locked);
    assert_eq!(back.lock_pin_hash.as_deref(), Some("$argon2"));
}

/// `data`/`expires_at` are nullable columns, so they serialize as `null`
/// rather than being dropped: the frontend distinguishes "absent" from "empty".
#[test]
fn nullable_user_session_fields_serialize_as_null() {
    let mut session = UserSession::from(botlib_session());
    session.data = None;
    session.expires_at = None;

    let value = serde_json::to_value(&session).expect("serialize");
    assert!(value.get("data").is_some(), "`data` must not be omitted");
    assert!(value["data"].is_null());
    assert!(value.get("expires_at").is_some(), "`expires_at` must not be omitted");
    assert!(value["expires_at"].is_null());
}

/// `bot_memories.branch_id` became `NOT NULL` in `6.5.23-branch-scope-cleanup`;
/// a payload without it must be rejected instead of defaulting to nil.
#[test]
fn bot_memory_rejects_a_payload_without_a_branch() {
    let complete = serde_json::json!({
        "id": Uuid::from_u128(1),
        "bot_id": Uuid::from_u128(2),
        "key": "preferred_language",
        "value": "pt-BR",
        "created_at": timestamp(),
        "updated_at": timestamp(),
        "branch_id": Uuid::from_u128(3),
    });
    let memory: BotMemory = serde_json::from_value(complete).expect("branch_id present");
    assert_eq!(memory.key, "preferred_language");

    let mut missing = serde_json::json!({
        "id": Uuid::from_u128(1),
        "bot_id": Uuid::from_u128(2),
        "key": "preferred_language",
        "value": "pt-BR",
        "created_at": timestamp(),
        "updated_at": timestamp(),
    });
    assert!(
        serde_json::from_value::<BotMemory>(missing.clone()).is_err(),
        "a memory without branch_id must not deserialize"
    );

    // An explicit `null` is equally invalid for a NOT NULL column.
    missing["branch_id"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<BotMemory>(missing).is_err());
}

/// A zero-byte attachment is a real size, not a missing one, and the size is a
/// `u64` on the wire.
#[test]
fn attachment_size_distinguishes_zero_from_absent() {
    let base = Attachment {
        attachment_type: "image".to_string(),
        url: "/api/files/zero".to_string(),
        mime_type: None,
        filename: None,
        size: Some(0),
    };
    let back: Attachment =
        serde_json::from_str(&serde_json::to_string(&base).expect("serialize"))
            .expect("deserialize");
    assert_eq!(back.size, Some(0));
    assert_eq!(back.mime_type, None);

    let max = Attachment {
        size: Some(u64::MAX),
        ..base.clone()
    };
    let back: Attachment =
        serde_json::from_str(&serde_json::to_string(&max).expect("serialize"))
            .expect("deserialize");
    assert_eq!(back.size, Some(u64::MAX));

    let absent = Attachment {
        size: None,
        ..base
    };
    let back: Attachment =
        serde_json::from_str(&serde_json::to_string(&absent).expect("serialize"))
            .expect("deserialize");
    assert_eq!(back.size, None);
}

/// A workflow execution may legitimately be at step 0 and carry a large JSON
/// blob; neither is special-cased on the way through serde.
#[test]
fn workflow_execution_keeps_step_and_state_independently() {
    let execution = WorkflowExecution {
        id: Uuid::from_u128(1),
        bot_id: Uuid::from_u128(2),
        workflow_name: "onboarding".to_string(),
        current_step: 0,
        state_json: "null".to_string(),
        status: "pending".to_string(),
        created_at: timestamp(),
        updated_at: timestamp(),
    };

    let back: WorkflowExecution =
        serde_json::from_str(&serde_json::to_string(&execution).expect("serialize"))
            .expect("deserialize");
    assert_eq!(back.current_step, 0);
    assert_eq!(back.state_json, "null");
    assert_eq!(back.status, "pending");

    let mut later = execution;
    later.current_step = -1;
    later.state_json = "{}".to_string();
    let back: WorkflowExecution =
        serde_json::from_str(&serde_json::to_string(&later).expect("serialize"))
            .expect("deserialize");
    assert_eq!(back.current_step, -1);
    assert_eq!(back.state_json, "{}");
}

/// #1497/#1513 — `schema::tables` is a re-export of `botschema`, never a local
/// copy. Every module must resolve to the canonical table name.
#[test]
fn every_schema_reexport_is_the_canonical_table() {
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

    assert_eq!(table_sql(&crate::schema::tables::bots::table), "bots");
    assert_eq!(
        table_sql(&crate::schema::tables::user_sessions::table),
        "user_sessions"
    );
    assert_eq!(
        table_sql(&crate::schema::tables::bot_memories::table),
        "bot_memories"
    );
    assert_eq!(
        table_sql(&crate::schema::tables::bot_shared_memory::table),
        "bot_shared_memory"
    );
    assert_eq!(
        table_sql(&crate::schema::tables::system_automations::table),
        "system_automations"
    );
    assert_eq!(
        table_sql(&crate::schema::tables::session_tool_associations::table),
        "session_tool_associations"
    );
    assert_eq!(
        table_sql(&crate::schema::tables::message_history::table),
        "message_history"
    );
    assert_eq!(
        table_sql(&crate::schema::tables::workflow_executions::table),
        "workflow_executions"
    );
    assert_eq!(
        table_sql(&crate::schema::tables::workflow_events::table),
        "workflow_events"
    );
    assert_eq!(
        table_sql(&crate::schema::tables::calendar_events::table),
        "calendar_events"
    );
    assert_eq!(
        table_sql(&crate::schema::tables::organizations::table),
        "organizations"
    );
    assert_eq!(
        table_sql(&crate::schema::tables::bot_configuration::table),
        "bot_configuration"
    );
}

/// An event without a workflow (`workflow_id: null`) is how ad-hoc triggers are
/// queued, and an event without a payload must stay `None`.
#[test]
fn workflow_event_keeps_both_optional_fields_nullable() {
    let event = WorkflowEvent {
        id: Uuid::from_u128(1),
        workflow_id: None,
        event_name: "chat.message".to_string(),
        event_data_json: None,
        processed: false,
        created_at: timestamp(),
    };

    let value = serde_json::to_value(&event).expect("serialize");
    assert!(value["workflow_id"].is_null());
    assert!(value["event_data_json"].is_null());

    let back: WorkflowEvent = serde_json::from_value(value).expect("deserialize");
    assert!(back.workflow_id.is_none());
    assert!(back.event_data_json.is_none());
    assert!(!back.processed);
}
