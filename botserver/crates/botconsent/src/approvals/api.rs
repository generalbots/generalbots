//! HTTP surface for the approvals workflow. Routes are merged into the
//! consent router, which already carries `Arc<ConsentService>` as state.
//!
//! Every handler derives its actor and tenant from the authenticated caller.
//! No identity or tenant value is ever read from a request body or query
//! string (issue #2901).

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use super::store::ApprovalStore;
use super::types::{ApprovalStep, CallerPrincipal, DecisionKind, Delegation};
use crate::ConsentService;

fn status_for(error: &str) -> StatusCode {
    match error {
        "approval not found" => StatusCode::NOT_FOUND,
        "published approval is immutable" | "already published" => StatusCode::CONFLICT,
        "version_mismatch" | "invalidated" => StatusCode::CONFLICT,
        "owner_cannot_approve"
        | "not_an_assignee"
        | "out_of_turn"
        | "already_decided"
        | "delegator is not an assignee"
        | "delegate is not a tenant user" => StatusCode::FORBIDDEN,
        _ => StatusCode::BAD_REQUEST,
    }
}

type ApiResult<T> = Result<T, (StatusCode, String)>;

fn approvals(service: &Arc<ConsentService>) -> &ApprovalStore {
    &service.approvals
}

#[derive(QueryableByName)]
struct TenantRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    tenant: String,
}

/// Resolves a user's tenant strictly from persisted membership. No header,
/// query parameter or request body can influence this value.
fn tenant_of_user(
    pool: &crate::DbPool,
    user_id: Uuid,
    default_only: bool,
) -> Result<Option<String>, (StatusCode, String)> {
    let mut conn = pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {e}")))?;
    diesel::sql_query(
        "SELECT COALESCE(o.tenant_id::text, o.org_id::text) AS tenant \
         FROM users u \
         JOIN user_organizations uo ON uo.user_id = u.id \
         JOIN organizations o ON o.org_id = uo.org_id \
         WHERE u.id = $1 AND COALESCE(u.is_active, true) = true \
         AND ($2 = false OR COALESCE(uo.is_default, false) = true) \
         ORDER BY uo.joined_at DESC LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Bool, _>(default_only)
    .get_result::<TenantRow>(&mut conn)
    .optional()
    .map(|row| row.map(|r| r.tenant))
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("tenant lookup failed: {e}"),
        )
    })
}

/// Binds a request to the identity returned by authentication and to the
/// tenant that identity belongs to. The actor token is the user's UUID, which
/// a client cannot forge with a body field.
fn principal(
    service: &Arc<ConsentService>,
    headers: &HeaderMap,
) -> Result<CallerPrincipal, (StatusCode, String)> {
    let user_id = crate::api::require_consent_user(headers, &service.pool)?;
    let tenant =
        tenant_of_user(&service.pool, user_id, true)?.unwrap_or_else(|| user_id.to_string());
    Ok(CallerPrincipal::new(user_id, tenant))
}

/// A delegation target must be an active user whose membership includes the
/// caller's tenant. Without this check an attacker could point a delegation
/// at an arbitrary account in another organisation.
fn ensure_tenant_user(
    service: &Arc<ConsentService>,
    target: &str,
    tenant: &str,
) -> Result<(), (StatusCode, String)> {
    #[derive(QueryableByName)]
    struct ExistsRow {
        #[diesel(sql_type = diesel::sql_types::Bool)]
        ok: bool,
    }
    let target_id = Uuid::parse_str(target)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid delegate".to_string()))?;
    let mut conn = service
        .pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {e}")))?;
    let found = diesel::sql_query(
        "SELECT EXISTS( \
             SELECT 1 FROM users u \
             JOIN user_organizations uo ON uo.user_id = u.id \
             JOIN organizations o ON o.org_id = uo.org_id \
             WHERE u.id = $1 AND COALESCE(u.is_active, true) = true \
             AND COALESCE(o.tenant_id::text, o.org_id::text) = $2 \
         ) AS ok",
    )
    .bind::<diesel::sql_types::Uuid, _>(target_id)
    .bind::<diesel::sql_types::Text, _>(tenant)
    .get_result::<ExistsRow>(&mut conn)
    .optional()
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("delegate lookup failed: {e}"),
        )
    })?
    .map(|row| row.ok)
    .unwrap_or(false);
    if found {
        Ok(())
    } else {
        Err((
            StatusCode::FORBIDDEN,
            "delegate is not a tenant user".to_string(),
        ))
    }
}

