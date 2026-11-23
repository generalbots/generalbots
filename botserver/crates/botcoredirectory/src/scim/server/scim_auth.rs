//! Per-route SCIM authentication and path-id validation (#2848).
//!
//! SCIM routes drive the Zitadel service account, so they require a dedicated
//! SCIM bearer token (never a normal user JWT) and must never format raw path
//! ids into privileged management URLs.

use axum::{
    body::Body,
    extract::{FromRequestParts, State},
    http::{header::AUTHORIZATION, request::Parts, HeaderValue, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::scim::types::ScimError;

/// Identity encoded in the SCIM token. Never taken from the path or the body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScimPrincipal {
    pub org_id: Uuid,
    pub branch_id: Uuid,
}

#[derive(Deserialize)]
struct ScimTokenEntry {
    org_id: Uuid,
    branch_id: Uuid,
}

/// Maps dedicated SCIM tokens to the org/branch each token may provision.
pub struct ScimTokenVerifier {
    tokens: HashMap<String, ScimPrincipal>,
}

impl ScimTokenVerifier {
    /// `SCIM_TOKENS` is a JSON object:
    /// `{"<token>": {"org_id": "<uuid>", "branch_id": "<uuid>"}}`.
    ///
    /// Fails closed: an unset or malformed value yields an empty map, so every
    /// SCIM route rejects requests until an operator provisions a token.
    pub fn from_env() -> Self {
        let raw = match std::env::var("SCIM_TOKENS") {
            Ok(value) => value,
            Err(_) => return Self { tokens: HashMap::new() },
        };
        let entries: HashMap<String, ScimTokenEntry> = match serde_json::from_str(&raw) {
            Ok(entries) => entries,
            Err(error) => {
                log::error!("SCIM_TOKENS is not valid JSON: {error}");
                return Self { tokens: HashMap::new() };
            }
        };
        let tokens = entries
            .into_iter()
            .map(|(token, entry)| {
                (
                    token,
                    ScimPrincipal { org_id: entry.org_id, branch_id: entry.branch_id },
                )
            })
            .collect();
        Self { tokens }
    }

    /// Verify an `Authorization: Bearer <scim-token>` header.
    /// A normal user JWT is never in this map and is therefore rejected.
    pub fn verify_header(&self, header: Option<&HeaderValue>) -> Result<ScimPrincipal, &'static str> {
        let header = header.ok_or("missing Authorization header")?;
        let raw = header.to_str().map_err(|_| "malformed Authorization header")?;
        let token = raw
            .get(..7)
            .filter(|prefix| prefix.eq_ignore_ascii_case("Bearer "))
            .and_then(|_| raw.get(7..))
            .filter(|token| !token.is_empty())
            .ok_or("malformed Authorization header")?;
        self.tokens
            .iter()
            .find(|(candidate, _)| constant_time_eq(candidate.as_bytes(), token.as_bytes()))
            .map(|(_, principal)| principal.clone())
            .ok_or("invalid SCIM token")
    }

    #[cfg(test)]
    fn from_entries(entries: &[(&str, Uuid, Uuid)]) -> Self {
        Self {
            tokens: entries
                .iter()
                .map(|(token, org_id, branch_id)| {
                    (
                        (*token).to_string(),
                        ScimPrincipal { org_id: *org_id, branch_id: *branch_id },
                    )
                })
                .collect(),
        }
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in left.iter().zip(right.iter()) {
        diff |= *a ^ *b;
    }
    diff == 0
}

pub(crate) fn scim_error(
    status: StatusCode,
    detail: &str,
    scim_status: u16,
) -> (StatusCode, Json<ScimError>) {
    (
        status,
        Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: Some("invalidValue".to_string()),
            detail: detail.to_string(),
            status: scim_status,
        }),
    )
}

fn scim_unauthorized() -> (StatusCode, Json<ScimError>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(ScimError {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:Error".to_string()],
            scim_type: None,
            detail: "A valid SCIM bearer token is required".to_string(),
            status: 401,
        }),
    )
}

/// Router layer attached to every SCIM route (#2848).
pub async fn scim_auth_middleware(
    State(verifier): State<Arc<ScimTokenVerifier>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let header = request.headers().get(AUTHORIZATION).cloned();
    match verifier.verify_header(header.as_ref()) {
        Ok(principal) => {
            request.extensions_mut().insert(principal);
            next.run(request).await
        }
        Err(reason) => {
            log::warn!("SCIM request rejected: {reason}");
            scim_unauthorized().into_response()
        }
    }
}

/// The verified principal, extracted by handlers from the request extensions.
#[derive(Clone, Debug)]
pub struct ScimCaller(pub ScimPrincipal);

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for ScimCaller
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ScimError>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<ScimPrincipal>()
            .cloned()
            .map(ScimCaller)
            .ok_or_else(|| {
                scim_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "SCIM principal missing from request",
                    500,
                )
            })
    }
}

