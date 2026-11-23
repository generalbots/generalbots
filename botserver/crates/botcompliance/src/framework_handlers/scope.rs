//! Branch scope for framework by-id mutations (#2750).
//!
//! The delete/update handlers used to address a single row by primary key
//! alone. Any authenticated caller holding a UUID from another tenant could
//! therefore delete or overwrite that row: the branch resolved from the
//! server-minted claims was computed but never applied to the statement.
//! Every by-id mutation now goes through [`scoped_evidence`],
//! [`scoped_framework`] or [`scoped_control`], which bind both `id` and
//! `branch_id`. A foreign row cannot satisfy that predicate, so the affected
//! row count is zero and the handler returns 404 — no client-supplied
//! `org_id`/`branch_id` is ever trusted.

use super::*;

pub(super) type ScopedEvidence = diesel::dsl::Filter<
    diesel::dsl::Filter<
        compliance_control_evidence::table,
        diesel::dsl::Eq<compliance_control_evidence::id, Uuid>,
    >,
    diesel::dsl::Eq<compliance_control_evidence::branch_id, Uuid>,
>;

pub(super) type ScopedFramework = diesel::dsl::Filter<
    diesel::dsl::Filter<compliance_frameworks::table, diesel::dsl::Eq<compliance_frameworks::id, Uuid>>,
    diesel::dsl::Eq<compliance_frameworks::branch_id, Uuid>,
>;

pub(super) type ScopedControl = diesel::dsl::Filter<
    diesel::dsl::Filter<compliance_controls::table, diesel::dsl::Eq<compliance_controls::id, Uuid>>,
    diesel::dsl::Eq<compliance_controls::branch_id, Uuid>,
>;

/// `compliance_control_evidence` row addressed by id **and** caller branch.
pub(super) fn scoped_evidence(id: Uuid, branch_id: Uuid) -> ScopedEvidence {
    compliance_control_evidence::table
        .filter(compliance_control_evidence::id.eq(id))
        .filter(compliance_control_evidence::branch_id.eq(branch_id))
}

/// `compliance_frameworks` row addressed by id **and** caller branch.
pub(super) fn scoped_framework(id: Uuid, branch_id: Uuid) -> ScopedFramework {
    compliance_frameworks::table
        .filter(compliance_frameworks::id.eq(id))
        .filter(compliance_frameworks::branch_id.eq(branch_id))
}

/// `compliance_controls` row addressed by id **and** caller branch.
pub(super) fn scoped_control(id: Uuid, branch_id: Uuid) -> ScopedControl {
    compliance_controls::table
        .filter(compliance_controls::id.eq(id))
        .filter(compliance_controls::branch_id.eq(branch_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn where_clause(sql: &str) -> String {
        sql.split("WHERE").nth(1).unwrap_or_default().to_string()
    }

    fn assert_scoped(sql: &str, table: &str) {
        let clause = where_clause(sql);
        assert!(
            clause.contains(&format!("\"{table}\".\"id\" = $")),
            "by-id statement lost its primary key predicate: {sql}"
        );
        assert!(
            clause.contains(&format!("\"{table}\".\"branch_id\" = $")),
            "by-id statement lost its tenant branch predicate: {sql}"
        );
    }

    /// Evidence deletion must carry both predicates; a foreign id matches no
    /// row, so the handler's affected-row check returns 404.
    #[test]
    fn evidence_delete_is_scoped_to_id_and_tenant_branch() {
        let id = Uuid::new_v4();
        let branch_id = Uuid::new_v4();
        let sql = diesel::debug_query::<diesel::pg::Pg, _>(&diesel::delete(scoped_evidence(
            id,
            branch_id,
        )))
        .to_string();

        assert_scoped(&sql, "compliance_control_evidence");
        assert!(sql.contains(&id.to_string()));
        assert!(sql.contains(&branch_id.to_string()));
    }

    /// Framework update must not be able to overwrite another tenant's row.
    #[test]
    fn framework_update_is_scoped_to_id_and_tenant_branch() {
        let id = Uuid::new_v4();
        let branch_id = Uuid::new_v4();
        let sql = diesel::debug_query::<diesel::pg::Pg, _>(&diesel::update(scoped_framework(
            id,
            branch_id,
        ))
        .set(compliance_frameworks::status.eq("archived")))
        .to_string();

        assert_scoped(&sql, "compliance_frameworks");
        assert!(sql.contains(&branch_id.to_string()));
    }

    /// Archive is a second update path on the same table — it needs the same
    /// branch predicate, otherwise a foreign framework gets archived.
    #[test]
    fn framework_archive_update_keeps_the_branch_predicate() {
        let id = Uuid::new_v4();
        let branch_id = Uuid::new_v4();
        let sql = diesel::debug_query::<diesel::pg::Pg, _>(&diesel::update(scoped_framework(
            id,
            branch_id,
        ))
        .set(compliance_frameworks::updated_at.eq(Utc::now())))
        .to_string();

        assert_scoped(&sql, "compliance_frameworks");
        assert!(sql.contains(&branch_id.to_string()));
    }

    #[test]
    fn control_update_is_scoped_to_id_and_tenant_branch() {
        let id = Uuid::new_v4();
        let branch_id = Uuid::new_v4();
        let sql = diesel::debug_query::<diesel::pg::Pg, _>(&diesel::update(scoped_control(
            id,
            branch_id,
        ))
        .set(compliance_controls::title.eq("x")))
        .to_string();

        assert_scoped(&sql, "compliance_controls");
        assert!(sql.contains(&branch_id.to_string()));
    }

    #[test]
    fn evidence_approve_update_is_scoped_to_id_and_tenant_branch() {
        let id = Uuid::new_v4();
        let branch_id = Uuid::new_v4();
        let sql = diesel::debug_query::<diesel::pg::Pg, _>(&diesel::update(scoped_evidence(
            id,
            branch_id,
        ))
        .set(compliance_control_evidence::status.eq("approved")))
        .to_string();

        assert_scoped(&sql, "compliance_control_evidence");
        assert!(sql.contains(&branch_id.to_string()));
    }

    /// Two-tenant check: the predicate binds the *caller's* branch, so a row
    /// owned by another branch can never satisfy it and reads/updates/deletes
    /// return zero rows for foreign ids.
    #[test]
    fn foreign_branch_cannot_match_own_row() {
        let id = Uuid::new_v4();
        let own = Uuid::new_v4();
        let foreign = Uuid::new_v4();

        for sql in [
            diesel::debug_query::<diesel::pg::Pg, _>(&scoped_evidence(id, own)).to_string(),
            diesel::debug_query::<diesel::pg::Pg, _>(&scoped_framework(id, own)).to_string(),
            diesel::debug_query::<diesel::pg::Pg, _>(&scoped_control(id, own)).to_string(),
        ] {
            assert!(sql.contains(&own.to_string()), "own branch not bound: {sql}");
            assert!(
                !sql.contains(&foreign.to_string()),
                "foreign branch leaked into the predicate: {sql}"
            );
        }

        let foreign_sql =
            diesel::debug_query::<diesel::pg::Pg, _>(&scoped_evidence(id, foreign)).to_string();
        assert!(foreign_sql.contains(&foreign.to_string()));
        assert!(!foreign_sql.contains(&own.to_string()));
    }
}
