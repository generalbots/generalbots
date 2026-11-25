//! #2448 criterion 4 — the LIVE, scoped agent-batch undo endpoint.
//!
//! The typed data model lives in `core::bot::consent_gate::undo`; this module
//! is its only HTTP consumer and also hosts the shared in-process service used
//! by the chat-native `docs.undo` verb. The request body carries a single batch id:
//! user, bot and session are derived exclusively from the verified
//! `AuthenticatedUser` inserted by the auth middleware. A batch is consumed
//! once (`UndoStore::consume`, conditional UPDATE) and reverted in LIFO order
//! from recorded before-states (`UndoBatch::revert_plan`). Where a before-state
//! names a collaboration version, the restore is performed by the existing
//! `restore_version` handler with the same server-side identity — never with
//! an owner/bot supplied by the caller.

use super::*;
use crate::core::bot::consent_gate::undo::{
    BatchKey, RevertOperation, UndoError, UndoScope, UndoStore, consume_latest,
    shared_pool_store,
};
use uuid::Uuid;

/// The ONLY client-supplied field. Expanding this struct with an owner, bot or
/// session would break the ownership invariant enforced by [`verified_scope`].
#[derive(Debug, Deserialize)]
pub struct UndoBody {
    pub batch_id: String,
}

/// Build the ownership scope from verified session claims only.
///
/// `current_bot_id` is set by the auth middleware from the verified `X-Bot-ID`
/// header; a token carrying exactly one bot grant is the alternate server-side
/// source. When the session carries its own id it is used verbatim, otherwise
/// a stable UUIDv5 is derived from the verified user+bot pair — never from a
/// header, query string or request body.
pub(crate) fn verified_scope(user: &AuthenticatedUser) -> Result<UndoScope, UndoError> {
    if !user.is_authenticated() {
        return Err(UndoError::InvalidScope);
    }
    let bot_id = user
        .current_bot_id
        .filter(|id| !id.is_nil())
        .or_else(|| {
            let accessible = user.accessible_bot_ids();
            if accessible.len() == 1 {
                accessible.first().copied()
            } else {
                None
            }
        })
        .ok_or(UndoError::InvalidScope)?;
    let session_id = match user.session_id.as_deref() {
        Some(id) if !id.trim().is_empty() => id.to_string(),
        _ => Uuid::new_v5(
            &Uuid::NAMESPACE_OID,
            format!("undo:{}:{}", user.user_id, bot_id).as_bytes(),
        )
        .to_string(),
    };
    let scope = UndoScope::new(user.user_id, bot_id, &session_id);
    scope.validate()?;
    Ok(scope)
}

/// Single-use + LIFO core shared by the handler and its deterministic tests.
pub(crate) fn undo_plan_for<S: UndoStore>(
    store: &S,
    scope: &UndoScope,
    batch_id: &str,
) -> Result<Vec<RevertOperation>, UndoError> {
    let key = BatchKey::new(scope.clone(), batch_id)?;
    let batch = store.consume(&key)?;
    Ok(batch.revert_plan())
}

/// A restore target is only read from the recorded before-state. The
/// after-state is diagnostic and can never select a version to restore.
fn before_version_id(before: &serde_json::Value) -> Option<Uuid> {
    before
        .get("version_id")
        .and_then(serde_json::Value::as_str)
        .and_then(|value| Uuid::parse_str(value).ok())
}

/// Typed failures map to existing localized error bodies; foreign, missing and
/// already-consumed batches are indistinguishable to the caller so the route
/// cannot be used to enumerate another user's batch ids.
fn undo_error(
    locale: &botcore::i18n::Locale,
    error: UndoError,
) -> (StatusCode, Json<serde_json::Value>) {
    match error {
        UndoError::InvalidScope | UndoError::InvalidBatch => err(
            locale,
            StatusCode::BAD_REQUEST,
            "error-collab-invalid-resource",
        ),
        UndoError::Storage => err(
            locale,
            StatusCode::INTERNAL_SERVER_ERROR,
            "error-db-unavailable",
        ),
        UndoError::ScopeMismatch
        | UndoError::NotFound
        | UndoError::NotReversible
        | UndoError::AlreadyRecorded
        | UndoError::AlreadyUndone => err(
            locale,
            StatusCode::NOT_FOUND,
            "error-collab-version-not-found",
        ),
    }
}

