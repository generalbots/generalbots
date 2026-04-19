use axum::{
    body::Body,
    extract::State,
    http::{header::HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::billing::quotas::{QuotaAction, QuotaManager};
use crate::billing::UsageMetric;

#[derive(Clone)]
pub struct QuotaMiddlewareState {
    pub quota_manager: Arc<QuotaManager>,
    pub enabled: Arc<RwLock<bool>>,
}

impl QuotaMiddlewareState {
    pub fn new(quota_manager: Arc<QuotaManager>) -> Self {
        Self {
            quota_manager,
            enabled: Arc::new(RwLock::new(true)),
        }
    }

    pub async fn set_enabled(&self, enabled: bool) {
        let mut guard = self.enabled.write().await;
        *guard = enabled;
    }

    pub async fn is_enabled(&self) -> bool {
        *self.enabled.read().await
    }
}

pub async fn quota_middleware(
    State(state): State<Arc<QuotaMiddlewareState>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !state.is_enabled().await {
        return next.run(request).await;
    }

    let org_id = extract_organization_id(&request);

    let org_id = match org_id {
        Some(id) => id,
        None => return next.run(request).await,
    };

    let metric = determine_metric_from_request(&request);

    let action = state.quota_manager.check_action(org_id, metric).await;

    match action {
        QuotaAction::Allow => {
            let response = next.run(request).await;

            if response.status().is_success() {
                if let Err(e) = state
                    .quota_manager
                    .increment_usage(org_id, metric, 1)
                    .await
                {
                    tracing::warn!("Failed to increment usage for org {}: {}", org_id, e);
                }
            }

            response
        }
        QuotaAction::Warn { message, percentage } => {
            let mut response = next.run(request).await;

            if response.status().is_success() {
                if let Err(e) = state
                    .quota_manager
                    .increment_usage(org_id, metric, 1)
                    .await
                {
                    tracing::warn!("Failed to increment usage for org {}: {}", org_id, e);
                }
            }

            let headers = response.headers_mut();
            headers.insert(
                "X-Quota-Warning",
                message.parse().unwrap_or_else(|_| HeaderValue::from_static("quota warning")),
            );
            headers.insert(
                "X-Quota-Usage-Percent",
                percentage
                    .to_string()
                    .parse()
                    .unwrap_or_else(|_| HeaderValue::from_static("0")),
            );

            response
        }
        QuotaAction::Block { message } => QuotaExceededResponse { message }.into_response(),
    }
}

pub async fn api_rate_limit_middleware(
    State(state): State<Arc<QuotaMiddlewareState>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !state.is_enabled().await {
        return next.run(request).await;
    }

    let org_id = match extract_organization_id(&request) {
        Some(id) => id,
        None => return next.run(request).await,
    };

    let action = state
        .quota_manager
        .check_action(org_id, UsageMetric::ApiCalls)
        .await;

    match action {
        QuotaAction::Allow => {
            if let Err(e) = state
                .quota_manager
                .increment_usage(org_id, UsageMetric::ApiCalls, 1)
                .await
            {
                tracing::warn!("Failed to increment API call count for org {}: {}", org_id, e);
            }
            next.run(request).await
        }
        QuotaAction::Warn { message, percentage } => {
            if let Err(e) = state
                .quota_manager
                .increment_usage(org_id, UsageMetric::ApiCalls, 1)
                .await
            {
                tracing::warn!("Failed to increment API call count for org {}: {}", org_id, e);
            }

            let mut response = next.run(request).await;
            let headers = response.headers_mut();
            headers.insert(
                "X-RateLimit-Warning",
                message.parse().unwrap_or_else(|_| HeaderValue::from_static("rate limit warning")),
            );
            headers.insert(
                "X-RateLimit-Usage-Percent",
                percentage
                    .to_string()
                    .parse()
                    .unwrap_or_else(|_| HeaderValue::from_static("0")),
            );
            response
        }
        QuotaAction::Block { message } => RateLimitExceededResponse { message }.into_response(),
    }
}

pub async fn message_quota_middleware(
    State(state): State<Arc<QuotaMiddlewareState>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !state.is_enabled().await {
        return next.run(request).await;
    }

    let org_id = match extract_organization_id(&request) {
        Some(id) => id,
        None => return next.run(request).await,
    };

    let action = state
        .quota_manager
        .check_action(org_id, UsageMetric::Messages)
        .await;

    match action {
        QuotaAction::Allow => {
            let response = next.run(request).await;
            if response.status().is_success() {
                if let Err(e) = state
                    .quota_manager
                    .increment_usage(org_id, UsageMetric::Messages, 1)
                    .await
                {
                    tracing::warn!("Failed to increment message count for org {}: {}", org_id, e);
                }
            }
            response
        }
        QuotaAction::Warn { message, percentage } => {
            let response = next.run(request).await;
            if response.status().is_success() {
                if let Err(e) = state
                    .quota_manager
                    .increment_usage(org_id, UsageMetric::Messages, 1)
                    .await
                {
                    tracing::warn!("Failed to increment message count for org {}: {}", org_id, e);
                }
            }

            let mut response = response;
            let headers = response.headers_mut();
            headers.insert(
                "X-Message-Quota-Warning",
                message.parse().unwrap_or_else(|_| HeaderValue::from_static("message quota warning")),
            );
            headers.insert(
                "X-Message-Quota-Usage-Percent",
                percentage
                    .to_string()
                    .parse()
                    .unwrap_or_else(|_| HeaderValue::from_static("0")),
            );
            response
        }
        QuotaAction::Block { message } => MessageQuotaExceededResponse { message }.into_response(),
    }
}

pub async fn storage_check_middleware(
    State(state): State<Arc<QuotaMiddlewareState>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !state.is_enabled().await {
        return next.run(request).await;
    }

    let org_id = match extract_organization_id(&request) {
        Some(id) => id,
        None => return next.run(request).await,
    };

    let content_length = request
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    if content_length == 0 {
        return next.run(request).await;
    }

    let check_result = state
        .quota_manager
        .check_quota(org_id, UsageMetric::StorageBytes, content_length)
        .await;

    match check_result {
        Ok(result) => {
            if !result.allowed {
                return StorageQuotaExceededResponse {
                    message: format!(
                        "Storage quota exceeded. Current: {} bytes, Limit: {:?} bytes",
                        result.current, result.limit
                    ),
                    current_usage: result.current,
                    limit: result.limit,
                }
                .into_response();
            }

            let response = next.run(request).await;

            if response.status().is_success() {
                if let Err(e) = state
                    .quota_manager
                    .increment_usage(org_id, UsageMetric::StorageBytes, content_length)
                    .await
                {
                    tracing::warn!("Failed to increment storage for org {}: {}", org_id, e);
                }
            }

            response
        }
        Err(e) => {
            tracing::error!("Failed to check storage quota for org {}: {}", org_id, e);
            next.run(request).await
        }
    }
}

fn extract_organization_id(request: &Request<Body>) -> Option<Uuid> {
    if let Some(org_header) = request.headers().get("X-Organization-Id") {
        if let Ok(org_str) = org_header.to_str() {
            if let Ok(org_id) = Uuid::parse_str(org_str) {
                return Some(org_id);
            }
        }
    }

    if let Some(query) = request.uri().query() {
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                if key == "org_id" || key == "organization_id" {
                    if let Ok(org_id) = Uuid::parse_str(value) {
                        return Some(org_id);
                    }
                }
            }
        }
    }

    request
        .extensions()
        .get::<OrganizationContext>()
        .map(|ctx| ctx.organization_id)
}