#[derive(Debug, Deserialize)]
struct CreateBody {
    doc_id: String,
    #[serde(default)]
    doc_class: Option<String>,
    #[serde(default)]
    title: Option<String>,
    content: String,
}

#[derive(Debug, Deserialize)]
struct RouteBody {
    steps: Vec<ApprovalStep>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    due_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct DelegateBody {
    to: String,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct DecideBody {
    decision: DecisionKind,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    comment: Option<String>,
    #[serde(default)]
    version_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SignBody {
    #[serde(default)]
    comment: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ContentBody {
    content: String,
}

#[derive(Debug, Deserialize)]
struct SlaBody {
    #[serde(default)]
    at: Option<DateTime<Utc>>,
}

pub fn router() -> Router<Arc<ConsentService>> {
    Router::new()
        .route("/api/approvals", post(create).get(list))
        .route("/api/approvals/sla-sweep", post(sla_sweep))
        .route("/api/approvals/:id", get(get_approval))
        .route("/api/approvals/:id/route", post(route))
        .route("/api/approvals/:id/delegate", post(delegate))
        .route("/api/approvals/:id/decide", post(decide))
        .route("/api/approvals/:id/sign", post(sign))
        .route("/api/approvals/:id/update-content", post(update_content))
        .route("/api/approvals/:id/check-sla", post(check_sla))
        .route("/api/approvals/:id/publish", post(publish))
        .route("/api/approvals/:id/verify", get(verify))
}

/// `POST /api/approvals` — creates a draft owned by the authenticated caller
/// and scoped to that caller's tenant.
pub async fn create(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Json(body): Json<CreateBody>,
) -> ApiResult<Json<serde_json::Value>> {
    let caller = principal(&service, &headers)?;
    let doc_class = body.doc_class.unwrap_or_else(|| "document".to_string());
    let title = body.title.unwrap_or_else(String::new);
    let record = approvals(&service)
        .create(
            &caller.tenant,
            &body.doc_id,
            &doc_class,
            &title,
            &caller.actor,
            &body.content,
        )
        .map_err(|e| (status_for(&e), e))?;
    Ok(Json(serde_json::json!({ "approval": record })))
}

/// `GET /api/approvals` — lists only the authenticated caller's tenant. No
/// `?actor=` override is honoured: identity comes from the session alone.
pub async fn list(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    let caller = principal(&service, &headers)?;
    let rows = approvals(&service)
        .list(&caller.tenant)
        .map_err(|e| (status_for(&e), e))?;
    Ok(Json(serde_json::json!({ "approvals": rows })))
}

pub async fn get_approval(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let caller = principal(&service, &headers)?;
    let record = approvals(&service)
        .get(&caller.tenant, id)
        .map_err(|e| (status_for(&e), e))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "approval not found".to_string()))?;
    Ok(Json(serde_json::json!({ "approval": record })))
}

pub async fn route(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<RouteBody>,
) -> ApiResult<Json<serde_json::Value>> {
    let caller = principal(&service, &headers)?;
    let message = body.message.unwrap_or_else(String::new);
    let record = approvals(&service)
        .set_route(&caller.tenant, id, body.steps, &message, body.due_at)
        .map_err(|e| (status_for(&e), e))?;
    Ok(Json(serde_json::json!({ "approval": record })))
}

pub async fn delegate(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<DelegateBody>,
) -> ApiResult<Json<serde_json::Value>> {
    let caller = principal(&service, &headers)?;
    ensure_tenant_user(&service, &body.to, &caller.tenant)?;
    let delegation = Delegation {
        from: caller.actor.clone(),
        to: body.to,
        reason: body.reason.unwrap_or_else(String::new),
        expires_at: body.expires_at,
        created_at: Utc::now(),
    };
    let record = approvals(&service)
        .delegate(&caller.tenant, id, delegation)
        .map_err(|e| (status_for(&e), e))?;
    Ok(Json(serde_json::json!({ "approval": record })))
}

pub async fn decide(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<DecideBody>,
) -> ApiResult<Json<serde_json::Value>> {
    let caller = principal(&service, &headers)?;
    let reason = body.reason.unwrap_or_else(String::new);
    let comment = body.comment.unwrap_or_else(String::new);
    let record = approvals(&service)
        .decide(
            &caller.tenant,
            id,
            &caller.actor,
            body.decision,
            &reason,
            &comment,
            body.version_hash.as_deref(),
        )
        .map_err(|e| (status_for(&e), e))?;
    Ok(Json(serde_json::json!({ "approval": record })))
}

pub async fn sign(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<SignBody>,
) -> ApiResult<Json<serde_json::Value>> {
    let caller = principal(&service, &headers)?;
    let comment = body.comment.unwrap_or_else(String::new);
    let record = approvals(&service)
        .sign(&caller.tenant, id, &caller.actor, &comment)
        .map_err(|e| (status_for(&e), e))?;
    Ok(Json(serde_json::json!({ "approval": record })))
}

pub async fn update_content(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<ContentBody>,
) -> ApiResult<Json<serde_json::Value>> {
    let caller = principal(&service, &headers)?;
    let record = approvals(&service)
        .update_content(&caller.tenant, id, &body.content)
        .map_err(|e| (status_for(&e), e))?;
    Ok(Json(serde_json::json!({ "approval": record })))
}

/// `POST /api/approvals/:id/check-sla` — read-only preview of due events for
/// this record. The global sweep marks events as delivered.
pub async fn check_sla(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<SlaBody>,
) -> ApiResult<Json<serde_json::Value>> {
    let caller = principal(&service, &headers)?;
    let record = approvals(&service)
        .get(&caller.tenant, id)
        .map_err(|e| (status_for(&e), e))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "approval not found".to_string()))?;
    let now = body.at.unwrap_or_else(Utc::now);
    let events = super::engine::check_sla(&record, now);
    Ok(Json(serde_json::json!({ "events": events })))
}

