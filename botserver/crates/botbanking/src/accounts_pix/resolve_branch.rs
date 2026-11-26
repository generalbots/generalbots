use super::*;

// #2948 — all banking reads and writes now share the same verified-claims
// scope resolver. A signed-in admin with a nil/absent workspace claim is
// scoped to the explicit global tenant instead of being rejected with 401;
// anonymous and unverified tokens stay denied.
pub(super) use crate::branch_scope::resolve_branch;

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
    fn anonymous_callers_are_denied_without_a_verified_claim() {
        let (status, _) = resolve_branch(&HeaderMap::new())
            .expect_err("no bearer token must never resolve a tenant");
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
}
