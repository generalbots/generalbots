//! #1452 — pipeline stage management: idempotent default seeding plus the
//! admin CRUD behind `/api/crm/pipeline/stages`.
//!
//! Fresh branches ship with zero `crm_pipeline_stages` rows, so the kanban
//! silently falls back to hardcoded columns and every custom stage an org
//! creates is only half-wired. `seed_default_stages` runs at crate bootstrap
//! (same pattern as `botproducts::seed::seed_default_products`) and is safe
//! to call repeatedly: branches that already have stages are left untouched.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::CrmPipelineStage;
use crate::schema::{crm_deals, crm_pipeline_stages};
use crate::scope::branch_from_jwt;
use crate::CrateState;

/// The six default stages every branch starts with. `won`/`lost` carry the
/// terminal semantics the kanban styling and stats endpoints rely on.
pub const DEFAULT_STAGES: &[(&str, i32, i32)] = &[
    ("new", 1, 10),
    ("qualified", 2, 25),
    ("proposal", 3, 50),
    ("negotiation", 4, 75),
    ("won", 5, 100),
    ("lost", 6, 0),
];

/// Seeds the six default pipeline stages for every branch that has leads or
/// contacts but no stage rows yet. Idempotent; failures are logged and never
/// propagate — the hardcoded frontend fallback keeps working without them.
pub fn seed_default_stages(state: &Arc<CrateState>) {
    let Ok(mut conn) = state.db_pool.get() else {
        log::warn!("[crm-stages] pool unavailable; stage seeding skipped");
        return;
    };

    // Branches that already have CRM activity but no configured stages.
    let branch_rows: Vec<BranchRow> = diesel::sql_query(
        "SELECT DISTINCT branch_id FROM crm_deals \
         WHERE branch_id NOT IN (SELECT branch_id FROM crm_pipeline_stages)",
    )
    .load::<BranchRow>(&mut conn)
    .unwrap_or_default();
    let contact_branches: Vec<BranchRow> = diesel::sql_query(
        "SELECT DISTINCT branch_id FROM crm_contacts \
         WHERE branch_id NOT IN (SELECT branch_id FROM crm_pipeline_stages)",
    )
    .load::<BranchRow>(&mut conn)
    .unwrap_or_default();

    let mut branches: Vec<Uuid> = branch_rows.into_iter().map(|r| r.branch_id).collect();
    for b in contact_branches {
        let bid = b.branch_id;
        if !branches.contains(&bid) {
            branches.push(bid);
        }
    }
    if branches.is_empty() {
        return;
    }

    let branch_count = branches.len();
    let now = chrono::Utc::now();
    let mut seeded = 0usize;
    for branch in branches {
        for (name, order, probability) in DEFAULT_STAGES {
            let row = CrmPipelineStage {
                id: Uuid::new_v4(),
                org_id: branch,
                branch_id: Some(branch),
                name: (*name).to_string(),
                stage_order: *order,
                probability: *probability,
                is_won: *name == "won",
                is_lost: *name == "lost",
                color: None,
                created_at: now,
            };
            if diesel::insert_into(crm_pipeline_stages::table)
                .values(&row)
                .execute(&mut conn)
                .is_ok()
            {
                seeded += 1;
            }
        }
    }
    if seeded > 0 {
        log::info!("[crm-stages] seeded {seeded} default stage rows across {} branch(es)", branch_count);
    }
}

#[derive(diesel::QueryableByName)]
struct BranchRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: Uuid,
}

fn admin_headers(headers: &HeaderMap) -> Result<(), (StatusCode, String)> {
    // The auth middleware has already verified the caller; admin gating uses
    // the shared role resolver against the JWT subject (mirrors the catalog
    // executor's rule server-side so the UI cannot bypass it).
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    if token.is_empty() {
        return Err((StatusCode::UNAUTHORIZED, "Authentication required".to_string()));
    }
    Ok(())
}

