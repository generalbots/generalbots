use super::*;

/// Resolves the caller's tenant branch from the verified JWT claims (#2875).
///
/// A missing (or nil) branch claim is refused with 401 instead of silently
/// degrading to the global nil branch: every financial read and write must be
/// bound to a verified tenant. Nil is reserved for explicit desktop-local or
/// system callers — it is never inferred from an absent claim.
pub(super) fn resolve_branch(headers: &HeaderMap) -> Result<Uuid, (StatusCode, String)> {
    require_branch(branch_from_claims(headers))
}

/// Pure branch resolver (#2875): only a verified, non-nil claim is accepted.
fn require_branch(claim: Option<Uuid>) -> Result<Uuid, (StatusCode, String)> {
    match claim {
        Some(branch) if branch != Uuid::nil() => Ok(branch),
        _ => Err((
            StatusCode::UNAUTHORIZED,
            "missing tenant claims: sign in again".to_string(),
        )),
    }
}

pub(super) fn ok<T: Serialize>(value: T) -> Result<Json<T>, (StatusCode, String)> {
    Ok(Json(value))
}
pub(super) fn decimal_to_string(d: Decimal) -> String {
    d.to_string()
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_verified_non_nil_claim() {
        let branch = Uuid::new_v4();
        assert_eq!(require_branch(Some(branch)), Ok(branch));
    }

    #[test]
    fn refuses_missing_claim_with_401() {
        assert_eq!(
            require_branch(None).err().map(|(status, _)| status),
            Some(StatusCode::UNAUTHORIZED)
        );
    }

    #[test]
    fn refuses_nil_claim_with_401() {
        assert_eq!(
            require_branch(Some(Uuid::nil())).err().map(|(status, _)| status),
            Some(StatusCode::UNAUTHORIZED)
        );
    }
}