/// Zitadel user ids are UUIDs. Reject anything else before it reaches a URL.
pub fn parse_scim_user_id(raw: &str) -> Result<Uuid, String> {
    Uuid::parse_str(raw).map_err(|_| "Invalid SCIM user id: expected a UUID".to_string())
}

/// SCIM group ids are metadata keys: alphanumeric, `_` or `-` only.
pub fn validate_scim_group_key(raw: &str) -> Result<String, String> {
    if raw.is_empty()
        || raw.len() > 128
        || !raw.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err("Invalid SCIM group id: expected an alphanumeric key".to_string());
    }
    Ok(raw.to_string())
}

/// Percent-encode a single path segment as a second layer of traversal defence.
pub fn percent_encode_path_segment(raw: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(raw.len());
    for byte in raw.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            other => {
                encoded.push('%');
                encoded.push(HEX[(other >> 4) as usize] as char);
                encoded.push(HEX[(other & 0x0F) as usize] as char);
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    const ORG: &str = "11111111-1111-4111-8111-111111111111";
    const BRANCH: &str = "22222222-2222-4222-8222-222222222222";

    #[test]
    fn hostile_user_ids_are_rejected() {
        let hostile = [
            "",
            "group_admins",
            "..%2F..%2Fmetadata%2Forganization%2Fgroup_admins",
            "../../metadata/organization/group_admins",
            "%2e%2e%2f%2e%2e%2fmetadata",
            "11111111-1111-4111-8111-111111111111/../../x",
            "11111111-1111-4111-8111-111111111111?admin=true",
        ];
        for id in hostile {
            assert!(
                parse_scim_user_id(id).is_err(),
                "hostile user id must be rejected: {id}"
            );
        }
    }

    #[test]
    fn valid_uuid_user_id_is_accepted() {
        assert_eq!(
            parse_scim_user_id(ORG).unwrap(),
            Uuid::parse_str(ORG).unwrap()
        );
    }

    #[test]
    fn hostile_group_keys_are_rejected() {
        let hostile = [
            "",
            "../../metadata/organization/group_admins",
            "..%2F..%2Fmetadata%2Forganization%2Fgroup_admins",
            "group_admins/extra",
            "group_admins%2f..",
            "group_admins\nX-Evil: 1",
        ];
        for key in hostile {
            assert!(
                validate_scim_group_key(key).is_err(),
                "hostile group key must be rejected: {key}"
            );
        }
        assert_eq!(
            validate_scim_group_key("group_7f3a1c2e-9b11-4c4e-8f2a-000000000001").unwrap(),
            "group_7f3a1c2e-9b11-4c4e-8f2a-000000000001"
        );
    }

    #[test]
    fn traversal_segments_are_percent_encoded() {
        assert_eq!(
            percent_encode_path_segment("../.."),
            "..%2F.."
        );
        assert_eq!(
            percent_encode_path_segment("../metadata/organization/group_admins"),
            "..%2Fmetadata%2Forganization%2Fgroup_admins"
        );
    }

    #[test]
    fn missing_malformed_and_wrong_tokens_are_rejected() {
        let verifier = ScimTokenVerifier::from_entries(&[("scim-secret-token", Uuid::parse_str(ORG).unwrap(), Uuid::parse_str(BRANCH).unwrap())]);
        assert_eq!(
            verifier.verify_header(None).err(),
            Some("missing Authorization header")
        );
        assert_eq!(
            verifier.verify_header(Some(&HeaderValue::from_static("Bearer wrong-token"))).err(),
            Some("invalid SCIM token")
        );
        assert_eq!(
            verifier.verify_header(Some(&HeaderValue::from_static("Basic scim-secret-token"))).err(),
            Some("malformed Authorization header")
        );
        assert_eq!(
            verifier.verify_header(Some(&HeaderValue::from_static("Bearer "))).err(),
            Some("malformed Authorization header")
        );
    }

    #[test]
    fn valid_scim_token_yields_its_org_and_branch() {
        let verifier = ScimTokenVerifier::from_entries(&[("scim-secret-token", Uuid::parse_str(ORG).unwrap(), Uuid::parse_str(BRANCH).unwrap())]);
        let header = HeaderValue::from_static("bearer scim-secret-token");
        assert_eq!(
            verifier.verify_header(Some(&header)).unwrap(),
            ScimPrincipal { org_id: Uuid::parse_str(ORG).unwrap(), branch_id: Uuid::parse_str(BRANCH).unwrap() }
        );
    }

    #[test]
    fn normal_user_jwt_is_not_a_scim_token() {
        let verifier = ScimTokenVerifier::from_entries(&[("scim-secret-token", Uuid::parse_str(ORG).unwrap(), Uuid::parse_str(BRANCH).unwrap())]);
        let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJhZG1pbiJ9.signature";
        let header = HeaderValue::from_str(&format!("Bearer {jwt}")).unwrap();
        assert_eq!(verifier.verify_header(Some(&header)).err(), Some("invalid SCIM token"));
    }
}
