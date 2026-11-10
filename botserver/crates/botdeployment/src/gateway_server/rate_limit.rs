//! IMP-07 — the deploy gateway is served by its own listener (port 5860), so
//! the rate-limit layer installed by `main_module::server::server_layers`
//! never sees these requests. Install the canonical limiter here instead of
//! inventing a second policy.
//!
//! `validate_deploy_key` compares the `X-Deploy-Key` header against a static
//! list, so without a quota the endpoint is an unlimited guessing oracle for a
//! secret that can create containers and rewrite Caddy routes.

use std::sync::Arc;

use axum::Extension;
use botlib::limits::SystemLimits;
use botsecurity_auth::rate_limiter::{CombinedRateLimiter, HttpRateLimitConfig};

/// The gateway shares the main server's shape (global HTTP quota + a per-client
/// budget) but with a tighter ceiling: alm-ci sends a handful of deploys per
/// release, never a sustained stream.
pub(super) const GATEWAY_REQUESTS_PER_SECOND: u32 = 20;
pub(super) const GATEWAY_BURST: u32 = 40;

/// Builds the limiter and the extension the middleware extracts.
pub(super) fn gateway_rate_limit(
) -> (Extension<Arc<CombinedRateLimiter>>, Arc<CombinedRateLimiter>) {
    let limiter = Arc::new(CombinedRateLimiter::new(
        HttpRateLimitConfig {
            requests_per_second: GATEWAY_REQUESTS_PER_SECOND,
            burst_size: GATEWAY_BURST,
        },
        SystemLimits::default(),
    ));
    (Extension(Arc::clone(&limiter)), limiter)
}

#[cfg(test)]
mod tests {
    use super::{gateway_rate_limit, GATEWAY_BURST, GATEWAY_REQUESTS_PER_SECOND};
    use botsecurity_auth::rate_limiter::HttpRateLimitConfig;

    #[test]
    fn the_gateway_quota_is_tighter_than_the_api_default() {
        let api = HttpRateLimitConfig::api();
        assert!(GATEWAY_REQUESTS_PER_SECOND < api.requests_per_second);
        assert!(GATEWAY_BURST < api.burst_size);
    }

    #[test]
    fn the_limiter_accepts_a_first_request() {
        let (_extension, limiter) = gateway_rate_limit();
        assert!(limiter.check_http_limit());
    }
}