fn determine_metric_from_request(request: &Request<Body>) -> UsageMetric {
    let path = request.uri().path();
    let method = request.method();

    if path.contains("/chat") || path.contains("/message") || path.contains("/conversation") {
        return UsageMetric::Messages;
    }

    if path.contains("/upload") || path.contains("/file") || path.contains("/storage") {
        return UsageMetric::StorageBytes;
    }

    if path.contains("/bot") && method == "POST" {
        return UsageMetric::Bots;
    }

    if path.contains("/user") && method == "POST" {
        return UsageMetric::Users;
    }

    if path.contains("/kb") || path.contains("/document") {
        return UsageMetric::KbDocuments;
    }

    if path.contains("/app") || path.contains("/form") || path.contains("/site") {
        return UsageMetric::Apps;
    }

    UsageMetric::ApiCalls
}

#[derive(Clone)]
pub struct OrganizationContext {
    pub organization_id: Uuid,
    pub user_id: Option<Uuid>,
    pub plan_id: Option<String>,
}

pub async fn organization_context_middleware(
    request: Request<Body>,
    next: Next,
) -> Response {
    let org_id = extract_organization_id(&request);

    if let Some(org_id) = org_id {
        let mut request = request;
        request.extensions_mut().insert(OrganizationContext {
            organization_id: org_id,
            user_id: None,
            plan_id: None,
        });
        next.run(request).await
    } else {
        next.run(request).await
    }
}

