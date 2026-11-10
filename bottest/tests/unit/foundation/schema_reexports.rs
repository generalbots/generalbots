//! #1497/#1513 — `botbasic_types::schema::tables` must re-export the canonical
//! `botschema` tables, not carry a private copy. This test lives in `bottest`
//! because it spans two crates.

use diesel::prelude::*;

fn table_sql<T>(table: &T) -> String
where
    T: diesel::query_builder::QueryFragment<diesel::pg::Pg>
        + diesel::query_builder::QueryId,
{
    diesel::debug_query::<diesel::pg::Pg, _>(table)
        .to_string()
        .split(" -- binds")
        .next()
        .unwrap_or_default()
        .replace('"', "")
        .trim()
        .to_string()
}

#[test]
fn basic_types_tables_are_the_botschema_tables() {
    assert_eq!(
        table_sql(&botbasic_types::schema::tables::bots::table),
        table_sql(&botschema::bots::table)
    );
    assert_eq!(
        table_sql(&botbasic_types::schema::tables::user_sessions::table),
        table_sql(&botschema::user_sessions::table)
    );
    assert_eq!(
        table_sql(&botbasic_types::schema::tables::bot_memories::table),
        table_sql(&botschema::bot_memories::table)
    );
}

#[test]
fn the_reexports_are_usable_with_diesel_column_names() {
    // Column constants come from the same macro expansion; a local copy would
    // diverge on the next migration.
    assert_eq!(
        botbasic_types::schema::tables::bots::name::NAME,
        botschema::bots::name::NAME
    );
    assert_eq!(
        botbasic_types::schema::tables::user_sessions::bot_id::NAME,
        botschema::user_sessions::bot_id::NAME
    );
}
