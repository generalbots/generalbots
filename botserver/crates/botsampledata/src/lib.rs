//! `botsampledata` — idempotent demo data seeding for a production-ready
//! General Bots instance.
//!
//! Seeding is safe to run on every boot: every statement is guarded by a
//! `WHERE NOT EXISTS`-style check so repeated runs never duplicate rows.
//!
//! What it creates:
//!   * DB rows for every suite app (people, CRM, tickets, billing, calendar,
//!     research, compliance, goals, workspaces, social, campaigns, lists,
//!     o365, drive) — always scoped to the dedicated sample tenant.
//!   * A demo email account + a handful of messages so the Mail app is usable.
//!
//! Entry point: [`seed_all`].

pub mod db;
pub mod drive;
pub mod email;
pub mod sample;
pub mod seed_all_apps;

use botcore::shared::utils::DbPool;

/// Seed every demo entity for the given pool.
///
/// Safe to call on every startup — all operations are idempotent. All demo
/// data is written exclusively under the dedicated sample tenant; if that
/// tenant cannot be created the seeding is skipped (never a real tenant).
pub fn seed_all(pool: &DbPool) {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            log::error!("botsampledata: could not acquire DB connection: {e}");
            return;
        }
    };

    // db::seed runs each domain independently (one app's legacy/missing table
    // does not abort the rest of the demo data). This also seeds the extended
    // app surface (products, dashboards, meet, learn, project, canvas,
    // attendant, OKR, database, integrations, sources, monitoring).
    match db::seed(&mut conn) {
        Ok(()) => log::info!("botsampledata: database demo data seeded (sample tenant)"),
        Err(e) => log::warn!("botsampledata: database seeding skipped (sample tenant unavailable): {e}"),
    }

    match email::seed(&mut conn) {
        Ok(()) => log::info!("botsampledata: demo email account + messages seeded (sample tenant)"),
        Err(e) => log::warn!("botsampledata: email seeding skipped (sample tenant unavailable): {e}"),
    }
}

/// Seeds the fiscal Drive objects (invoice folder + cash-flow spreadsheets)
/// into the default bot's MinIO bucket. Must run after the drive client is
/// available; guards every object with existence checks.
pub async fn seed_drive_fiscal(pool: &DbPool, drive: &dyn botlib::traits::DriveRepository) {
    match drive::seed_drive_objects(pool, drive).await {
        Ok(()) => log::info!("botsampledata: fiscal drive objects seeded"),
        Err(e) => log::error!("botsampledata: fiscal drive seeding failed: {e}"),
    }
}

/// Seeds the Pragmatismo reference bot payload (#750) into the
/// `pragmatismo.gbai` bucket. Safe to run every boot (existence-guarded).
pub async fn seed_pragmatismo_payload(drive: &dyn botlib::traits::DriveRepository) {
    match drive::seed_pragmatismo_payload(drive).await {
        Ok(()) => log::info!("botsampledata: pragmatismo payload seeded"),
        Err(e) => log::error!("botsampledata: pragmatismo payload seeding failed: {e}"),
    }
}
