//! Schema invariants for the Diesel macro output (#2064).
//!
//! `botschema` owns every `table!`/`joinable!` in the workspace, and the guard
//! that used to catch duplicate or malformed table names (`schema_guard.py`)
//! is not in the tree. These tests replace it: they render the `FROM` target of
//! the always-compiled modules and assert the Postgres identifier contract,
//! plus the primary-key/column-type contracts the query layer relies on.

use crate::*;
use diesel::prelude::*;
use diesel::sql_types::{Int4, Nullable, Uuid, Varchar};
use diesel::{Expression, Table};

/// The `FROM` target diesel will emit for a table.
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

/// Pairs of `(Rust table path, SQL table name)` for every table compiled in.
macro_rules! table_names {
    ($($t:path),* $(,)?) => {
        vec![$( (stringify!($t), table_sql(&$t)) ),*]
    };
}

fn core_tables() -> Vec<(&'static str, String)> {
    table_names![
        tenants::table,
        organizations::table,
        organization_invitations::table,
        branches::table,
        bots::table,
        system_automations::table,
        user_sessions::table,
        workflow_executions::table,
        workflow_events::table,
        bot_shared_memory::table,
        chat_preferences::table,
        message_history::table,
        users::table,
        clicks::table,
        bot_memories::table,
        basic_tools::table,
        session_tool_associations::table,
        bot_configuration::table,
        rbac_roles::table,
        rbac_groups::table,
        rbac_permissions::table,
        rbac_role_permissions::table,
        rbac_user_roles::table,
        rbac_user_groups::table,
        rbac_group_roles::table,
        user_llm_accounts::user_llm_accounts::table,
    ]
}

fn always_compiled_tables() -> Vec<(&'static str, String)> {
    let mut tables = core_tables();
    tables.extend(table_names![
        drive_files::table,
        drive_user_permissions::table,
        drive_starred::table,
        drive_share_links::table,
        desktop_connections::table,
        desktop_connection_log::table,
        database_query_history::table,
        database_saved_queries::table,
        feature_flags::table,
        email_crm_links::table,
        email_campaign_links::table,
        email_snooze::table,
        email_flags::table,
        email_nudges::table,
        run_events::table,
    ]);
    tables
}

#[test]
fn core_tables_keep_their_sql_names() {
    for (rust_path, sql_name) in core_tables() {
        let expected = rust_path.split("::").next().unwrap_or(rust_path);
        assert_eq!(
            sql_name, expected,
            "table! block {rust_path} renamed its SQL table"
        );
    }
}

#[test]
fn no_two_tables_share_a_sql_name() {
    let mut tables = always_compiled_tables();
    tables.sort_unstable_by(|a, b| a.1.cmp(&b.1));

    for pair in tables.windows(2) {
        let (left_path, left_name) = (&pair[0].0, &pair[0].1);
        let (right_path, right_name) = (&pair[1].0, &pair[1].1);
        assert_ne!(
            left_name, right_name,
            "{left_path} and {right_path} both declare the table `{left_name}`"
        );
    }
}

#[test]
fn every_table_name_is_a_postgres_identifier() {
    for (rust_path, sql_name) in always_compiled_tables() {
        assert!(
            sql_name.len() <= 63,
            "{rust_path}: `{sql_name}` exceeds the 63-byte identifier limit"
        );
        assert!(
            sql_name
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_'),
            "{rust_path}: `{sql_name}` must be lowercase snake_case"
        );
        assert!(
            sql_name.starts_with(|c: char| c.is_ascii_lowercase()),
            "{rust_path}: `{sql_name}` must start with a letter"
        );
        assert!(
            !sql_name.ends_with('_') && !sql_name.contains("__"),
            "{rust_path}: `{sql_name}` has an empty identifier segment"
        );
    }
}

