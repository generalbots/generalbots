//! TEST / UAT SEED ROUTE for the agent-batch undo verb (#2448).
//!
//! This is deliberately **not** a production writer. The real agent-batch
//! executor lives outside this module and does not yet call `UndoStore::record`;
//! until it does, this authenticated route records a batch for the verified
//! caller from a typed body of *real* before-states so the chat-native
//! `docs.undo` verb can be exercised end to end. Every before-state must name
//! an existing collaboration version the verified caller can access, so a
//! fabricated id can never become a "restore target".
//!
//! No user, bot or session id is read from the request: the scope is rebuilt
//! from the authenticated principal exactly as `undo_last_batch_for_chat`
//! builds it (no session claim -> the same stable UUIDv5), so the chat verb can
//! find the batch.

use super::undo::{before_version_id, undo_error, verified_scope};
use super::*;
use crate::core::bot::consent_gate::undo::{
    shared_pool_store, BatchKey, UndoBatch, UndoError, UndoOperation, UndoScope, UndoStore,
};

/// Client-supplied payload: a batch id plus typed before-states. Identity
/// fields are intentionally absent — they are derived from `AuthenticatedUser`.
#[derive(Debug, Deserialize)]
pub struct SeedUndoBody {
    pub batch_id: String,
    pub operations: Vec<SeedUndoOperation>,
}

/// One before-state to record. `before` must be an object with a real
/// `version_id` (validated by [`seed_batch_for`] and by the handler's DB check).
#[derive(Debug, Deserialize)]
pub struct SeedUndoOperation {
    pub op_id: String,
    pub app_id: String,
    pub target: String,
    pub before: serde_json::Value,
}

/// Pure core shared by the handler and the deterministic tests.
///
/// The caller passes a *chat-equivalent* authenticated identity (verified user
/// and bot, no session claim) so [`verified_scope`] derives the UUIDv5 session
/// id that `undo_last_batch_for_chat` also derives. A different session claim
/// is therefore irrelevant to lookup — the batch always belongs to the one
/// verified user/bot pair.
fn seed_batch_for<S: UndoStore>(
    store: &S,
    chat_user: &AuthenticatedUser,
    batch_id: &str,
    operations: Vec<SeedUndoOperation>,
) -> Result<UndoBatch, UndoError> {
    let scope = verified_scope(chat_user)?;
    let key = BatchKey::new(scope, batch_id)?;
    let mut typed = Vec::with_capacity(operations.len());
    for operation in operations {
        // The restore path selects a target only from `before.version_id`; a
        // before-state without one is a guess, not a recorded state.
        if before_version_id(&operation.before).is_none() {
            return Err(UndoError::NotReversible);
        }
        typed.push(UndoOperation::new(
            &operation.op_id,
            &operation.app_id,
            &operation.target,
            operation.before,
        )?);
    }
    store.record(key, typed)
}

/// `POST /api/collab/undo/seed` — authenticated test/UAT recorder.
///
/// Gated by the verified principal (`is_authenticated`) and by collaboration
/// access to every referenced version. Responses use existing localized error
/// bodies; no new user-facing string is introduced.
pub async fn seed_undo_batch(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    request_locale: botcore::i18n::RequestLocale,
    Json(body): Json<SeedUndoBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let locale = request_locale.locale();
    if !user.is_authenticated() {
        return Err(err(
            locale,
            StatusCode::FORBIDDEN,
            "error-collab-invalid-resource",
        ));
    }
    let scope = verified_scope(&user).map_err(|_| {
        err(
            locale,
            StatusCode::FORBIDDEN,
            "error-collab-invalid-resource",
        )
    })?;
    let store = shared_pool_store().ok_or_else(|| {
        err(
            locale,
            StatusCode::INTERNAL_SERVER_ERROR,
            "error-db-unavailable",
        )
    })?;

    // Every before-state must name a version that really exists and is owned
    // (or accessible) by the verified caller before it can be recorded.
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    for operation in &body.operations {
        let version_id = match before_version_id(&operation.before) {
            Some(id) => id,
            None => return Err(undo_error(locale, UndoError::NotReversible)),
        };
        let resource = match version_resource(&mut conn, version_id) {
            Some(resource) => resource,
            None => return Err(undo_error(locale, UndoError::NotFound)),
        };
        if !has_collab_access(&mut conn, &user, &resource.0, &resource.1) {
            return Err(undo_error(locale, UndoError::ScopeMismatch));
        }
    }

    // Rebuild an identity with no session claim so `verified_scope` uses the
    // same UUIDv5 derivation as the chat verb.
    let chat_user =
        AuthenticatedUser::new(user.user_id, user.username.clone()).with_current_bot(scope.bot_id);
    let batch = seed_batch_for(&store, &chat_user, &body.batch_id, body.operations)
        .map_err(|error| undo_error(locale, error))?;
    let count = batch.operations.len();
    let batch_id = batch.key.batch_id;
    warn!(
        "agent-undo seed route recorded batch {batch_id} ({count} operations) for verified user {}",
        user.user_id
    );
    Ok(Json(serde_json::json!({
        "batch_id": batch_id,
        "seeded": true,
        "operations": count,
    })))
}