/// `POST /api/collab/undo` — consume the caller's active batch and revert it.
pub async fn undo_batch(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    request_locale: botcore::i18n::RequestLocale,
    Json(body): Json<UndoBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let locale = request_locale.locale();
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
    let plan =
        undo_plan_for(&store, &scope, &body.batch_id).map_err(|error| undo_error(locale, error))?;

    let restored = restore_plan(&state, &user, locale, &plan).await?;
    let result = UndoResult {
        batch_id: body.batch_id,
        reverted: plan,
        restored,
    };
    let value = serde_json::to_value(result).map_err(|_| {
        err(
            locale,
            StatusCode::INTERNAL_SERVER_ERROR,
            "error-db-unavailable",
        )
    })?;
    Ok(Json(value))
}

/// Typed, structured result shared by the HTTP route and the chat-native verb.
/// The LLM turns these fields into user-facing text; no prose is hardcoded here.
#[derive(Debug, serde::Serialize)]
pub struct UndoResult {
    pub batch_id: String,
    pub reverted: Vec<RevertOperation>,
    pub restored: Vec<String>,
}

/// Reverts a consumed plan through the existing version-restore handler.
///
/// The restore handler derives the target resource from the stored version and
/// the actor from the verified session. No caller-supplied owner, bot or
/// resource id is ever passed through this call.
async fn restore_plan(
    state: &Arc<AppState>,
    user: &AuthenticatedUser,
    locale: &botcore::i18n::Locale,
    plan: &[RevertOperation],
) -> Result<Vec<String>, (StatusCode, Json<serde_json::Value>)> {
    let mut restored = Vec::new();
    for operation in plan {
        let Some(version_id) = before_version_id(&operation.before) else {
            continue;
        };
        let response = super::restore_version(
            State(state.clone()),
            Extension(user.clone()),
            Path(version_id),
            botcore::i18n::RequestLocale(locale.clone()),
        )
        .await?;
        restored.push(response.id.clone());
    }
    Ok(restored)
}

