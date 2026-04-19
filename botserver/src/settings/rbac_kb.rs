//! KB-group access management handlers
//!
//! Provides endpoints to assign/remove RBAC groups to/from knowledge bases,
//! and to query which KBs a specific user may access.
//!
//! Endpoints:
//!   GET    /api/rbac/kbs/{kb_id}/groups
//!   POST   /api/rbac/kbs/{kb_id}/groups/{group_id}
//!   DELETE /api/rbac/kbs/{kb_id}/groups/{group_id}
//!   GET    /api/rbac/users/{user_id}/accessible-kbs

use crate::core::shared::models::RbacGroup;
use crate::core::shared::state::AppState;
use crate::security::error_sanitizer::log_and_sanitize_str;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::task::JoinError;
use uuid::Uuid;

/// Auxiliary row type for fetching group_id values via raw SQL.
#[derive(QueryableByName)]
pub struct GroupIdQueryRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub group_id: Uuid,
}

/// Auxiliary row type for fetching kb_collections via raw SQL.
#[derive(QueryableByName, Serialize, Deserialize)]
pub struct KbCollectionRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub bot_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub folder_path: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub qdrant_collection: String,
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub document_count: i32,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub created_at: chrono::DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub updated_at: chrono::DateTime<Utc>,
}

