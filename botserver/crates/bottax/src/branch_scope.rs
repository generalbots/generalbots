//! Verified tenant-scope resolution for the tax REST handlers (#2966).
//!
//! Every tax query filters on `branch_id`. That value must come from a
//! verified, server-minted JWT claim — never from a client-supplied header.
//! The cloud login only embeds a `branch_id` claim when the identity resolves
//! to a real workspace binding, so the global SaaS admin carries either a nil
//! or an absent claim. Rejecting both with 401 (the old `require_branch`
//! behaviour) made every signed-in admin call to this app fail. A verified
//! token with no workspace branch is now scoped to the explicit global tenant
//! (nil); unverified/anonymous callers are still refused.

use axum::http::{HeaderMap, StatusCode};
use botcore::shared::tenant;
use uuid::Uuid;

const DENIED: &str = "missing tenant claims: sign in again";

/// Resolves the caller's branch from verified claims only.
pub(crate) fn resolve_branch(headers: &HeaderMap) -> Result<Uuid, (StatusCode, String)> {
    // `sub` is mandatory in a verified SaaS JWT. Its presence proves the
    // token passed signature/expiry verification, so a forged or anonymous
    // caller can never reach the global-scope fallback below.
    let verified = tenant::user_id_from_claims(headers).is_some();
    branch_for_verified_claims(verified, tenant::branch_from_claims(headers))
}

/// Pure decision helper: keeps the accept/reject boundary testable offline
/// without minting a JWT (signature verification is exercised by `verified`).
fn branch_for_verified_claims(
    verified: bool,
    claim: Option<Uuid>,
) -> Result<Uuid, (StatusCode, String)> {
    if !verified {
        return Err((StatusCode::UNAUTHORIZED, DENIED.to_string()));
    }
    Ok(claim.unwrap_or_else(Uuid::nil))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_verified_workspace_claim_is_used_verbatim() {
        let branch = Uuid::new_v4();
        assert_eq!(branch_for_verified_claims(true, Some(branch)), Ok(branch));
    }

    #[test]
    fn a_verified_nil_claim_is_the_global_admin_scope_not_a_401() {
        assert_eq!(
            branch_for_verified_claims(true, Some(Uuid::nil())),
            Ok(Uuid::nil())
        );
    }

    #[test]
    fn a_verified_token_without_a_branch_claim_gets_the_global_scope() {
        assert_eq!(branch_for_verified_claims(true, None), Ok(Uuid::nil()));
    }

    #[test]
    fn an_unverified_caller_is_denied_even_with_a_forged_branch_claim() {
        let forged = Uuid::new_v4();
        let (status, message) = branch_for_verified_claims(false, Some(forged))
            .expect_err("an unverified caller must be denied");
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(message, DENIED);
    }

    #[test]
    fn a_request_without_headers_is_denied() {
        assert_eq!(
            resolve_branch(&HeaderMap::new())
                .err()
                .map(|(status, _)| status),
            Some(StatusCode::UNAUTHORIZED)
        );
    }

    #[test]
    fn a_non_jwt_header_is_denied() {
        let mut headers = HeaderMap::new();
        if let Ok(value) = "Bearer not-a-jwt".parse() {
            headers.insert("authorization", value);
        }
        assert_eq!(
            resolve_branch(&headers).err().map(|(status, _)| status),
            Some(StatusCode::UNAUTHORIZED)
        );
    }
}
