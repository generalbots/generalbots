pub(crate) use axum::{ body::Body, extract::State, http::{header::HeaderValue, Request, StatusCode}, middleware::Next, response::{IntoResponse, Response}, Json, };
pub(crate) use std::sync::Arc;
pub(crate) use tokio::sync::RwLock;
pub(crate) use uuid::Uuid;
pub(crate) use crate::quotas::{QuotaAction, QuotaManager};
pub(crate) use crate::UsageMetric;

#[derive(Clone)]
pub struct QuotaMiddlewareState {
    pub quota_manager: Arc<QuotaManager>,
    pub enabled: Arc<RwLock<bool>>,
}

#[derive(Clone)]
pub struct OrganizationContext {
    pub organization_id: Uuid,
    pub user_id: Option<Uuid>,
    pub plan_id: Option<String>,
}

struct QuotaExceededResponse {
    message: String,
}

struct RateLimitExceededResponse {
    message: String,
}

struct MessageQuotaExceededResponse {
    message: String,
}

struct StorageQuotaExceededResponse {
    message: String,
    current_usage: u64,
    limit: Option<u64>,
}

mod set_enabled;
mod determine_metric_from_request;
mod organization_context_middleware;
mod into_response;
mod rate_limit_exceeded_response;
mod message_quota_exceeded_response;
mod create_quota_middleware_state;

pub use set_enabled::*;
use determine_metric_from_request::*;
pub use organization_context_middleware::*;
pub use create_quota_middleware_state::*;
