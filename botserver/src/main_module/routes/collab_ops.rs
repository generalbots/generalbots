// Server-authoritative operation log with Lamport clocks and version vectors.
//
// This replaces last-write-wins for concurrent collaborative editing. Editors
// submit operations tagged with the version they observed (`base_version`);
// when that base is behind the resource's converged version, the op is a
// concurrent conflict that must be resolved (accept-server / accept-client).
//
// Every op carries an `actor_type` (`human` | `llm`) so a concurrent AI agent's
// edits are ordered, filtered and visualized separately from a person's.

use super::collab_routes::{collab_user_id, collab_user_name, sanitize_resource};
use crate::security::auth_api::types::AuthenticatedUser;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Extension, Router,
};
use botcore::shared::state::AppState;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool, Text, Timestamptz, Uuid as SqlUuid};
use diesel::PgConnection;
use log::warn;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    warn!("collab ops error ({}): {msg}", status.as_u16());
    (status, Json(serde_json::json!({ "error": msg })))
}

fn normalize_actor_type(actor_type: &str) -> &'static str {
    if actor_type == "llm" {
        "llm"
    } else {
        "human"
    }
}

// ---------------------------------------------------------------------------
// Request types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct SubmitOpBody {
    pub resource_type: String,
    pub resource_id: String,
    pub op_type: String,
    pub base_version: i64,
    #[serde(default)]
    pub actor_type: String,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Deserialize)]
pub struct OpsQuery {
    pub resource_type: String,
    pub resource_id: String,
    #[serde(default)]
    pub since: Option<i64>,
}

#[derive(Deserialize)]
pub struct ConflictQuery {
    pub resource_type: String,
    pub resource_id: String,
}

#[derive(Deserialize)]
pub struct ResolveBody {
    /// "accept-server" keeps the newer server state; "accept-client" rebases
    /// the conflicting op on top of it.
    pub resolution: String,
}

// ---------------------------------------------------------------------------
// DB helpers (all return diesel::result::Error so they compose in a txn)
// ---------------------------------------------------------------------------

fn read_current_version(
    conn: &mut PgConnection,
    ty: &str,
    id: &str,
) -> Result<i64, diesel::result::Error> {
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = BigInt)]
        current_version: i64,
    }
    let opt = diesel::sql_query(
        "SELECT current_version FROM collab_resource_state \
         WHERE resource_type = $1 AND resource_id = $2",
    )
    .bind::<Text, _>(ty)
    .bind::<Text, _>(id)
    .get_result::<Row>(conn)
    .optional()?;
    Ok(opt.map(|r| r.current_version).unwrap_or(0))
}

fn read_vector(
    conn: &mut PgConnection,
    ty: &str,
    id: &str,
) -> Result<HashMap<String, i64>, diesel::result::Error> {
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        version_vector: String,
    }
    let opt = diesel::sql_query(
        "SELECT version_vector FROM collab_resource_state \
         WHERE resource_type = $1 AND resource_id = $2",
    )
    .bind::<Text, _>(ty)
    .bind::<Text, _>(id)
    .get_result::<Row>(conn)
    .optional()?;
    Ok(opt
        .and_then(|r| serde_json::from_str::<HashMap<String, i64>>(&r.version_vector).ok())
        .unwrap_or_default())
}

fn upsert_state(
    conn: &mut PgConnection,
    ty: &str,
    id: &str,
    current_version: i64,
    vector_json: &str,
) -> Result<usize, diesel::result::Error> {
    diesel::sql_query(
        "INSERT INTO collab_resource_state \
         (resource_type, resource_id, current_version, version_vector, updated_at) \
         VALUES ($1, $2, $3, $4, NOW()) \
         ON CONFLICT (resource_type, resource_id) \
         DO UPDATE SET current_version = $3, version_vector = $4, updated_at = NOW()",
    )
    .bind::<Text, _>(ty)
    .bind::<Text, _>(id)
    .bind::<BigInt, _>(current_version)
    .bind::<Text, _>(vector_json)
    .execute(conn)
}

