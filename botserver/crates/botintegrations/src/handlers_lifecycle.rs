use std::sync::Arc;

use axum::extract::{Json, Path, State};
use botsecurity_auth::auth_api::types::AuthenticatedUser;
use serde_json::Value;

use crate::error::IntegrationError;
use crate::models;
use crate::repository as repo;
use crate::scope::resolve_scope;
use crate::state::IntegrationState;

use crate::request::{parse_bot_id, parse_connection_id, split_rotation_secrets};

use super::handlers_connections::record_outcome;

fn encode_record(record: models::ConnectionRecord) -> Result<Value, IntegrationError> {
    serde_json::to_value(record)
        .map_err(|error| IntegrationError::Storage(format!("record encode failed: {error}")))
}

/// POST /api/bots/:bot_id/integration-connections/:connection_id/test
///
/// Placeholder-safe connectivity probe for slice 1: credentials are loaded
/// strictly from Vault and the stored configuration shape is validated. No
/// outbound network call is performed; the recorded outcome is always
/// `unverified` until a later slice introduces real probes.
pub async fn test_connection(
    State(state): State<Arc<IntegrationState>>,
    user: AuthenticatedUser,
    Path((bot_id, connection_id)): Path<(String, String)>,
) -> Result<Json<Value>, IntegrationError> {
    let bot_uuid = parse_bot_id(&bot_id)?;
    let connection_uuid = parse_connection_id(&connection_id)?;
    let scope = resolve_scope(&state.pool, &user, bot_uuid)?;

    let row = {
        let mut conn = state.pool.get()?;
        repo::get(&mut conn, &scope, connection_uuid)?.ok_or(IntegrationError::NotFound)?
    };

    match state.vault.load_strict(&row.vault_path).await {
        Err(error @ IntegrationError::VaultUnavailable) => {
            let mut conn = state.pool.get()?;
            if let Err(mark_error) =
                repo::mark_tested(&mut conn, &scope, connection_uuid, "unverified")
            {
                log::error!("failed to mark test outcome for {connection_uuid}: {mark_error:?}");
            }
            record_outcome(
                &mut conn,
                &scope,
                Some(connection_uuid),
                "connection.tested",
                "failed",
                "medium",
                &serde_json::json!({
                    "provider": row.provider_slug,
                    "outcome_detail": "credential store unavailable"
                }),
            );
            Err(error)
        }
        Err(other) => Err(other),
        Ok(credentials) => {
            let shape_ok = credentials
                .as_object()
                .map(|entries| !entries.is_empty())
                .unwrap_or(false)
                && row.configuration.is_object();
            let mut conn = state.pool.get()?;
            if let Err(mark_error) =
                repo::mark_tested(&mut conn, &scope, connection_uuid, "unverified")
            {
                log::error!("failed to mark test outcome for {connection_uuid}: {mark_error:?}");
            }
            if shape_ok {
                record_outcome(
                    &mut conn,
                    &scope,
                    Some(connection_uuid),
                    "connection.tested",
                    "ok",
                    "low",
                    &serde_json::json!({
                        "provider": row.provider_slug,
                        "test_status": "unverified",
                        "outcome_detail": "shape validated; no outbound probe in this slice"
                    }),
                );
                Ok(Json(serde_json::json!({
                    "id": connection_uuid.to_string(),
                    "outcome": "unverified",
                    "detail": "credentials present and configuration shape valid; connectivity probing is not performed in this slice",
                })))
            } else {
                record_outcome(
                    &mut conn,
                    &scope,
                    Some(connection_uuid),
                    "connection.tested",
                    "failed",
                    "low",
                    &serde_json::json!({
                        "provider": row.provider_slug,
                        "outcome_detail": "invalid credential or configuration shape"
                    }),
                );
                Err(IntegrationError::Validation(
                    "stored credential envelope or configuration has an invalid shape".to_string(),
                ))
            }
        }
    }
}

/// POST /api/bots/:bot_id/integration-connections/:connection_id/rotate
pub async fn rotate(
    State(state): State<Arc<IntegrationState>>,
    user: AuthenticatedUser,
    Path((bot_id, connection_id)): Path<(String, String)>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, IntegrationError> {
    let bot_uuid = parse_bot_id(&bot_id)?;
    let connection_uuid = parse_connection_id(&connection_id)?;
    let scope = resolve_scope(&state.pool, &user, bot_uuid)?;
    let secrets = split_rotation_secrets(&body)?;

    let row = {
        let mut conn = state.pool.get()?;
        repo::get(&mut conn, &scope, connection_uuid)?.ok_or(IntegrationError::NotFound)?
    };

    // New credential material overwrites the canonical path; the database
    // version counter only advances once Vault accepted the write.
    state.vault.store(&scope, connection_uuid, &secrets).await?;

    let version = {
        let mut conn = state.pool.get()?;
        let version = repo::increment_credential_version(&mut conn, &scope, connection_uuid)?;
        record_outcome(
            &mut conn,
            &scope,
            Some(connection_uuid),
            "connection.rotated",
            "ok",
            "medium",
            &serde_json::json!({ "provider": row.provider_slug, "credential_version": version }),
        );
        version
    };

    let mut conn = state.pool.get()?;
    let stored =
        repo::get(&mut conn, &scope, connection_uuid)?.ok_or(IntegrationError::NotFound)?;
    let mut value = encode_record(stored.into_record())?;
    if let Some(object) = value.as_object_mut() {
        object.insert("credential_version".to_string(), Value::from(version));
    }
    Ok(Json(value))
}

/// GET /api/bots/:bot_id/integration-connections/:connection_id/events
pub async fn list_events(
    State(state): State<Arc<IntegrationState>>,
    user: AuthenticatedUser,
    Path((bot_id, connection_id)): Path<(String, String)>,
) -> Result<Json<Value>, IntegrationError> {
    let bot_uuid = parse_bot_id(&bot_id)?;
    let connection_uuid = parse_connection_id(&connection_id)?;
    let scope = resolve_scope(&state.pool, &user, bot_uuid)?;
    let mut conn = state.pool.get()?;
    if repo::get(&mut conn, &scope, connection_uuid)?.is_none() {
        return Err(IntegrationError::NotFound);
    }
    let events = repo::list_events(&mut conn, &scope, connection_uuid)?;
    let items = events
        .into_iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<Value>, _>>()
        .map_err(|error| IntegrationError::Storage(format!("event encode failed: {error}")))?;
    Ok(Json(serde_json::json!({ "items": items })))
}
