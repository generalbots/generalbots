use super::*;
use crate::auth::{AuthenticatedUser, Role};
use botsecurity_core::audit::{
    record_security_decision, AuditActor, AuditEventType, AuditOutcome, SecurityDecision,
};

/// #2765 — write a CSRF rejection to the tamper-evident audit chain.
///
/// The actor is taken from the server-populated request extension (never from
/// a header), and only `method path` is fingerprinted: tokens, cookies and
/// form values are not part of the record.
/// Extract the audit actor into owned data. `Request<Body>` is not `Sync`, so
/// a reference to it must never be held across an `.await` in an async fn.
fn csrf_audit_actor(request: &Request<Body>) -> AuditActor {
    match request.extensions().get::<AuthenticatedUser>() {
        Some(user) if user.user_id != uuid::Uuid::nil() => {
            let mut actor = AuditActor::user(user.user_id).with_username(user.username.clone());
            if let Some(ref session) = user.session_id {
                actor = actor.with_session(session.clone());
            }
            actor
        }
        _ => AuditActor::anonymous(),
    }
}

async fn audit_csrf_reject(
    actor: AuditActor,
    method: &str,
    path: &str,
    reason_code: &'static str,
) {
    let payload = format!("{method} {path}");
    let decision = SecurityDecision {
        event_type: AuditEventType::UnauthorizedAccess,
        actor,
        outcome: AuditOutcome::Failure,
        action: "csrf.reject",
        reason_code,
        payload: &payload,
    };
    if let Err(error) = record_security_decision(&decision).await {
        warn!("CSRF security decision audit write failed: {error}");
    }
}

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

/// Outcome of the pure CSRF exemption decision (#2683).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CsrfDecision {
    /// The request is safe (read-only) or authenticated by a non-cookie means.
    Skip,
    /// A credential-entry POST carried a foreign Origin/Referer (#2815).
    RejectCrossOrigin,
    /// A non-loopback Origin/Referer in `no_security` desktop mode.
    RejectNonLoopback,
    /// No exemption applies: the double-submit token must be validated.
    Validate,
}

/// Pure exemption decision, free of axum types so it can be unit-tested
/// without a running server (#2683).
///
/// The order preserves the previous middleware behaviour exactly:
/// read-only methods → same-origin checks → path exemptions → the `no_security`
/// loopback guard → Bearer/internal-service identities → token validation.
///
/// Crucially, a path exemption is only one input: a cookie/session-authenticated
/// state-changing request whose path is NOT exempt reaches `Validate`, so no
/// ambient-credential write is left unguarded. Bearer callers are stateless and
/// skip the token check here, independent of any path. `loopback_verdict` is
/// `Some(true/false)` for a loopback / absent Origin and `None` for a foreign
/// non-loopback Origin, keeping the desktop-mode CSRF guard active.
fn csrf_decision(
    config: &CsrfConfig,
    method: &str,
    path: &str,
    has_bearer: bool,
    is_same_origin: bool,
    loopback_verdict: Option<bool>,
    is_internal_service: bool,
) -> CsrfDecision {
    if !config.enabled {
        return CsrfDecision::Skip;
    }

    let exempt_method = config
        .exempt_methods
        .iter()
        .any(|m| m.eq_ignore_ascii_case(method));
    if exempt_method {
        return CsrfDecision::Skip;
    }

    // #2815 — login/signup cannot carry a double-submit token (the forms post
    // JSON), so require a same-origin Origin/Referer before the path exemption
    // can apply. Bearer callers stay exempt.
    let origin_check_required = config
        .origin_check_paths
        .iter()
        .any(|pattern| path_matches(pattern, path));
    if origin_check_required && !has_bearer && !is_same_origin {
        return CsrfDecision::RejectCrossOrigin;
    }

    let exempt_path = config
        .exempt_paths
        .iter()
        .any(|pattern| path_matches(pattern, path));
    if exempt_path {
        return CsrfDecision::Skip;
    }

    // CSRF stays enabled in `no_security` desktop mode: a browser cross-site
    // request sends a non-loopback Origin/Referer and is rejected. A loopback
    // origin or no Origin at all (a local, non-browser tool) is allowed — the
    // loopback check replaces the double-submit token the WebView cannot send.
    if config.no_security_mode {
        return match loopback_verdict {
            None => CsrfDecision::RejectNonLoopback,
            Some(_) => CsrfDecision::Skip,
        };
    }

    // Bearer tokens are stateless; a cross-origin page cannot read one from a
    // cookie jar. The internal service identity is inserted by the auth
    // middleware after matching the configured secret (#2122).
    if has_bearer || is_internal_service {
        return CsrfDecision::Skip;
    }

    CsrfDecision::Validate
}

pub async fn csrf_middleware(
    csrf_manager: Arc<CsrfManager>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let config = csrf_manager.config();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    // Gather the request facts as pure values, then delegate the decision so
    // it can be exercised in tests without an axum runtime (#2683).
    let has_bearer = has_bearer_auth(&request);
    let is_same_origin = is_same_origin(&request);
    let loopback_verdict = match loopback_origin_ok(&request) {
        Ok(value) => Some(value),
        Err(()) => None,
    };
    let is_internal_service = request
        .extensions()
        .get::<AuthenticatedUser>()
        .is_some_and(|user| user.has_role(&Role::Service) && user.username == "service:internal");

    let decision = csrf_decision(
        config,
        &method,
        &path,
        has_bearer,
        is_same_origin,
        loopback_verdict,
        is_internal_service,
    );

    match decision {
        CsrfDecision::Skip => return next.run(request).await,
        CsrfDecision::RejectCrossOrigin => {
            warn!("CSRF blocked cross-origin {} {} (#2815)", method, path);
            audit_csrf_reject(csrf_audit_actor(&request), &method, &path, "csrf.cross_origin").await;
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": "csrf_validation_failed",
                    "message": "Cross-origin requests are not allowed for this endpoint"
                })),
            )
                .into_response();
        }
        CsrfDecision::RejectNonLoopback => {
            warn!(
                "CSRF blocked non-loopback origin for {} {} in no_security mode",
                method, path
            );
            audit_csrf_reject(csrf_audit_actor(&request), &method, &path, "csrf.non_loopback").await;
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "error": "csrf_validation_failed",
                    "message": "Cross-origin requests are not allowed in desktop-local mode"
                })),
            )
                .into_response();
        }
        CsrfDecision::Validate => {}
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
        audit_csrf_reject(csrf_audit_actor(&request), &method, &path, "csrf.token_invalid").await;

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
        let req = request("login.generalbots.org", None, Some("https://login.generalbots.org"), None);
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
        let req = request("login.generalbots.org", None, Some("https://evil.example"), None);
        assert!(!is_same_origin(&req));
    }

    #[test]
    fn rejects_a_request_without_origin_or_referer() {
        let req = request("login.generalbots.org", None, None, None);
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

#[cfg(test)]
#[path = "csrf_middleware_exemption_tests.rs"]
mod exemption_decision_tests;