fn insert_op(
    conn: &mut PgConnection,
    ty: &str,
    id: &str,
    actor_id: &str,
    actor_name: &str,
    actor_type: &str,
    op_type: &str,
    base_version: i64,
    lamport: i64,
    payload_json: &str,
    conflict: bool,
) -> Result<String, diesel::result::Error> {
    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        id: String,
    }
    diesel::sql_query(
        "INSERT INTO collab_ops \
         (resource_type, resource_id, actor_id, actor_name, actor_type, op_type, \
          base_version, lamport_ts, payload, conflict) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
         RETURNING id::text",
    )
    .bind::<Text, _>(ty)
    .bind::<Text, _>(id)
    .bind::<Text, _>(actor_id)
    .bind::<Text, _>(actor_name)
    .bind::<Text, _>(actor_type)
    .bind::<Text, _>(op_type)
    .bind::<BigInt, _>(base_version)
    .bind::<BigInt, _>(lamport)
    .bind::<Text, _>(payload_json)
    .bind::<Bool, _>(conflict)
    .get_result::<Row>(conn)
    .map(|r| r.id)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `POST /api/collab/ops` — submit an operation. Returns the op id, its Lamport
/// timestamp, the new converged version, and whether it conflicted.
pub async fn submit_op(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(body): Json<SubmitOpBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&body.resource_type, &body.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    if body.base_version < 0 {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid base_version"));
    }
    if body.op_type.trim().is_empty() || body.op_type.len() > 32 {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid op_type"));
    }

    let ty = body.resource_type;
    let id = body.resource_id;
    let op_type = body.op_type;
    let base_version = body.base_version;
    let actor_type = normalize_actor_type(&body.actor_type);
    let actor_id = collab_user_id(&user);
    let actor_name = collab_user_name(&user);
    let payload_json = serde_json::to_string(&body.payload).unwrap_or_else(|_| "{}".to_string());

    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let outcome: Result<serde_json::Value, diesel::result::Error> = conn.transaction(|tx| {
        let current = read_current_version(tx, &ty, &id)?;
        let conflict = base_version < current;
        let lamport = current.max(base_version) + 1;

        let op_id = insert_op(
            tx,
            &ty,
            &id,
            &actor_id,
            &actor_name,
            actor_type,
            &op_type,
            base_version,
            lamport,
            &payload_json,
            conflict,
        )?;

        let mut vector = read_vector(tx, &ty, &id)?;
        vector.insert(actor_id.clone(), lamport);
        let vector_json = serde_json::to_string(&vector).unwrap_or_else(|_| "{}".to_string());

        // A conflicting op does not advance the converged state — resolution
        // decides whether the editor's change is applied on top (accept-client)
        // or discarded in favour of the newer server state (accept-server).
        let new_version = if conflict { current } else { lamport };
        upsert_state(tx, &ty, &id, new_version, &vector_json)?;

        Ok(serde_json::json!({
            "op_id": op_id,
            "lamport": lamport,
            "current_version": new_version,
            "conflict": conflict,
            "base_version": base_version,
        }))
    });

    let body = outcome.map_err(|e| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            &format!("submit op: {e}"),
        )
    })?;

    Ok(Json(body))
}

