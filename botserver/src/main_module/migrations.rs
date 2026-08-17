//! migrations - extracted from bootstrap.rs

use diesel::prelude::*;
use diesel::sql_query;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

/// Diesel 2.x computes a migration's version by stripping dashes from the
/// directory name (`version_from_string`). Databases migrated by older diesel
/// releases stored the full directory name (e.g. `6.0.0-01-core`), so those
/// rows never match the embedded versions and every migration is treated as
/// pending again — the very first one then fails with "relation already exists".
/// Normalize legacy version strings once so `run_pending_migrations` sees the
/// already-applied migrations and only runs genuinely new ones.
pub fn run_diesel_migrations(
    conn: &mut PgConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Err(e) = sql_query(
        "UPDATE __diesel_schema_migrations SET version = replace(version, '-', '')",
    )
    .execute(conn)
    {
        log::warn!("Failed to normalize migration versions (continuing): {}", e);
    }

    match conn.run_pending_migrations(MIGRATIONS) {
        Ok(applied) => {
            if applied.is_empty() {
                log::info!("All migrations already applied");
            } else {
                log::info!("Applied {} migration(s)", applied.len());
            }
            Ok(())
        }
        Err(e) => {
            log::error!("Migration pipeline failed (schema may be partially updated): {}", e);
            log::warn!("Continuing despite migration errors - database might be partially migrated");
            Ok(())
        }
    }
}
