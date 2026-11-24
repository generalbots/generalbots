//! Shared HTTP state and tenant resolution for the fraud endpoints.

use crate::engine::FraudEngine;
use axum::http::HeaderMap;
use botsecurity_core::verified_jwt::{bearer_token, verify_saas_jwt, VerifiedJwt};
use uuid::Uuid;

pub use botlib::db_pool::DbPool;

#[derive(Clone)]
pub struct FraudState {
    pub pool: DbPool,
    pub engine: FraudEngine,
}

impl FraudState {
    pub fn new(pool: DbPool) -> Self {
        let engine = FraudEngine::new(pool.clone());
        Self { pool, engine }
    }
}

/// Tenant of a **verified** token: the workspace branch first, then the owning
/// organization. No payload byte is read until `verify_saas_jwt` has pinned
/// HS256 and checked the signature, expiry and subject, so a forged
/// `branch_id`/`org_id`/`role` claim cannot select another tenant (#2773).
fn branch_from_verified(claims: &VerifiedJwt) -> Option<Uuid> {
    claims.branch_id.or(claims.org_id)
}

/// Verified tenant branch of the request, or `None` for an absent, unsigned,
/// tampered, foreign-key or expired token.
fn verified_branch(headers: &HeaderMap) -> Option<Uuid> {
    let claims = verify_saas_jwt(bearer_token(headers)?).ok()?;
    branch_from_verified(&claims)
}

/// Resolves the caller's tenant branch from the verified JWT claims, never from
/// client-supplied query params (issue #734). Falls back to the global nil
/// branch for anonymous/internal callers so system-wide operations still work.
pub fn resolve_branch(headers: &HeaderMap) -> Uuid {
    verified_branch(headers).unwrap_or(Uuid::nil())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    const BRANCH: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
    const ORG: &str = "cccccccc-cccc-cccc-cccc-cccccccccccc";

    fn verified(branch: Option<&str>, org: Option<&str>) -> VerifiedJwt {
        VerifiedJwt {
            sub: "user-1".to_string(),
            email: None,
            org_id: org.and_then(|value| Uuid::parse_str(value).ok()),
            branch_id: branch.and_then(|value| Uuid::parse_str(value).ok()),
            roles: Vec::new(),
            exp: chrono::Utc::now().timestamp() + 3600,
            jti: None,
        }
    }

    #[test]
    fn verified_branch_wins_over_org() {
        assert_eq!(
            branch_from_verified(&verified(Some(BRANCH), Some(ORG))),
            Some(Uuid::parse_str(BRANCH).expect("uuid"))
        );
    }

    #[test]
    fn org_is_the_fallback_when_no_branch_is_signed() {
        assert_eq!(
            branch_from_verified(&verified(None, Some(ORG))),
            Some(Uuid::parse_str(ORG).expect("uuid"))
        );
        assert_eq!(branch_from_verified(&verified(None, None)), None);
    }

    #[test]
    fn missing_token_uses_the_global_branch() {
        assert_eq!(resolve_branch(&HeaderMap::new()), Uuid::nil());
    }

    #[test]
    fn forged_branch_claim_is_ignored() {
        // `alg:none` payload claiming another tenant's branch/org and the
        // `admin` role. The old base64 decoder trusted it outright (#2773).
        let forged = "eyJhbGciOiJub25lIn0.eyJicmFuY2hfaWQiOiJiYmJiYmJiYi1iYmJiLWJiYmItYmJiYi1iYmJiYmJiYmJiYmIiLCJvcmdfaWQiOiJjY2NjY2NjYy1jY2NjLWNjY2MtY2NjYy1jY2NjY2NjY2NjY2MiLCJyb2xlIjoiYWRtaW4iLCJleHAiOjQxMDI0NDQ4MDAsInN1YiI6IjAwMDAwMDAwLTAwMDAtMDAwMC0wMDAwLTAwMDAwMDAwMDAwMSJ9.";
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_str(&format!("Bearer {forged}")).expect("header"),
        );
        assert!(verified_branch(&headers).is_none());
        assert_eq!(resolve_branch(&headers), Uuid::nil());
    }
}
