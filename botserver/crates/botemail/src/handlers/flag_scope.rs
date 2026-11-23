//! Verified tenant scope for the feature-flags endpoint (issue #2856).
//!
//! The org id in the URL used to be trusted verbatim, so any authenticated
//! user could read another organization's flags by editing the path. The org
//! id is now resolved from verified caller claims; a client-supplied value is
//! accepted only when it matches. An administrator targeting another tenant
//! uses the separate explicit admin path, never this read endpoint.

use axum::http::HeaderMap;
use uuid::Uuid;

/// Why a feature-flags request was refused before touching the database.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FlagScopeError {
    /// No verified org claim: the caller's identity cannot authorize a tenant.
    MissingClaim,
    /// The path param names an org other than the verified caller's org.
    ForeignOrg,
}

/// #2856 — pure resolution predicate for the tenant whose flags may be read.
///
/// The verified claim is authoritative. A client-supplied path param is
/// accepted only when it matches; it can never select another tenant, even
/// for an administrator (cross-org reads belong on the explicit admin path).
pub(crate) fn resolve_flag_org(
    claims_org: Option<Uuid>,
    requested_org: Uuid,
) -> Result<Uuid, FlagScopeError> {
    match claims_org {
        Some(org) if org == requested_org => Ok(org),
        Some(_) => Err(FlagScopeError::ForeignOrg),
        None => Err(FlagScopeError::MissingClaim),
    }
}

/// Verified org id of the caller (issue #2856).
///
/// JWT claims come only from the signature-verifying resolver in
/// `botsecurity_core`; opaque `gb_*` session tokens resolve through the cache
/// populated from the login JWT. A JWT that fails verification is never
/// downgraded into a session lookup — that would reopen the forged-token path.
pub(crate) fn caller_org(headers: &HeaderMap) -> Option<Uuid> {
    if let Some(org) = botsecurity_core::tenant::org_from_claims(headers) {
        return Some(org);
    }
    let auth = headers.get("authorization")?.to_str().ok()?;
    let token = botsecurity_core::tenant::token_from_authorization(auth)?;
    if token.contains('.') {
        return None;
    }
    botsecurity_core::lookup_session_cache(&token).and_then(|entry| entry.organization_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_claim_resolves_to_the_caller_org() {
        let org = Uuid::new_v4();
        assert_eq!(resolve_flag_org(Some(org), org), Ok(org));
    }

    #[test]
    fn a_foreign_path_org_is_refused() {
        let claimed = Uuid::new_v4();
        let requested = Uuid::new_v4();
        assert_eq!(
            resolve_flag_org(Some(claimed), requested),
            Err(FlagScopeError::ForeignOrg)
        );
    }

    #[test]
    fn a_missing_verified_claim_is_refused() {
        let requested = Uuid::new_v4();
        assert_eq!(
            resolve_flag_org(None, requested),
            Err(FlagScopeError::MissingClaim)
        );
    }

    #[test]
    fn nil_org_scope_resolves_only_for_a_nil_path_param() {
        assert_eq!(
            resolve_flag_org(Some(Uuid::nil()), Uuid::nil()),
            Ok(Uuid::nil())
        );
        assert_eq!(
            resolve_flag_org(Some(Uuid::nil()), Uuid::new_v4()),
            Err(FlagScopeError::ForeignOrg)
        );
    }
}
