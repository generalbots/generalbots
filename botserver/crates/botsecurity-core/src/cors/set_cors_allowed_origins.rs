use super::*;

pub fn set_cors_allowed_origins(origins: Vec<String>) {
    if let Ok(mut guard) = CORS_ALLOWED_ORIGINS.write() {
        info!("Setting CORS allowed origins: {:?}", origins);
        *guard = Some(origins);
    }
}
pub fn get_cors_allowed_origins() -> Vec<String> {
    get_allowed_origins_from_config()
}
/// Predicate form of the origin check, kept for callers that build their own
/// `CorsLayer`.
///
/// It fails closed: an empty configuration denies every cross-origin origin
/// (#2129). Only an explicit allowlist entry, or the documented
/// `CORS_ALLOW_ANY_ORIGIN=1` development override, can return `true`.
pub fn validate_origin(origin: &HeaderValue, _request: &axum::http::request::Parts) -> bool {
    let origin_str = match origin.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    if origin_str.is_empty() {
        return false;
    }

    match resolve_origin_policy(&get_allowed_origins_from_config()) {
        OriginPolicy::Deny => false,
        OriginPolicy::Allowlist(origins) => origins.iter().any(|allowed| allowed == origin_str),
        OriginPolicy::AnyNoCredentials => is_valid_origin_format(origin_str),
    }
}
pub(super) fn is_valid_origin_format(origin: &str) -> bool {
    // A request without an `Origin` header is same-origin by definition (a
    // direct navigation, or a non-browser client). There is no foreign origin
    // to validate, so the format check passes.
    if origin.is_empty() {
        return true;
    }

    if !origin.starts_with("http://") && !origin.starts_with("https://") {
        return false;
    }

    if origin.contains("..") || origin.matches("//").count() > 1 {
        return false;
    }

    let dangerous_patterns = [
        "<script",
        "javascript:",
        "data:",
        "vbscript:",
        "%3c",
        "%3e",
        "\\x",
        "\\u",
    ];

    let origin_lower = origin.to_lowercase();
    for pattern in &dangerous_patterns {
        if origin_lower.contains(pattern) {
            return false;
        }
    }

    true
}
/// Build the CORS layer for the process-wide configuration.
///
/// With no configured origin the result is same-origin only: no
/// `Access-Control-Allow-Origin` is emitted for a foreign origin, so a browser
/// blocks the cross-origin read (#2129).
pub fn create_cors_layer() -> CorsLayer {
    let configured = get_allowed_origins_from_config();
    cors_layer_for_policy(resolve_origin_policy(&configured), "create_cors_layer")
}
/// Middleware that strips CORS headers from responses when the request
/// came through a reverse proxy (detected via X-Forwarded-* headers).
/// This prevents duplicate Access-Control-Allow-Origin headers caused by
/// both the application and the proxy (e.g., Caddy) setting them.
pub async fn strip_proxy_cors_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let behind_proxy = request.headers().contains_key("x-forwarded-for")
        || request.headers().contains_key("x-forwarded-host")
        || request.headers().contains_key("x-forwarded-proto");

    let mut response = next.run(request).await;

    if behind_proxy {
        let headers = response.headers_mut();
        headers.remove(header::ACCESS_CONTROL_ALLOW_ORIGIN);
        headers.remove(header::ACCESS_CONTROL_ALLOW_METHODS);
        headers.remove(header::ACCESS_CONTROL_ALLOW_HEADERS);
        headers.remove(header::ACCESS_CONTROL_EXPOSE_HEADERS);
        headers.remove(header::ACCESS_CONTROL_MAX_AGE);
        headers.remove(header::ACCESS_CONTROL_ALLOW_CREDENTIALS);
    }

    response
}
/// Production builder: an empty list means "no cross-origin access", exactly
/// like the other builders (#2129).
pub fn create_cors_layer_for_production(allowed_origins: Vec<String>) -> CorsLayer {
    cors_layer_for_policy(
        resolve_origin_policy(&allowed_origins),
        "create_cors_layer_for_production",
    )
}
/// Explicit-allowlist builder. An empty list denies cross-origin access; it
/// never falls back to reflecting the caller's origin (#2129).
pub fn create_cors_layer_with_origins(origins: Vec<String>) -> CorsLayer {
    cors_layer_for_policy(
        resolve_origin_policy(&origins),
        "create_cors_layer_with_origins",
    )
}
fn cors_layer_for_policy(policy: OriginPolicy, context: &str) -> CorsLayer {
    log_origin_policy(&policy, context);

    let base = CorsConfig::production();

    build_cors_layer(
        &policy,
        base.allowed_methods,
        base.allowed_headers,
        base.exposed_headers,
        base.max_age_secs,
        base.allow_credentials,
    )
}
