use std::sync::Arc;

use axum::extract::{Json, Path, State};
use botsecurity_auth::auth_api::types::AuthenticatedUser;
use serde_json::Value;
use uuid::Uuid;

use crate::error::IntegrationError;
use crate::request::{parse_bot_id, parse_connection_id, split_request};
use crate::scope::{resolve_scope, ConnectionScope};
use crate::secrets::build_connection_path;
use crate::state::IntegrationState;
use crate::{models, repository};

pub(crate) fn record_outcome(
    conn: &mut diesel::PgConnection,
    scope: &ConnectionScope,
    connection_id: Option<Uuid>,
    event_type: &str,
    outcome: &str,
    risk_level: &str,
    metadata: &Value,
) {
    let event = repository::NewConnectionEvent {
        connection_id,
        actor_user_id: scope.owner_user_id(),
        event_type,
        outcome,
        risk_level,
        metadata,
    };
    if let Err(error) = repository::record_event(conn, scope, &event) {
        log::error!("failed to append integration connection event {event_type}: {error:?}");
    }
}

fn record_to_value(record: models::ConnectionRecord) -> Result<Value, IntegrationError> {
    serde_json::to_value(record)
        .map_err(|e| IntegrationError::Storage(format!("record encode failed: {e}")))
}

/// POST /api/bots/:bot_id/integration-connections
pub async fn create(
    State(state): State<Arc<IntegrationState>>,
    user: AuthenticatedUser,
    Path(bot_id): Path<String>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, IntegrationError> {
    let bot_uuid = parse_bot_id(&bot_id)?;
    let scope = resolve_scope(&state.pool, &user, bot_uuid)?;
    let parsed = split_request(&body)?;

    {
        let mut conn = state.pool.get()?;
        if repository::find_active_by_provider(&mut conn, &scope, &parsed.provider_slug)?.is_some()
        {
            record_outcome(
                &mut conn,
                &scope,
                None,
                "connection.create_denied",
                "denied",
                "low",
                &serde_json::json!({ "provider": parsed.provider_slug }),
            );
            return Err(IntegrationError::Conflict);
        }
    }

    let connection_id = Uuid::new_v4();
    let vault_path = build_connection_path(&scope, connection_id);

    // Credentials go to Vault first and fail closed: a Vault outage never
    // leaves behind a database row pointing at unwritten credentials.
    let has_secrets = parsed
        .secrets
        .as_object()
        .map(|envelope| !envelope.is_empty())
        .unwrap_or(false);
    if has_secrets {
        state
            .vault
            .store(&scope, connection_id, &parsed.secrets)
            .await?;
    }

    let granted_scopes = serde_json::to_value(&parsed.granted_scopes)
        .map_err(|error| IntegrationError::Storage(format!("scopes encode failed: {error}")))?;
    let insert = repository::NewConnectionInsert {
        connection_id,
        provider_slug: &parsed.provider_slug,
        display_name: &parsed.display_name,
        auth_kind: &parsed.auth_kind,
        vault_path: &vault_path,
        granted_scopes: &granted_scopes,
        visibility: &parsed.visibility,
        configuration: &parsed.configuration,
        expires_at: parsed.expires_at,
    };
    let insert_result = {
        let mut conn = state.pool.get()?;
        repository::insert(&mut conn, &scope, &insert)
    };
    if let Err(error) = insert_result {
        if has_secrets {
            if let Err(cleanup) = state.vault.delete(&vault_path).await {
                log::error!("orphan credential cleanup failed for {vault_path}: {cleanup:?}");
            }
        }
        return Err(error);
    }

    let mut conn = state.pool.get()?;
    record_outcome(
        &mut conn,
        &scope,
        Some(connection_id),
        "connection.created",
        "ok",
        "low",
        &serde_json::json!({
            "provider": parsed.provider_slug,
            "auth_kind": parsed.auth_kind,
            "status": "active",
        }),
    );
    let stored =
        repository::get(&mut conn, &scope, connection_id)?.ok_or(IntegrationError::NotFound)?;
    Ok(Json(record_to_value(stored.into_record())?))
}

/// GET /api/bots/:bot_id/integration-connections
pub async fn list(
    State(state): State<Arc<IntegrationState>>,
    user: AuthenticatedUser,
    Path(bot_id): Path<String>,
) -> Result<Json<Value>, IntegrationError> {
    let bot_uuid = parse_bot_id(&bot_id)?;
    let scope = resolve_scope(&state.pool, &user, bot_uuid)?;
    let mut conn = state.pool.get()?;
    let records: Result<Vec<Value>, IntegrationError> = repository::list(&mut conn, &scope, false)?
        .into_iter()
        .map(|row| record_to_value(row.into_record()))
        .collect();
    Ok(Json(serde_json::json!({ "items": records? })))
}

/// GET /api/bots/:bot_id/integration-connections/:connection_id
pub async fn get_one(
    State(state): State<Arc<IntegrationState>>,
    user: AuthenticatedUser,
    Path((bot_id, connection_id)): Path<(String, String)>,
) -> Result<Json<Value>, IntegrationError> {
    let bot_uuid = parse_bot_id(&bot_id)?;
    let connection_uuid = parse_connection_id(&connection_id)?;
    let scope = resolve_scope(&state.pool, &user, bot_uuid)?;
    let mut conn = state.pool.get()?;
    let stored =
        repository::get(&mut conn, &scope, connection_uuid)?.ok_or(IntegrationError::NotFound)?;
    Ok(Json(record_to_value(stored.into_record())?))
}

/// DELETE /api/bots/:bot_id/integration-connections/:connection_id
pub async fn remove(
    State(state): State<Arc<IntegrationState>>,
    user: AuthenticatedUser,
    Path((bot_id, connection_id)): Path<(String, String)>,
) -> Result<Json<Value>, IntegrationError> {
    let bot_uuid = parse_bot_id(&bot_id)?;
    let connection_uuid = parse_connection_id(&connection_id)?;
    let scope = resolve_scope(&state.pool, &user, bot_uuid)?;

    let vault_path = {
        let mut conn = state.pool.get()?;
        let stored = repository::get(&mut conn, &scope, connection_uuid)?
            .ok_or(IntegrationError::NotFound)?;
        record_outcome(
            &mut conn,
            &scope,
            Some(connection_uuid),
            "connection.deleted",
            "ok",
            "medium",
            &serde_json::json!({
                "provider": stored.provider_slug,
                "reason": "deleted by owner",
            }),
        );
        stored.vault_path
    };

    let deleted = {
        let mut conn = state.pool.get()?;
        repository::delete_row(&mut conn, &scope, connection_uuid)?
    };
    if !deleted {
        return Err(IntegrationError::NotFound);
    }

    if let Err(error) = state.vault.delete(&vault_path).await {
        // The row is already gone; surface the leftover credential loudly in
        // the server log so operators can purge the Vault entry manually.
        log::error!("vault cleanup failed after deleting {connection_uuid}: {error:?}");
    }

    Ok(Json(
        serde_json::json!({ "id": connection_uuid.to_string(), "deleted": true }),
    ))
}
