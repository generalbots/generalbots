use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::schema::marketing_lists;
use crate::segment::{
    self, Segment, SegmentCondition, SegmentError, SegmentMatch, SegmentStoreError, WirePreviewRequest,
    FIELD_CATALOG,
};
use crate::state::AppState;

/// Sanitised HTTP error body for this module's handlers. Every internal failure
/// reaches this tuple through `internal_error`, which calls
/// `log_and_sanitize_str` first; validation failures carry typed `SegmentError`
/// JSON. Named once so the #2414 migration to a typed envelope stays local.
type ApiError = (StatusCode, String);

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = marketing_lists)]
pub struct MarketingList {
    pub id: Uuid,
    pub org_id: Option<Uuid>,
    pub bot_id: Option<Uuid>,
    pub name: String,
    pub list_type: String,
    pub query_text: Option<String>,
    pub contact_count: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub branch_id: Uuid,
    pub description: Option<String>,
    pub member_count: Option<i32>,
    pub is_dynamic: Option<bool>,
    pub criteria: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct CreateListRequest {
    pub name: String,
    pub list_type: String,
    pub query_text: Option<String>,
    /// Structured segment (#2537). When absent, `query_text` is compiled into
    /// the same model so legacy lists keep working.
    pub segment: Option<Segment>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateListRequest {
    pub name: Option<String>,
    pub list_type: Option<String>,
    pub query_text: Option<String>,
    pub segment: Option<Segment>,
}

#[derive(Debug, Deserialize)]
pub struct PreviewListRequest {
    /// Optional unsaved segment to preview before storing it.
    pub segment: Option<Segment>,
}

fn internal_error(error: &dyn std::fmt::Display, context: &str) -> ApiError {
    let sanitized =
        botsecurity_core::error_sanitizer::log_and_sanitize_str(&error.to_string(), context, None);
    (StatusCode::INTERNAL_SERVER_ERROR, sanitized.error)
}

fn validation_error(error: &SegmentError) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        serde_json::to_string(error).unwrap_or_else(|_| error.to_string()),
    )
}

fn segment_error(error: &SegmentStoreError) -> ApiError {
    match error {
        SegmentStoreError::Validation(validation) => validation_error(validation),
        SegmentStoreError::Database(database) => internal_error(database, "segment_database"),
        SegmentStoreError::Serialization(serialization) => {
            internal_error(serialization, "segment_serialization")
        }
    }
}

fn catch_all_segment() -> Segment {
    Segment {
        match_mode: SegmentMatch::All,
        groups: vec![SegmentCondition {
            field: "status".to_string(),
            op: "is_not_empty".to_string(),
            value: Value::Null,
        }],
        meta: None,
    }
}

