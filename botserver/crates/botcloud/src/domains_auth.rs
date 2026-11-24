//! Write-authorization guard for `bot_domains` (#2677).
//!
//! The old `domains.rs` UPDATE/DELETE statements admitted every row whose
//! `branch_id` was NULL — and auto-provisioned domains always have a NULL
//! branch — so any authenticated tenant user could repoint or delete another
//! tenant's domain. This module makes the decision explicit and testable and
//! fails closed: platform admins may manage every row, tenant users only rows
//! in their own branch.

use axum::http::StatusCode;
use diesel::prelude::*;
use uuid::Uuid;

/// Pure decision for a write to a `bot_domains` row.
///
/// A platform admin may manage any row, including legacy auto-provisioned rows
/// whose branch is NULL. A tenant user may only touch a row belonging to their
/// own branch; a NULL row is global (NULL-scope) and never writable by them.
/// A branch belongs to exactly one organization, so equality on `branch_id`
/// is the org+branch scope check. No caller branch means no identity — deny.
pub(crate) fn may_write_domain(
    admin: bool,
    caller_branch: Option<Uuid>,
    row_branch: Option<Uuid>,
) -> bool {
    if admin {
        return true;
    }
    match (caller_branch, row_branch) {
        (Some(caller), Some(row)) => caller == row,
        _ => false,
    }
}

/// Load the target row's scope and enforce [`may_write_domain`] before any
/// write. Missing row → 404, out-of-scope row → 403, DB failure → 500 (all
/// fail closed). Called by both `update_domain` and `delete_domain`.
pub(crate) fn ensure_write_allowed(
    conn: &mut diesel::PgConnection,
    domain_id: Uuid,
    admin: bool,
    caller_branch: Option<Uuid>,
) -> Result<(), (StatusCode, String)> {
    use crate::schema_ext::bot_domains::dsl::{bot_domains, branch_id, id as domain_id_col};

    let row_branch = bot_domains
        .filter(domain_id_col.eq(domain_id))
        .select(branch_id)
        .first::<Option<Uuid>>(conn)
        .optional()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query domain scope: {e}")))?
        .ok_or((StatusCode::NOT_FOUND, "Domain mapping not found".to_string()))?;

    if !may_write_domain(admin, caller_branch, row_branch) {
        return Err((StatusCode::FORBIDDEN, "Not authorized".to_string()));
    }
    Ok(())
}

#[cfg(test)]
#[path = "domains_auth_tests.rs"]
mod tests;
