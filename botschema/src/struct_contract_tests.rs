//! Contract tests for the row structs that model the Drive, Desktop and
//! database-manager tables (#2064).
//!
//! `#[diesel(check_for_backend(diesel::pg::Pg))]` on each struct turns
//! struct↔table drift into a compile error inside this module, and the runtime
//! assertions below pin the SQL names the queries rely on. `DriveShareLink` had
//! `download_count: i32` against a `Nullable<Int4>` column, so a link that was
//! never downloaded failed to load.

use crate::{
    database_query_history, database_saved_queries, desktop_connection_log,
    desktop_connections, drive_files, drive_share_links, drive_starred,
    drive_user_permissions,
};
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
fn the_row_structs_model_the_documented_tables() {
    assert_eq!(table_sql(&drive_files::table), "drive_files");
    assert_eq!(
        table_sql(&drive_user_permissions::table),
        "drive_user_permissions"
    );
    assert_eq!(table_sql(&drive_starred::table), "drive_starred");
    assert_eq!(table_sql(&drive_share_links::table), "drive_share_links");
    assert_eq!(table_sql(&desktop_connections::table), "desktop_connections");
    assert_eq!(
        table_sql(&desktop_connection_log::table),
        "desktop_connection_log"
    );
    assert_eq!(
        table_sql(&database_query_history::table),
        "database_query_history"
    );
    assert_eq!(
        table_sql(&database_saved_queries::table),
        "database_saved_queries"
    );
}

#[test]
fn the_row_structs_read_their_declared_columns() {
    assert_eq!(drive_user_permissions::permission::NAME, "permission");
    assert_eq!(drive_user_permissions::granted_by::NAME, "granted_by");
    assert_eq!(drive_starred::path::NAME, "path");
    assert_eq!(drive_share_links::token::NAME, "token");
    assert_eq!(drive_share_links::download_count::NAME, "download_count");
    assert_eq!(desktop_connections::auto_connect::NAME, "auto_connect");
    assert_eq!(desktop_connections::port::NAME, "port");
    assert_eq!(
        desktop_connection_log::bytes_transferred::NAME,
        "bytes_transferred"
    );
    assert_eq!(database_query_history::is_mutation::NAME, "is_mutation");
    assert_eq!(database_query_history::row_count::NAME, "row_count");
    assert_eq!(database_saved_queries::is_shared::NAME, "is_shared");
}

/// `download_count` is nullable in the schema, so the struct must model it as
/// `Option<i32>`: a never-downloaded link loads as `NULL`.
#[test]
fn a_share_link_without_downloads_is_representable() {
    let link = crate::DriveShareLink {
        id: uuid::Uuid::nil(),
        created_by: uuid::Uuid::nil(),
        bucket: "b".to_string(),
        path: "/a".to_string(),
        token: "t".to_string(),
        permission: "read".to_string(),
        expires_at: None,
        max_downloads: Some(10),
        download_count: None,
        created_at: chrono::Utc::now(),
    };
    assert_eq!(link.download_count, None);
    assert_eq!(link.max_downloads, Some(10));
}

/// Compile-time guard: the queryable structs stay `Selectable`, so a renamed
/// column or a dropped field is a compile error rather than a runtime
/// `DeserializationError`.
#[test]
fn every_queryable_struct_is_selectable() {
    fn assert_selectable<T: Selectable<diesel::pg::Pg>>() {}

    assert_selectable::<crate::DriveFile>();
    assert_selectable::<crate::DriveUserPermission>();
    assert_selectable::<crate::DriveStarred>();
    assert_selectable::<crate::DriveShareLink>();
    assert_selectable::<crate::DesktopConnectionDb>();
    assert_selectable::<crate::DesktopConnectionLogDb>();
    assert_selectable::<crate::QueryHistoryDb>();
    assert_selectable::<crate::SavedQueryDb>();
}

/// SCHEMA-13 — Diesel's tuple support stops at 32 columns; a wider `table!`
/// block cannot be selected from at all.
trait Arity {
    const ARITY: usize;
}

macro_rules! impl_arity {
    ($($name:ident),+) => {
        impl<$($name),+> Arity for ($($name,)+) {
            const ARITY: usize = impl_arity!(@count $($name)+);
        }
    };
    (@count $($name:ident)+) => { <[()]>::len(&[$(impl_arity!(@unit $name)),+]) };
    (@unit $name:ident) => { () };
}

impl_arity!(A);
impl_arity!(A, B);
impl_arity!(A, B, C);
impl_arity!(A, B, C, D);
impl_arity!(A, B, C, D, E);
impl_arity!(A, B, C, D, E, F);
impl_arity!(A, B, C, D, E, F, G);
impl_arity!(A, B, C, D, E, F, G, H);
impl_arity!(A, B, C, D, E, F, G, H, I);
impl_arity!(A, B, C, D, E, F, G, H, I, J);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD, AE);
impl_arity!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, AA, AB, AC, AD, AE, AF);

fn column_count<T>() -> usize
where
    T: Table,
    T::AllColumns: Arity,
{
    <T::AllColumns as Arity>::ARITY
}

#[test]
fn no_modeled_table_exceeds_the_tuple_limit() {
    let counts = [
        ("drive_files", column_count::<drive_files::table>()),
        (
            "drive_user_permissions",
            column_count::<drive_user_permissions::table>(),
        ),
        ("drive_starred", column_count::<drive_starred::table>()),
        (
            "drive_share_links",
            column_count::<drive_share_links::table>(),
        ),
        (
            "desktop_connections",
            column_count::<desktop_connections::table>(),
        ),
        (
            "desktop_connection_log",
            column_count::<desktop_connection_log::table>(),
        ),
        (
            "database_query_history",
            column_count::<database_query_history::table>(),
        ),
        (
            "database_saved_queries",
            column_count::<database_saved_queries::table>(),
        ),
    ];

    for (name, count) in counts {
        assert!(count > 0, "{name} declares no columns");
        assert!(count <= 32, "{name} declares {count} columns, the limit is 32");
    }
}
