//! #2138 — every by-id ticket query must carry the caller's branch.
//!
//! Handlers address a single ticket only through [`scoped_ticket`], so these
//! tests pin the SQL shape (`WHERE id = $1 AND branch_id = $2`). A row that
//! belongs to another branch can never satisfy that predicate, which is what
//! turns a cross-tenant request into a 404 instead of a leak. No database is
//! needed: `debug_query` renders the statement the handler would execute.

use super::*;

fn where_clause(sql: &str) -> String {
    sql.split("WHERE").nth(1).unwrap_or_default().to_string()
}

#[test]
fn scoped_ticket_select_filters_on_id_and_branch() {
    let id = Uuid::new_v4();
    let branch_id = Uuid::new_v4();

    let sql = diesel::debug_query::<diesel::pg::Pg, _>(
        &scoped_ticket(id, branch_id).select(support_tickets::all_columns),
    )
    .to_string();

    let where_sql = where_clause(&sql);
    assert!(
        where_sql.contains("\"support_tickets\".\"id\" = $1"),
        "by-id read lost its primary key predicate: {sql}"
    );
    assert!(
        where_sql.contains("\"support_tickets\".\"branch_id\" = $2"),
        "by-id read lost its branch predicate: {sql}"
    );
    assert!(
        sql.contains(&id.to_string()) && sql.contains(&branch_id.to_string()),
        "the ticket id and the caller branch must both be bind parameters: {sql}"
    );
}

#[test]
fn scoped_ticket_update_keeps_the_branch_predicate() {
    let id = Uuid::new_v4();
    let branch_id = Uuid::new_v4();

    let sql = diesel::debug_query::<diesel::pg::Pg, _>(
        &diesel::update(scoped_ticket(id, branch_id)).set(support_tickets::subject.eq("x")),
    )
    .to_string();

    let where_sql = where_clause(&sql);
    assert!(
        where_sql.contains("\"support_tickets\".\"id\" = $"),
        "by-id update lost its primary key predicate: {sql}"
    );
    assert!(
        where_sql.contains("\"support_tickets\".\"branch_id\" = $"),
        "by-id update would rewrite another tenant's ticket: {sql}"
    );
    assert!(
        sql.contains(&branch_id.to_string()),
        "the caller branch must be a bind parameter: {sql}"
    );
}

#[test]
fn scoped_ticket_delete_keeps_the_branch_predicate() {
    let id = Uuid::new_v4();
    let branch_id = Uuid::new_v4();

    let sql =
        diesel::debug_query::<diesel::pg::Pg, _>(&diesel::delete(scoped_ticket(id, branch_id)))
            .to_string();

    let where_sql = where_clause(&sql);
    assert!(
        where_sql.contains("\"support_tickets\".\"id\" = $1"),
        "by-id delete lost its primary key predicate: {sql}"
    );
    assert!(
        where_sql.contains("\"support_tickets\".\"branch_id\" = $2"),
        "by-id delete would remove another tenant's ticket: {sql}"
    );
}

/// The predicate binds the *caller's* branch: two branches produce two
/// different statements, so a foreign row can never match.
#[test]
fn scoped_ticket_predicate_binds_the_caller_branch() {
    let id = Uuid::new_v4();
    let own = Uuid::new_v4();
    let foreign = Uuid::new_v4();

    let own_sql = diesel::debug_query::<diesel::pg::Pg, _>(&scoped_ticket(id, own)).to_string();
    let foreign_sql =
        diesel::debug_query::<diesel::pg::Pg, _>(&scoped_ticket(id, foreign)).to_string();

    assert!(own_sql.contains(&own.to_string()));
    assert!(!own_sql.contains(&foreign.to_string()));
    assert!(foreign_sql.contains(&foreign.to_string()));
    assert!(!foreign_sql.contains(&own.to_string()));
}
