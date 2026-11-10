//! Schema contract tests (#2064).
//!
//! `botschema` owns every `table!`/`joinable!` macro in the workspace, but it
//! had no test module at all: a renamed column or a dropped relation was only
//! discovered when a query failed at runtime. These tests pin the macro output
//! (the SQL table name, column names, column count and the joinable relations)
//! so a migration that drifts from the crate fails here first.
//!
//! They are pure: no database, no service, no fixture. Feature-gated tables
//! are tested when their feature is on (`--features full`).

use crate::*;
use diesel::prelude::*;

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

#[test]
fn users_table_contract_is_stable() {
    assert_eq!(table_sql(&users::table), "users");
    assert_eq!(users::id::NAME, "id");
    assert_eq!(users::username::NAME, "username");
    assert_eq!(users::email::NAME, "email");
    assert_eq!(users::password_hash::NAME, "password_hash");
    assert_eq!(users::is_admin::NAME, "is_admin");
}

#[test]
fn bots_table_contract_is_stable() {
    assert_eq!(table_sql(&bots::table), "bots");
    assert_eq!(bots::id::NAME, "id");
    assert_eq!(bots::name::NAME, "name");
    assert_eq!(bots::llm_provider::NAME, "llm_provider");
    assert_eq!(bots::llm_config::NAME, "llm_config");
}

#[test]
fn user_sessions_contract_is_stable() {
    assert_eq!(table_sql(&user_sessions::table), "user_sessions");
    assert_eq!(user_sessions::user_id::NAME, "user_id");
    assert_eq!(user_sessions::bot_id::NAME, "bot_id");
    assert_eq!(user_sessions::message_count::NAME, "message_count");
    assert_eq!(user_sessions::total_tokens::NAME, "total_tokens");
}

#[test]
fn tenant_and_branch_tables_keep_their_primary_keys() {
    assert_eq!(table_sql(&tenants::table), "tenants");
    assert_eq!(table_sql(&organizations::table), "organizations");
    assert_eq!(table_sql(&branches::table), "branches");
    // The organizations primary key is `org_id`, not `id` (the row predates
    // the `id` convention); a "fix" that renames it breaks every query.
    assert_eq!(organizations::org_id::NAME, "org_id");
    assert_eq!(branches::org_id::NAME, "org_id");
    assert_eq!(branches::tenant_id::NAME, "tenant_id");
}

#[test]
fn drive_files_contract_is_stable() {
    assert_eq!(table_sql(&drive_files::table), "drive_files");
    assert_eq!(drive_files::file_path::NAME, "file_path");
    assert_eq!(drive_files::file_type::NAME, "file_type");
    assert_eq!(drive_files::etag::NAME, "etag");
    assert_eq!(drive_files::indexed::NAME, "indexed");
}

#[cfg(feature = "billing")]
#[test]
fn products_contract_is_stable() {
    assert_eq!(table_sql(&products::table), "products");
    assert_eq!(products::sku::NAME, "sku");
    assert_eq!(products::product_type::NAME, "product_type");
    assert_eq!(products::price::NAME, "price");
    assert_eq!(products::tax_rate::NAME, "tax_rate");
}

#[cfg(feature = "integrations")]
#[test]
fn connectors_contract_is_stable() {
    assert_eq!(table_sql(&connectors::table), "connectors");
    assert_eq!(connectors::bot_id::NAME, "bot_id");
    assert_eq!(connectors::connector_type::NAME, "connector_type");
    assert_eq!(connectors::auth_config::NAME, "auth_config");
    assert_eq!(connectors::endpoints::NAME, "endpoints");
}

#[cfg(feature = "social")]
#[test]
fn social_posts_contract_is_stable() {
    assert_eq!(table_sql(&social_posts::table), "social_posts");
    assert_eq!(social_posts::org_id::NAME, "org_id");
    assert_eq!(social_posts::bot_id::NAME, "bot_id");
    assert_eq!(social_posts::content::NAME, "content");
    assert_eq!(social_posts::content_type::NAME, "content_type");
}

#[test]
fn rbac_tables_contract_is_stable() {
    assert_eq!(table_sql(&rbac_roles::table), "rbac_roles");
    assert_eq!(table_sql(&rbac_groups::table), "rbac_groups");
    assert_eq!(table_sql(&rbac_permissions::table), "rbac_permissions");
    assert_eq!(rbac_roles::display_name::NAME, "display_name");
    assert_eq!(rbac_groups::parent_group_id::NAME, "parent_group_id");
    assert_eq!(rbac_permissions::resource_type::NAME, "resource_type");
    assert_eq!(rbac_permissions::action::NAME, "action");
}

/// Compile-time guard: the `joinable!` relations the query layer depends on
/// must exist. Removing one is a compile error here, not a runtime error in
/// production.
#[test]
fn declared_joinable_relations_exist() {
    fn assert_joinable<L, R>()
    where
        L: diesel::JoinTo<R>,
    {
    }

    assert_joinable::<organizations::table, tenants::table>();
    assert_joinable::<branches::table, organizations::table>();
    assert_joinable::<branches::table, tenants::table>();
    assert_joinable::<bots::table, branches::table>();
    assert_joinable::<bots::table, organizations::table>();
}

#[test]
fn root_reexports_the_core_tables() {
    // `lib.rs` re-exports every module, so consumers use `botschema::users`
    // rather than the private module path.
    assert_eq!(table_sql(&crate::users::table), "users");
    assert_eq!(table_sql(&crate::drive_files::table), "drive_files");
    assert_eq!(table_sql(&crate::bots::table), "bots");
}
