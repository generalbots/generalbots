//! Deterministic unit coverage for the scoped agent-batch undo data model.
//! No server, database, browser or network is needed.

use chrono::{DateTime, TimeZone, Utc};
use serde_json::{json, Value};
use uuid::Uuid;

use super::{
    BatchKey, BatchStatus, InMemoryUndoStore, UndoError, UndoOperation, UndoScope, UndoStore,
};

fn scope() -> UndoScope {
    let mut scope = UndoScope::new(Uuid::from_u128(11), Uuid::from_u128(22), "session-1");
    scope.thread_id = Some("thread-9".to_string());
    scope
}

fn key(batch_id: &str) -> BatchKey {
    BatchKey::new(scope(), batch_id).unwrap_or_else(|_| BatchKey {
        batch_id: batch_id.to_string(),
        scope: scope(),
    })
}

fn at(offset_secs: i64) -> DateTime<Utc> {
    match Utc.timestamp_opt(1_700_000_000 + offset_secs, 0) {
        chrono::LocalResult::Single(value) => value,
        _ => Utc::now(),
    }
}

fn operation(op_id: &str, before: Value) -> UndoOperation {
    UndoOperation::new(op_id, "docs", "doc-42", before).unwrap_or_else(|_| UndoOperation {
        op_id: op_id.to_string(),
        app_id: "docs".to_string(),
        target: "doc-42".to_string(),
        after: None,
        before: Value::Null,
    })
}

#[test]
fn record_then_lookup_returns_the_exact_scoped_batch() -> Result<(), UndoError> {
    let store = InMemoryUndoStore::new();
    let batch_key = key("batch-1");
    let op = operation("op-1", json!({ "title": "before" }))
        .with_after(json!({ "title": "after" }));
    let recorded = store.record(batch_key.clone(), vec![op.clone()])?;

    assert_eq!(recorded.key, batch_key);
    assert_eq!(recorded.status, BatchStatus::Active);
    let found = store.find(&batch_key)?;
    assert!(found.is_some(), "a recorded batch must be findable by its key");
    assert_eq!(found.map(|batch| batch.operations), Some(vec![op]));
    assert_eq!(store.find(&key("other-batch"))?, None);
    Ok(())
}

#[test]
fn lookup_rejects_foreign_user_bot_and_session() -> Result<(), UndoError> {
    let store = InMemoryUndoStore::new();
    let original = key("batch-1");
    store.record(original.clone(), vec![operation("op-1", json!(1))])?;

    let foreign_user = BatchKey::new(
        UndoScope::new(Uuid::from_u128(99), Uuid::from_u128(22), "session-1"),
        "batch-1",
    )?;
    let foreign_bot = BatchKey::new(
        UndoScope::new(Uuid::from_u128(11), Uuid::from_u128(99), "session-1"),
        "batch-1",
    )?;
    let foreign_session = BatchKey::new(
        UndoScope::new(Uuid::from_u128(11), Uuid::from_u128(22), "session-2"),
        "batch-1",
    )?;
    for foreign in [&foreign_user, &foreign_bot, &foreign_session] {
        assert_eq!(
            store.find(foreign),
            Err(UndoError::ScopeMismatch),
            "a different user, bot or session must never resolve the batch"
        );
        assert_eq!(store.consume(foreign), Err(UndoError::ScopeMismatch));
    }
    Ok(())
}

#[test]
fn consume_succeeds_once_and_second_undo_is_rejected() -> Result<(), UndoError> {
    let store = InMemoryUndoStore::new();
    let batch_key = key("batch-2");
    store.record(batch_key.clone(), vec![operation("op-1", json!("old"))])?;

    let consumed = store.consume(&batch_key)?;
    assert_eq!(consumed.status, BatchStatus::Undone);
    assert_eq!(consumed.revert_plan().len(), 1);
    assert_eq!(store.consume(&batch_key), Err(UndoError::AlreadyUndone));
    assert_eq!(store.latest(&scope())?, None, "undone batches are not active");
    let stored = match store.find(&batch_key)? {
        Some(batch) => batch,
        None => return Err(UndoError::NotFound),
    };
    assert_eq!(stored.status, BatchStatus::Undone);
    Ok(())
}

