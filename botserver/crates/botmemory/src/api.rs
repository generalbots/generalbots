//! Memory HTTP surface (routes per the shared API contract):
//! - `GET    /api/memory/items?kind=&q=&scope=&offset=` — filtered listing
//! - `POST   /api/memory/items`                        — create (dedupe aware)
//! - `PUT    /api/memory/items/{id}`                   — partial update / pin
//! - `DELETE /api/memory/items/{id}`                   — delete owned row
//! - `POST   /api/memory/import`                       — bulk import / dry-run
//! - `GET    /api/memory/export`                       — export live memories
//!
//! Authorization is owner-only: reads and writes require a JWT user claim and
//! target rows owned by the caller. Branch-shared rows (`scope = 'branch'`)
//! additionally appear in listings when their branch matches the JWT branch
//! claim; writes never cross ownership.

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post, put},
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::models::{ImportBody, MemoryBody, UpdateMemoryBody};
use crate::store::{self, DedupeOutcome};
use crate::MemoryService;

const B64_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn b64_decode_flexible(input: &str) -> Option<Vec<u8>> {
    let mut normalized = String::with_capacity(input.len());
    for c in input.trim().chars() {
        match c {
            '-' => normalized.push('+'),
            '_' => normalized.push('/'),
            c if !c.is_whitespace() => normalized.push(c),
            _ => {}
        }
    }
    if normalized.len() % 4 == 1 {
        return None;
    }
    let mut out = Vec::with_capacity(normalized.len() / 4 * 3);
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    for byte in normalized.bytes() {
        if byte == b'=' {
            break;
        }
        let value = match byte {
            b'A'..=b'Z' => (byte - b'A') as u32,
            b'a'..=b'z' => (byte - b'a' + 26) as u32,
            b'0'..=b'9' => (byte - b'0' + 52) as u32,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        };
        acc = (acc << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((acc >> bits) & 0xFF) as u8);
        }
    }
    Some(out)
}

fn jwt_claims(headers: &HeaderMap) -> Option<Value> {
    let token = headers
        .get("authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")?
        .trim();
    let payload = token.split('.').nth(1)?;
    let decoded = b64_decode_flexible(payload)?;
    serde_json::from_slice(&decoded).ok()
}

fn claim_uuid(claims: &Value, keys: &[&str]) -> Option<Uuid> {
    for key in keys {
        if let Some(raw) = claims.get(key).and_then(|v| v.as_str()) {
            if let Ok(id) = Uuid::parse_str(raw) {
                return Some(id);
            }
        }
    }
    None
}

fn jwt_user_id(headers: &HeaderMap) -> Option<Uuid> {
    let claims = jwt_claims(headers)?;
    claim_uuid(&claims, &["user_id", "sub", "uid"])
}

fn jwt_scope(headers: &HeaderMap) -> (Option<Uuid>, Option<Uuid>) {
    let claims = jwt_claims(headers);
    match claims {
        Some(claims) => (
            claim_uuid(&claims, &["org_id", "org"]),
            claim_uuid(&claims, &["branch_id", "branch"]),
        ),
        None => (None, None),
    }
}

fn require_user(headers: &HeaderMap) -> Result<Uuid, (StatusCode, String)> {
    jwt_user_id(headers).ok_or_else(|| (StatusCode::UNAUTHORIZED, "Authentication required".to_string()))
}

fn internal(context: &str, err: impl std::fmt::Display) -> (StatusCode, String) {
    tracing::error!("memory {context} failed: {err}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Memory service error".to_string())
}

pub fn configure_routes() -> Router<Arc<MemoryService>> {
    Router::new()
        .route("/api/memory/items", get(list_items).post(create_item))
        .route("/api/memory/items/:id", put(update_item).delete(delete_item))
        .route("/api/memory/import", post(import_items))
        .route("/api/memory/export", get(export_items))
}

/// `GET /api/memory/items?kind=&q=&scope=&offset=`
async fn list_items(
    State(service): State<Arc<MemoryService>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let owner = require_user(&headers)?;
    let (_, branch_claim) = jwt_scope(&headers);
    let filter = store::ListFilter {
        kind: params.get("kind").map(String::as_str),
        q: params.get("q").map(String::as_str),
        scope: params.get("scope").map(String::as_str),
    };
    let offset = params.get("offset").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);

    let items = {
        let mut conn = service
            .pool
            .get()
            .map_err(|e| internal("db pool", e))?;
        store::list(&mut conn, owner, branch_claim, filter, offset)
            .map_err(|e| internal("list", e))?
    };

    Ok(Json(json!({ "items": items, "limit": store::LIST_LIMIT, "offset": offset })))
}

