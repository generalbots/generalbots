//! Tenant branch resolution for compliance reads and writes (#1387).
//!
//! Every compliance table is branch-scoped. Handlers used to write and read
//! with `Uuid::nil()`, so all tenants shared one dataset. The branch now comes
//! from the server-minted JWT claims; when a request carries no claims (CLI or
//! internal call) it falls back to the default bot's branch.

use axum::http::HeaderMap;
use diesel::prelude::*;
use uuid::Uuid;

use crate::DbPool;

/// Resolves the branch that owns the caller's compliance data.
pub fn resolve_branch(headers: &HeaderMap, pool: &DbPool) -> Uuid {
    if let Some(branch) = botsecurity_core::tenant::branch_from_claims(headers) {
        return branch;
    }

    default_branch(pool).unwrap_or(Uuid::nil())
}

fn default_branch(pool: &DbPool) -> Option<Uuid> {
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct BranchRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        branch_id: Uuid,
    }

    let mut conn = pool.get().ok()?;

    diesel::sql_query(
        "SELECT branch_id FROM bots WHERE is_default_for_branch = TRUE ORDER BY created_at ASC LIMIT 1",
    )
    .get_result::<BranchRow>(&mut conn)
    .map(|row| row.branch_id)
    .ok()
}
