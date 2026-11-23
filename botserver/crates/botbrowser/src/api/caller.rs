//! Verified caller identity for browser sessions (#2879).
//!
//! Every browser session is bound to the authenticated owner and to an
//! unguessable capability minted at creation. This module is the only place
//! that turns request headers (or, for WebSockets, query parameters) into a
//! [`SessionCaller`]. A request without a verifiable Bearer token gets no
//! identity and is refused, so a leaked session id alone is never enough to
//! reach another tenant's page.

use super::*;
use axum::http::{request, StatusCode};
use botsecurity_core::verified_jwt::{bearer_token, verify_saas_jwt};
use uuid::Uuid;

/// Verified identity and capability presented by the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SessionCaller {
    pub owner: SessionOwner,
    pub capability: Option<String>,
}

impl SessionCaller {
    fn query_value(query: Option<&str>, key: &str) -> Option<String> {
        let query = query?;
        query.split('&').find_map(|pair| {
            let (name, value) = pair.split_once('=')?;
            if name == key && !value.is_empty() {
                Some(value.to_string())
            } else {
                None
            }
        })
    }

    /// Resolve the caller from a request's parts. Kept separate from the axum
    /// trait so the pure logic is easy to reason about and reuse.
    pub(crate) fn from_parts(parts: &request::Parts) -> Result<Self, (StatusCode, String)> {
        let token = bearer_token(&parts.headers)
            .map(str::to_string)
            .or_else(|| Self::query_value(parts.uri.query(), "token"))
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Authentication required".to_string(),
                )
            })?;
        let claims = verify_saas_jwt(&token).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "Invalid or expired token".to_string(),
            )
        })?;
        let owner_user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                "Invalid token subject".to_string(),
            )
        })?;
        let capability = parts
            .headers
            .get("x-browser-capability")
            .and_then(|value| value.to_str().ok())
            .map(str::to_string)
            .or_else(|| Self::query_value(parts.uri.query(), "capability"));
        Ok(Self {
            owner: SessionOwner {
                owner_user_id,
                org_id: claims.org_id,
                branch_id: claims.branch_id,
            },
            capability,
        })
    }
}

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for SessionCaller
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Self::from_parts(parts)
    }
}

#[cfg(test)]
mod tests {
    use super::SessionCaller;

    #[test]
    fn query_value_parses_a_key() {
        let value = SessionCaller::query_value(Some("a=1&capability=xyz&b=2"), "capability");
        assert_eq!(value.as_deref(), Some("xyz"));
    }

    #[test]
    fn query_value_ignores_missing_and_empty_values() {
        assert_eq!(SessionCaller::query_value(None, "capability"), None);
        assert_eq!(
            SessionCaller::query_value(Some("other=1&capability="), "capability"),
            None
        );
    }
}
