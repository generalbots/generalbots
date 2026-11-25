//! Deterministic coverage for the chat-native, verified-scope undo lookup.
//! No server, database, browser or network is needed.

use chrono::{TimeZone, Utc};
use serde_json::{json, Value};
use uuid::Uuid;

use super::{
    BatchKey, BatchStatus, InMemoryUndoStore, UndoError, UndoOperation, UndoScope, UndoStore,
    consume_latest, latest_batch_key,
};

fn scope(session: &str) -> UndoScope {
    UndoScope::new(Uuid::from_u128(11), Uuid::from_u128(22), session)
}

fn operation(before: Value, after: Value) -> UndoOperation {
    match UndoOperation::new("op-1", "docs", "doc-42", before) {
        Ok(operation) => operation.with_after(after),
        Err(_) => UndoOperation {
            op_id: "op-1".to_string(),
            app_id: "docs".to_string(),
            target: "doc-42".to_string(),
            after: None,
            before: Value::Null,
        },
    }
}

fn record(
    store: &InMemoryUndoStore,
    session: &str,
    batch_id: &str,
    offset: i64,
    before: Value,
) -> Result<(), UndoError> {
    let key = BatchKey::new(scope(session), batch_id)?;
    let at = match Utc.timestamp_opt(1_700_000_000 + offset, 0) {
        chrono::LocalResult::Single(value) => value,
        _ => return Err(UndoError::Storage),
    };
    store.record_at(key, vec![operation(before, json!("new"))], at)?;
    Ok(())
}

#[test]
fn chat_undo_resolves_only_the_verified_scopes_newest_batch() -> Result<(), UndoError> {
    let store = InMemoryUndoStore::new();
    record(&store, "session-1", "older", 0, json!("old-a"))?;
    record(&store, "session-1", "newest", 60, json!("old-b"))?;
    record(&store, "session-2", "foreign", 120, json!("foreign-old"))?;

    let resolved = consume_latest(&store, &scope("session-1"))?;
    assert_eq!(resolved.key.batch_id, "newest");
    assert_eq!(resolved.revert_plan.len(), 1);
    assert_eq!(resolved.revert_plan[0].before, json!("old-b"));

    // The foreign session's batch was neither selected nor consumed.
    let foreign = latest_batch_key(&store, &scope("session-2"))?;
    assert_eq!(foreign.batch_id, "foreign");
    assert_eq!(
        store
            .find(&foreign)?
            .ok_or(UndoError::NotFound)?
            .status,
        BatchStatus::Active
    );
    Ok(())
}

#[test]
fn chat_undo_denies_a_foreign_session_without_consuming_the_owners_batch() -> Result<(), UndoError> {
    let store = InMemoryUndoStore::new();
    record(&store, "session-1", "owned", 0, json!("old"))?;

    let foreign = consume_latest(&store, &scope("session-2"));
    assert!(foreign.is_err(), "a different session must never resolve the batch");

    // The rightful session still owns an active batch afterwards.
    let resolved = consume_latest(&store, &scope("session-1"))?;
    assert_eq!(resolved.key.batch_id, "owned");
    Ok(())
}

#[test]
fn chat_undo_second_invocation_is_denied() -> Result<(), UndoError> {
    let store = InMemoryUndoStore::new();
    record(&store, "session-1", "single-use", 0, json!("old"))?;

    let first = consume_latest(&store, &scope("session-1"))?;
    assert_eq!(first.key.batch_id, "single-use");
    let second = consume_latest(&store, &scope("session-1"));
    assert!(second.is_err(), "a consumed batch must not be restorable twice");
    Ok(())
}

#[test]
fn chat_undo_restores_before_state_only() -> Result<(), UndoError> {
    let store = InMemoryUndoStore::new();
    record(&store, "session-1", "scope-check", 0, json!({ "version_id": "before" }))?;

    let resolved = consume_latest(&store, &scope("session-1"))?;
    assert_eq!(
        resolved.revert_plan[0].before,
        json!({ "version_id": "before" })
    );
    // A revert plan has no `after` field at all, so restore cannot pick it.
    let encoded = serde_json::to_value(&resolved.revert_plan).map_err(|_| UndoError::Storage)?;
    assert!(encoded[0].get("after").is_none());
    Ok(())
}
