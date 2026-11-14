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
    collections::HashSet,
    net::{IpAddr, SocketAddr},
    num::NonZeroU32,
    sync::{Arc, OnceLock},
};

pub type GlobalRateLimiter = GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Per-client limiter for the authentication endpoints. Keyed by the resolved
/// client address (trusted-proxy aware), so one client cannot exhaust the login
/// endpoint for everyone and spoofed forwarding headers from untrusted peers
/// cannot rotate the key.
pub type AuthKeyedRateLimiter =
    GovernorRateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>;
const TRUSTED_PROXIES_ENV: &str = "AUTH_TRUSTED_PROXIES";

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
    /// Strict profile for authentication surfaces. Matches IMP-07: 10 req/s
    /// with a small burst to absorb legitimate retries while stopping
    /// brute-force attempts.
    pub fn strict() -> Self {
        Self {
            requests_per_second: 10,
            burst_size: 10,
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
    static_limiter: Arc<GlobalRateLimiter>,
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
        .allow_burst(NonZeroU32::new(http_config.burst_size).unwrap_or(DEFAULT_BURST));
        // Authentication endpoints get their own quota: 10 req/s with a burst
        // of 10 per client (IMP-07). This stops online password guessing without
        // throttling normal API traffic.
        const AUTH_RPS: NonZeroU32 = match NonZeroU32::new(10) {
            Some(v) => v,
            None => DEFAULT_RPS,
        };
        const AUTH_BURST: NonZeroU32 = match NonZeroU32::new(10) {
            Some(v) => v,
            None => DEFAULT_BURST,
        };
        let auth_quota = Quota::per_second(AUTH_RPS).allow_burst(AUTH_BURST);

        Self {
            http_limiter: Arc::new(GovernorRateLimiter::direct(quota)),
            static_limiter: Arc::new(GovernorRateLimiter::direct(
                super::rate_limiter_static::static_asset_quota(),
            )),
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

    /// Quota for the suite's own static files, kept apart from the API quota
    /// (see `rate_limiter_static`).
    pub fn check_static_limit(&self) -> bool {
        self.static_limiter.check().is_ok()
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
            static_limiter: Arc::clone(&self.static_limiter),
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

    let user_id = addr.ip().to_string();

    match limiter.check_user_limit(&user_id).await {
        Ok(()) => next.run(request).await,
        Err(limit_exceeded) => {
            let (status, body) = format_limit_error_response(&limit_exceeded);
            (
                StatusCode::from_u16(status).unwrap_or(StatusCode::TOO_MANY_REQUESTS),
                body,
            )
                .into_response()
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
/// per-client auth quota. Forwarding headers are trusted only when the direct
/// peer is a configured trusted proxy; otherwise the peer IP is used so clients
/// cannot spoof `X-Forwarded-For`/`X-Real-IP` to bypass brute-force protection.
pub async fn auth_rate_limit_middleware(
    addr: Option<ConnectInfo<SocketAddr>>,
    axum::Extension(limiter): axum::Extension<Arc<CombinedRateLimiter>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if super::rate_limiter_static::is_static_asset_request(request.method(), request.uri().path()) {
        if !limiter.check_static_limit() {
            return http_rate_limit_response(1);
        }
        return next.run(request).await;
    }

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
fn trusted_proxies() -> &'static HashSet<IpAddr> {
    static TRUSTED: OnceLock<HashSet<IpAddr>> = OnceLock::new();
    TRUSTED.get_or_init(|| {
        std::env::var(TRUSTED_PROXIES_ENV)
            .ok()
            .map(|value| parse_trusted_proxies(&value))
            .unwrap_or_default()
    })
}
fn parse_trusted_proxies(value: &str) -> HashSet<IpAddr> {
    value
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .filter_map(|entry| entry.parse::<IpAddr>().ok())
        .collect()
}
fn is_trusted_proxy(ip: IpAddr) -> bool {
    trusted_proxies().contains(&ip)
}
fn forwarded_client_ip(request: &Request<Body>, trusted: &HashSet<IpAddr>) -> Option<IpAddr> {
    if let Some(chain) = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
    {
        let mut rightmost_untrusted = None;
        for part in chain.split(',') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(ip) = trimmed.parse::<IpAddr>() {
                if trusted.contains(&ip) {
                    continue;
                }
                rightmost_untrusted = Some(ip);
            }
        }
        if let Some(ip) = rightmost_untrusted {
            return Some(ip);
        }
    }

    request
        .headers()
        .get("x-real-ip")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<IpAddr>().ok())
        .filter(|ip| !trusted.contains(ip))
}
fn client_key(request: &Request<Body>, addr: Option<SocketAddr>) -> String {
    let peer_ip = addr.map(|socket| socket.ip());

    match peer_ip {
        Some(peer) if is_trusted_proxy(peer) => {
            if let Some(client) = forwarded_client_ip(request, trusted_proxies()) {
                return client.to_string();
            }
            if let Some(chain) = request
                .headers()
                .get("x-forwarded-for")
                .and_then(|value| value.to_str().ok())
            {
                if let Some(last) = chain
                    .split(',')
                    .map(str::trim)
                    .rfind(|value| !value.is_empty())
                {
                    return last.to_string();
                }
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
            peer.to_string()
        }
        Some(peer) => peer.to_string(),
        None => "unknown".to_string(),
    }
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
#[path = "rate_limiter_tests.rs"]
mod tests;