/// `GET /api/collab/ops?resource_type=&resource_id=&since=` — the op-log since a
/// Lamport timestamp (exclusive), or the whole log when omitted.
pub async fn list_ops(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(params): Query<OpsQuery>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Text)]
        actor_id: String,
        #[diesel(sql_type = Text)]
        actor_name: String,
        #[diesel(sql_type = Text)]
        actor_type: String,
        #[diesel(sql_type = Text)]
        op_type: String,
        #[diesel(sql_type = BigInt)]
        base_version: i64,
        #[diesel(sql_type = BigInt)]
        lamport_ts: i64,
        #[diesel(sql_type = Text)]
        payload: String,
        #[diesel(sql_type = Bool)]
        conflict: bool,
        #[diesel(sql_type = Bool)]
        resolved: bool,
        #[diesel(sql_type = Text)]
        resolution: String,
        #[diesel(sql_type = Timestamptz)]
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let since = params.since.unwrap_or(0);
    let rows = diesel::sql_query(
        "SELECT id::text, actor_id, actor_name, actor_type, op_type, base_version, \
                lamport_ts, payload, conflict, resolved, resolution, created_at \
         FROM collab_ops \
         WHERE resource_type = $1 AND resource_id = $2 AND lamport_ts > $3 \
         ORDER BY lamport_ts ASC",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .bind::<BigInt, _>(since)
    .load::<Row>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("list ops: {e}")))?;

    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let payload = serde_json::from_str::<serde_json::Value>(&r.payload).unwrap_or(serde_json::Value::Null);
            serde_json::json!({
                "id": r.id,
                "actor_id": r.actor_id,
                "actor_name": r.actor_name,
                "actor_type": r.actor_type,
                "op_type": r.op_type,
                "base_version": r.base_version,
                "lamport": r.lamport_ts,
                "payload": payload,
                "conflict": r.conflict,
                "resolved": r.resolved,
                "resolution": r.resolution,
                "created_at": r.created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(items))
}

/// `GET /api/collab/conflicts?resource_type=&resource_id=` — unresolved
/// concurrent ops awaiting a resolution decision.
pub async fn list_conflicts(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(params): Query<ConflictQuery>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        id: String,
        #[diesel(sql_type = Text)]
        actor_id: String,
        #[diesel(sql_type = Text)]
        actor_name: String,
        #[diesel(sql_type = Text)]
        actor_type: String,
        #[diesel(sql_type = Text)]
        op_type: String,
        #[diesel(sql_type = BigInt)]
        base_version: i64,
        #[diesel(sql_type = BigInt)]
        lamport_ts: i64,
        #[diesel(sql_type = Text)]
        payload: String,
        #[diesel(sql_type = Timestamptz)]
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let rows = diesel::sql_query(
        "SELECT id::text, actor_id, actor_name, actor_type, op_type, base_version, \
                lamport_ts, payload, created_at \
         FROM collab_ops \
         WHERE resource_type = $1 AND resource_id = $2 \
           AND conflict = TRUE AND resolved = FALSE \
         ORDER BY lamport_ts ASC",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .load::<Row>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("list conflicts: {e}")))?;

    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let payload = serde_json::from_str::<serde_json::Value>(&r.payload).unwrap_or(serde_json::Value::Null);
            serde_json::json!({
                "id": r.id,
                "actor_id": r.actor_id,
                "actor_name": r.actor_name,
                "actor_type": r.actor_type,
                "op_type": r.op_type,
                "base_version": r.base_version,
                "lamport": r.lamport_ts,
                "payload": payload,
                "created_at": r.created_at.to_rfc3339(),
            })
        })
        .collect();

    Ok(Json(items))
}

/// `POST /api/collab/conflicts/:id/resolve` — resolve a concurrent op.
/// `accept-server` discards the editor change; `accept-client` rebases it on
/// top of the converged state (advancing the resource version).
pub async fn resolve_conflict(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(id): Path<uuid::Uuid>,
    Json(body): Json<ResolveBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let resolution = match body.resolution.as_str() {
        "accept-server" => "rejected",
        "accept-client" => "accepted",
        _ => return Err(err(StatusCode::BAD_REQUEST, "resolution must be accept-server or accept-client")),
    };

    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    // Fetch the op's Lamport timestamp and resource address first.
    #[derive(QueryableByName)]
    struct OpRef {
        #[diesel(sql_type = Text)]
        resource_type: String,
        #[diesel(sql_type = Text)]
        resource_id: String,
        #[diesel(sql_type = Text)]
        actor_id: String,
        #[diesel(sql_type = BigInt)]
        lamport_ts: i64,
    }
    let op = diesel::sql_query(
        "SELECT resource_type, resource_id, actor_id, lamport_ts FROM collab_ops WHERE id = $1",
    )
    .bind::<SqlUuid, _>(id)
    .get_result::<OpRef>(&mut conn)
    .optional()
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("load op: {e}")))?;

    let Some(op) = op else {
        return Err(err(StatusCode::NOT_FOUND, "Operation not found"));
    };

    let updated = diesel::sql_query(
        "UPDATE collab_ops SET resolved = TRUE, resolution = $2 \
         WHERE id = $1 AND conflict = TRUE AND resolved = FALSE",
    )
    .bind::<SqlUuid, _>(id)
    .bind::<Text, _>(resolution)
    .execute(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("resolve op: {e}")))?;

    if updated == 0 {
        return Err(err(StatusCode::CONFLICT, "Operation is not an unresolved conflict"));
    }

    // accept-client advances the converged version past the applied change and
    // records the rebased op in the version vector.
    if resolution == "accepted" {
        let current = read_current_version(&mut conn, &op.resource_type, &op.resource_id)
            .unwrap_or(0);
        let new_version = current.max(op.lamport_ts);
        let mut vector = read_vector(&mut conn, &op.resource_type, &op.resource_id).unwrap_or_default();
        vector.insert(op.actor_id.clone(), op.lamport_ts);
        let vector_json = serde_json::to_string(&vector).unwrap_or_else(|_| "{}".to_string());
        if let Err(e) = upsert_state(&mut conn, &op.resource_type, &op.resource_id, new_version, &vector_json) {
            warn!("resolve op: state upsert failed: {e}");
        }
    }

    Ok(Json(serde_json::json!({ "success": true, "resolution": resolution })))
}

