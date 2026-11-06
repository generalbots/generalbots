use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use botlib::limits::{
    format_limit_error_response, LimitExceeded, RateLimiter as BotlibRateLimiter, SystemLimits,
};
use governor::{
    clock::DefaultClock,
    state::{keyed::DefaultKeyedStateStore, InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use serde_json::json;
use std::{
    net::SocketAddr,
    num::NonZeroU32,
    sync::Arc,
};

pub type GlobalRateLimiter = GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Per-client limiter for the authentication endpoints. Keyed by the client
/// address (the first `X-Forwarded-For` hop behind Caddy), so one client
/// cannot exhaust the login endpoint for everyone.
pub type AuthKeyedRateLimiter =
    GovernorRateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>;

#[derive(Debug, Clone)]
pub struct HttpRateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

impl Default for HttpRateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 100,
            burst_size: 200,
        }
    }
}

impl HttpRateLimitConfig {
    pub fn strict() -> Self {
        Self {
            requests_per_second: 50,
            burst_size: 100,
        }
    }

    pub fn relaxed() -> Self {
        Self {
            requests_per_second: 500,
            burst_size: 1000,
        }
    }

    pub fn api() -> Self {
        Self {
            requests_per_second: 100,
            burst_size: 150,
        }
    }
}

pub struct CombinedRateLimiter {
    http_limiter: Arc<GlobalRateLimiter>,
    auth_limiter: Arc<AuthKeyedRateLimiter>,
    botlib_limiter: Arc<BotlibRateLimiter>,
}

impl CombinedRateLimiter {
    pub fn new(http_config: HttpRateLimitConfig, system_limits: SystemLimits) -> Self {
        const DEFAULT_RPS: NonZeroU32 = match NonZeroU32::new(100) {
            Some(v) => v,
            None => unreachable!(),
        };
        const DEFAULT_BURST: NonZeroU32 = match NonZeroU32::new(200) {
            Some(v) => v,
            None => unreachable!(),
        };

        let quota = Quota::per_second(
            NonZeroU32::new(http_config.requests_per_second).unwrap_or(DEFAULT_RPS),
        )
        .allow_burst(
            NonZeroU32::new(http_config.burst_size).unwrap_or(DEFAULT_BURST),
        );

        // Authentication endpoints get their own quota: 10 req/s with a burst
        // of 20 per client. This is the IMP-07 auth limit; it stops online
        // password guessing without throttling normal API traffic.
        const AUTH_RPS: NonZeroU32 = match NonZeroU32::new(10) {
            Some(v) => v,
            None => DEFAULT_RPS,
        };
        const AUTH_BURST: NonZeroU32 = match NonZeroU32::new(20) {
            Some(v) => v,
            None => DEFAULT_BURST,
        };
        let auth_quota = Quota::per_second(AUTH_RPS).allow_burst(AUTH_BURST);

        Self {
            http_limiter: Arc::new(GovernorRateLimiter::direct(quota)),
            auth_limiter: Arc::new(GovernorRateLimiter::keyed(auth_quota)),
            botlib_limiter: Arc::new(BotlibRateLimiter::new(system_limits)),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(HttpRateLimitConfig::default(), SystemLimits::default())
    }

    pub fn check_http_limit(&self) -> bool {
        self.http_limiter.check().is_ok()
    }

    /// Enforce the stricter per-client quota used by the authentication
    /// endpoints (login, refresh, bootstrap).
    pub fn check_auth_limit(&self, key: &str) -> bool {
        self.auth_limiter.check_key(&key.to_string()).is_ok()
    }

    pub async fn check_user_limit(&self, user_id: &str) -> Result<(), LimitExceeded> {
        self.botlib_limiter.check_rate_limit(user_id).await
    }

    pub fn botlib_limiter(&self) -> &Arc<BotlibRateLimiter> {
        &self.botlib_limiter
    }

    pub async fn cleanup(&self) {
        self.botlib_limiter.cleanup_stale_entries().await;
    }
}

impl Clone for CombinedRateLimiter {
    fn clone(&self) -> Self {
        Self {
            http_limiter: Arc::clone(&self.http_limiter),
            auth_limiter: Arc::clone(&self.auth_limiter),
            botlib_limiter: Arc::clone(&self.botlib_limiter),
        }
    }
}

pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    axum::Extension(limiter): axum::Extension<Arc<CombinedRateLimiter>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !limiter.check_http_limit() {
        return http_rate_limit_response(30);
    }

    let user_id = extract_user_id(&request).unwrap_or_else(|| addr.ip().to_string());

    match limiter.check_user_limit(&user_id).await {
        Ok(()) => next.run(request).await,
        Err(limit_exceeded) => {
            let (status, body) = format_limit_error_response(&limit_exceeded);
            (StatusCode::from_u16(status).unwrap_or(StatusCode::TOO_MANY_REQUESTS), body).into_response()
        }
    }
}

