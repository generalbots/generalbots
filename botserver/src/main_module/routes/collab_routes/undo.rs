//! #2448 criterion 4 — the LIVE, scoped agent-batch undo endpoint.
//!
//! The typed data model lives in `core::bot::consent_gate::undo`; this module
//! is its only HTTP consumer and also hosts the shared in-process service used
//! by the chat-native `docs.undo` verb. The request body carries a single batch id:
//! user, bot and session are derived exclusively from the verified
//! `AuthenticatedUser` inserted by the auth middleware. A batch is reverted in
//! LIFO order from recorded before-states (`UndoBatch::revert_plan`) and only
//! then consumed once (`UndoStore::consume`, conditional UPDATE): a failed
//! restore leaves the batch active and retryable. Where a before-state names a
//! collaboration version, the restore is performed by the existing
//! `restore_version` handler with the same server-side identity — never with
//! an owner/bot supplied by the caller.

use super::*;
pub(super) use crate::core::bot::consent_gate::undo::before_version_id;
use crate::core::bot::consent_gate::undo::{
    apply_restore_plan, consume_after_restore, peek, peek_latest, shared_pool_store, BatchKey,
    RestoreSink, RevertOperation, UndoError, UndoScope, UndoStore,
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
///
/// Both callers peek first and invoke this unconditional consume primitive only
/// after a successful restore, so a failed restore leaves the batch active.
pub(crate) fn undo_plan_for<S: UndoStore>(
    store: &S,
    scope: &UndoScope,
    batch_id: &str,
) -> Result<Vec<RevertOperation>, UndoError> {
    let key = BatchKey::new(scope.clone(), batch_id)?;
    let batch = store.consume(&key)?;
    Ok(batch.revert_plan())
}

/// Typed failures map to existing localized error bodies; foreign, missing and
/// already-consumed batches are indistinguishable to the caller so the route
/// cannot be used to enumerate another user's batch ids.
pub(super) fn undo_error(
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

/// `POST /api/collab/undo` — peek and revert the caller's active batch, then
/// consume it once on success.
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
    // Peek (ownership-checked, read-only) so a failed restore cannot eat the
    // caller's only copy of the batch; consume only after restore succeeds.
    let key =
        BatchKey::new(scope.clone(), &body.batch_id).map_err(|error| undo_error(locale, error))?;
    let plan = peek(&store, &key)
        .map_err(|error| undo_error(locale, error))?
        .revert_plan();

    let restored = restore_plan(&state, &user, locale, &plan).await?;
    // Restore succeeded: apply the scoped single-use consume gate now.
    let reverted =
        undo_plan_for(&store, &scope, &body.batch_id).map_err(|error| undo_error(locale, error))?;
    let result = UndoResult {
        batch_id: body.batch_id,
        reverted,
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

type RestoreError = (StatusCode, Json<serde_json::Value>);

/// Thin, fail-open wrapper around the undo module's version-history probe. A
/// probe error is logged and ignored: the restore itself surfaces any real
/// database failure, so a transient probe outage can never block a restore.
fn already_restored(state: &Arc<AppState>, actor_id: &str, version_id: Uuid) -> bool {
    let mut conn = match state.conn.get() {
        Ok(conn) => conn,
        Err(error) => {
            log::error!("undo restore idempotency: pool error: {error}");
            return false;
        }
    };
    match crate::core::bot::consent_gate::undo::history_has_newer_restore(
        &mut conn, actor_id, version_id,
    ) {
        Ok(found) => found,
        Err(error) => {
            log::error!("undo restore idempotency: history query failed: {error}");
            false
        }
    }
}

struct PgRestoreSink<'a> {
    state: &'a Arc<AppState>,
    user: &'a AuthenticatedUser,
    locale: &'a botcore::i18n::Locale,
    actor_id: String,
}

impl RestoreSink<RestoreError> for PgRestoreSink<'_> {
    fn was_restored(&self, version_id: Uuid) -> bool {
        already_restored(self.state, &self.actor_id, version_id)
    }

    fn restore(
        &self,
        version_id: Uuid,
    ) -> impl std::future::Future<Output = Result<String, RestoreError>> + '_ {
        let state = self.state;
        let user = self.user;
        let locale = self.locale;
        async move {
            let response = super::restore_version(
                State(state.clone()),
                Extension(user.clone()),
                Path(version_id),
                botcore::i18n::RequestLocale(locale.clone()),
            )
            .await?;
            Ok(response.id.clone())
        }
    }
}

