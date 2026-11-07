//! Bearer token parsing and caller authorization.

use axum::http::{HeaderMap, StatusCode};
use uuid::Uuid;

use super::{external_token_user, unauthorized};

pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()
        .map(|v| v.to_string())
        .filter(|v| v.starts_with("Bearer ") || v.starts_with("bearer "))
}

fn b64_decode(input: &str) -> Option<Vec<u8>> {
    fn value(c: u8) -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'a'..=b'z' => Some((c - b'a') as u32 + 26),
            b'0'..=b'9' => Some((c - b'0') as u32 + 52),
            b'+' | b'-' => Some(62),
            b'/' | b'_' => Some(63),
            _ => None,
        }
    }
    let mut out = Vec::with_capacity(input.len() / 4 * 3);
    let mut buffer = 0u32;
    let mut bits = 0u32;
    for &byte in input.as_bytes() {
        if byte == b'=' || byte == b'\r' || byte == b'\n' {
            continue;
        }
        buffer = (buffer << 6) | value(byte)?;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xFF) as u8);
        }
    }
    Some(out)
}

pub fn jwt_payload(token: &str) -> Option<serde_json::Value> {
    let payload_segment = token.split('.').nth(1)?;
    let decoded = b64_decode(payload_segment)?;
    serde_json::from_slice(&decoded).ok()
}

pub fn b64_json_sub(token: &str) -> Option<String> {
    jwt_payload(token)?
        .get("sub")
        .and_then(|s| s.as_str())
        .map(str::to_string)
}

pub fn is_admin_claims(claims: &serde_json::Value) -> bool {
    claims.get("role").and_then(|r| r.as_str()) == Some("admin")
}

/// Verifies the token signature with the SaaS JWT secret handed over by the
/// server at boot (`GB_AGENT_JWT_SECRET`).
fn verified_sub(raw: &str) -> Option<Uuid> {
    // `bearer_token` returns the whole header value; the signature covers the
    // JWT only, so drop the scheme prefix here.
    let token = raw
        .strip_prefix("Bearer ")
        .or_else(|| raw.strip_prefix("bearer "))
        .unwrap_or(raw)
        .trim();
    let secret = std::env::var("GB_AGENT_JWT_SECRET").ok()?;
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;
    // Suite tokens are claim-light (no `aud`).
    validation.validate_aud = false;
    let data = jsonwebtoken::decode::<serde_json::Value>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| tracing::warn!("agent bearer rejected: {e}"))
    .ok()?;
    let sub = data.claims.get("sub")?.as_str()?;
    Uuid::parse_str(sub).ok()
}

pub(super) fn caller_uuid(
    headers: &HeaderMap,
) -> Result<(String, Uuid, bool), (StatusCode, String)> {
    let token = bearer_token(headers).ok_or_else(|| unauthorized("Missing bearer token"))?;
    // Suite/session tokens (`gb_<uuid>_<ts>`) are not JWTs: the server wires a
    // verifier at boot so file links and VM endpoints accept the same token the
    // chat page holds. Without it every generated-file URL answered 401 and the
    // attachment cards were dead links.
    if let (Some(payload), Some(sub)) = (jwt_payload(&token), b64_json_sub(&token)) {
        // SECURITY: the payload segment is attacker-controlled. Every caller must
        // pass signature verification before acting on another user's VM — a
        // handcrafted `sub` used to reach any workspace.
        let user_id = match verified_sub(&token) {
            Some(verified) => verified,
            None if std::env::var("GB_AGENT_JWT_SECRET").is_err() => {
                tracing::warn!(
                    "GB_AGENT_JWT_SECRET not set; falling back to unverified claims (tests only)"
                );
                Uuid::parse_str(&sub).unwrap_or_else(|_| {
                    Uuid::new_v5(&Uuid::NAMESPACE_URL, format!("zitadel:{sub}").as_bytes())
                })
            }
            None => match external_token_user(&token) {
                Some(id) => id,
                None => return Err(unauthorized("Invalid or unverified token")),
            },
        };
        let admin = is_admin_claims(&payload);
        return Ok((sub, user_id, admin));
    }

    if let Some(user_id) = external_token_user(&token) {
        return Ok((user_id.to_string(), user_id, false));
    }
    Err(unauthorized("Invalid or unverified token"))
}

pub fn authorize_session_access(
    headers: &HeaderMap,
    owner_user_id: &Uuid,
) -> Result<(), (StatusCode, String)> {
    let (_, user_id, admin) = caller_uuid(headers)?;
    if admin || user_id == *owner_user_id {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, "Not authorized".to_string()))
    }
}
