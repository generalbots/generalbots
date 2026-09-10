//! Org API keys (#1172 backend): generation (48 random chars from two UUIDv4
//! concatenations), HMAC-SHA256 storage-only hashing (#1297), authentication
//! and the `/api/cloud/orgkeys` CRUD handlers. Raw keys are returned exactly
//! once. Legacy SHA-256 hashes are accepted with a transparent re-hash on
//! successful authentication.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::{bearer_token, is_admin_claims, jwt_payload};
use crate::models::{KeyCreateBody, OrgApiKeyRow};
use crate::schema::org_api_keys;
use crate::AgentService;

/// Server-side pepper for key hashing (#1297). Required in production — the
/// value comes from the environment, never from source. When unset, an
/// ephemeral pepper is used so hashes reset on restart (keys must be re-authed
/// via the legacy path… which is also ephemeral; operators should set the env).
static KEY_HASH_PEPPER: std::sync::OnceLock<String> = std::sync::OnceLock::new();

fn key_hash_pepper() -> &'static str {
    KEY_HASH_PEPPER.get_or_init(|| {
        std::env::var("GB_API_KEY_HASH_PEPPER").unwrap_or_else(|_| {
            tracing::warn!(
                "GB_API_KEY_HASH_PEPPER not set; falling back to ephemeral pepper — hashes reset on restart"
            );
            Uuid::new_v4().to_string()
        })
    })
}

