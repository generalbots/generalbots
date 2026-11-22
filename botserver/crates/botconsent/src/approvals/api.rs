//! HTTP surface for the approvals workflow. Routes are merged into the
//! consent router, which already carries `Arc<ConsentService>` as state.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::store::ApprovalStore;
use super::types::{ApprovalStep, DecisionKind, Delegation};
use crate::ConsentService;

fn status_for(error: &str) -> StatusCode {
    match error {
        "approval not found" => StatusCode::NOT_FOUND,
        "published approval is immutable" | "already published" => StatusCode::CONFLICT,
        "version_mismatch" | "invalidated" => StatusCode::CONFLICT,
        _ => StatusCode::BAD_REQUEST,
    }
}

type ApiResult<T> = Result<T, (StatusCode, String)>;

fn approvals(service: &Arc<ConsentService>) -> &ApprovalStore {
    &service.approvals
}

#[derive(Debug, Deserialize)]
struct CreateBody {
    #[serde(default)]
    tenant: Option<String>,
    doc_id: String,
    #[serde(default)]
    doc_class: Option<String>,
    #[serde(default)]
    title: Option<String>,
    owner: String,
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
    from: String,
    to: String,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct DecideBody {
    actor: String,
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
    actor: String,
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

/// `POST /api/approvals` — creates a draft bound to the content hash.
pub async fn create(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Json(body): Json<CreateBody>,
) -> ApiResult<Json<serde_json::Value>> {
    crate::api::require_consent_user(&headers, &service.pool)?;
    let tenant = body.tenant.unwrap_or_else(|| "default".to_string());
    let doc_class = body.doc_class.unwrap_or_else(|| "document".to_string());
    let title = body.title.unwrap_or_else(String::new);
    let record = approvals(&service)
        .create(
            &tenant,
            &body.doc_id,
            &doc_class,
            &title,
            &body.owner,
            &body.content,
        )
        .map_err(|e| (status_for(&e), e))?;
    Ok(Json(serde_json::json!({ "approval": record })))
}

/// `GET /api/approvals` — dashboard; `?actor=` filters to that person's queue.
pub async fn list(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<serde_json::Value>> {
    crate::api::require_consent_user(&headers, &service.pool)?;
    let rows = match params.get("actor") {
        Some(actor) if !actor.is_empty() => approvals(&service).pending_for(actor),
        _ => approvals(&service).list(),
    }
    .map_err(|e| (status_for(&e), e))?;
    Ok(Json(serde_json::json!({ "approvals": rows })))
}

pub async fn get_approval(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    crate::api::require_consent_user(&headers, &service.pool)?;
    let record = approvals(&service)
        .get(id)
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
    crate::api::require_consent_user(&headers, &service.pool)?;
    let message = body.message.unwrap_or_else(String::new);
    let record = approvals(&service)
        .set_route(id, body.steps, &message, body.due_at)
        .map_err(|e| (status_for(&e), e))?;
    Ok(Json(serde_json::json!({ "approval": record })))
}

pub async fn delegate(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<DelegateBody>,
) -> ApiResult<Json<serde_json::Value>> {
    crate::api::require_consent_user(&headers, &service.pool)?;
    let delegation = Delegation {
        from: body.from,
        to: body.to,
        reason: body.reason.unwrap_or_else(String::new),
        expires_at: body.expires_at,
        created_at: Utc::now(),
    };
    let record = approvals(&service)
        .delegate(id, delegation)
        .map_err(|e| (status_for(&e), e))?;
    Ok(Json(serde_json::json!({ "approval": record })))
}

pub async fn decide(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<DecideBody>,
) -> ApiResult<Json<serde_json::Value>> {
    crate::api::require_consent_user(&headers, &service.pool)?;
    let reason = body.reason.unwrap_or_else(String::new);
    let comment = body.comment.unwrap_or_else(String::new);
    let record = approvals(&service)
        .decide(
            id,
            &body.actor,
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
    crate::api::require_consent_user(&headers, &service.pool)?;
    let comment = body.comment.unwrap_or_else(String::new);
    let record = approvals(&service)
        .sign(id, &body.actor, &comment)
        .map_err(|e| (status_for(&e), e))?;
    Ok(Json(serde_json::json!({ "approval": record })))
}

pub async fn update_content(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<ContentBody>,
) -> ApiResult<Json<serde_json::Value>> {
    crate::api::require_consent_user(&headers, &service.pool)?;
    let record = approvals(&service)
        .update_content(id, &body.content)
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
    crate::api::require_consent_user(&headers, &service.pool)?;
    let record = approvals(&service)
        .get(id)
        .map_err(|e| (status_for(&e), e))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "approval not found".to_string()))?;
    let now = body.at.unwrap_or_else(Utc::now);
    let events = super::engine::check_sla(&record, now);
    Ok(Json(serde_json::json!({ "events": events })))
}

/// `POST /api/approvals/sla-sweep` — global sweep; marks reminder/escalation
/// as sent so each fires once per step. `{"at":"..."}` is optional.
pub async fn sla_sweep(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Json(body): Json<SlaBody>,
) -> ApiResult<Json<serde_json::Value>> {
    crate::api::require_consent_user(&headers, &service.pool)?;
    let now = body.at.unwrap_or_else(Utc::now);
    let events = approvals(&service)
        .check_sla_at(now)
        .map_err(|e| (status_for(&e), e))?;
    Ok(Json(serde_json::json!({ "events": events })))
}

pub async fn publish(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    crate::api::require_consent_user(&headers, &service.pool)?;
    let artifact = approvals(&service)
        .publish(id)
        .map_err(|e| (status_for(&e), e))?;
    Ok(Json(serde_json::json!({ "artifact": artifact })))
}

pub async fn verify(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<serde_json::Value>> {
    crate::api::require_consent_user(&headers, &service.pool)?;
    let verified = approvals(&service)
        .verify_trail(id)
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
    }

    #[test]
    fn router_builds_without_panics() {
        router();
    }
}
