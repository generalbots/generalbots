//! Rate Limiting Module
//!
//! Provides API rate limiting using the governor library.
//! Supports per-IP and per-user rate limiting with configurable limits.

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::{collections::HashMap, net::SocketAddr, num::NonZeroU32, sync::Arc};
use tokio::sync::RwLock;

/// Rate limiter type alias
type Limiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>;

/// Per-key rate limiter for IP-based or user-based limiting
pub struct KeyedRateLimiter {
    limiters: RwLock<HashMap<String, Arc<Limiter>>>,
    quota: Quota,
    cleanup_threshold: usize,
}

impl KeyedRateLimiter {
    /// Create a new keyed rate limiter
    pub fn new(requests_per_second: u32, burst_size: u32) -> Self {
        let quota =
            Quota::per_second(NonZeroU32::new(requests_per_second).unwrap_or(NonZeroU32::MIN))
                .allow_burst(NonZeroU32::new(burst_size).unwrap_or(NonZeroU32::MIN));

        Self {
            limiters: RwLock::new(HashMap::new()),
            quota,
            cleanup_threshold: 10000,
        }
    }

    /// Check if a key is rate limited
    pub async fn check(&self, key: &str) -> bool {
        let limiter = {
            let limiters = self.limiters.read().await;
            limiters.get(key).cloned()
        };

        let limiter = match limiter {
            Some(l) => l,
            None => {
                let mut limiters = self.limiters.write().await;

                // Cleanup old limiters if threshold exceeded
                if limiters.len() > self.cleanup_threshold {
                    limiters.clear();
                }

                let new_limiter = Arc::new(RateLimiter::direct(self.quota));
                limiters.insert(key.to_string(), Arc::clone(&new_limiter));
                new_limiter
            }
        };

        limiter.check().is_ok()
    }

    /// Get remaining quota for a key
    pub async fn remaining(&self, key: &str) -> Option<u32> {
        let limiters = self.limiters.read().await;
        limiters.get(key).map(|l| l.check().map(|_| 1).unwrap_or(0))
    }
}

impl std::fmt::Debug for KeyedRateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyedRateLimiter")
            .field("cleanup_threshold", &self.cleanup_threshold)
            .finish()
    }
}

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Requests per second for API endpoints
    pub api_rps: u32,
    /// Burst size for API endpoints
    pub api_burst: u32,
    /// Requests per second for auth endpoints (stricter)
    pub auth_rps: u32,
    /// Burst size for auth endpoints
    pub auth_burst: u32,
    /// Requests per second for LLM endpoints (most restrictive)
    pub llm_rps: u32,
    /// Burst size for LLM endpoints
    pub llm_burst: u32,
    /// Enable rate limiting
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            api_rps: 100,
            api_burst: 200,
            auth_rps: 10,
            auth_burst: 20,
            llm_rps: 5,
            llm_burst: 10,
            enabled: true,
        }
    }
}

/// Rate limit state shared across requests
#[derive(Debug)]
pub struct RateLimitState {
    pub config: RateLimitConfig,
    pub api_limiter: KeyedRateLimiter,
    pub auth_limiter: KeyedRateLimiter,
    pub llm_limiter: KeyedRateLimiter,
}

impl RateLimitState {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            api_limiter: KeyedRateLimiter::new(config.api_rps, config.api_burst),
            auth_limiter: KeyedRateLimiter::new(config.auth_rps, config.auth_burst),
            llm_limiter: KeyedRateLimiter::new(config.llm_rps, config.llm_burst),
            config,
        }
    }

    pub fn from_env() -> Self {
        let config = RateLimitConfig {
            api_rps: std::env::var("RATE_LIMIT_API_RPS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            api_burst: std::env::var("RATE_LIMIT_API_BURST")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(200),
            auth_rps: std::env::var("RATE_LIMIT_AUTH_RPS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            auth_burst: std::env::var("RATE_LIMIT_AUTH_BURST")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(20),
            llm_rps: std::env::var("RATE_LIMIT_LLM_RPS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            llm_burst: std::env::var("RATE_LIMIT_LLM_BURST")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            enabled: std::env::var("RATE_LIMIT_ENABLED")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true),
        };
        Self::new(config)
    }
}

/// Extract client IP from request
fn get_client_ip(req: &Request) -> String {
    // Try X-Forwarded-For header first (for reverse proxies)
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
            if let Some(ip) = value.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }

    // Try X-Real-IP header
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(value) = real_ip.to_str() {
            return value.to_string();
        }
    }

    // Fall back to connection info
    req.extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Determine which limiter to use based on path
fn get_limiter_type(path: &str) -> LimiterType {
    if path.contains("/auth") || path.contains("/login") || path.contains("/token") {
        LimiterType::Auth
    } else if path.contains("/llm") || path.contains("/chat") || path.contains("/generate") {
        LimiterType::Llm
    } else {
        LimiterType::Api
    }
}

#[derive(Debug, Clone, Copy)]
enum LimiterType {
    Api,
    Auth,
    Llm,
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(state): State<Arc<RateLimitState>>,
    req: Request,
    next: Next,
) -> Response {
    if !state.config.enabled {
        return next.run(req).await;
    }

    let client_ip = get_client_ip(&req);
    let path = req.uri().path();
    let limiter_type = get_limiter_type(path);

    let allowed = match limiter_type {
        LimiterType::Api => state.api_limiter.check(&client_ip).await,
        LimiterType::Auth => state.auth_limiter.check(&client_ip).await,
        LimiterType::Llm => state.llm_limiter.check(&client_ip).await,
    };

    if allowed {
        next.run(req).await
    } else {
        rate_limit_response(limiter_type)
    }
}

/// Generate rate limit exceeded response
fn rate_limit_response(limiter_type: LimiterType) -> Response {
    let (retry_after, message) = match limiter_type {
        LimiterType::Api => (1, "API rate limit exceeded"),
        LimiterType::Auth => (
            60,
            "Authentication rate limit exceeded. Please wait before trying again.",
        ),
        LimiterType::Llm => (
            10,
            "LLM rate limit exceeded. Please wait before sending another request.",
        ),
    };

    let body = serde_json::json!({
        "error": "rate_limit_exceeded",
        "message": message,
        "retry_after": retry_after
    });

    (
        StatusCode::TOO_MANY_REQUESTS,
        [
            ("Retry-After", retry_after.to_string()),
            ("Content-Type", "application/json".to_string()),
        ],
        body.to_string(),
    )
        .into_response()
}

/// Create rate limit state for use with axum middleware
pub fn create_rate_limit_state(config: RateLimitConfig) -> Arc<RateLimitState> {
    Arc::new(RateLimitState::new(config))
}
