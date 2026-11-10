//! Environment, connection and skip helpers shared by the integrity tests.
//!
//! Everything here fails loudly under `INTEGRITY_STRICT=1` (CI) and skips with a
//! precise reason otherwise, so a developer without PostgreSQL still gets a green
//! run while the CI job cannot pass without actually exercising the database.

use std::path::PathBuf;

use diesel::prelude::*;

use super::{dataset, restore};

pub const STRICT_ENV: &str = "INTEGRITY_STRICT";
pub const DATASET_ENV: &str = "INTEGRITY_DATASET";
pub const DEFAULT_DATASET_DIR: &str = "/tmp/gb-integrity";

/// Skips with a reason, or fails when the suite is running in strict (CI) mode.
macro_rules! skip_or_fail {
    ($strict:expr, $ret:expr, $($arg:tt)*) => {{
        let reason = format!($($arg)*);
        if $strict {
            panic!("data-integrity suite cannot run: {reason}");
        }
        eprintln!("SKIP (data-integrity): {reason}");
        return $ret;
    }};
}

pub fn strict() -> bool {
    std::env::var(STRICT_ENV).is_ok_and(|value| value == "1")
}

pub fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    match manifest_dir.parent() {
        Some(root) => PathBuf::from(root),
        None => manifest_dir,
    }
}

pub fn dataset_dir() -> PathBuf {
    match std::env::var(DATASET_ENV) {
        Ok(dir) if !dir.trim().is_empty() => PathBuf::from(dir),
        _ => PathBuf::from(DEFAULT_DATASET_DIR),
    }
}

pub fn database_url(strict: bool) -> Option<String> {
    match std::env::var("DATABASE_URL") {
        Ok(url) if !url.trim().is_empty() => Some(url),
        _ => skip_or_fail!(
            strict,
            None,
            "DATABASE_URL is not set; point it at a migrated throwaway database"
        ),
    }
}

pub fn connect(strict: bool) -> Option<PgConnection> {
    let url = match database_url(strict) {
        Some(url) => url,
        None => return None,
    };
    match PgConnection::establish(&url) {
        Ok(conn) => Some(conn),
        Err(e) => skip_or_fail!(strict, None, "cannot connect to DATABASE_URL: {e}"),
    }
}

fn count_integrity_orgs(conn: &mut PgConnection) -> Result<i64, String> {
    #[derive(QueryableByName)]
    struct Count {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        row_count: i64,
    }
    diesel::sql_query(
        "SELECT count(*) AS row_count FROM organizations WHERE slug LIKE 'integrity-%'",
    )
    .get_result::<Count>(conn)
    .map(|row| row.row_count)
    .map_err(|e| format!("count integrity orgs: {e}"))
}

/// Loads the manifest and makes sure the dataset is in the database, loading it
/// through `psql` when it is not.
pub fn ensure_dataset(
    strict: bool,
    conn: &mut PgConnection,
    url: &str,
) -> Option<dataset::Manifest> {
    let dir = dataset_dir();
    let manifest = match dataset::load(&dir) {
        Ok(manifest) => manifest,
        Err(e) => skip_or_fail!(
            strict,
            None,
            "{e} — generate it with: python3 scripts/gen_integrity_dataset.py --out {}",
            dir.display()
        ),
    };
    let loaded = match count_integrity_orgs(conn) {
        Ok(count) => count,
        Err(e) => skip_or_fail!(strict, None, "{e}"),
    };
    if loaded == manifest.orgs.len() as i64 {
        return Some(manifest);
    }
    if !restore::available("psql") {
        skip_or_fail!(
            strict,
            None,
            "psql is not on PATH, so {} cannot be loaded",
            dir.join(&manifest.sql_file).display()
        );
    }
    let sql_path = dir.join(&manifest.sql_file);
    if let Err(e) = restore::psql_file(url, &sql_path) {
        skip_or_fail!(strict, None, "dataset load failed: {e}");
    }
    Some(manifest)
}
