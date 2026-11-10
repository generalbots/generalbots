//! Contract tests for the workflow DTOs (#2064).
//!
//! `TriggerKind::from_i32` is the only conversion between the persisted integer
//! and the typed trigger, so its exact table is pinned: an unknown code must be
//! `None` (never a silent default) or an unrelated workflow starts firing.

use super::{TriggerKind, WorkflowExecution};
use chrono::Utc;
use uuid::Uuid;

#[test]
fn every_documented_trigger_code_maps_to_its_variant() {
    let cases = [
        (0, TriggerKind::Scheduled),
        (1, TriggerKind::TableUpdate),
        (2, TriggerKind::TableInsert),
        (3, TriggerKind::TableDelete),
        (4, TriggerKind::Webhook),
        (5, TriggerKind::EmailReceived),
        (6, TriggerKind::FolderChange),
        (7, TriggerKind::DealStageChange),
        (8, TriggerKind::ContactChange),
        (9, TriggerKind::EmailOpened),
    ];
    for (code, expected) in cases {
        assert_eq!(TriggerKind::from_i32(code), Some(expected), "code {code}");
    }
}

#[test]
fn unknown_trigger_codes_are_none() {
    for code in [-1, 10, 42, i32::MIN, i32::MAX] {
        assert_eq!(TriggerKind::from_i32(code), None, "code {code}");
    }
}

#[test]
fn trigger_kinds_are_copy_and_comparable() {
    let kind = TriggerKind::Webhook;
    let copy = kind;
    assert_eq!(kind, copy);
}

#[test]
fn a_workflow_execution_with_empty_optionals_round_trips() {
    let execution = WorkflowExecution {
        id: Uuid::new_v4(),
        bot_id: Uuid::new_v4(),
        workflow_name: "onboarding".to_string(),
        current_step: None,
        state_json: None,
        status: "pending".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let encoded = serde_json::to_string(&execution).expect("serializable");
    let decoded: WorkflowExecution = serde_json::from_str(&encoded).expect("deserializable");

    assert!(decoded.current_step.is_none());
    assert!(decoded.state_json.is_none());
    assert_eq!(decoded.workflow_name, "onboarding");
}

#[test]
fn current_step_and_state_are_independent() {
    let base = WorkflowExecution {
        id: Uuid::nil(),
        bot_id: Uuid::nil(),
        workflow_name: "w".to_string(),
        current_step: Some(3),
        state_json: Some(serde_json::json!({"seen": true})),
        status: "running".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let decoded: WorkflowExecution =
        serde_json::from_str(&serde_json::to_string(&base).expect("serializable"))
            .expect("deserializable");
    assert_eq!(decoded.current_step, Some(3));
    assert_eq!(decoded.state_json, Some(serde_json::json!({"seen": true})));

    // `Option` fields may be absent on the wire: the derived deserializer maps
    // a missing key to `None`, which is what an older row looks like.
    let mut json = serde_json::to_value(&base).expect("serializable");
    json.as_object_mut()
        .expect("object")
        .remove("current_step");
    let decoded: WorkflowExecution = serde_json::from_value(json).expect("deserializable");
    assert!(decoded.current_step.is_none());
    assert_eq!(decoded.state_json, Some(serde_json::json!({"seen": true})));
}