/// Compile-time identity: the declared primary key column, not merely a column
/// with the same name. Renaming the key in a `table!` block fails to compile.
#[test]
fn primary_keys_are_the_documented_columns() {
    fn assert_pk<T, K>()
    where
        T: Table<PrimaryKey = K>,
    {
    }

    assert_pk::<tenants::table, tenants::id>();
    assert_pk::<organizations::table, organizations::org_id>();
    assert_pk::<branches::table, branches::id>();
    assert_pk::<users::table, users::id>();
    assert_pk::<bots::table, bots::id>();
    assert_pk::<user_sessions::table, user_sessions::id>();
    assert_pk::<drive_files::table, drive_files::id>();
    assert_pk::<rbac_user_groups::table, rbac_user_groups::id>();
}

/// `organizations.tenant_id` was added by `6.5.15-org-branches` without NOT
/// NULL, so a row without a tenant must still be loadable.
#[test]
fn organizations_tenant_id_is_a_nullable_uuid() {
    fn expects_nullable_uuid<T: Expression<SqlType = Nullable<Uuid>>>() {}
    expects_nullable_uuid::<organizations::tenant_id>();
}

#[test]
fn message_history_keeps_its_message_type_column() {
    fn expects_int4<T: Expression<SqlType = Int4>>() {}

    assert_eq!(table_sql(&message_history::table), "message_history");
    assert_eq!(message_history::message_type::NAME, "message_type");
    assert_eq!(message_history::session_id::NAME, "session_id");
    assert_eq!(message_history::message_index::NAME, "message_index");
    assert_eq!(message_history::content_encrypted::NAME, "content_encrypted");
    expects_int4::<message_history::message_type>();
    expects_int4::<message_history::token_count>();
}

/// The RBAC resolution query walks users → groups → roles; losing one edge
/// breaks admin detection for every request.
#[test]
fn rbac_user_groups_joins_users_and_groups() {
    fn assert_joinable<L, R>()
    where
        L: diesel::JoinTo<R>,
    {
    }

    assert_joinable::<rbac_user_groups::table, users::table>();
    assert_joinable::<rbac_user_groups::table, rbac_groups::table>();
    assert_joinable::<rbac_group_roles::table, rbac_groups::table>();
    assert_joinable::<rbac_group_roles::table, rbac_roles::table>();
    assert_joinable::<rbac_role_permissions::table, rbac_permissions::table>();
}

#[test]
fn rbac_columns_keep_their_names() {
    assert_eq!(rbac_user_groups::user_id::NAME, "user_id");
    assert_eq!(rbac_user_groups::group_id::NAME, "group_id");
    assert_eq!(rbac_group_roles::role_id::NAME, "role_id");
    assert_eq!(rbac_role_permissions::permission_id::NAME, "permission_id");
}

#[cfg(feature = "billing")]
#[test]
fn billing_contract_pins_the_unlimited_stock_sentinel() {
    fn expects_int4<T: Expression<SqlType = Int4>>() {}
    fn expects_nullable_varchar<T: Expression<SqlType = Nullable<Varchar>>>() {}

    // `botproducts::seed` writes `stock_quantity = -1` for "unlimited"; the
    // column must stay a signed Int4 for that sentinel to be storable.
    expects_int4::<products::stock_quantity>();
    expects_nullable_varchar::<products::sku>();
    assert_eq!(table_sql(&products::table), "products");
    assert_eq!(products::branch_id::NAME, "branch_id");
    assert_eq!(products::is_public::NAME, "is_public");
}

#[cfg(feature = "billing")]
#[test]
fn billing_invoices_join_their_items() {
    fn assert_joinable<L, R>()
    where
        L: diesel::JoinTo<R>,
    {
    }

    assert_joinable::<billing_invoice_items::table, billing_invoices::table>();
    assert_joinable::<billing_payments::table, billing_invoices::table>();
    assert_joinable::<billing_quote_items::table, billing_quotes::table>();
    assert_joinable::<price_list_items::table, price_lists::table>();
    assert_joinable::<price_list_items::table, products::table>();
}

#[cfg(feature = "billing")]
#[test]
fn billing_tables_are_reachable_from_the_crate_root() {
    // `lib.rs` re-exports the module, so consumers write `botschema::products`
    // and never the private `botschema::billing::catalog::products` path.
    assert_eq!(table_sql(&crate::products::table), "products");
    assert_eq!(table_sql(&crate::billing_invoices::table), "billing_invoices");
}