/// GET /api/rbac/kbs/{kb_id}/groups — list groups that have access to a KB
pub async fn get_kb_groups(
    State(state): State<Arc<AppState>>,
    Path(kb_id): Path<Uuid>,
) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result: Result<Result<Vec<RbacGroup>, String>, JoinError> = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        
        #[derive(QueryableByName)]
        struct GroupRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            name: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            display_name: String,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            description: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Bool)]
            is_active: bool,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            parent_group_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            created_by: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: chrono::DateTime<Utc>,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            updated_at: chrono::DateTime<Utc>,
        }
        
        let rows: Vec<GroupRow> = diesel::sql_query(
            "SELECT rg.id, rg.name, rg.display_name, rg.description, rg.is_active,
                    rg.parent_group_id, rg.created_by, rg.created_at, rg.updated_at
             FROM research.kb_group_associations kga
             JOIN core.rbac_groups rg ON rg.id = kga.group_id
             WHERE kga.kb_id = $1 AND rg.is_active = true"
        )
        .bind::<diesel::sql_types::Uuid, _>(kb_id)
        .load(&mut db_conn)
        .map_err(|e| format!("Query error: {e}"))?;
        
        let groups: Vec<RbacGroup> = rows.into_iter().map(|r| RbacGroup {
            id: r.id,
            name: r.name,
            display_name: r.display_name,
            description: r.description,
            is_active: r.is_active,
            parent_group_id: r.parent_group_id,
            created_by: r.created_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }).collect();
        
        Ok(groups)
    })
    .await;

    match result {
        Ok(Ok(groups)) => Json(serde_json::json!({ "groups": groups })).into_response(),
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "get_kb_groups", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "get_kb_groups", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

/// POST /api/rbac/kbs/{kb_id}/groups/{group_id} — grant a group access to a KB
pub async fn assign_kb_to_group(
    State(state): State<Arc<AppState>>,
    Path((kb_id, group_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let conn = state.conn.clone();
    let now = Utc::now();
    let result = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::core::shared::models::kb_group_associations;
        let existing: Option<Uuid> = kb_group_associations::table
            .filter(kb_group_associations::kb_id.eq(kb_id))
            .filter(kb_group_associations::group_id.eq(group_id))
            .select(kb_group_associations::id)
            .first::<Uuid>(&mut db_conn)
            .optional()
            .map_err(|e| format!("Query error: {e}"))?;
        if existing.is_some() {
            return Err("Group already has access to this KB".to_string());
        }
        diesel::sql_query(
            "INSERT INTO kb_group_associations (id, kb_id, group_id, granted_at)
             VALUES ($1, $2, $3, $4)",
        )
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Uuid, _>(kb_id)
        .bind::<diesel::sql_types::Uuid, _>(group_id)
        .bind::<diesel::sql_types::Timestamptz, _>(now)
        .execute(&mut db_conn)
        .map_err(|e| format!("Insert error: {e}"))?;
        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            info!("Assigned KB {kb_id} to group {group_id}");
            StatusCode::CREATED.into_response()
        }
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "assign_kb_to_group", None);
            (StatusCode::BAD_REQUEST, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "assign_kb_to_group", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

/// DELETE /api/rbac/kbs/{kb_id}/groups/{group_id} — revoke a group's access to a KB
pub async fn remove_kb_from_group(
    State(state): State<Arc<AppState>>,
    Path((kb_id, group_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<(), String> {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;
        use crate::core::shared::models::kb_group_associations;
        diesel::delete(
            kb_group_associations::table
                .filter(kb_group_associations::kb_id.eq(kb_id))
                .filter(kb_group_associations::group_id.eq(group_id)),
        )
        .execute(&mut db_conn)
        .map_err(|e| format!("Delete error: {e}"))?;
        Ok(())
    })
    .await;

    match result {
        Ok(Ok(())) => {
            info!("Removed group {group_id} from KB {kb_id}");
            StatusCode::NO_CONTENT.into_response()
        }
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "remove_kb_from_group", None);
            (StatusCode::BAD_REQUEST, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "remove_kb_from_group", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}

/// GET /api/rbac/users/{user_id}/accessible-kbs — list KBs accessible to a user
pub async fn get_accessible_kbs_for_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    let conn = state.conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut db_conn = conn.get().map_err(|e| format!("DB error: {e}"))?;

        let group_ids: Vec<Uuid> = diesel::sql_query(
            "SELECT group_id FROM rbac_user_groups WHERE user_id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .load::<GroupIdQueryRow>(&mut db_conn)
        .map_err(|e| format!("Failed to get user groups: {e}"))?
        .into_iter()
        .map(|r| r.group_id)
        .collect();

        let kbs: Vec<KbCollectionRow> = if group_ids.is_empty() {
            diesel::sql_query(
                "SELECT kc.id, kc.bot_id, kc.name, kc.folder_path, kc.qdrant_collection,
                        kc.document_count, kc.created_at, kc.updated_at
                 FROM kb_collections kc
                 WHERE NOT EXISTS (
                     SELECT 1 FROM kb_group_associations kga WHERE kga.kb_id = kc.id
                 )",
            )
            .load::<KbCollectionRow>(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))?
        } else {
            diesel::sql_query(
                "SELECT kc.id, kc.bot_id, kc.name, kc.folder_path, kc.qdrant_collection,
                        kc.document_count, kc.created_at, kc.updated_at
                 FROM kb_collections kc
                 WHERE NOT EXISTS (
                     SELECT 1 FROM kb_group_associations kga WHERE kga.kb_id = kc.id
                 )
                 OR EXISTS (
                     SELECT 1 FROM kb_group_associations kga
                     WHERE kga.kb_id = kc.id
                       AND kga.group_id = ANY($1::uuid[])
                 )",
            )
            .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(group_ids)
            .load::<KbCollectionRow>(&mut db_conn)
            .map_err(|e| format!("Query error: {e}"))?
        };

        Ok::<_, String>(kbs)
    })
    .await;

    match result {
        Ok(Ok(kbs)) => Json(serde_json::json!({ "kbs": kbs })).into_response(),
        Ok(Err(e)) => {
            let sanitized = log_and_sanitize_str(&e, "get_accessible_kbs_for_user", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
        Err(e) => {
            let sanitized = log_and_sanitize_str(&e.to_string(), "get_accessible_kbs_for_user", None);
            (StatusCode::INTERNAL_SERVER_ERROR, sanitized).into_response()
        }
    }
}
