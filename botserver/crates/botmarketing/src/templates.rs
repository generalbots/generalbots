use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::{OptionalExtension, PgConnection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::schema::marketing_templates;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = marketing_templates)]
pub struct MarketingTemplate {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub channel: String,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub media_url: Option<String>,
    pub ai_prompt: Option<String>,
    pub variables: Option<serde_json::Value>,
    pub approved: Option<bool>,
    pub meta_template_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub branch_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub channel: String,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub media_url: Option<String>,
    pub ai_prompt: Option<String>,
    pub variables: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub channel: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub media_url: Option<String>,
    pub ai_prompt: Option<String>,
    pub variables: Option<serde_json::Value>,
    pub approved: Option<bool>,
}

/// Resolves the caller's verified branch for a by-id request (#2887).
///
/// The server-minted JWT claim wins; the only fallback is the server-side
/// default scope. A client-supplied org/branch is never consulted.
fn caller_branch(state: &AppState, headers: &HeaderMap, conn: &mut PgConnection) -> Uuid {
    let (_, default_branch_id, _) = state.get_scope();
    crate::scope::branch_from_jwt(headers, conn).unwrap_or(default_branch_id)
}

/// Pure form of the #2887 scoping rule: a row is visible to a caller only
/// when the row's `branch_id` matches the caller's verified branch. Every
/// by-id Diesel query carries the equivalent `.filter(branch_id.eq(branch))`;
/// this predicate also re-checks the loaded row as defence in depth and makes
/// the two-tenant behaviour unit-testable without a database.
fn in_branch_scope(row_branch_id: Uuid, caller_branch_id: Uuid) -> bool {
    row_branch_id == caller_branch_id
}

/// Loads a template by id only when it belongs to `branch_id`. A foreign or
/// unknown id yields `None`, which the handlers turn into `404`.
fn load_scoped_template(
    conn: &mut PgConnection,
    id: Uuid,
    branch_id: Uuid,
) -> Result<Option<MarketingTemplate>, diesel::result::Error> {
    let row = marketing_templates::table
        .filter(marketing_templates::id.eq(id))
        .filter(marketing_templates::branch_id.eq(branch_id))
        .first::<MarketingTemplate>(conn)
        .optional()?;
    Ok(row.filter(|template| in_branch_scope(template.branch_id, branch_id)))
}

