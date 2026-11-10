//! Memory HTTP surface (routes per the shared API contract):
//! - `GET    /api/memory/items?kind=&q=&scope=&offset=` — filtered listing
//! - `POST   /api/memory/items`                        — create (dedupe aware)
//! - `PUT    /api/memory/items/{id}`                   — partial update / pin
//! - `DELETE /api/memory/items/{id}`                   — delete owned row
//! - `POST   /api/memory/import`                       — bulk import / dry-run
//! - `GET    /api/memory/export`                       — export live memories
//!
//! Authorization is owner-only: reads and writes require a **verified** JWT and
//! target rows owned by the caller. Branch-shared rows (`scope = 'branch'`)
//! additionally appear in listings when their branch matches the verified
//! branch claim; writes never cross ownership.
//!
//! #2123 — the bearer token used to be base64-decoded on its own, so a forged
//! payload (`alg: none`, or any key) chose its own `sub`, `org_id` and
//! `branch_id`. [`botsecurity_core::verified_jwt::verify_saas_jwt`] checks the
//! HS256 signature in constant time, the algorithm and the `exp`/`nbf` window
//! before a single claim is read; anything it refuses is a 401.

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post, put},
    Json, Router,
};
use botsecurity_core::verified_jwt::{bearer_token, verify_saas_jwt};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::models::{ImportBody, MemoryBody, UpdateMemoryBody};
use crate::store::{self, DedupeOutcome};
use crate::MemoryService;

/// The caller identity carried by a token whose signature and expiry verified.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Caller {
    user_id: Uuid,
    org_id: Option<Uuid>,
    branch_id: Option<Uuid>,
}

/// Resolves the caller from the `Authorization: Bearer` header, or `None`.
///
/// Fails closed: an unverifiable token (wrong key, `alg: none`, expired,
/// missing `exp`, malformed) yields `None`, never the unverified payload.
fn caller_from_headers(headers: &HeaderMap) -> Option<Caller> {
    let verified = verify_saas_jwt(bearer_token(headers)?).ok()?;
    let user_id = Uuid::parse_str(verified.sub.trim()).ok()?;
    Some(Caller {
        user_id,
        org_id: verified.org_id,
        branch_id: verified.branch_id,
    })
}

fn require_caller(headers: &HeaderMap) -> Result<Caller, (StatusCode, String)> {
    caller_from_headers(headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Authentication required".to_string()))
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
    let caller = require_caller(&headers)?;
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
        store::list(&mut conn, caller.user_id, caller.branch_id, filter, offset)
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
    let caller = require_caller(&headers)?;
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
                org_id: caller.org_id,
                branch_id: caller.branch_id,
                owner_user_id: caller.user_id,
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
    let caller = require_caller(&headers)?;
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
        store::update(&mut conn, memory_id, caller.user_id, &patch)
            .map_err(|e| internal("update", e))?
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
    let caller = require_caller(&headers)?;
    let deleted = {
        let mut conn = service
            .pool
            .get()
            .map_err(|e| internal("db pool", e))?;
        store::delete(&mut conn, memory_id, caller.user_id).map_err(|e| internal("delete", e))?
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
    let caller = require_caller(&headers)?;
    if body.items.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "items must not be empty".to_string()));
    }

    let report = {
        let mut conn = service
            .pool
            .get()
            .map_err(|e| internal("db pool", e))?;
        crate::import::import(
            &mut conn,
            caller.user_id,
            caller.org_id,
            caller.branch_id,
            &body.items,
            body.dry_run,
        )
        .map_err(|e| internal("import", e))?
    };

    Ok(Json(json!({ "report": report, "dry_run": body.dry_run })))
}

/// `GET /api/memory/export`
async fn export_items(
    State(service): State<Arc<MemoryService>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, String)> {
    let caller = require_caller(&headers)?;
    let items = {
        let mut conn = service
            .pool
            .get()
            .map_err(|e| internal("db pool", e))?;
        store::export_all(&mut conn, caller.user_id).map_err(|e| internal("export", e))?
    };

    Ok(Json(json!({ "items": items, "count": items.len() })))
}

fn labeled_memory(status: &str, memory: crate::models::UserMemory) -> Value {
    json!({ "status": status, "memory": memory })
}

fn bad_request(message: String) -> (StatusCode, String) {
    (StatusCode::BAD_REQUEST, message)
}

#[cfg(test)]
#[path = "api_tests.rs"]
mod api_tests;
