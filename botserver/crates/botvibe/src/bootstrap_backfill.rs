//! Reform #1500 — boot backfill: every branch gets its default-bot Vibe
//! project and every bot-kind project its PROD/TEST bot pair (#1504).
//! Runs at boot via `configure_vibe_routes` and is fully idempotent.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use botcore::shared::utils::DbPool;
use diesel::prelude::*;
use uuid::Uuid;

use super::bootstrap::{alm_org_from_slug, ensure_bot_rows_conn, BotRowRef};

/// Pool handed over by the vibe router mount so naming helpers can resolve
/// branch slugs without every caller threading a pool through the API.
fn shared_pool_slot() -> &'static Mutex<Option<DbPool>> {
    static SLOT: OnceLock<Mutex<Option<DbPool>>> = OnceLock::new();
    SLOT.get_or_init(|| Mutex::new(None))
}

/// Store the app pool once at boot (`configure_vibe_routes`).
pub fn init_shared_pool(pool: DbPool) {
    if let Ok(mut slot) = shared_pool_slot().lock() {
        if slot.is_none() {
            *slot = Some(pool);
        }
    }
}

fn shared_pool() -> Option<DbPool> {
    shared_pool_slot().lock().ok().and_then(|slot| slot.clone())
}

/// Resolve the ALM org for a branch (slug preferred, short-uuid fallback).
/// Cached per process — slugs never change within a boot.
pub fn alm_org_for_branch(pool: &DbPool, branch_id: Uuid) -> String {
    static CACHE: OnceLock<Mutex<HashMap<Uuid, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(map) = cache.lock() {
        if let Some(org) = map.get(&branch_id) {
            return org.clone();
        }
    }
    let org = match query_branch_slug(pool, branch_id) {
        Some(slug) => alm_org_from_slug(&slug),
        None => crate::bootstrap::alm_org_from_slug(""),
    };
    if let Ok(mut map) = cache.lock() {
        map.insert(branch_id, org.clone());
    }
    org
}

fn query_branch_slug(pool: &DbPool, branch_id: Uuid) -> Option<String> {
    #[derive(diesel::QueryableByName)]
    struct BranchSlug {
        #[diesel(sql_type = diesel::sql_types::Text)]
        slug: String,
    }
    let mut conn = pool.get().ok()?;
    diesel::sql_query("SELECT slug FROM branches WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .get_result::<BranchSlug>(&mut conn)
        .ok()
        .map(|row| row.slug)
}

#[derive(diesel::QueryableByName, Clone)]
struct BackfillProject {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    org_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

fn count_branch_bots(conn: &mut PgConnection, branch_id: Uuid) -> i64 {
    #[derive(diesel::QueryableByName)]
    struct Count {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        n: i64,
    }
    diesel::sql_query("SELECT COUNT(*) AS n FROM bots WHERE branch_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .get_result::<Count>(conn)
        .map(|c| c.n)
        .unwrap_or(0)
}

fn backfill_sync(pool: &DbPool) -> Result<(usize, usize), String> {
    #[derive(diesel::QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        n: i64,
    }
    let mut conn = pool.get().map_err(|e| format!("db pool: {e}"))?;
    let mut projects = 0usize;

    #[derive(diesel::QueryableByName)]
    struct WorkspaceRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        branch_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    let workspaces: Vec<WorkspaceRow> = diesel::sql_query(
        "SELECT cw.org_id, cw.branch_id, cw.name FROM cloud_workspaces cw \
         JOIN organizations o ON o.org_id = cw.org_id \
         WHERE cw.branch_id <> '00000000-0000-0000-0000-000000000000'",
    )
    .load(&mut conn)
    .unwrap_or_default();
    // The inner JOIN drops workspaces whose org was deleted (bots.org_id has
    // an FK to organizations) — creating bot rows for them would violate
    // `bots_org_id_fkey`. Stale rows are the drive-monitor's responsibility
    // to re-create orgs for; here we simply skip what cannot exist.
    let stale: i64 = diesel::sql_query(
        "SELECT COUNT(*) AS n FROM cloud_workspaces cw \
         LEFT JOIN organizations o ON o.org_id = cw.org_id \
         WHERE cw.branch_id <> '00000000-0000-0000-0000-000000000000' \
           AND o.org_id IS NULL",
    )
    .get_result::<CountRow>(&mut conn)
    .map(|c| c.n)
    .unwrap_or(0);
    if stale > 0 {
        log::warn!(
            "vibe bootstrap: {stale} workspace(s) skipped — their org no longer exists"
        );
    }
    for ws in &workspaces {
        match crate::bootstrap::ensure_branch_default_project_conn(&mut conn, ws.org_id, ws.branch_id, &ws.name)
        {
            Ok(_) => projects += 1,
            Err(e) => log::warn!("vibe bootstrap: workspace '{}': {e}", ws.name),
        }
    }

    let mut twins = 0usize;
    let bot_projects: Vec<BackfillProject> = diesel::sql_query(
        "SELECT vp.id, vp.org_id, vp.branch_id, vp.name FROM vibe_projects vp \
         JOIN organizations o ON o.org_id = vp.org_id \
         WHERE vp.project_type = 'bot'",
    )
    .load(&mut conn)
    .unwrap_or_default();
    // Per-project tolerance: one broken row must never abort the whole
    // backfill (a warn keeps the rest converging on the next boot).
    for p in &bot_projects {
        let before = count_branch_bots(&mut conn, p.branch_id);
        if let Err(e) = ensure_bot_rows_conn(
            &mut conn,
            &BotRowRef {
                id: p.id,
                org_id: p.org_id,
                branch_id: p.branch_id,
                name: p.name.clone(),
            },
            None,
        ) {
            log::warn!("vibe bootstrap: project '{}': {e}", p.name);
            continue;
        }
        if count_branch_bots(&mut conn, p.branch_id) > before {
            twins += 1;
        }
    }
    Ok((projects, twins))
}

/// Boot backfill entry point (async wrapper over the blocking SQL work).
pub async fn backfill_default_projects(pool: DbPool) {
    let outcome = tokio::task::spawn_blocking(move || backfill_sync(&pool))
        .await
        .unwrap_or_else(|e| Err(format!("backfill join: {e}")));
    match outcome {
        Ok((projects, twins)) => log::info!(
            "vibe bootstrap backfill: {projects} default project(s) ensured, {twins} bot pair(s) ensured"
        ),
        Err(e) => log::error!("vibe bootstrap backfill failed: {e}"),
    }
}

/// Shared-pool convenience for the org resolution used by the git monitor.
pub fn alm_org_for_branch_shared(branch_id: Uuid) -> Option<String> {
    shared_pool().map(|pool| alm_org_for_branch(&pool, branch_id))
}