#[test]
fn operations_without_a_before_state_cannot_form_a_batch() -> Result<(), UndoError> {
    assert_eq!(
        UndoOperation::new("op-1", "docs", "doc-42", Value::Null),
        Err(UndoError::NotReversible)
    );
    let store = InMemoryUndoStore::new();
    assert_eq!(
        store.record(key("batch-3"), Vec::new()),
        Err(UndoError::InvalidBatch)
    );
    let no_target = UndoOperation {
        op_id: "op-1".to_string(),
        app_id: "docs".to_string(),
        target: String::new(),
        after: None,
        before: json!("old"),
    };
    assert_eq!(
        store.record(key("batch-4"), vec![no_target]),
        Err(UndoError::NotReversible)
    );
    assert_eq!(
        store.record(
            key("batch-5"),
            vec![operation("op-1", json!(1)), operation("op-1", json!(2))]
        ),
        Err(UndoError::NotReversible),
        "duplicate op ids are ambiguous and must not be recorded"
    );
    assert_eq!(store.latest(&scope())?, None);
    Ok(())
}

#[test]
fn revert_plan_is_reverse_lifo_and_carries_only_before_states() -> Result<(), UndoError> {
    let first = operation("op-1", json!({ "cell": "A1", "value": 1 }))
        .with_after(json!({ "cell": "A1", "value": 2 }));
    let second = operation("op-2", json!({ "cell": "B2", "value": 3 }));
    let batch = super::UndoBatch::new(key("batch-6"), vec![first, second], at(0))?;
    let plan = batch.revert_plan();

    assert_eq!(plan.len(), 2);
    assert_eq!(plan[0].op_id, "op-2");
    assert_eq!(plan[0].before, json!({ "cell": "B2", "value": 3 }));
    assert_eq!(plan[1].op_id, "op-1");
    assert_eq!(plan[1].before, json!({ "cell": "A1", "value": 1 }));
    Ok(())
}

#[test]
fn latest_undo_targets_only_the_newest_active_batch() -> Result<(), UndoError> {
    let store = InMemoryUndoStore::new();
    store.record_at(key("older"), vec![operation("op-1", json!(1))], at(0))?;
    store.record_at(key("newer"), vec![operation("op-2", json!(2))], at(60))?;

    let latest = match store.latest(&scope())? {
        Some(batch) => batch,
        None => return Err(UndoError::NotFound),
    };
    assert_eq!(latest.key.batch_id, "newer");
    store.consume(&key("newer"))?;
    let remaining = match store.latest(&scope())? {
        Some(batch) => batch,
        None => return Err(UndoError::NotFound),
    };
    assert_eq!(remaining.key.batch_id, "older");
    Ok(())
}

#[test]
fn destructive_marker_carries_only_an_explicit_batch_id() {
    use base64::Engine as _;
    let decode = |marker: &str| -> Value {
        let encoded = marker.split("GB-CONSENT-REQUEST:").nth(1).unwrap_or("");
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(encoded.trim())
            .unwrap_or_default();
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    let mut request = botconsent::PendingRequest {
        request_id: "req-1".to_string(),
        user_id: Uuid::from_u128(11),
        app_id: "docs".to_string(),
        action_class: "delete".to_string(),
        detail: json!({ "command": "docs.rows.delete", "params": { "batch_id": "batch-9", "ids": [1] } }),
        thread_id: None,
        run_id: None,
        risk_tier: "high".to_string(),
    };
    let payload = decode(&super::super::marker_content(&request));
    assert_eq!(payload["confirm_required"], json!(true));
    assert_eq!(payload["batch_id"], json!("batch-9"));

    request.action_class = "read".to_string();
    assert!(decode(&super::super::marker_content(&request))
        .get("batch_id")
        .is_none());

    request.action_class = "delete".to_string();
    request.detail = json!({ "command": "docs.rows.delete", "params": { "ids": [1] } });
    assert!(decode(&super::super::marker_content(&request))
        .get("batch_id")
        .is_none());
}

#[test]
fn nil_identity_and_empty_ids_are_rejected_before_storage() -> Result<(), UndoError> {
    let nil_user = UndoScope::new(Uuid::nil(), Uuid::from_u128(22), "session-1");
    assert_eq!(BatchKey::new(nil_user, "batch-1"), Err(UndoError::InvalidScope));
    assert_eq!(
        BatchKey::new(scope(), "  "),
        Err(UndoError::InvalidBatch)
    );
    let store = InMemoryUndoStore::new();
    assert_eq!(
        store.record(key("batch-7"), vec![operation("op-1", json!(1))])?.status,
        BatchStatus::Active
    );
    Ok(())
}
