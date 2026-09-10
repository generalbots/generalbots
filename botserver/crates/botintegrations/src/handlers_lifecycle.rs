use std::sync::Arc;

use axum::extract::{Json, Path, State};
use botsecurity_auth::auth_api::types::AuthenticatedUser;
use serde::Deserialize;
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

/// Agent type names for the three Vibe LLM slots. Each agent type resolves
/// its own provider (Settings → Vibe), so a branch can route Reasoning,
/// Agentic and Fast runs to different providers.
pub const VIBE_AGENT_TYPES: [&str; 3] = ["reasoning", "agentic", "fast"];

#[derive(Debug, Deserialize)]
pub struct UseForVibeRequest {
    /// One of `reasoning` | `agentic` | `fast` (defaults to `agentic`).
    agent: Option<String>,
}

/// POST /api/bots/:bot_id/integration-connections/:connection_id/use-for-vibe
///
/// Assigns an LLM integration connection to one of the three Vibe agent
/// slots (`reasoning` | `agentic` | `fast`, default `agentic`) for its bot.
/// Resolves the provider defaults (endpoint + model) from the catalog table,
/// loads the API key strictly from Vault, then persists the resolved
/// `vibe-llm-{agent}-*` config keys through `ConfigManager` — sensitive keys
/// land in the per-bot Vault path, the rest in `bot_configuration`. The Vibe
/// agent loop reads these keys with no further integration lookups.
pub async fn use_for_vibe(
    State(state): State<Arc<IntegrationState>>,
    user: AuthenticatedUser,
    Path((bot_id, connection_id)): Path<(String, String)>,
    body: Option<Json<UseForVibeRequest>>,
) -> Result<Json<Value>, IntegrationError> {
    let agent = body
        .and_then(|Json(req)| req.agent)
        .unwrap_or_else(|| "agentic".to_string());
    if !VIBE_AGENT_TYPES.contains(&agent.as_str()) {
        return Err(IntegrationError::Validation(format!(
            "agent must be one of {}",
            VIBE_AGENT_TYPES.join(", ")
        )));
    }
    let prefix = format!("vibe-llm-{agent}-");

    let bot_uuid = parse_bot_id(&bot_id)?;
    let connection_uuid = parse_connection_id(&connection_id)?;
    let scope = resolve_scope(&state.pool, &user, bot_uuid)?;

    let row = {
        let mut conn = state.pool.get()?;
        repo::get(&mut conn, &scope, connection_uuid)?.ok_or(IntegrationError::NotFound)?
    };

    let (default_url, default_model) =
        crate::llm_providers::llm_provider_defaults(&row.provider_slug).ok_or_else(|| {
            IntegrationError::Validation(format!(
                "connection is not an LLM provider: {}",
                row.provider_slug
            ))
        })?;

    // Keyless providers (free gateway models) carry no Vault secret; the
    // connection may be stored without any secrets envelope at all.
    let api_key = if crate::llm_providers::llm_provider_keyless(&row.provider_slug) {
        String::new()
    } else {
        let credentials = state.vault.load_strict(&row.vault_path).await?;
        credentials
            .as_object()
            .and_then(|entries| {
                // Accept either the canonical `api_key` field or a generic `key`.
                entries
                    .get("api_key")
                    .or_else(|| entries.get("key"))
                    .and_then(|v| v.as_str())
            })
            .filter(|k| !k.is_empty())
            .ok_or_else(|| {
                IntegrationError::Validation("connection has no stored API key".to_string())
            })?
            .to_string()
    };

    let config = botcore::config::ConfigManager::new(state.pool.clone());
    let write = |key: &str, value: &str| {
        config.set_config(&bot_uuid, key, value).map_err(|e| {
            log::error!("use_for_vibe: cannot persist {key} for {bot_uuid}: {e}");
            IntegrationError::Storage(format!("cannot persist {key}"))
        })
    };
    write(&format!("{prefix}provider"), &row.provider_slug)?;
    write(&format!("{prefix}url"), default_url)?;
    write(&format!("{prefix}model"), default_model)?;
    write(&format!("{prefix}key"), &api_key)?;

    let mut conn = state.pool.get()?;
    record_outcome(
        &mut conn,
        &scope,
        Some(connection_uuid),
        "connection.used_for_vibe",
        "ok",
        "low",
        &serde_json::json!({
            "provider": row.provider_slug,
            "agent": agent,
            "model": default_model,
        }),
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "agent": agent,
        "provider": row.provider_slug,
        "model": default_model,
        "url": default_url,
    })))
}
