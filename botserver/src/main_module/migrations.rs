//! migrations - extracted from bootstrap.rs

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub fn run_diesel_migrations(
    conn: &mut impl MigrationHarness<diesel::pg::Pg>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
