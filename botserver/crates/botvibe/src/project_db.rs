//! #1386 — Per-project databases for every Vibe project kind and environment.
//!
//! Each project owns TWO dedicated PostgreSQL databases so an experiment on
//! the dev twin can never read or write production data:
//!
//! | env        | database name              |
//! |------------|----------------------------|
//! | production | `app_{branch}_{project}`   |
//! | test/dev   | `app_{branch}_{project}_dev` |
//!
//! Naming mirrors `botcore::bot_database::generate_database_name`
//! (`bot_{branch}_{bot}`) so bots and apps share one convention. `website`
//! projects are static and get no database; they keep the twin layout only
//! for their payload dirs (see `site_env.rs`).
//!
//! Security: database names are built exclusively from alphanumeric +
//! underscore fragments and validated before reaching SQL, so identifier
//! injection is impossible. Connection URLs are injected as environment
//! variables and never logged.

use diesel::prelude::*;
use uuid::Uuid;

pub use botcore::project_db::{
    database_url_for, is_production_env, project_database_name, validate_db_name,
    MAX_DB_NAME_LEN,
};

/// Whether a project kind needs a database at all. `website` is static HTMX
/// served by the proxy; bots own their DB through `botcore::bot_database`.
pub fn kind_needs_database(project_type: &str) -> bool {
    project_type != "website"
}

#[derive(diesel::QueryableByName)]
struct DbExistsRow {
    #[diesel(sql_type = diesel::sql_types::Bool)]
    exists: bool,
}

fn database_exists(conn: &mut diesel::PgConnection, db_name: &str) -> Result<bool, String> {
    diesel::sql_query(format!(
        "SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = '{db_name}') AS exists"
    ))
    .get_result::<DbExistsRow>(conn)
    .map(|r| r.exists)
    .map_err(|e| format!("check database {db_name}: {e}"))
}

/// Ensure the database exists and return the full connection URL for it.
///
/// Credentials come from the botserver main `DATABASE_URL` (Vault-managed) so
/// project databases live on the same PostgreSQL instance the stack already
/// provisions. Creation is idempotent; concurrent creates converge via the
/// `already exists` check.
pub fn ensure_project_database(
    pool: &crate::types::DbPool,
    branch_id: Uuid,
    project_name: &str,
    env: &str,
) -> Result<String, String> {
    use diesel::prelude::*;

    let db_name = project_database_name(branch_id, project_name, env);
    validate_db_name(&db_name)?;

    let mut conn = pool.get().map_err(|e| format!("db pool: {e}"))?;

    if !database_exists(&mut conn, &db_name)? {
        if let Err(e) = diesel::sql_query(format!("CREATE DATABASE {db_name}")).execute(&mut conn) {
            let err = e.to_string();
            // Concurrent creation racing us is success, anything else is not.
            if !err.contains("already exists") {
                return Err(format!("create database {db_name}: {err}"));
            }
        }
        log::info!("Vibe project db: created {db_name} (env {env})");
    }

    Ok(database_url_for(&db_name))
}

/// Drop both environments' databases for a project (delete/eviction path).
/// Best-effort: one failure does not stop the other from being dropped.
pub fn drop_project_databases(
    pool: &crate::types::DbPool,
    branch_id: Uuid,
    project_name: &str,
) -> Vec<String> {
    use diesel::prelude::*;

    let mut errors = Vec::new();
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return vec![format!("db pool: {e}")],
    };
    for env in ["production", "test"] {
        let db_name = project_database_name(branch_id, project_name, env);
        if let Err(e) = validate_db_name(&db_name) {
            errors.push(e);
            continue;
        }
        match database_exists(&mut conn, &db_name) {
            Ok(false) => continue,
            Ok(true) => {}
            Err(e) => {
                errors.push(e);
                continue;
            }
        }
        // WITH (FORCE) terminates remaining connections so eviction can
        // reclaim a busy project cleanly (PostgreSQL 13+).
        match diesel::sql_query(format!("DROP DATABASE IF EXISTS {db_name} WITH (FORCE)"))
            .execute(&mut conn)
        {
            Ok(_) => log::info!("Vibe project db: dropped {db_name}"),
            Err(e) => errors.push(format!("drop {db_name}: {e}")),
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    const BRANCH: Uuid = Uuid::nil();

    #[test]
    fn prod_and_dev_names_differ() {
        let prod = project_database_name(BRANCH, "my-store", "production");
        let dev = project_database_name(BRANCH, "my-store", "test");
        assert_eq!(prod, "app_00000000_my_store");
        assert_eq!(dev, "app_00000000_my_store_dev");
        assert_ne!(prod, dev);
    }

    #[test]
    fn legacy_dev_spellings_map_to_dev_db() {
        assert_eq!(
            project_database_name(BRANCH, "x", "development"),
            project_database_name(BRANCH, "x", "dev")
        );
        assert_eq!(
            project_database_name(BRANCH, "x", "development"),
            project_database_name(BRANCH, "x", "test")
        );
    }

    #[test]
    fn names_are_identifier_safe() {
        for name in ["My Store!", "../etc", "a b-c_d"] {
            let db = project_database_name(BRANCH, name, "production");
            assert!(validate_db_name(&db).is_ok(), "{name} → {db}");
        }
    }

    #[test]
    fn website_needs_no_db() {
        assert!(!kind_needs_database("website"));
        assert!(kind_needs_database("bot"));
        assert!(kind_needs_database("apps"));
        assert!(kind_needs_database("custom"));
    }

    #[test]
    fn long_names_keep_suffix_under_limit() {
        let long = "a".repeat(80);
        let dev = project_database_name(BRANCH, &long, "test");
        assert!(dev.len() <= MAX_DB_NAME_LEN);
        assert!(dev.ends_with("_dev"));
    }

    #[test]
    fn url_appends_database_after_last_slash() {
        // Scoped so the env mutation cannot race other tests in this module.
        std::env::set_var(
            "DATABASE_URL",
            "postgres://u:p@db.local:5432/botserver?sslmode=disable",
        );
        let url = database_url_for("app_x");
        assert!(url.starts_with("postgres://u:p@db.local:5432/app_x?sslmode=disable"));
        assert!(!url.ends_with("/botserver"));
    }
}
