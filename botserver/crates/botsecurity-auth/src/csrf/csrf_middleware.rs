use super::*;
use crate::auth::{AuthenticatedUser, Role};

fn is_loopback_host(host: &str) -> bool {
    matches!(
        host.to_ascii_lowercase().as_str(),
        "localhost" | "127.0.0.1" | "::1" | "[::1]"
    )
}

fn origin_or_referer_host(value: &str) -> Option<String> {
    let parsed = url::Url::parse(value).ok()?;
    parsed.host_str().map(ToString::to_string)
}

fn has_bearer_auth(request: &Request<Body>) -> bool {
    request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("Bearer "))
}

fn path_matches(pattern: &str, path: &str) -> bool {
    match pattern.strip_suffix('*') {
        Some(prefix) => path.starts_with(prefix),
        None => pattern == path,
    }
}

/// Host and explicit port of an authority (`example.com:5000`, `[::1]:5000`).
fn authority_parts(authority: &str) -> Option<(String, Option<u16>)> {
    let parsed = url::Url::parse(&format!("http://{authority}")).ok()?;
    Some((parsed.host_str()?.to_ascii_lowercase(), parsed.port()))
}

/// The host the browser believes it is talking to. The botui reverse proxy
/// drops `Host` and forwards the original in `x-forwarded-host` (see
/// `botui/src/ui_server/proxy.rs`), so prefer that header when present.
fn expected_host(request: &Request<Body>) -> Option<(String, Option<u16>)> {
    if let Some(forwarded) = request
        .headers()
        .get("x-forwarded-host")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(str::trim)
    {
        if let Some(parts) = authority_parts(forwarded) {
            return Some(parts);
        }
    }
    request
        .headers()
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .and_then(authority_parts)
}

fn origin_matches(expected: &(String, Option<u16>), value: &str) -> bool {
    let parsed = match url::Url::parse(value) {
        Ok(parsed) => parsed,
        Err(_) => return false,
    };
    let host = match parsed.host_str() {
        Some(host) => host.to_ascii_lowercase(),
        None => return false,
    };
    if host != expected.0 {
        return false;
    }
    match (parsed.port(), expected.1) {
        (Some(from), Some(to)) => from == to,
        // A proxy may add or drop a port. When only one side declares one, the
        // host match is the meaningful same-origin signal.
        _ => true,
    }
}

/// #2815 — same-origin check for the auth POSTs that are exempt from the
/// double-submit token. Browsers send `Origin` on every POST; `Referer` is the
/// fallback for older clients. Both absent means "not proven", so fail closed.
fn is_same_origin(request: &Request<Body>) -> bool {
    let expected = match expected_host(request) {
        Some(expected) => expected,
        None => return false,
    };
    if let Some(origin) = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
    {
        return origin_matches(&expected, origin);
    }
    if let Some(referer) = request
        .headers()
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
    {
        return origin_matches(&expected, referer);
    }
    false
}

fn loopback_origin_ok(request: &Request<Body>) -> Result<bool, ()> {
    if let Some(origin) = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
    {
        if origin.eq_ignore_ascii_case("null") {
            return Err(());
        }
        let host = origin_or_referer_host(origin).ok_or(())?;
        return if is_loopback_host(&host) {
            Ok(true)
        } else {
            Err(())
        };
    }

    if let Some(referer) = request
        .headers()
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
    {
        let host = origin_or_referer_host(referer).ok_or(())?;
        return if is_loopback_host(&host) {
            Ok(true)
        } else {
            Err(())
        };
    }

    Ok(false)
}