fn hmac_hash(raw: &str, pepper: &str) -> String {
    // Standard HMAC (RFC 2104) over SHA-256, implemented directly so there is
    // no panic path from key-length errors. Pepper is normalized to the
    // 64-byte block size first (hashed when longer).
    let mut block = [0u8; 64];
    let pepper_bytes = pepper.as_bytes();
    if pepper_bytes.len() > 64 {
        block[..32].copy_from_slice(&Sha256::digest(pepper_bytes));
    } else {
        block[..pepper_bytes.len()].copy_from_slice(pepper_bytes);
    }
    let ipad: Vec<u8> = block.iter().map(|b| b ^ 0x36).collect();
    let opad: Vec<u8> = block.iter().map(|b| b ^ 0x5c).collect();
    let mut inner = Sha256::new();
    inner.update(&ipad);
    inner.update(raw.as_bytes());
    let inner_digest = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(&opad);
    outer.update(inner_digest);
    hex_encode(&outer.finalize())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

/// Identity resolved from a presented API key.
#[derive(Debug, Clone)]
pub struct KeyContext {
    pub key_id: Uuid,
    pub org_id: Uuid,
    pub scopes: Vec<String>,
}

fn internal(msg: String) -> (StatusCode, String) {
    tracing::error!("{msg}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "API key operation failed".to_string(),
    )
}

fn unauthorized(msg: &str) -> (StatusCode, String) {
    (StatusCode::UNAUTHORIZED, msg.to_string())
}

/// 48 hex chars derived from two concatenated UUIDv4 simples.
pub fn generate_api_key() -> String {
    let combined = format!(
        "{}{}",
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple()
    );
    combined.chars().take(48).collect()
}

pub fn hash_key(raw: &str) -> String {
    // #1297 — plain SHA-256 over high-entropy keys is flagged as weak hashing
    // on sensitive data; HMAC-SHA256 with a server-side pepper hardens it
    // against offline rainbow/dictionary attacks on a leaked DB.
    hmac_hash(raw, key_hash_pepper())
}

/// Authenticate an `Authorization` header carrying an org API key
/// (`Bearer <key>` or the bare key). Updates last_used_at best-effort.
pub fn authenticate_key(
    header_value: &str,
    conn: &mut diesel::PgConnection,
) -> Result<KeyContext, (StatusCode, String)> {
    let token = header_value
        .strip_prefix("Bearer ")
        .or_else(|| header_value.strip_prefix("bearer "))
        .unwrap_or(header_value)
        .trim();
    if token.is_empty() || token.len() != 48 {
        return Err(unauthorized("Invalid API key"));
    }

    use crate::schema::org_api_keys::dsl::{id, key_hash, org_api_keys, revoked_at};
    // #1297 — accept legacy SHA-256 hashes during the transition window and
    // transparently re-hash with HMAC on successful authentication.
    let legacy_hash = {
        let digest = Sha256::digest(token.as_bytes());
        hex_encode(&digest)
    };
    let current_hash = hash_key(token);
    let row: OrgApiKeyRow = org_api_keys
        .filter(
            key_hash
                .eq(&current_hash)
                .or(key_hash.eq(&legacy_hash)),
        )
        .filter(revoked_at.is_null())
        .first(conn)
        .optional()
        .map_err(|e| internal(format!("org_api_keys select: {e}")))?
        .ok_or_else(|| unauthorized("Invalid API key"))?;
    if row.key_hash == legacy_hash {
        if let Err(e) = diesel::update(org_api_keys.filter(id.eq(row.id)))
            .set(key_hash.eq(&current_hash))
            .execute(conn)
        {
            tracing::warn!("org_api_keys re-hash failed for key {}: {e}", row.id);
        }
    }

    let context = KeyContext {
        key_id: row.id,
        org_id: row.org_id,
        scopes: row
            .scopes
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
    };

    // Best-effort usage stamping — never fail the request over it.
    use crate::schema::org_api_keys::dsl::{last_used_at, org_api_keys as keys};
    if let Err(e) = diesel::update(keys.filter(id.eq(row.id)))
        .set(last_used_at.eq(Utc::now()))
        .execute(conn)
    {
        tracing::warn!("last_used_at update failed for key {}: {e}", row.id);
    }
    Ok(context)
}

fn redacted(row: &OrgApiKeyRow) -> serde_json::Value {
    json!({
        "id": row.id,
        "org_id": row.org_id,
        "name": row.name,
        "key_prefix": row.key_prefix,
        "scopes": row.scopes,
        "last_used_at": row.last_used_at,
        "revoked_at": row.revoked_at,
        "created_at": row.created_at,
    })
}

enum CallerIdentity {
    Admin,
    Org(KeyContext),
}

fn resolve_caller(
    state: &AgentService,
    headers: &HeaderMap,
) -> Result<CallerIdentity, (StatusCode, String)> {
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    if let Some(token) = bearer_token(headers) {
        if let Ok(ctx) = authenticate_key(&token, &mut conn) {
            return Ok(CallerIdentity::Org(ctx));
        }
        if let Some(claims) = jwt_payload(&token) {
            if is_admin_claims(&claims) {
                return Ok(CallerIdentity::Admin);
            }
        }
    }
    Err(unauthorized("Missing or invalid credentials"))
}

/// `POST /api/cloud/orgkeys`
pub async fn create_org_key(
    State(state): State<Arc<AgentService>>,
    headers: HeaderMap,
    Json(body): Json<KeyCreateBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let caller = resolve_caller(&state, &headers)?;
    let name = body.name.trim().to_string();
    if name.is_empty() || name.chars().count() > 120 {
        return Err((StatusCode::BAD_REQUEST, "Name is required".to_string()));
    }

    let org_id = match &caller {
        CallerIdentity::Org(ctx) => ctx.org_id,
        CallerIdentity::Admin => body.org_id.ok_or_else(|| {
            (StatusCode::BAD_REQUEST, "org_id is required for admin-created keys".to_string())
        })?,
    };

    let raw = generate_api_key();
    let now: DateTime<Utc> = Utc::now();
    let row = OrgApiKeyRow {
        id: Uuid::new_v4(),
        org_id,
        name,
        key_hash: hash_key(&raw),
        key_prefix: raw.chars().take(12).collect(),
        scopes: serde_json::Value::Array(
            body.scopes.iter().map(|s| serde_json::Value::String(s.clone())).collect(),
        ),
        last_used_at: None,
        revoked_at: None,
        created_at: now,
    };

    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    diesel::insert_into(org_api_keys::table)
        .values(&row)
        .execute(&mut conn)
        .map_err(|e| internal(format!("org_api_keys insert: {e}")))?;

    tracing::info!("created org API key {} for org {org_id}", row.id);
    Ok(Json(json!({
        "status": "created",
        "item": redacted(&row),
        "key": raw,
    })))
}

/// `GET /api/cloud/orgkeys` — admins see all keys, org keys see their own org.
pub async fn list_org_keys(
    State(state): State<Arc<AgentService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let caller = resolve_caller(&state, &headers)?;

    use crate::schema::org_api_keys::dsl::{created_at, org_api_keys, org_id};
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let rows: Vec<OrgApiKeyRow> = match caller {
        CallerIdentity::Admin => org_api_keys
            .order(created_at.desc())
            .load(&mut conn)
            .map_err(|e| internal(format!("org_api_keys select: {e}")))?,
        CallerIdentity::Org(ctx) => org_api_keys
            .filter(org_id.eq(ctx.org_id))
            .order(created_at.desc())
            .load(&mut conn)
            .map_err(|e| internal(format!("org_api_keys select: {e}")))?,
    };

    Ok(Json(json!({ "items": rows.iter().map(redacted).collect::<Vec<_>>() })))
}

/// `DELETE /api/cloud/orgkeys/{id}` — soft revoke via `revoked_at`.
pub async fn revoke_org_key(
    State(state): State<Arc<AgentService>>,
    headers: HeaderMap,
    Path(key_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let caller = resolve_caller(&state, &headers)?;

    use crate::schema::org_api_keys::dsl::{id, org_api_keys, org_id, revoked_at};
    let mut conn = state.pool().get().map_err(|e| internal(format!("DB pool: {e}")))?;
    let affected = match &caller {
        CallerIdentity::Admin => diesel::update(org_api_keys.filter(id.eq(key_id)))
            .set(revoked_at.eq(Utc::now()))
            .execute(&mut conn)
            .map_err(|e| internal(format!("org_api_keys revoke: {e}")))?,
        CallerIdentity::Org(ctx) => {
            let target_org: Option<Uuid> = org_api_keys
                .filter(id.eq(key_id))
                .select(org_id)
                .first(&mut conn)
                .optional()
                .map_err(|e| internal(format!("org_api_keys select: {e}")))?;
            match target_org {
                Some(org) if org == ctx.org_id => {
                    diesel::update(org_api_keys.filter(id.eq(key_id)))
                        .set(revoked_at.eq(Utc::now()))
                        .execute(&mut conn)
                        .map_err(|e| internal(format!("org_api_keys revoke: {e}")))?
                }
                _ => return Err((StatusCode::FORBIDDEN, "Not authorized".to_string())),
            }
        }
    };

    if affected == 0 {
        return Err((StatusCode::NOT_FOUND, "API key not found".to_string()));
    }
    Ok(Json(json!({ "status": "revoked" })))
}