/// Shared service for the chat-native "undo my last agent batch" verb (#2448).
///
/// The scope is built from the authenticated chat identity only — `user_id` and
/// `bot_id` come from the trusted command executor, never from command params.
/// With no session claim the verifier derives the same stable UUIDv5 the
/// loopback HTTP hop uses, so chat and the consent card address one batch.
/// The newest active batch of that scope is consumed once and reverted through
/// the same restore path as the HTTP endpoint.
pub async fn undo_last_batch_for_chat(
    state: &Arc<AppState>,
    user_id: Uuid,
    bot_id: Uuid,
) -> Result<UndoResult, UndoError> {
    let user = AuthenticatedUser::new(user_id, "chat".to_string()).with_current_bot(bot_id);
    let scope = verified_scope(&user)?;
    let store = shared_pool_store().ok_or(UndoError::Storage)?;
    let resolved = consume_latest(&store, &scope)?;
    let locale = botcore::i18n::Locale::new("en").ok_or(UndoError::InvalidScope)?;
    let restored = restore_plan(state, &user, &locale, &resolved.revert_plan)
        .await
        .map_err(|(status, _)| {
            log::error!("chat undo version restore failed with HTTP {}", status.as_u16());
            UndoError::Storage
        })?;
    Ok(UndoResult {
        batch_id: resolved.key.batch_id,
        reverted: resolved.revert_plan,
        restored,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::bot::consent_gate::undo::{InMemoryUndoStore, UndoOperation};
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    fn scope(session: &str) -> UndoScope {
        UndoScope::new(Uuid::from_u128(11), Uuid::from_u128(22), session)
    }

    fn operation(version_id: &str) -> UndoOperation {
        UndoOperation {
            op_id: "op-1".to_string(),
            app_id: "docs".to_string(),
            target: "doc-42".to_string(),
            after: Some(json!({ "version_id": "11111111-1111-1111-1111-111111111111" })),
            before: json!({ "version_id": version_id }),
        }
    }

    fn store_with(batch_id: &str) -> Result<InMemoryUndoStore, UndoError> {
        let store = InMemoryUndoStore::new();
        store.record_at(
            BatchKey::new(scope("session-1"), batch_id)?,
            vec![operation("22222222-2222-2222-2222-222222222222")],
            Utc.timestamp_opt(1_700_000_000, 0)
                .single()
                .ok_or(UndoError::Storage)?,
        )?;
        Ok(store)
    }

    #[test]
    fn cross_session_undo_is_denied_and_leaves_the_batch_active() -> Result<(), UndoError> {
        let store = store_with("batch-cross")?;
        // Same user and bot, different authenticated session: no undo.
        assert_eq!(
            undo_plan_for(&store, &scope("session-2"), "batch-cross"),
            Err(UndoError::ScopeMismatch)
        );
        let still_active = store
            .find(&BatchKey::new(scope("session-1"), "batch-cross")?)?
            .ok_or(UndoError::NotFound)?;
        assert_eq!(
            still_active.status,
            crate::core::bot::consent_gate::undo::BatchStatus::Active
        );
        // The rightful session can still consume it afterwards.
        assert_eq!(
            undo_plan_for(&store, &scope("session-1"), "batch-cross")?.len(),
            1
        );
        Ok(())
    }

    #[test]
    fn double_undo_is_denied_by_the_single_use_core() -> Result<(), UndoError> {
        let store = store_with("batch-double")?;
        assert_eq!(
            undo_plan_for(&store, &scope("session-1"), "batch-double")?.len(),
            1
        );
        assert_eq!(
            undo_plan_for(&store, &scope("session-1"), "batch-double"),
            Err(UndoError::AlreadyUndone)
        );
        Ok(())
    }

    #[test]
    fn revert_uses_before_state_only_never_the_after_state() -> Result<(), UndoError> {
        let store = store_with("batch-before")?;
        let plan = undo_plan_for(&store, &scope("session-1"), "batch-before")?;
        assert_eq!(plan.len(), 1);
        assert_eq!(
            plan[0].before,
            json!({ "version_id": "22222222-2222-2222-2222-222222222222" })
        );
        // The plan type has no `after` field at all.
        let encoded = serde_json::to_value(&plan).map_err(|_| UndoError::Storage)?;
        assert!(encoded[0].get("after").is_none());
        // Restore selection reads only `before.version_id`.
        assert_eq!(
            before_version_id(&plan[0].before),
            Uuid::parse_str("22222222-2222-2222-2222-222222222222").ok()
        );
        let after = json!({ "version_id": "11111111-1111-1111-1111-111111111111" });
        assert_ne!(
            before_version_id(&after),
            before_version_id(&plan[0].before)
        );
        Ok(())
    }

    #[test]
    fn verified_scope_never_accepts_anonymous_or_botless_identities() {
        let anonymous = AuthenticatedUser::anonymous();
        assert_eq!(verified_scope(&anonymous), Err(UndoError::InvalidScope));

        let mut user = AuthenticatedUser::new(Uuid::from_u128(11), "user".to_string());
        assert_eq!(verified_scope(&user), Err(UndoError::InvalidScope));
        user = user.with_current_bot(Uuid::from_u128(22));
        let derived = match verified_scope(&user) {
            Ok(scope) => scope,
            Err(_) => return,
        };
        assert_eq!(derived.user_id, Uuid::from_u128(11));
        assert_eq!(derived.bot_id, Uuid::from_u128(22));
        assert!(!derived.session_id.is_empty());

        let with_session = user.with_session("verified-session");
        assert_eq!(
            verified_scope(&with_session).map(|scope| scope.session_id),
            Ok("verified-session".to_string())
        );
    }
}