struct QuotaExceededResponse {
    message: String,
}

impl IntoResponse for QuotaExceededResponse {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "error": "quota_exceeded",
            "message": self.message,
            "code": "QUOTA_EXCEEDED"
        });

        (
            StatusCode::TOO_MANY_REQUESTS,
            [
                ("Content-Type", "application/json"),
                ("X-Quota-Exceeded", "true"),
            ],
            Json(body),
        )
            .into_response()
    }
}

struct RateLimitExceededResponse {
    message: String,
}

impl IntoResponse for RateLimitExceededResponse {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "error": "rate_limit_exceeded",
            "message": self.message,
            "code": "RATE_LIMIT_EXCEEDED",
            "retry_after": 60
        });

        (
            StatusCode::TOO_MANY_REQUESTS,
            [
                ("Content-Type", "application/json"),
                ("Retry-After", "60"),
                ("X-RateLimit-Exceeded", "true"),
            ],
            Json(body),
        )
            .into_response()
    }
}

struct MessageQuotaExceededResponse {
    message: String,
}

impl IntoResponse for MessageQuotaExceededResponse {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "error": "message_quota_exceeded",
            "message": self.message,
            "code": "MESSAGE_QUOTA_EXCEEDED",
            "upgrade_url": "/billing/upgrade"
        });

        (
            StatusCode::TOO_MANY_REQUESTS,
            [
                ("Content-Type", "application/json"),
                ("X-Message-Quota-Exceeded", "true"),
            ],
            Json(body),
        )
            .into_response()
    }
}

struct StorageQuotaExceededResponse {
    message: String,
    current_usage: u64,
    limit: Option<u64>,
}

impl IntoResponse for StorageQuotaExceededResponse {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "error": "storage_quota_exceeded",
            "message": self.message,
            "code": "STORAGE_QUOTA_EXCEEDED",
            "current_usage_bytes": self.current_usage,
            "limit_bytes": self.limit,
            "upgrade_url": "/billing/upgrade"
        });

        (
            StatusCode::PAYLOAD_TOO_LARGE,
            [
                ("Content-Type", "application/json"),
                ("X-Storage-Quota-Exceeded", "true"),
            ],
            Json(body),
        )
            .into_response()
    }
}

pub fn create_quota_middleware_state(quota_manager: Arc<QuotaManager>) -> Arc<QuotaMiddlewareState> {
    Arc::new(QuotaMiddlewareState::new(quota_manager))
}

pub async fn toggle_saas_mode(state: &QuotaMiddlewareState, enabled: bool) {
    state.set_enabled(enabled).await;
    if enabled {
        tracing::info!("SaaS quota enforcement enabled");
    } else {
        tracing::info!("SaaS quota enforcement disabled (local mode)");
    }
}
