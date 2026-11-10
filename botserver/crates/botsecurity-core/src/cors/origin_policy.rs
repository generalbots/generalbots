//! Fail-closed origin policy shared by every CORS builder (#2129).
//!
//! There are exactly three behaviours, resolved in one place so no builder can
//! drift back to "reflect whatever `Origin` arrives":
//!
//! | Resolved policy          | `Access-Control-Allow-Origin` | credentials |
//! |--------------------------|-------------------------------|-------------|
//! | `Deny` (empty config)    | never emitted                 | never       |
//! | `Allowlist(origins)`     | exact echo of a listed origin | listed only |
//! | `AnyNoCredentials` (dev) | echo of a format-valid origin | never       |
//!
//! An empty configuration means **same-origin only**: the response carries no
//! CORS header, so a browser blocks every cross-origin read. The historical
//! default — reflecting any `http(s)` origin together with
//! `Access-Control-Allow-Credentials: true` — is gone.
//!
//! The development escape hatch is `CORS_ALLOW_ANY_ORIGIN=1`. It is off unless
//! explicitly set, it only accepts format-valid origins, and it always refuses
//! credentials, so the classic "reflected origin + cookies" attack stays closed
//! even when the override is left enabled by mistake.

use super::*;
use tower_http::cors::AllowCredentials;

/// Environment switch that re-enables cross-origin access for local
/// development. Unset (or any value other than `1`/`true`/`yes`/`on`) keeps the
/// fail-closed default; when set it never enables credentials.
pub const CORS_ALLOW_ANY_ORIGIN_ENV: &str = "CORS_ALLOW_ANY_ORIGIN";

/// What the layer is allowed to do with an incoming `Origin` header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OriginPolicy {
    /// No cross-origin access: no `Access-Control-Allow-Origin` is emitted.
    Deny,
    /// Exact-match allowlist. Credentials are permitted for listed origins.
    Allowlist(Vec<String>),
    /// Development override: any format-valid origin, credentials refused.
    AnyNoCredentials,
}

/// True when the documented development override is enabled.
pub fn allow_any_origin_override() -> bool {
    match std::env::var(CORS_ALLOW_ANY_ORIGIN_ENV) {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

/// Normalise a configured origin list: trim entries, drop empties, drop
/// duplicates and drop anything that cannot be an exact `Origin` header value.
///
/// A `*` entry is dropped as well. A wildcard can never belong to a
/// credentialed allowlist, and silently widening it to "allow everything" is
/// exactly the bug this module closes; deployments must list their origins.
pub fn normalize_origins(origins: &[String]) -> Vec<String> {
    let mut normalized: Vec<String> = Vec::new();

    for raw in origins {
        let candidate = raw.trim();

        if candidate.is_empty() || candidate == "*" {
            continue;
        }

        if candidate.parse::<HeaderValue>().is_err() {
            continue;
        }

        if normalized
            .iter()
            .any(|existing| existing.as_str() == candidate)
        {
            continue;
        }

        normalized.push(candidate.to_string());
    }

    normalized
}

/// Resolve the policy for a configuration snapshot.
///
/// Explicit origins always win (an operator who configured an allowlist must
/// never be widened by the dev switch); an empty list means deny unless the dev
/// override is explicitly enabled.
pub fn resolve_origin_policy(configured: &[String]) -> OriginPolicy {
    let allowlist = normalize_origins(configured);

    if !allowlist.is_empty() {
        return OriginPolicy::Allowlist(allowlist);
    }

    if allow_any_origin_override() {
        return OriginPolicy::AnyNoCredentials;
    }

    OriginPolicy::Deny
}

/// Log the resolved policy once per layer construction, so a deployment can see
/// from its own logs whether cross-origin access is off, allowlisted or in the
/// development override.
pub(super) fn log_origin_policy(policy: &OriginPolicy, context: &str) {
    match policy {
        OriginPolicy::Deny => info!(
            "{context}: no CORS origin configured — cross-origin access disabled \
             (same-origin only); set cors-allowed-origins to allow explicit origins"
        ),
        OriginPolicy::Allowlist(origins) => info!(
            "{context}: CORS restricted to {} configured origin(s)",
            origins.len()
        ),
        OriginPolicy::AnyNoCredentials => info!(
            "{context}: {CORS_ALLOW_ANY_ORIGIN_ENV} is set — allowing any format-valid \
             origin WITHOUT credentials (development only)"
        ),
    }
}

/// Exact-match check used by the credentials predicate.
fn is_listed_origin(allowlist: &[String], origin: &HeaderValue) -> bool {
    match origin.to_str() {
        Ok(candidate) => allowlist
            .iter()
            .any(|allowed| allowed.as_str() == candidate),
        Err(_) => false,
    }
}

/// Build the layer for a resolved policy.
///
/// `Deny` deliberately configures nothing: tower-http's default origin set is an
/// empty list, so no `Access-Control-Allow-Origin` is emitted and a preflight
/// response carries no CORS grant the browser would accept.
pub(super) fn build_cors_layer(
    policy: &OriginPolicy,
    methods: Vec<Method>,
    allowed_headers: Vec<String>,
    exposed_headers: Vec<String>,
    max_age_secs: u64,
    allow_credentials: bool,
) -> CorsLayer {
    let mut cors = CorsLayer::new();

    match policy {
        OriginPolicy::Deny => return cors,
        OriginPolicy::Allowlist(origins) => {
            let values: Vec<HeaderValue> = origins
                .iter()
                .filter_map(|origin| origin.parse::<HeaderValue>().ok())
                .collect();

            if values.is_empty() {
                return cors;
            }

            cors = cors.allow_origin(AllowOrigin::list(values));

            if allow_credentials {
                // Credentials are only announced for an origin that is on the
                // list — never for a caller-supplied origin that merely looks
                // like one.
                let allowlist = origins.clone();
                cors = cors.allow_credentials(AllowCredentials::predicate(
                    move |origin: &HeaderValue, _parts: &axum::http::request::Parts| {
                        is_listed_origin(&allowlist, origin)
                    },
                ));
            }
        }
        OriginPolicy::AnyNoCredentials => {
            cors = cors.allow_origin(AllowOrigin::predicate(validate_origin));
            // Credentials are intentionally never combined with a reflected
            // origin: a browser must not attach cookies to this dev-only mode.
        }
    }

    cors = cors.allow_methods(methods);

    let headers: Vec<header::HeaderName> = allowed_headers
        .iter()
        .filter_map(|name| name.parse().ok())
        .collect();
    cors = cors.allow_headers(headers);

    let exposed: Vec<header::HeaderName> = exposed_headers
        .iter()
        .filter_map(|name| name.parse().ok())
        .collect();
    cors = cors.expose_headers(exposed);

    cors.max_age(std::time::Duration::from_secs(max_age_secs))
}
