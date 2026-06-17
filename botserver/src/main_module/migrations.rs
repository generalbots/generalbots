//! migrations - extracted from bootstrap.rs

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};


const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub fn run_diesel_migrations(
    conn: &mut impl MigrationHarness<diesel::pg::Pg>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    conn.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}