pub async fn list_templates(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<MarketingTemplate>>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (_, branch_id, _) = state.get_scope();

    let templates: Vec<MarketingTemplate> = marketing_templates::table
        .filter(marketing_templates::branch_id.eq(branch_id))
        .order(marketing_templates::created_at.desc())
        .load(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?;

    Ok(Json(templates))
}

pub async fn get_template(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<MarketingTemplate>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;
    let branch_id = caller_branch(&state, &headers, &mut conn);

    let template = load_scoped_template(&mut conn, id, branch_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?
        .ok_or((StatusCode::NOT_FOUND, "Template not found".to_string()))?;

    Ok(Json(template))
}

pub async fn create_template(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateTemplateRequest>,
) -> Result<Json<MarketingTemplate>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;

    let (org_id, default_branch_id, bot_id) = state.get_scope();
    let branch_id = crate::scope::branch_from_jwt(&headers, &mut conn)
        .unwrap_or(default_branch_id);
    let id = Uuid::new_v4();
    let now = Utc::now();

    let template = MarketingTemplate {
        id,
        org_id,
        bot_id,
        branch_id,
        name: req.name,
        channel: req.channel,
        subject: req.subject,
        body: req.body,
        media_url: req.media_url,
        ai_prompt: req.ai_prompt,
        variables: req.variables,
        approved: Some(false),
        meta_template_id: None,
        created_at: Some(now),
        updated_at: Some(now),
    };

    diesel::insert_into(marketing_templates::table)
        .values(&template)
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert error: {e}")))?;

    Ok(Json(template))
}

pub async fn update_template(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateTemplateRequest>,
) -> Result<Json<MarketingTemplate>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;
    let branch_id = caller_branch(&state, &headers, &mut conn);
    // Reject a foreign id before touching any row (#2887).
    load_scoped_template(&mut conn, id, branch_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {e}")))?
        .ok_or((StatusCode::NOT_FOUND, "Template not found".to_string()))?;

    let now = Utc::now();

    if let Some(name) = req.name {
        diesel::update(
            marketing_templates::table
                .filter(marketing_templates::id.eq(id))
                .filter(marketing_templates::branch_id.eq(branch_id)),
        )
        .set(marketing_templates::name.eq(name))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(channel) = req.channel {
        diesel::update(
            marketing_templates::table
                .filter(marketing_templates::id.eq(id))
                .filter(marketing_templates::branch_id.eq(branch_id)),
        )
        .set(marketing_templates::channel.eq(channel))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(subject) = req.subject {
        diesel::update(
            marketing_templates::table
                .filter(marketing_templates::id.eq(id))
                .filter(marketing_templates::branch_id.eq(branch_id)),
        )
        .set(marketing_templates::subject.eq(Some(subject)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(body) = req.body {
        diesel::update(
            marketing_templates::table
                .filter(marketing_templates::id.eq(id))
                .filter(marketing_templates::branch_id.eq(branch_id)),
        )
        .set(marketing_templates::body.eq(Some(body)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(media_url) = req.media_url {
        diesel::update(
            marketing_templates::table
                .filter(marketing_templates::id.eq(id))
                .filter(marketing_templates::branch_id.eq(branch_id)),
        )
        .set(marketing_templates::media_url.eq(Some(media_url)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(ai_prompt) = req.ai_prompt {
        diesel::update(
            marketing_templates::table
                .filter(marketing_templates::id.eq(id))
                .filter(marketing_templates::branch_id.eq(branch_id)),
        )
        .set(marketing_templates::ai_prompt.eq(Some(ai_prompt)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(variables) = req.variables {
        diesel::update(
            marketing_templates::table
                .filter(marketing_templates::id.eq(id))
                .filter(marketing_templates::branch_id.eq(branch_id)),
        )
        .set(marketing_templates::variables.eq(variables))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }
    if let Some(approved) = req.approved {
        diesel::update(
            marketing_templates::table
                .filter(marketing_templates::id.eq(id))
                .filter(marketing_templates::branch_id.eq(branch_id)),
        )
        .set(marketing_templates::approved.eq(Some(approved)))
        .execute(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;
    }

    diesel::update(
        marketing_templates::table
            .filter(marketing_templates::id.eq(id))
            .filter(marketing_templates::branch_id.eq(branch_id)),
    )
    .set(marketing_templates::updated_at.eq(now))
    .execute(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update error: {e}")))?;

    get_template(State(state), headers, Path(id)).await
}

pub async fn delete_template(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut conn = state.conn.get().map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}"))
    })?;
    let branch_id = caller_branch(&state, &headers, &mut conn);

    let deleted = diesel::delete(
        marketing_templates::table
            .filter(marketing_templates::id.eq(id))
            .filter(marketing_templates::branch_id.eq(branch_id)),
    )
    .execute(&mut conn)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete error: {e}")))?;
    if deleted == 0 {
        return Err((StatusCode::NOT_FOUND, "Template not found".to_string()));
    }

    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

#[cfg(test)]
mod tests {
    use super::in_branch_scope;
    use uuid::Uuid;

    #[test]
    fn scoped_predicate_accepts_own_branch() {
        let branch = Uuid::new_v4();
        assert!(in_branch_scope(branch, branch));
    }

    #[test]
    fn scoped_predicate_rejects_foreign_branch() {
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        assert_ne!(tenant_a, tenant_b);
        assert!(!in_branch_scope(tenant_b, tenant_a));
        assert!(!in_branch_scope(tenant_a, tenant_b));
    }

    #[test]
    fn scoped_predicate_never_matches_a_different_tenant_even_when_one_is_nil() {
        let real = Uuid::new_v4();
        assert!(!in_branch_scope(real, Uuid::nil()));
        assert!(!in_branch_scope(Uuid::nil(), real));
        assert!(in_branch_scope(Uuid::nil(), Uuid::nil()));
    }
}