#[cfg(test)]
mod tests {
    use super::super::undo::undo_plan_for;
    use super::*;
    use crate::core::bot::consent_gate::undo::{consume_latest, InMemoryUndoStore};
    use serde_json::json;
    use uuid::Uuid;

    fn user() -> AuthenticatedUser {
        AuthenticatedUser::new(Uuid::from_u128(31), "chat".to_string())
            .with_current_bot(Uuid::from_u128(32))
    }

    fn operation(op_id: &str) -> SeedUndoOperation {
        SeedUndoOperation {
            op_id: op_id.to_string(),
            app_id: "docs".to_string(),
            target: "doc-7".to_string(),
            before: json!({ "version_id": "22222222-2222-2222-2222-222222222222" }),
        }
    }

    fn record(store: &InMemoryUndoStore, batch_id: &str) -> Result<UndoBatch, UndoError> {
        seed_batch_for(store, &user(), batch_id, vec![operation("op-1")])
    }

    #[test]
    fn seed_records_batch_under_verified_chat_scope() -> Result<(), UndoError> {
        let store = InMemoryUndoStore::new();
        let scope = verified_scope(&user())?;
        let batch = record(&store, "batch-seed-scope")?;
        assert_eq!(batch.key.scope, scope);
        assert_eq!(batch.key.scope.user_id, Uuid::from_u128(31));
        assert_eq!(batch.key.scope.bot_id, Uuid::from_u128(32));
        assert_eq!(batch.operations.len(), 1);
        Ok(())
    }

    #[test]
    fn chat_verb_finds_and_consumes_the_seeded_batch() -> Result<(), UndoError> {
        let store = InMemoryUndoStore::new();
        record(&store, "batch-seed-chat")?;
        let scope = verified_scope(&user())?;
        let resolved = consume_latest(&store, &scope)?;
        assert_eq!(resolved.key.batch_id, "batch-seed-chat");
        assert_eq!(resolved.revert_plan.len(), 1);
        assert_eq!(
            resolved.revert_plan[0].before,
            json!({ "version_id": "22222222-2222-2222-2222-222222222222" })
        );
        Ok(())
    }

    #[test]
    fn cross_session_cannot_consume_the_seeded_batch() -> Result<(), UndoError> {
        let store = InMemoryUndoStore::new();
        record(&store, "batch-seed-cross")?;
        let foreign = UndoScope::new(
            Uuid::from_u128(31),
            Uuid::from_u128(32),
            "a-different-session",
        );
        assert_eq!(
            undo_plan_for(&store, &foreign, "batch-seed-cross"),
            Err(UndoError::ScopeMismatch)
        );
        // The rightful chat scope can still consume it afterwards.
        assert_eq!(
            consume_latest(&store, &verified_scope(&user())?)?
                .key
                .batch_id,
            "batch-seed-cross"
        );
        Ok(())
    }

    #[test]
    fn second_undo_is_denied_after_the_chat_verb_consumes() -> Result<(), UndoError> {
        let store = InMemoryUndoStore::new();
        record(&store, "batch-seed-twice")?;
        let scope = verified_scope(&user())?;
        assert_eq!(
            consume_latest(&store, &scope)?.key.batch_id,
            "batch-seed-twice"
        );
        // No active batch remains for the same scope.
        assert_eq!(consume_latest(&store, &scope), Err(UndoError::NotFound));
        assert_eq!(
            undo_plan_for(&store, &scope, "batch-seed-twice"),
            Err(UndoError::AlreadyUndone)
        );
        Ok(())
    }

    #[test]
    fn guessed_before_state_is_rejected() {
        let store = InMemoryUndoStore::new();
        let guessed = SeedUndoOperation {
            op_id: "op-guess".to_string(),
            app_id: "docs".to_string(),
            target: "doc-7".to_string(),
            before: json!({ "version_id": "" }),
        };
        assert_eq!(
            seed_batch_for(&store, &user(), "batch-guess", vec![guessed]),
            Err(UndoError::NotReversible)
        );
    }
}
