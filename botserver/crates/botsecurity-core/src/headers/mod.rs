pub(crate) use axum::{
    body::Body,
    http::{header::HeaderName, HeaderValue, Request},
    middleware::Next,
    response::Response,
    Router,
};
pub(crate) use std::collections::HashMap;

/// Environment variable that drops HSTS for plain-HTTP local development.
///
/// HSTS is emitted by default on every server. A browser that receives it on a
/// `http://localhost` origin pins that host to HTTPS for the whole `max-age`,
/// and the dev server does not speak TLS — the tab then refuses to load at all
/// and the only cure is clearing the browser's transport-security state.
/// Local dev therefore sets `GB_SECURITY_HSTS=off`; anything else (unset,
/// `1`, `on`, `true`) keeps HSTS. Production never sets it.
pub const HSTS_ENV: &str = "GB_SECURITY_HSTS";

/// The IMP-09 header set. One config drives every server in the workspace.
#[derive(Debug, Clone)]
pub struct SecurityHeadersConfig {
    pub content_security_policy: Option<String>,
    pub x_frame_options: Option<String>,
    pub x_content_type_options: Option<String>,
    pub x_xss_protection: Option<String>,
    pub strict_transport_security: Option<String>,
    pub referrer_policy: Option<String>,
    pub permissions_policy: Option<String>,
    pub cache_control: Option<String>,
    pub custom_headers: HashMap<String, String>,
}

pub struct CspBuilder {
    directives: HashMap<String, Vec<String>>,
}

impl SecurityHeadersConfig {
    /// The config every public server uses: the strict default, with HSTS
    /// dropped only when [`HSTS_ENV`] explicitly asks for it.
    pub fn from_env() -> Self {
        if hsts_enabled() {
            Self::default()
        } else {
            Self::default().disable_hsts()
        }
    }
}

/// Whether HSTS may be emitted for this process. See [`HSTS_ENV`].
pub fn hsts_enabled() -> bool {
    hsts_value_enabled(std::env::var(HSTS_ENV).ok().as_deref())
}

/// Interprets one `GB_SECURITY_HSTS` value. Split from [`hsts_enabled`] so the
/// decision is testable without mutating process-wide state.
fn hsts_value_enabled(value: Option<&str>) -> bool {
    match value {
        Some(raw) => !matches!(
            raw.trim().to_ascii_lowercase().as_str(),
            "0" | "off" | "false" | "no"
        ),
        None => true,
    }
}

/// Applies the IMP-09 headers to every response a router produces.
///
/// This is the single place a router becomes compliant: call it once, after the
/// router is fully built. Axum only applies a layer to routes that already
/// exist, so the call must follow the last `route`/`nest`/`fallback`. Because
/// it wraps the whole router, fallbacks, nested static services (`ServeDir`)
/// and error responses — 401/403/404/429/500 — carry the headers too.
pub fn with_security_headers<S>(router: Router<S>, config: SecurityHeadersConfig) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router.layer(axum::middleware::from_fn(
        move |request: Request<Body>, next: Next| {
            let config = config.clone();
            async move {
                let mut response = next.run(request).await;
                apply_security_headers(&mut response, &config);
                response
            }
        },
    ))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

mod default_src;
mod strict;

pub use strict::*;
