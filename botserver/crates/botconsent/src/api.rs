//! Consent HTTP surface (routes per the shared API contract):
//! - `GET    /api/consent/permissions`        — list the caller's grants
//! - `DELETE /api/consent/permissions/{id}`   — revoke one owned grant
//! - `POST   /api/consent/resolve`            — resolve one pending request
//! - `GET    /api/consent/pending`            — list pending prompts for caller
//! - `GET    /api/ui/consent/table`           — server-rendered settings fragment

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::cards;
use crate::enforce::{self, PendingRequest};
use crate::models::{Decision, ResolveBody};
use crate::store;
use crate::ConsentService;

const B64_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn b64_decode_flexible(input: &str) -> Option<Vec<u8>> {
    let mut normalized = String::with_capacity(input.len());
    for c in input.trim().chars() {
        match c {
            '-' => normalized.push('+'),
            '_' => normalized.push('/'),
            c if !c.is_whitespace() => normalized.push(c),
            _ => {}
        }
    }
    if normalized.len() % 4 == 1 {
        return None;
    }
    let mut out = Vec::with_capacity(normalized.len() / 4 * 3);
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    for b in normalized.bytes() {
        if b == b'=' {
            break;
        }
        let v = match b {
            b'A'..=b'Z' => (b - b'A') as u32,
            b'a'..=b'z' => (b - b'a' + 26) as u32,
            b'0'..=b'9' => (b - b'0' + 52) as u32,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        };
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((acc >> bits) & 0xFF) as u8);
        }
    }
    Some(out)
}

fn jwt_claims(headers: &HeaderMap) -> Option<serde_json::Value> {
    let token = headers
        .get("authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")?
        .trim();
    let payload = token.split('.').nth(1)?;
    let decoded = b64_decode_flexible(payload)?;
    serde_json::from_slice(&decoded).ok()
}

fn jwt_user_id(headers: &HeaderMap) -> Option<Uuid> {
    let claims = jwt_claims(headers)?;
    for key in ["user_id", "sub", "uid"] {
        if let Some(raw) = claims.get(key).and_then(|v| v.as_str()) {
            if let Ok(id) = Uuid::parse_str(raw) {
                return Some(id);
            }
        }
    }
    None
}

