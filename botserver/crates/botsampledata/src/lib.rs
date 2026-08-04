//! `botsampledata` — idempotent demo data seeding for a production-ready
//! General Bots instance.
//!
//! Seeding is safe to run on every boot: every statement is guarded by a
//! `WHERE NOT EXISTS`-style check so repeated runs never duplicate rows.
//!
//! What it creates:
//!   * DB rows for every suite app (people, CRM, tickets, billing, calendar,
//!     research, compliance, goals, workspaces, social, campaigns, lists,
//!     o365, drive) — scoped correctly per handler (real default branch vs nil).
//!   * A demo email account + a handful of messages so the Mail app is usable.
//!
//! Entry point: [`seed_all`].

pub mod db;
pub mod email;

use botcore::shared::utils::DbPool;

/// Seed every demo entity for the given pool.
///
/// Safe to call on every startup — all operations are idempotent.
pub fn seed_all(pool: &DbPool) {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            log::error!("botsampledata: could not acquire DB connection: {e}");
            return;
        }
    };

    match db::seed(&mut conn) {
        Ok(()) => log::info!("botsampledata: database demo data seeded"),
        Err(e) => log::error!("botsampledata: database seeding failed: {e}"),
    }

    match email::seed(&mut conn) {
        Ok(()) => log::info!("botsampledata: demo email account + messages seeded"),
        Err(e) => log::error!("botsampledata: email seeding failed: {e}"),
    }
}