pub async fn list_lists(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<MarketingList>>, ApiError> {
    let mut conn = state.conn.get().map_err(|e| internal_error(&e, "list_lists_pool"))?;
    let (_, branch_id, _) = state.get_scope();

    let lists: Vec<MarketingList> = marketing_lists::table
        .filter(marketing_lists::branch_id.eq(branch_id))
        .order(marketing_lists::created_at.desc())
        .load(&mut conn)
        .map_err(|e| internal_error(&e, "list_lists_query"))?;

    Ok(Json(lists))
}

pub async fn get_list(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<MarketingList>, ApiError> {
    let mut conn = state.conn.get().map_err(|e| internal_error(&e, "get_list_pool"))?;

    let list: MarketingList = marketing_lists::table
        .filter(marketing_lists::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "List not found".to_string()))?;

    Ok(Json(list))
}

pub async fn create_list(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateListRequest>,
) -> Result<Json<MarketingList>, ApiError> {
    let mut conn = state.conn.get().map_err(|e| internal_error(&e, "create_list_pool"))?;
    let (_, branch_id, _) = state.get_scope();

    let has_segment = req.segment.is_some();
    let is_dynamic = has_segment || req.list_type == "dynamic";
    let segment = match req.segment {
        Some(segment) => segment,
        None => match req.query_text.as_deref() {
            Some(text) if !text.trim().is_empty() => segment::compile_legacy_query(text),
            _ => catch_all_segment(),
        },
    };
    segment::parse_segment(&segment).map_err(|e| validation_error(&e))?;
    let criteria = serde_json::to_value(&segment)
        .map_err(|e| internal_error(&e, "create_list_segment"))?;

    let id = Uuid::new_v4();
    let now = Utc::now();
    let list = MarketingList {
        id,
        branch_id,
        name: req.name,
        list_type: req.list_type,
        query_text: req.query_text,
        criteria: Some(criteria),
        is_dynamic: Some(is_dynamic),
        member_count: None,
        description: None,
        contact_count: Some(0),
        created_at: Some(now),
        updated_at: Some(now),
        org_id: None,
        bot_id: None,
    };

    diesel::insert_into(marketing_lists::table)
        .values(&list)
        .execute(&mut conn)
        .map_err(|e| internal_error(&e, "create_list_insert"))?;

    Ok(Json(list))
}

pub async fn update_list(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateListRequest>,
) -> Result<Json<MarketingList>, ApiError> {
    let mut conn = state.conn.get().map_err(|e| internal_error(&e, "update_list_pool"))?;
    let now = Utc::now();

    if let Some(name) = req.name {
        diesel::update(marketing_lists::table.filter(marketing_lists::id.eq(id)))
            .set(marketing_lists::name.eq(name))
            .execute(&mut conn)
            .map_err(|e| internal_error(&e, "update_list_name"))?;
    }
    if let Some(list_type) = req.list_type {
        diesel::update(marketing_lists::table.filter(marketing_lists::id.eq(id)))
            .set(marketing_lists::list_type.eq(list_type))
            .execute(&mut conn)
            .map_err(|e| internal_error(&e, "update_list_type"))?;
    }

    let segment = match req.segment {
        Some(segment) => Some(segment),
        None => req.query_text.as_deref().map(|text| {
            if text.trim().is_empty() {
                catch_all_segment()
            } else {
                segment::compile_legacy_query(text)
            }
        }),
    };
    if let Some(segment) = segment {
        segment::parse_segment(&segment).map_err(|e| validation_error(&e))?;
        let criteria = serde_json::to_value(&segment)
            .map_err(|e| internal_error(&e, "update_list_segment"))?;
        diesel::update(marketing_lists::table.filter(marketing_lists::id.eq(id)))
            .set((
                marketing_lists::criteria.eq(Some(criteria)),
                marketing_lists::is_dynamic.eq(Some(true)),
            ))
            .execute(&mut conn)
            .map_err(|e| internal_error(&e, "update_list_criteria"))?;
    }
    if let Some(query_text) = req.query_text {
        diesel::update(marketing_lists::table.filter(marketing_lists::id.eq(id)))
            .set(marketing_lists::query_text.eq(Some(query_text)))
            .execute(&mut conn)
            .map_err(|e| internal_error(&e, "update_list_query_text"))?;
    }

    diesel::update(marketing_lists::table.filter(marketing_lists::id.eq(id)))
        .set(marketing_lists::updated_at.eq(now))
        .execute(&mut conn)
        .map_err(|e| internal_error(&e, "update_list_timestamp"))?;

    get_list(State(state), Path(id)).await
}

pub async fn delete_list(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let mut conn = state.conn.get().map_err(|e| internal_error(&e, "delete_list_pool"))?;

    diesel::delete(marketing_lists::table.filter(marketing_lists::id.eq(id)))
        .execute(&mut conn)
        .map_err(|e| internal_error(&e, "delete_list_delete"))?;

    Ok(Json(json!({ "status": "deleted" })))
}

/// `POST /api/crm/lists/:id/refresh` — replaces membership with the current
/// eligible set and records the refresh snapshot in `criteria.meta`.
pub async fn refresh_marketing_list(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let mut conn = state.conn.get().map_err(|e| internal_error(&e, "refresh_pool"))?;
    let (_, branch_id, _) = state.get_scope();

    let list: MarketingList = marketing_lists::table
        .filter(marketing_lists::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "List not found".to_string()))?;

    let resolved = segment::segment_from_list(&list).map_err(|e| segment_error(&e))?;
    // Drift is measured against the snapshot stored at the last refresh,
    // before this run replaces it. Otherwise it would always read as clean.
    let current_members = segment::current_member_count(&mut conn, id)
        .map_err(|e| internal_error(&e, "refresh_member_count"))?;
    let drift = segment::detect_drift(&resolved, resolved.meta.as_ref(), current_members);
    let result = segment::materialize_segment(&mut conn, branch_id, &list, &resolved)
        .map_err(|e| segment_error(&e))?;

    Ok(Json(json!({
        "status": "refreshed",
        "list_id": id,
        "matched_count": result.matched_count,
        "suppressed_count": result.suppressed_count,
        "member_count": result.member_count,
        "query_hash": result.query_hash,
        "last_refreshed_at": result.last_refreshed_at,
        "drift": drift,
    })))
}

/// `POST /api/crm/lists/:id/preview` — counts matches and suppressed matches
/// and returns at most ten samples without writing any membership row. An
/// optional body can preview an unsaved segment from the editor.
pub async fn preview_list(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    body: Option<Json<PreviewListRequest>>,
) -> Result<Json<Value>, ApiError> {
    let mut conn = state.conn.get().map_err(|e| internal_error(&e, "preview_pool"))?;
    let (_, branch_id, _) = state.get_scope();

    let list: MarketingList = marketing_lists::table
        .filter(marketing_lists::id.eq(id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "List not found".to_string()))?;

    let resolved = match body.and_then(|Json(request)| request.segment) {
        Some(segment) => segment,
        None => segment::segment_from_list(&list).map_err(|e| segment_error(&e))?,
    };
    let preview = segment::preview_segment(&mut conn, branch_id, &resolved)
        .map_err(|e| segment_error(&e))?;
    let stored_members = segment::current_member_count(&mut conn, id)
        .map_err(|e| internal_error(&e, "preview_member_count"))?;
    let drift = segment::detect_drift(&resolved, resolved.meta.as_ref(), stored_members);

    Ok(Json(json!({
        "list_id": id,
        "segment": resolved,
        "preview": preview,
        "drift": drift,
    })))
}

/// `GET /api/crm/lists/segment/fields` — the field catalog the editor uses to
/// build valid conditions (kept in sync with the evaluator by construction).
pub async fn segment_fields() -> Json<Value> {
    Json(json!({ "fields": FIELD_CATALOG }))
}

/// `POST /api/crm/segments/preview` — validates a recursive UI segment and
/// returns `{ matched, suppressed, matches }` (first ten) without writing any
/// membership row. Falls back to no data when the database pool is exhausted.
pub async fn segment_preview(
    State(state): State<Arc<AppState>>,
    Json(request): Json<WirePreviewRequest>,
) -> Result<Json<Value>, ApiError> {
    let mut conn = state.conn.get().map_err(|e| internal_error(&e, "segment_preview_pool"))?;
    let (_, branch_id, _) = state.get_scope();
    let preview = segment::preview_wire_segment(&mut conn, branch_id, &request.segment)
        .map_err(|e| segment_error(&e))?;
    Ok(Json(json!({
        "matched": preview.matched,
        "suppressed": preview.suppressed,
        "matches": preview.matches,
    })))
}