fn require_user(headers: &HeaderMap) -> Result<Uuid, (StatusCode, String)> {
    jwt_user_id(headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Authentication required".to_string()))
}

pub fn configure_routes() -> Router<Arc<ConsentService>> {
    Router::new()
        .route("/api/consent/permissions", get(list_permissions))
        .route(
            "/api/consent/permissions/:id",
            axum::routing::delete(revoke_permission),
        )
        .route("/api/consent/resolve", post(resolve_consent))
        .route("/api/consent/pending", get(pending_requests))
        .route("/api/ui/consent/table", get(permissions_table))
}

/// `GET /api/consent/permissions`
pub async fn list_permissions(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = require_user(&headers)?;
    let rows = {
        let mut conn = service
            .pool
            .get()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {e}")))?;
        store::list_for_user(&mut conn, user_id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query failed: {e}")))?
    };
    Ok(Json(serde_json::json!({ "permissions": rows })))
}

/// `DELETE /api/consent/permissions/{id}`
pub async fn revoke_permission(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    Path(permission_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = require_user(&headers)?;
    let deleted = {
        let mut conn = service
            .pool
            .get()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {e}")))?;
        store::revoke(&mut conn, permission_id, user_id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Revoke failed: {e}")))?
    };
    if !deleted {
        return Err((StatusCode::NOT_FOUND, "Permission not found".to_string()));
    }
    Ok(Json(serde_json::json!({ "status": "revoked", "id": permission_id })))
}

/// `GET /api/consent/pending`
pub async fn pending_requests(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let user_id = require_user(&headers)?;
    let now = std::time::Instant::now();
    let pend = service.pending.lock().await;
    let items: Vec<&PendingRequest> = pend
        .values()
        .filter(|(req, born)| req.user_id == user_id && now.duration_since(*born) < enforce_ttl())
        .map(|(req, _)| req)
        .collect();
    Ok(Json(serde_json::json!({ "pending": items })))
}

fn enforce_ttl() -> std::time::Duration {
    std::time::Duration::from_secs(enforce::PENDING_TTL_SECS)
}

/// Accepts both JSON bodies and HTMX form-encoded `hx-vals` payloads.
fn parse_resolve_body(headers: &HeaderMap, body: &[u8]) -> Result<ResolveBody, String> {
    let is_form = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.contains("application/x-www-form-urlencoded"))
        .unwrap_or(false);
    if is_form {
        parse_urlencoded(std::str::from_utf8(body).map_err(|e| format!("encoding: {e}"))?)
    } else {
        serde_json::from_slice(body).map_err(|e| format!("invalid body: {e}"))
    }
}

fn decode_percent(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(b) => {
                        out.push(b);
                        i += 3;
                    }
                    Err(_) => {
                        out.push(bytes[i]);
                        i += 1;
                    }
                }
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

fn parse_urlencoded(body: &str) -> Result<ResolveBody, String> {
    let mut request_id: Option<String> = None;
    let mut decision_raw: Option<String> = None;
    for pair in body.split('&') {
        let (k, v) = match pair.split_once('=') {
            Some(kv) => kv,
            None => continue,
        };
        match k {
            "request_id" => request_id = Some(decode_percent(v)),
            "decision" => decision_raw = Some(decode_percent(v)),
            _ => {}
        }
    }
    let request_id = request_id.filter(|s| !s.is_empty()).ok_or("request_id required")?;
    let decision_raw = decision_raw.ok_or("decision required")?;
    let decision: Decision =
        serde_json::from_value(serde_json::Value::String(decision_raw)).map_err(|e| format!("{e}"))?;
    Ok(ResolveBody { request_id, decision })
}

/// `POST /api/consent/resolve` — HTMX callers receive the summary card; plain
/// API callers receive JSON.
pub async fn resolve_consent(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<Response, (StatusCode, String)> {
    let user_id = require_user(&headers)?;
    let parsed = parse_resolve_body(&headers, &body)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid body: {e}")))?;

    let outcome = match enforce::resolve(&service, &parsed.request_id, parsed.decision, user_id).await {
        Ok(o) => o,
        Err(e) => {
            tracing::error!("consent resolve failed: {e}");
            if is_hx_request(&headers) {
                return Ok(Html(cards::summary_card_html("error")).into_response());
            }
            return Err((StatusCode::NOT_FOUND, "Consent request not found".to_string()));
        }
    };

    let label = match outcome {
        enforce::ResolvedOutcome::ConsumeOnce(_) => "allow_once",
        enforce::ResolvedOutcome::PersistGrant(_) => "always",
        enforce::ResolvedOutcome::RecordDenial(_) => "deny",
    };
    if is_hx_request(&headers) {
        return Ok(Html(cards::summary_card_html(label)).into_response());
    }
    Ok(Json(serde_json::json!({ "status": "resolved", "outcome": label })).into_response())
}

fn is_hx_request(headers: &HeaderMap) -> bool {
    headers
        .get("hx-request")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// `GET /api/ui/consent/table` — server-rendered settings fragment.
pub async fn permissions_table(
    State(service): State<Arc<ConsentService>>,
    headers: HeaderMap,
) -> Result<Html<String>, (StatusCode, String)> {
    let user_id = require_user(&headers)?;
    let rows = {
        let mut conn = service
            .pool
            .get()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {e}")))?;
        store::list_for_user(&mut conn, user_id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query failed: {e}")))?
    };
    Ok(Html(cards::permissions_table_html(&rows)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_htmx_form_payload() {
        let body = "request_id=abc%20123&decision=allow_once";
        let parsed = parse_urlencoded(body).expect("form parse ok");
        assert_eq!(parsed.request_id, "abc 123");
        assert_eq!(parsed.decision, Decision::AllowOnce);
    }

    #[test]
    fn rejects_missing_fields() {
        assert!(parse_urlencoded("decision=always").is_err());
        assert!(parse_urlencoded("request_id=x").is_err());
        assert!(parse_urlencoded("request_id=x&decision=nope").is_err());
    }

    #[test]
    fn decodes_percent_and_plus() {
        assert_eq!(decode_percent("a%20b+c"), "a b c");
    }

    #[test]
    fn jwt_user_from_sub_claim() {
        let header = "Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIwMDAwMDAwMC0wMDAwLTAwMDAtMDAwMC0wMDAwMDAwMDAwMDEifQ.sig";
        let mut map = HeaderMap::new();
        map.insert(
            "authorization",
            axum::http::HeaderValue::from_static(header),
        );
        assert_eq!(
            jwt_user_id(&map),
            Some(Uuid::parse_str("00000000-0000-0000-0000-000000000001").expect("valid uuid"))
        );
    }

    #[test]
    fn missing_token_is_unauthorized() {
        let map = HeaderMap::new();
        assert!(require_user(&map).is_err());
    }
}