/// Reverts a plan through the existing handler, skipping already restored
/// operations so a failed batch can be retried without duplicate versions.
async fn restore_plan(
    state: &Arc<AppState>,
    user: &AuthenticatedUser,
    locale: &botcore::i18n::Locale,
    plan: &[RevertOperation],
) -> Result<Vec<String>, RestoreError> {
    let sink = PgRestoreSink {
        state,
        user,
        actor_id: collab_user_id(user),
        locale,
    };
    apply_restore_plan(&sink, plan).await
}

/// Resolves the verified chat principal's account email from the server-side
/// user record. `user_id` comes from the trusted command executor's verified
/// session, never from command params or a client-supplied header. The email is
/// required because `collab_user_id()` resolves the collab grant by email when
/// one exists; without it the restore ran as a bare UUID and 404'd on a version
/// owned by the same user's account (#2448 live UAT).
fn verified_user_email(state: &Arc<AppState>, user_id: Uuid) -> Option<String> {
    use diesel::sql_types::{Text, Uuid as SqlUuid};
    use diesel::{OptionalExtension, RunQueryDsl};

    #[derive(diesel::QueryableByName)]
    struct EmailRow {
        #[diesel(sql_type = Text)]
        email: String,
    }

    let mut conn = state.conn.get().ok()?;
    let row: Option<EmailRow> = diesel::sql_query("SELECT email FROM users WHERE id = $1 LIMIT 1")
        .bind::<SqlUuid, _>(user_id)
        .get_result::<EmailRow>(&mut conn)
        .optional()
        .ok()
        .flatten();
    let email = row?.email.trim().to_string();
    if email.is_empty() {
        return None;
    }
    Some(email)
}

/// Shared service for the chat-native "undo my last agent batch" verb (#2448).
///
/// The scope is built from the authenticated chat identity only — `user_id` and
/// `bot_id` come from the trusted command executor, never from command params.
/// The verified account email is carried into that identity so collab grants
/// resolve to the same principal that authored the restored version. With no
/// session claim the verifier derives the same stable UUIDv5 the loopback HTTP
/// hop uses, so chat and the consent card address one batch. The newest active
/// batch of that scope is reverted through the same restore path as the HTTP
/// endpoint and consumed once only after the restore succeeds.
pub async fn undo_last_batch_for_chat(
    state: &Arc<AppState>,
    user_id: Uuid,
    bot_id: Uuid,
) -> Result<UndoResult, UndoError> {
    let user = AuthenticatedUser::new(user_id, "chat".to_string()).with_current_bot(bot_id);
    let user = match verified_user_email(state, user_id) {
        Some(email) => user.with_email(email),
        None => user,
    };
    let scope = verified_scope(&user)?;
    let store = shared_pool_store().ok_or(UndoError::Storage)?;
    let resolved = peek_latest(&store, &scope)?;
    let locale = botcore::i18n::Locale::new("en").ok_or(UndoError::InvalidScope)?;
    let restore = restore_plan(state, &user, &locale, &resolved.revert_plan)
        .await
        .map_err(|(status, _)| {
            log::error!(
                "chat undo version restore failed with HTTP {}",
                status.as_u16()
            );
            UndoError::Storage
        });
    let restored = consume_after_restore(&store, &resolved.key, restore, |error| error)?;
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
    fn email_carrying_chat_identity_resolves_the_verified_collab_principal() -> Result<(), UndoError>
    {
        let user_id = Uuid::from_u128(11);
        let bot_id = Uuid::from_u128(22);
        let bare = AuthenticatedUser::new(user_id, "chat".to_string()).with_current_bot(bot_id);
        assert_eq!(collab_user_id(&bare), user_id.to_string());

        let with_email = bare.clone().with_email("admin@localhost");
        assert_eq!(collab_user_id(&with_email), "admin@localhost");
        assert_eq!(with_email.email.as_deref(), Some("admin@localhost"));
        // The account email must not change the batch scope: the chat verb and
        // the HTTP consent card still address the very same batch.
        assert_eq!(verified_scope(&with_email)?, verified_scope(&bare)?);
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
