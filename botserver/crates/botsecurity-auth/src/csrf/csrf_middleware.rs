use super::*;
use crate::auth::{AuthenticatedUser, Role};

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

    if csrf_manager.is_exempt_path(&path) {
        return next.run(request).await;
    }

    // Skip CSRF for API requests using Bearer token auth (stateless)
    let is_bearer = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("Bearer "));
    if is_bearer {
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