/// `POST /api/approvals/sla-sweep` — sweeps only the caller's tenant. `{"at":"..."}`
/// is optional.
pub async fn sla_sweep(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Json(body): Json<SlaBody>,
) -> ApiResult<Json<serde_json::Value>> {
    let caller = principal(&service, &headers)?;
    let now = body.at.unwrap_or_else(Utc::now);
    let events = approvals(&service)
        .check_sla_at(&caller.tenant, now)
        .map_err(|e| (status_for(&e), e))?;
    Ok(Json(serde_json::json!({ "events": events })))
}

pub async fn publish(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let caller = principal(&service, &headers)?;
    let artifact = approvals(&service)
        .publish(&caller.tenant, id)
        .map_err(|e| (status_for(&e), e))?;
    Ok(Json(serde_json::json!({ "artifact": artifact })))
}

pub async fn verify(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    let caller = principal(&service, &headers)?;
    let verified = approvals(&service)
        .verify_trail(&caller.tenant, id)
        .map_err(|e| (status_for(&e), e))?;
    Ok(Json(serde_json::json!({ "verified": verified })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_status_mapping() {
        assert_eq!(status_for("approval not found"), StatusCode::NOT_FOUND);
        assert_eq!(status_for("version_mismatch"), StatusCode::CONFLICT);
        assert_eq!(status_for("reason_required"), StatusCode::BAD_REQUEST);
        assert_eq!(status_for("owner_cannot_approve"), StatusCode::FORBIDDEN);
    }

    #[test]
    fn router_builds_without_panics() {
        router();
    }

    #[test]
    fn principal_actor_is_the_authenticated_id() {
        let user_id =
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap_or_else(|_| Uuid::nil());
        let caller = CallerPrincipal::new(user_id, "tenant-a".to_string());
        assert_eq!(caller.actor, user_id.to_string());
        assert_eq!(caller.user_id, user_id);
        assert_eq!(caller.tenant, "tenant-a");
    }
}
