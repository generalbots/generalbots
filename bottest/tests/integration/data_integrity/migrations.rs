//! Migration bookkeeping: what is on disk versus what diesel recorded as
//! applied. A fresh database must have applied every migration exactly once.

use std::fs;
use std::path::Path;

use diesel::prelude::*;
use diesel::sql_types::Text;

/// Migration directories on disk, sorted by diesel's version order.
pub fn migration_versions(dir: &Path) -> Result<Vec<String>, String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut versions = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("{}: {e}", dir.display()))?;
        let path = entry.path();
        if !path.is_dir() || !path.join("up.sql").is_file() {
            continue;
        }
        match path.file_name().and_then(|name| name.to_str()) {
            Some(name) => versions.push(name.to_string()),
            None => {
                return Err(format!(
                    "migration directory with a non-UTF8 name: {}",
                    path.display()
                ))
            }
        }
    }
    versions.sort();
    Ok(versions)
}

/// The version diesel stores in `__diesel_schema_migrations` for a migration
/// directory. Diesel derives it from the directory name by taking the part
/// before the first `_` and removing every `-` (`migrations_internals::
/// version_from_string`), so `6.0.0-01-core` is recorded as `6.0.001core`.
pub fn canonical_version(directory: &str) -> String {
    directory
        .split('_')
        .next()
        .unwrap_or(directory)
        .replace('-', "")
}

/// Versions diesel recorded as applied.
pub fn applied_versions(conn: &mut PgConnection) -> Result<Vec<String>, String> {
    #[derive(diesel::QueryableByName)]
    struct VersionRow {
        #[diesel(sql_type = Text)]
        version: String,
    }
    let rows: Vec<VersionRow> = diesel::sql_query("SELECT version FROM __diesel_schema_migrations")
        .load(conn)
        .map_err(|e| format!("__diesel_schema_migrations: {e}"))?;
    let mut versions: Vec<String> = rows.into_iter().map(|row| row.version).collect();
    versions.sort();
    Ok(versions)
}