pub async fn simple_rate_limit_middleware(
    axum::Extension(limiter): axum::Extension<Arc<CombinedRateLimiter>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !limiter.check_http_limit() {
        return http_rate_limit_response(30);
    }
    next.run(request).await
}

/// Enforces the global HTTP quota and, for `/api/auth/*`, the stricter
/// per-client auth quota. The client key is the first `X-Forwarded-For` hop
/// (Caddy sets it) with `X-Real-IP` and the peer address as fallbacks.
pub async fn auth_rate_limit_middleware(
    addr: Option<ConnectInfo<SocketAddr>>,
    axum::Extension(limiter): axum::Extension<Arc<CombinedRateLimiter>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if !limiter.check_http_limit() {
        return http_rate_limit_response(30);
    }

    if request.uri().path().starts_with("/api/auth/") {
        let key = client_key(&request, addr.map(|info| info.0));
        if !limiter.check_auth_limit(&key) {
            return http_rate_limit_response(30);
        }
    }

    next.run(request).await
}

fn client_key(request: &Request<Body>, addr: Option<SocketAddr>) -> String {
    if let Some(first) = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return first.to_string();
    }

    if let Some(real_ip) = request
        .headers()
        .get("x-real-ip")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return real_ip.to_string();
    }

    addr.map(|socket| socket.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn extract_user_id(request: &Request<Body>) -> Option<String> {
    if let Some(user_id) = request.headers().get("x-user-id") {
        if let Ok(id) = user_id.to_str() {
            return Some(id.to_string());
        }
    }

    if let Some(auth) = request.headers().get("authorization") {
        if let Ok(auth_str) = auth.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if token.len() > 10 {
                    // The header is caller-supplied: never slice it mid-character.
                    return Some(format!(
                        "token:{}",
                        botlib::text::truncate_at_char_boundary(token, 10)
                    ));
                }
            }
        }
    }

    None
}

fn http_rate_limit_response(retry_after: u64) -> Response {
    let mut response = (
        StatusCode::TOO_MANY_REQUESTS,
        Json(json!({
            "error": "rate_limit_exceeded",
            "message": "Too many requests. Please slow down.",
            "retry_after_secs": retry_after
        })),
    )
        .into_response();

    if let Ok(value) = retry_after.to_string().parse() {
        response.headers_mut().insert("Retry-After", value);
    }

    response
}

pub fn create_rate_limit_layer(
    http_config: HttpRateLimitConfig,
    system_limits: SystemLimits,
) -> (
    axum::Extension<Arc<CombinedRateLimiter>>,
    Arc<CombinedRateLimiter>,
) {
    let limiter = Arc::new(CombinedRateLimiter::new(http_config, system_limits));
    (axum::Extension(Arc::clone(&limiter)), limiter)
}

pub fn create_default_rate_limit_layer() -> (
    axum::Extension<Arc<CombinedRateLimiter>>,
    Arc<CombinedRateLimiter>,
) {
    create_rate_limit_layer(HttpRateLimitConfig::default(), SystemLimits::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_config_presets() {
        let default = HttpRateLimitConfig::default();
        assert_eq!(default.requests_per_second, 100);

        let strict = HttpRateLimitConfig::strict();
        assert_eq!(strict.requests_per_second, 50);

        let relaxed = HttpRateLimitConfig::relaxed();
        assert_eq!(relaxed.requests_per_second, 500);

        let api = HttpRateLimitConfig::api();
        assert_eq!(api.requests_per_second, 100);
    }

    #[test]
    fn test_combined_limiter_creation() {
        let limiter = CombinedRateLimiter::with_defaults();
        assert!(limiter.check_http_limit());
    }

    #[test]
    fn test_combined_limiter_clone() {
        let limiter = CombinedRateLimiter::with_defaults();
        let cloned = limiter.clone();
        assert!(cloned.check_http_limit());
    }

    #[tokio::test]
    async fn test_user_rate_limit() {
        let limiter = CombinedRateLimiter::with_defaults();
        let result = limiter.check_user_limit("test-user").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_user_id_none() {
        let request = Request::builder()
            .body(Body::empty())
            .expect("request builds");
        assert!(extract_user_id(&request).is_none());
    }

    #[test]
    fn test_auth_rate_limit_rejects_a_burst_for_one_client() {
        let limiter = CombinedRateLimiter::with_defaults();
        let mut rejected = 0;
        for _ in 0..40 {
            if !limiter.check_auth_limit("198.51.100.7") {
                rejected += 1;
            }
        }
        assert!(rejected > 0, "a 40-request burst must be throttled");
        assert!(
            limiter.check_auth_limit("198.51.100.8"),
            "a different client must keep its own budget"
        );
    }

    #[test]
    fn test_client_key_prefers_forwarded_for() {
        let request = Request::builder()
            .header("x-forwarded-for", "203.0.113.9, 10.0.0.1")
            .body(Body::empty())
            .expect("request builds");
        assert_eq!(client_key(&request, None), "203.0.113.9");
    }
}
