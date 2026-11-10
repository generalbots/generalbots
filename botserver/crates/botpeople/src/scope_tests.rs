//! #2138 — every by-id people/team/time-off query must carry the caller's
//! branch.
//!
//! Handlers address a single row only through [`scoped_person`],
//! [`scoped_team`] and [`scoped_time_off`], so these tests pin the SQL shape
//! (`WHERE id = $1 AND branch_id = $2`). A row from another branch can never
//! satisfy that predicate, which is what turns a cross-tenant request into a
//! 404 instead of a leak. No database is needed: `debug_query` renders the
//! statement the handler would execute.

use super::*;

fn where_clause(sql: &str) -> String {
    sql.split("WHERE").nth(1).unwrap_or_default().to_string()
}

fn assert_scoped(sql: &str, table: &str) {
    let where_sql = where_clause(sql);
    assert!(
        where_sql.contains(&format!("\"{table}\".\"id\" = $")),
        "by-id query lost its primary key predicate: {sql}"
    );
    assert!(
        where_sql.contains(&format!("\"{table}\".\"branch_id\" = $")),
        "by-id query lost its branch predicate: {sql}"
    );
}

#[test]
fn scoped_person_read_filters_on_id_and_branch() {
    let id = Uuid::new_v4();
    let branch_id = Uuid::new_v4();

    let sql = diesel::debug_query::<diesel::pg::Pg, _>(
        &scoped_person(id, branch_id).select(people::all_columns),
    )
    .to_string();

    let where_sql = where_clause(&sql);
    assert!(where_sql.contains("\"people\".\"id\" = $1"), "{sql}");
    assert!(where_sql.contains("\"people\".\"branch_id\" = $2"), "{sql}");
    assert!(sql.contains(&branch_id.to_string()), "{sql}");
}

#[test]
fn scoped_person_update_and_delete_keep_the_branch_predicate() {
    let id = Uuid::new_v4();
    let branch_id = Uuid::new_v4();

    let update_sql = diesel::debug_query::<diesel::pg::Pg, _>(
        &diesel::update(scoped_person(id, branch_id)).set(people::job_title.eq("x")),
    )
    .to_string();
    assert_scoped(&update_sql, "people");

    let delete_sql =
        diesel::debug_query::<diesel::pg::Pg, _>(&diesel::delete(scoped_person(id, branch_id)))
            .to_string();
    assert_scoped(&delete_sql, "people");
}

#[test]
fn scoped_team_update_and_delete_keep_the_branch_predicate() {
    let id = Uuid::new_v4();
    let branch_id = Uuid::new_v4();

    let update_sql = diesel::debug_query::<diesel::pg::Pg, _>(
        &diesel::update(scoped_team(id, branch_id)).set(people_teams::name.eq("x")),
    )
    .to_string();
    assert_scoped(&update_sql, "people_teams");

    let delete_sql =
        diesel::debug_query::<diesel::pg::Pg, _>(&diesel::delete(scoped_team(id, branch_id)))
            .to_string();
    assert_scoped(&delete_sql, "people_teams");
}

#[test]
fn scoped_time_off_update_keeps_the_branch_predicate() {
    let id = Uuid::new_v4();
    let branch_id = Uuid::new_v4();

    let update_sql = diesel::debug_query::<diesel::pg::Pg, _>(
        &diesel::update(scoped_time_off(id, branch_id)).set(people_time_off::status.eq("approved")),
    )
    .to_string();

    assert_scoped(&update_sql, "people_time_off");
    assert!(update_sql.contains(&branch_id.to_string()), "{update_sql}");
}

/// Two branches produce two different predicates, so a foreign row can never
/// match the caller's statement.
#[test]
fn scoped_person_predicate_binds_the_caller_branch() {
    let id = Uuid::new_v4();
    let own = Uuid::new_v4();
    let foreign = Uuid::new_v4();

    let own_sql = diesel::debug_query::<diesel::pg::Pg, _>(&scoped_person(id, own)).to_string();
    let foreign_sql =
        diesel::debug_query::<diesel::pg::Pg, _>(&scoped_person(id, foreign)).to_string();

    assert!(own_sql.contains(&own.to_string()));
    assert!(!own_sql.contains(&foreign.to_string()));
    assert!(foreign_sql.contains(&foreign.to_string()));
    assert!(!foreign_sql.contains(&own.to_string()));
}