pub async fn csrf_middleware(
    csrf_manager: Arc<CsrfManager>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let config = csrf_manager.config();

    if !config.enabled {
        return next.run(request).await;
    }

    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    if csrf_manager.is_exempt_method(&method) {
        return next.run(request).await;
    }

    // #2815 — the login/signup POSTs cannot carry a double-submit token (the
    // forms post JSON), so they are exempted from the token check. Enforce
    // same-origin Origin/Referer validation on them instead, before the
    // blanket exemption applies, so a foreign site cannot drive a session-
    // fixation login/signup. Bearer/API callers stay exempt.
    if config
        .origin_check_paths
        .iter()
        .any(|pattern| path_matches(pattern, &path))
        && !has_bearer_auth(&request)
        && !is_same_origin(&request)
    {
        warn!("CSRF blocked cross-origin {} {} (#2815)", method, path);
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "csrf_validation_failed",
                "message": "Cross-origin requests are not allowed for this endpoint"
            })),
        )
            .into_response();
    }

    if csrf_manager.is_exempt_path(&path) {
        return next.run(request).await;
    }

    if config.no_security_mode {
        match loopback_origin_ok(&request) {
            Ok(true) => return next.run(request).await,
            Ok(false) => {
                // No Origin/Referer: non-browser/local clients on a loopback-bound
                // desktop server are allowed; browser cross-site requests send Origin.
                return next.run(request).await;
            }
            Err(()) => {
                warn!(
                    "CSRF blocked non-loopback origin for {} {} in no_security mode",
                    method, path
                );
                return (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({
                        "error": "csrf_validation_failed",
                        "message": "Cross-origin requests are not allowed in desktop-local mode"
                    })),
                )
                    .into_response();
            }
        }
    }

    // Skip CSRF for API requests using Bearer token auth (stateless)
    if has_bearer_auth(&request) {
        return next.run(request).await;
    }

    // Skip CSRF only for the verified internal service identity that the auth
    // middleware inserted after matching `X-Internal-Token` against the
    // configured secret (e.g. the VIBE RUN BASIC keyword calling back into the
    // local API). A caller-supplied header value is never an identity, so a
    // bare `X-User-ID`/`X-Internal-Token` string must not exempt a request
    // from CSRF (#2122).
    let is_internal_service = request
        .extensions()
        .get::<AuthenticatedUser>()
        .is_some_and(|user| user.has_role(&Role::Service) && user.username == "service:internal");
    if is_internal_service {
        return next.run(request).await;
    }

    let header_token = request
        .headers()
        .get(&config.header_name)
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let cookie_token = request
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|c| extract_csrf_from_cookie(c, &config.cookie_name));

    let validation_result = if config.double_submit_cookie {
        match (cookie_token.as_ref(), header_token.as_ref()) {
            (Some(cookie), Some(hdr)) => csrf_manager.validate_double_submit(cookie, hdr),
            _ => CsrfValidationResult::Missing,
        }
    } else {
        match header_token.as_ref() {
            Some(token) => csrf_manager.validate_token(token).await,
            None => CsrfValidationResult::Missing,
        }
    };

    if !validation_result.is_valid() {
        warn!(
            "CSRF validation failed for {} {}: {}",
            method,
            path,
            validation_result.error_message()
        );

        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "error": "csrf_validation_failed",
                "message": validation_result.error_message()
            })),
        )
            .into_response();
    }

    next.run(request).await
}

#[cfg(test)]
mod origin_tests {
    use super::*;

    fn request(host: &str, forwarded: Option<&str>, origin: Option<&str>, referer: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().method("POST").uri("/api/cloud/auth/login");
        builder = builder.header(header::HOST, host);
        if let Some(value) = forwarded {
            builder = builder.header("x-forwarded-host", value);
        }
        if let Some(value) = origin {
            builder = builder.header(header::ORIGIN, value);
        }
        if let Some(value) = referer {
            builder = builder.header(header::REFERER, value);
        }
        builder.body(Body::empty()).expect("test request is valid")
    }

    #[test]
    fn accepts_a_same_origin_header() {
        let req = request("login.pragmatismo.com.br", None, Some("https://login.pragmatismo.com.br"), None);
        assert!(is_same_origin(&req));
    }

    #[test]
    fn accepts_the_forwarded_host_and_a_referer_fallback() {
        // botui drops Host and forwards the original in x-forwarded-host.
        let req = request("localhost:8080", Some("localhost:5000"), Some("http://localhost:5000"), None);
        assert!(is_same_origin(&req));

        let req = request("localhost:8080", Some("localhost:5000"), None, Some("http://localhost:5000/signup"));
        assert!(is_same_origin(&req));
    }

    #[test]
    fn rejects_a_foreign_origin_even_with_a_matching_forwarded_host() {
        let req = request("login.pragmatismo.com.br", None, Some("https://evil.example"), None);
        assert!(!is_same_origin(&req));
    }

    #[test]
    fn rejects_a_request_without_origin_or_referer() {
        let req = request("login.pragmatismo.com.br", None, None, None);
        assert!(!is_same_origin(&req));
    }

    #[test]
    fn origin_check_paths_support_exact_and_wildcard_matching() {
        let paths = vec!["/api/cloud/auth/login".to_string(), "/api/cloud/auth/*".to_string()];
        assert!(paths.iter().any(|p| path_matches(p, "/api/cloud/auth/login")));
        assert!(paths.iter().any(|p| path_matches(p, "/api/cloud/auth/signup")));
        assert!(!paths.iter().any(|p| path_matches(p, "/api/cloud/profile")));
    }
}