/// `POST /api/memory/items`
async fn create_item(
    State(service): State<Arc<MemoryService>>,
    headers: HeaderMap,
    Json(body): Json<MemoryBody>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let owner = require_user(&headers)?;
    let (org_id, branch_id) = jwt_scope(&headers);
    let scope = crate::models::ensure_scope(&body.scope).map_err(bad_request)?;
    let kind = crate::models::ensure_kind(&body.kind).map_err(bad_request)?;
    let content = crate::models::ensure_content(&body.content).map_err(bad_request)?;

    let outcome = {
        let mut conn = service
            .pool
            .get()
            .map_err(|e| internal("db pool", e))?;
        store::insert(
            &mut conn,
            store::NewMemory {
                org_id,
                branch_id,
                owner_user_id: owner,
                scope: &scope,
                kind: &kind,
                content: &content,
                source: "manual",
                confidence: 0.9,
                pinned: body.pinned,
            },
        )
        .map_err(|e| internal("create", e))?
    };

    let memory = match outcome {
        DedupeOutcome::Created(row) => labeled_memory("created", row),
        DedupeOutcome::Superseded { memory, .. } => labeled_memory("superseded", memory),
        DedupeOutcome::Skipped(row) => labeled_memory("skipped_existing", row),
    };
    Ok(Json(memory))
}

/// `PUT /api/memory/items/{id}`
async fn update_item(
    State(service): State<Arc<MemoryService>>,
    headers: HeaderMap,
    Path(memory_id): Path<Uuid>,
    Json(body): Json<UpdateMemoryBody>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let owner = require_user(&headers)?;
    let patch = store::MemoryPatch {
        scope: match body.scope.as_deref() {
            Some(raw) => Some(crate::models::ensure_scope(raw).map_err(bad_request)?),
            None => None,
        },
        kind: match body.kind.as_deref() {
            Some(raw) => Some(crate::models::ensure_kind(raw).map_err(bad_request)?),
            None => None,
        },
        content: match body.content.as_deref() {
            Some(raw) => Some(crate::models::ensure_content(raw).map_err(bad_request)?),
            None => None,
        },
        pinned: body.pinned,
    };

    let updated = {
        let mut conn = service
            .pool
            .get()
            .map_err(|e| internal("db pool", e))?;
        store::update(&mut conn, memory_id, owner, &patch).map_err(|e| internal("update", e))?
    };

    match updated {
        Some(memory) => Ok(Json(json!({ "memory": memory }))),
        None => Err((StatusCode::NOT_FOUND, "Memory not found".to_string())),
    }
}

/// `DELETE /api/memory/items/{id}`
async fn delete_item(
    State(service): State<Arc<MemoryService>>,
    headers: HeaderMap,
    Path(memory_id): Path<Uuid>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let owner = require_user(&headers)?;
    let deleted = {
        let mut conn = service
            .pool
            .get()
            .map_err(|e| internal("db pool", e))?;
        store::delete(&mut conn, memory_id, owner).map_err(|e| internal("delete", e))?
    };
    if !deleted {
        return Err((StatusCode::NOT_FOUND, "Memory not found".to_string()));
    }
    Ok(Json(json!({ "status": "deleted", "id": memory_id })))
}

/// `POST /api/memory/import`
async fn import_items(
    State(service): State<Arc<MemoryService>>,
    headers: HeaderMap,
    Json(body): Json<ImportBody>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let owner = require_user(&headers)?;
    let (org_id, branch_id) = jwt_scope(&headers);
    if body.items.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "items must not be empty".to_string()));
    }

    let report = {
        let mut conn = service
            .pool
            .get()
            .map_err(|e| internal("db pool", e))?;
        crate::import::import(&mut conn, owner, org_id, branch_id, &body.items, body.dry_run)
            .map_err(|e| internal("import", e))?
    };

    Ok(Json(json!({ "report": report, "dry_run": body.dry_run })))
}

/// `GET /api/memory/export`
async fn export_items(
    State(service): State<Arc<MemoryService>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, String)> {
    let owner = require_user(&headers)?;
    let items = {
        let mut conn = service
            .pool
            .get()
            .map_err(|e| internal("db pool", e))?;
        store::export_all(&mut conn, owner).map_err(|e| internal("export", e))?
    };

    Ok(Json(json!({ "items": items, "count": items.len() })))
}

fn labeled_memory(status: &str, memory: crate::models::UserMemory) -> Value {
    json!({ "status": status, "memory": memory })
}

fn bad_request(message: String) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, message)
}
