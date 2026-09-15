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

/// Longest legal PostgreSQL identifier (NAMEDATALEN - 1).
const MAX_DB_NAME_LEN: usize = 63;

/// Sanitize a path component into a legal database-name fragment.
/// Mirrors `vm_lifecycle::sanitize_part` but keeps underscores (DB names).
fn sanitize_db_part(s: &str) -> String {
    let mut out = String::new();
    let mut last_underscore = false;
    for ch in s.chars().take(32) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_underscore = false;
        } else if (ch == '-' || ch == '_' || ch.is_whitespace()) && !out.is_empty() && !last_underscore {
            out.push('_');
            last_underscore = true;
        }
    }
    if out.is_empty() {
        "app".to_string()
    } else {
        out
    }
}

/// Branch fragment for database names: the first UUID group is stable and
/// short (`alm_org` convention), keeping generated names well under 63 chars.
fn branch_part(branch_id: Uuid) -> String {
    sanitize_db_part(&branch_id.to_string().split('-').next().unwrap_or("default"))
}

/// `true` when `env` resolves to the production environment.
fn is_production(env: &str) -> bool {
    crate::site_env::SiteEnv::parse(env)
        .map(|e| e == crate::site_env::SiteEnv::Production)
        .unwrap_or(false)
}

/// Build the database name for a project in one environment.
///
/// - `production` → `app_{branch}_{name}`
/// - anything else (test/development/dev/staging) → `app_{branch}_{name}_dev`
pub fn project_database_name(branch_id: Uuid, project_name: &str, env: &str) -> String {
    let suffix = if is_production(env) { "" } else { "_dev" };
    let max_base = MAX_DB_NAME_LEN - suffix.len();
    let base = format!("app_{}_{}", branch_part(branch_id), sanitize_db_part(project_name));
    let base = if base.len() > max_base {
        base[..max_base].to_string()
    } else {
        base
    };
    format!("{base}{suffix}")
}

/// Validate a database name before it reaches SQL. Rejects anything that is
/// not `[a-z0-9_]` (the only characters we ever generate) or too long.
fn validate_db_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > MAX_DB_NAME_LEN {
        return Err(format!("invalid database name length: {} chars", name.len()));
    }
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        return Err(format!("invalid characters in database name '{name}'"));
    }
    Ok(())
}

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

/// Connection URL for a project database, derived from the botserver main
/// URL (credentials/host) with the database segment replaced. Callers must
/// inject it as an environment variable and never log it.
pub fn database_url_for(db_name: &str) -> String {
    let base = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/botserver".to_string());
    match base.rfind('/') {
        Some(pos) => format!("{}/{db_name}{}", &base[..pos], query_suffix(&base)),
        None => format!("{base}/{db_name}"),
    }
}

/// Preserve a query string (`?sslmode=...`) from the original URL.
fn query_suffix(original: &str) -> String {
    match original.find('?') {
        Some(pos) => original[pos..].to_string(),
        None => String::new(),
    }
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
