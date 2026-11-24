use super::*;

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

    // Reserve the upload bytes under the quota manager's write lock BEFORE
    // running the handler (#2788). Checking first and incrementing after the
    // upload let concurrent uploads all observe the same pre-upload usage and
    // exceed the cap. `increment_usage` now checks and reserves atomically; a
    // refused reservation leaves usage untouched.
    let reservation = state
        .quota_manager
        .increment_usage(org_id, UsageMetric::StorageBytes, content_length)
        .await;

    let result = match reservation {
        Ok(result) => result,
        Err(e) => {
            tracing::log::error!("Failed to reserve storage quota for org {}: {}", org_id, e);
            return next.run(request).await;
        }
    };

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

    if !response.status().is_success() {
        if let Err(e) = state
            .quota_manager
            .decrement_usage(org_id, UsageMetric::StorageBytes, content_length)
            .await
        {
            tracing::warn!(
                "Failed to roll back storage reservation for org {}: {}",
                org_id,
                e
            );
        }
    }

    response
}
/// Resolve the tenant that owns a request for quota/metering purposes (#2755).
///
/// The organization id is security-sensitive: it decides which tenant's quota
/// is charged and whose usage is metered. It must therefore come exclusively
/// from the `OrganizationContext` extension, which an authenticated upstream
/// middleware populates from verified credentials.
///
/// Client-controlled sources (`X-Organization-Id`, `?org_id=`,
/// `?organization_id=`) are deliberately NOT consulted. Otherwise an attacker
/// could attribute traffic to a victim org (quota DoS / skewed metering) or
/// bill usage to a forged org to evade their own quota.
pub(super) fn extract_organization_id(request: &Request<Body>) -> Option<Uuid> {
    request
        .extensions()
        .get::<OrganizationContext>()
        .map(|ctx| ctx.organization_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn authenticated_org() -> Uuid {
        Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap_or_else(|_| Uuid::nil())
    }

    fn forged_org() -> Uuid {
        Uuid::parse_str("22222222-2222-4222-8222-222222222222").unwrap_or_else(|_| Uuid::nil())
    }

    fn request_with_forged_org() -> Request<Body> {
        Request::builder()
            .uri(format!(
                "/api/usage?org_id={}&organization_id={}",
                forged_org(),
                forged_org()
            ))
            .header("X-Organization-Id", forged_org().to_string())
            .body(Body::empty())
            .unwrap_or_else(|_| Request::new(Body::empty()))
    }

    /// A forged header/query must never override the authenticated extension.
    /// This is the core regression test for #2755.
    #[test]
    fn forged_org_id_is_ignored_in_favor_of_authenticated_context() {
        let mut request = request_with_forged_org();
        request.extensions_mut().insert(OrganizationContext {
            organization_id: authenticated_org(),
            user_id: None,
            plan_id: None,
        });

        assert_eq!(extract_organization_id(&request), Some(authenticated_org()));
    }

    #[test]
    fn forged_header_without_context_is_ignored() {
        let request = Request::builder()
            .header("X-Organization-Id", forged_org().to_string())
            .body(Body::empty())
            .unwrap_or_else(|_| Request::new(Body::empty()));

        assert_eq!(extract_organization_id(&request), None);
    }

    #[test]
    fn forged_query_without_context_is_ignored() {
        let request = Request::builder()
            .uri(format!("/api/usage?org_id={}", forged_org()))
            .body(Body::empty())
            .unwrap_or_else(|_| Request::new(Body::empty()));

        assert_eq!(extract_organization_id(&request), None);
    }

    #[test]
    fn missing_context_returns_none() {
        let request = Request::new(Body::empty());
        assert_eq!(extract_organization_id(&request), None);
    }
}