/// `GET /api/collab/ops/actors?resource_type=&resource_id=` — distinct actors
/// grouped by `actor_type` (human vs llm) with their last Lamport timestamp and
/// op count, so a client can render who (or which AI) edited concurrently.
pub async fn list_actors(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(params): Query<ConflictQuery>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        actor_id: String,
        #[diesel(sql_type = Text)]
        actor_name: String,
        #[diesel(sql_type = Text)]
        actor_type: String,
        #[diesel(sql_type = BigInt)]
        last_ts: i64,
        #[diesel(sql_type = BigInt)]
        op_count: i64,
    }

    let rows = diesel::sql_query(
        "SELECT actor_id, MAX(actor_name) AS actor_name, actor_type, \
                MAX(lamport_ts) AS last_ts, COUNT(*) AS op_count \
         FROM collab_ops \
         WHERE resource_type = $1 AND resource_id = $2 \
         GROUP BY actor_id, actor_type \
         ORDER BY last_ts DESC",
    )
    .bind::<Text, _>(&params.resource_type)
    .bind::<Text, _>(&params.resource_id)
    .load::<Row>(&mut conn)
    .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("list actors: {e}")))?;

    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "actor_id": r.actor_id,
                "actor_name": r.actor_name,
                "actor_type": r.actor_type,
                "last_lamport": r.last_ts,
                "op_count": r.op_count,
            })
        })
        .collect();

    Ok(Json(items))
}

/// `GET /api/collab/ops/state?resource_type=&resource_id=` — the converged
/// version + version vector, so a client can base its next op correctly.
pub async fn ops_state(
    State(state): State<Arc<AppState>>,
    Extension(_user): Extension<AuthenticatedUser>,
    Query(params): Query<ConflictQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if !sanitize_resource(&params.resource_type, &params.resource_id) {
        return Err(err(StatusCode::BAD_REQUEST, "Invalid resource_type/resource_id"));
    }
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &format!("db pool: {e}")))?;

    let current = read_current_version(&mut conn, &params.resource_type, &params.resource_id)
        .unwrap_or(0);
    let vector = read_vector(&mut conn, &params.resource_type, &params.resource_id)
        .unwrap_or_default();

    Ok(Json(serde_json::json!({
        "resource_type": params.resource_type,
        "resource_id": params.resource_id,
        "current_version": current,
        "version_vector": vector,
    })))
}

pub fn configure_collab_ops_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/collab/ops", post(submit_op).get(list_ops))
        .route("/api/collab/ops/state", get(ops_state))
        .route("/api/collab/ops/actors", get(list_actors))
        .route("/api/collab/conflicts", get(list_conflicts))
        .route("/api/collab/conflicts/:id/resolve", post(resolve_conflict))
}
