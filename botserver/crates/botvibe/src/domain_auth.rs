//! Per-app access control at the proxy edge (#vibe security settings).
//!
//! Caddy routes with `access != "public"` are wrapped in a `forward_auth`
//! handler pointing at `GET /api/vibe/domain-auth?domain=<d>`. This endpoint
//! enforces the binding's access policy with the cloud JWT:
//!
//! 1. **No credentials** → 302 to the login page with the app URL as
//!    `redirect` (the login page already resolves absolute redirects).
//! 2. **`?token=` callback** (login redirect landed back) → validate the JWT
//!    against the binding policy, set a domain-scoped `gb_domain_auth` cookie
//!    and 302 to `https://<domain>/`.
//! 3. **Valid cookie** → 200 (Caddy passes the request through to the app).
//!
//! `public` bindings are never routed here; the endpoint still returns 200
//! defensively so a stale route can't lock out a public app.

use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Extension, Query};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use botsecurity_auth::auth_provider::AuthProvider;
use botsecurity_auth::saas_jwt_auth::SaasJwtAuthProvider;

use crate::domains::ProjectDomains;
use crate::types::DbPool;

pub type ProjectDomainsRef = Arc<ProjectDomains>;

const COOKIE_NAME: &str = "gb_domain_auth";
const COOKIE_MAX_AGE: &str = "86400";

/// Login base URL for unauthenticated visitors (env `LOGIN_URL`, default the
/// canonical login domain documented in AGENTS.md — port 5000).
fn login_url() -> String {
    std::env::var("LOGIN_URL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "https://login.pragmatismo.com.br".to_string())
}

fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn extract_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    let all = headers.get_all(header::COOKIE);
    for value in all.iter().filter_map(|v| v.to_str().ok()) {
        for pair in value.split(';') {
            let mut parts = pair.trim().splitn(2, '=');
            if parts.next().map(str::trim) == Some(name) {
                return parts.next().map(|v| v.trim().to_string());
            }
        }
    }
    None
}

/// Email allowed by the `rbac` policy? The allowlist is stored as a
/// comma-separated list; entries are compared case-insensitively after trim.
fn email_allowed(allowed: &str, email: &str) -> bool {
    allowed.split(',').any(|e| e.trim().eq_ignore_ascii_case(email))
}

async fn authorize(
    bind: &crate::domains::DomainBind,
    token: &str,
) -> Result<Option<String>, String> {
    let secret = ProjectDomains::saas_jwt_secret();
    let provider = SaasJwtAuthProvider::new(secret);
    let user = provider
        .authenticate(token)
        .await
        .map_err(|e| format!("invalid token: {e:?}"))?;
    let email = user.email.unwrap_or_default();
    if bind.access == "rbac" {
        let allowed = bind.allowed_emails.as_deref().unwrap_or_default();
        if allowed.is_empty() || !email_allowed(allowed, &email) {
            return Err(format!(
                "rbac denied: '{email}' is not in the allowlist for {}",
                bind.domain
            ));
        }
    }
    Ok(if email.is_empty() { None } else { Some(email) })
}

pub async fn domain_auth(
    Extension(domains): Extension<ProjectDomainsRef>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
) -> Response {
    let domain = match params.get("domain").and_then(|d| ProjectDomains::validate_domain(d).ok()) {
        Some(d) => d,
        None => {
            return (StatusCode::BAD_REQUEST, "missing or invalid 'domain'").into_response();
        }
    };
    // #922 — resolve the binding for the explicit environment (default
    // production) so multi-environment deployments cannot authorize/proxy the
    // wrong environment for a domain.
    let env = params
        .get("env")
        .cloned()
        .unwrap_or_else(|| "production".to_string());
    let bind = match domains.get_by_domain_env(&domain, &env) {
        Ok(b) => b,
        Err(e) => {
            log::warn!("domain-auth: no binding for {domain}/{env}: {e}");
            return (StatusCode::NOT_FOUND, "no binding for domain").into_response();
        }
    };

    if bind.access == "public" {
        // Stale security wrapper on a public app — never lock it out.
        return StatusCode::OK.into_response();
    }

    let token = params
        .get("token")
        .cloned()
        .or_else(|| extract_bearer(&headers))
        .or_else(|| extract_cookie(&headers, COOKIE_NAME));

    let Some(tok) = token else {
        let dest = format!("https://{domain}/");
        let redirect = format!(
            "{}/?redirect={}",
            login_url(),
            urlencode(&dest)
        );
        log::info!("domain-auth: {domain} unauthenticated → login");
        return Redirect::to(&redirect).into_response();
    };

    match authorize(&bind, &tok).await {
        Ok(_email) => {
            if params.contains_key("token") {
                // Login callback: mint the domain cookie, then land on the app.
                let cookie = format!(
                    "{COOKIE_NAME}={tok}; Path=/; Domain={domain}; HttpOnly; SameSite=Lax; Max-Age={COOKIE_MAX_AGE}"
                );
                let mut resp = Redirect::to(&format!("https://{domain}/")).into_response();
                if let Ok(cv) = header::HeaderValue::from_str(&cookie) {
                    resp.headers_mut().insert(header::SET_COOKIE, cv);
                }
                log::info!("domain-auth: {domain} authenticated, cookie set");
                resp
            } else {
                StatusCode::OK.into_response()
            }
        }
        Err(e) => {
            log::warn!("domain-auth: {domain} rejected: {e}");
            if params.contains_key("token") {
                (StatusCode::FORBIDDEN, "access denied for this account").into_response()
            } else {
                // Expired/stale cookie — bounce through login once.
                let dest = format!("https://{domain}/");
                let redirect = format!("{}/?redirect={}", login_url(), urlencode(&dest));
                Redirect::to(&redirect).into_response()
            }
        }
    }
}

fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

pub fn domain_auth_router(pool: DbPool) -> axum::Router {
    axum::Router::new()
        .route("/api/vibe/domain-auth", axum::routing::get(domain_auth))
        .layer(Extension(Arc::new(ProjectDomains::new(pool))))
}