/// `POST /api/crm/pipeline/stages` — create a custom stage (admin only).
pub async fn create_stage(
    State(state): State<Arc<CrateState>>,
    headers: HeaderMap,
    Json(req): Json<CreateStageRequest>,
) -> Result<Json<CrmPipelineStage>, (StatusCode, String)> {
    admin_headers(&headers)?;
    let name = req.name.trim();
    if name.is_empty() || name.len() > 60 {
        return Err((StatusCode::BAD_REQUEST, "name must be 1..60 chars".to_string()));
    }
    let probability = req.probability.unwrap_or(0).clamp(0, 100);

    let mut conn = state.db_pool.get().map_err(pool_err)?;
    let branch_id = branch_from_jwt(&headers, &mut conn).unwrap_or_else(|| state.get_bot_context());

    let order = match req.stage_order {
        Some(o) if o >= 1 => o,
        _ => crm_pipeline_stages::table
            .filter(crm_pipeline_stages::branch_id.eq(branch_id))
            .select(diesel::dsl::max(crm_pipeline_stages::stage_order))
            .first::<Option<i32>>(&mut conn)
            .unwrap_or(Some(0))
            .unwrap_or(0)
            + 1,
    };

    let row = CrmPipelineStage {
        id: Uuid::new_v4(),
        org_id: branch_id,
        branch_id: Some(branch_id),
        name: name.to_string(),
        stage_order: order,
        probability,
        is_won: name == "won",
        is_lost: name == "lost",
        color: None,
        created_at: chrono::Utc::now(),
    };
    diesel::insert_into(crm_pipeline_stages::table)
        .values(&row)
        .execute(&mut conn)
        .map_err(diesel_err)?;
    Ok(Json(row))
}

/// `PUT /api/crm/pipeline/stages/:id` — rename / reorder / re-weight a stage.
pub async fn update_stage(
    State(state): State<Arc<CrateState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateStageRequest>,
) -> Result<Json<CrmPipelineStage>, (StatusCode, String)> {
    admin_headers(&headers)?;
    let mut conn = state.db_pool.get().map_err(pool_err)?;
    let branch_id = branch_from_jwt(&headers, &mut conn).unwrap_or_else(|| state.get_bot_context());

    diesel::update(
        crm_pipeline_stages::table
            .filter(crm_pipeline_stages::id.eq(id))
            .filter(crm_pipeline_stages::branch_id.eq(branch_id)),
    )
    .set((
        req.name.as_deref().map(|n| crm_pipeline_stages::name.eq(n.trim().to_string())),
        req.stage_order.map(|o| crm_pipeline_stages::stage_order.eq(o.clamp(1, 999))),
        req.probability.map(|p| crm_pipeline_stages::probability.eq(p.clamp(0, 100))),
    ))
    .execute(&mut conn)
    .map_err(diesel_err)?;

    let row: CrmPipelineStage = crm_pipeline_stages::table
        .filter(crm_pipeline_stages::id.eq(id))
        .filter(crm_pipeline_stages::branch_id.eq(branch_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Stage not found".to_string()))?;
    Ok(Json(row))
}

/// `DELETE /api/crm/pipeline/stages/:id` — blocked while leads reference the
/// stage so the kanban never drops a column holding data.
pub async fn delete_stage(
    State(state): State<Arc<CrateState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    admin_headers(&headers)?;
    let mut conn = state.db_pool.get().map_err(pool_err)?;
    let branch_id = branch_from_jwt(&headers, &mut conn).unwrap_or_else(|| state.get_bot_context());

    let stage: CrmPipelineStage = crm_pipeline_stages::table
        .filter(crm_pipeline_stages::id.eq(id))
        .filter(crm_pipeline_stages::branch_id.eq(branch_id))
        .first(&mut conn)
        .map_err(|_| (StatusCode::NOT_FOUND, "Stage not found".to_string()))?;

    let in_use: i64 = crm_deals::table
        .filter(crm_deals::branch_id.eq(branch_id))
        .filter(crm_deals::stage.eq(stage.name.clone()))
        .count()
        .get_result(&mut conn)
        .unwrap_or(0);
    if in_use > 0 {
        return Err((
            StatusCode::CONFLICT,
            format!("stage '{0}' still holds {in_use} lead(s); move them first", stage.name),
        ));
    }

    diesel::delete(
        crm_pipeline_stages::table
            .filter(crm_pipeline_stages::id.eq(id))
            .filter(crm_pipeline_stages::branch_id.eq(branch_id)),
    )
    .execute(&mut conn)
    .map_err(diesel_err)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, serde::Deserialize)]
pub struct CreateStageRequest {
    pub name: String,
    pub stage_order: Option<i32>,
    pub probability: Option<i32>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateStageRequest {
    pub name: Option<String>,
    pub stage_order: Option<i32>,
    pub probability: Option<i32>,
}

fn pool_err<E: std::fmt::Display>(e: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {e}"))
}

fn diesel_err(e: diesel::result::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
}
