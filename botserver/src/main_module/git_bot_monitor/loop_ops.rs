//! Reform #1502/#1504 — git monitor loop, hooks dispatch and Drive import.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use botcore::shared::state::AppState;
use botcore::shared::utils::{get_work_path, DbPool};
use diesel::prelude::*;
use uuid::Uuid;

use botcoresecrets::hooks as vibe_hooks;

use super::core::{
    alm_repo, copy_dir_recursive, ensure_checkout_with_heal, materialize_checkout, run_git,
    MonitoredBot,
};

const DEFAULT_MONITOR_SECS: u64 = 15;

fn list_monitored_bots(pool: &DbPool) -> Vec<MonitoredBot> {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        branch_slug: Option<String>,
    }
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            log::error!("[git_monitor] pool: {e}");
            return Vec::new();
        }
    };
    diesel::sql_query(
        "SELECT vp.id, vp.name, br.slug AS branch_slug \
         FROM vibe_projects vp \
         LEFT JOIN branches br ON br.id = vp.branch_id \
         WHERE vp.project_type = 'bot' AND vp.source_control = 'git'",
    )
    .load::<Row>(&mut conn)
    .unwrap_or_default()
    .into_iter()
    .map(|row| MonitoredBot {
        repo_slug: alm_repo(&row.name),
        project_id: row.id,
        branch_slug: row.branch_slug,
        name: row.name,
    })
    .collect()
}

pub(crate) fn resolve_branch_id(pool: &DbPool, branch_slug: &str) -> Uuid {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
    }
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(_) => return Uuid::nil(),
    };
    diesel::sql_query("SELECT id FROM branches WHERE slug = $1 LIMIT 1")
        .bind::<diesel::sql_types::Text, _>(branch_slug)
        .get_result::<Row>(&mut conn)
        .map(|r| r.id)
        .unwrap_or_else(|_| Uuid::nil())
}

/// Bump `drive_files.etag` for the materialized paths so the existing
/// DriveCompiler pipeline recompiles them (etag change triggers compile; a
/// missing S3 object falls back to the fresh work copy).
fn mark_for_compile(pool: &DbPool, branch_id: Uuid, paths: &[String], etag: &str) {
    let repo = botdrive::DriveFileRepository::new(pool.clone());
    for fp in paths {
        if let Err(e) = repo.upsert_file(fp, "bas", Some(etag.to_string()), None, Some(branch_id)) {
            log::warn!("[git_monitor] drive_files upsert {fp}: {e}");
        }
    }
    if !paths.is_empty() {
        log::info!(
            "[git_monitor] queued {} file(s) for compile (etag {etag})",
            paths.len()
        );
    }
}

fn sync_one(pool: &DbPool, project: &MonitoredBot, work_root: &Path) {
    let branch_slug = match project.branch_slug.as_deref().filter(|s| !s.trim().is_empty()) {
        Some(slug) => slug.trim().to_string(),
        None => {
            log::debug!("[git_monitor] project {} has no branch slug — skipped", project.name);
            return;
        }
    };
    let org = botvibe::bootstrap::alm_org_from_slug(&branch_slug);
    // Self-healing checkout: a missing Forgejo repo is provisioned from the
    // deployed PROD sources (#1503 backfill) instead of warn-looping forever.
    let checkout = match ensure_checkout_with_heal(pool, project, &org) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("[git_monitor] checkout {}/{}: {e}", org, project.repo_slug);
            return;
        }
    };
    // Fetch and compare without touching the working tree on no-op ticks.
    if let Err(e) = run_git(&checkout, &["fetch", "origin", "main"]) {
        log::debug!("[git_monitor] fetch {}/{}: {e}", org, project.repo_slug);
        return;
    }
    let local = run_git(&checkout, &["rev-parse", "HEAD"]).unwrap_or_default();
    let remote = run_git(&checkout, &["rev-parse", "origin/main"]).unwrap_or_default();
    let remote = remote.trim().to_string();
    if remote.is_empty() {
        return; // empty remote (new repo) — nothing to pull yet
    }
    if local.trim() == remote {
        return; // already in sync
    }
    if let Err(e) = run_git(&checkout, &["reset", "--hard", "origin/main"]) {
        log::warn!("[git_monitor] reset {}/{}: {e}", org, project.repo_slug);
        return;
    }
    log::info!("[git_monitor] {} pulled to {}", project.name, checkout.display());

    // PROD layout follows main (git is the deployable truth); the TEST twin
    // mirrors the same content — Run re-materializes from the workspace.
    let branch_id = resolve_branch_id(pool, &branch_slug);
    match materialize_checkout(&checkout, work_root, &branch_slug, &project.name) {
        Ok(paths) => mark_for_compile(pool, branch_id, &paths, &remote),
        Err(e) => log::error!("[git_monitor] materialize {}: {e}", project.name),
    }
    let test_bot = format!("{}-test", project.name);
    match materialize_checkout(&checkout, work_root, &branch_slug, &test_bot) {
        Ok(paths) => mark_for_compile(pool, branch_id, &paths, &remote),
        Err(e) => log::error!("[git_monitor] materialize {test_bot}: {e}"),
    }
}

/// Copy the TEST materialization over the PROD layout and queue recompiles —
/// the #1504 "deploy copies all to the default bot" step.
fn promote_test_to_prod(
    pool: &DbPool,
    branch_slug: &str,
    bot_name: &str,
    etag: &str,
) -> Result<(), String> {
    let work_root = PathBuf::from(get_work_path());
    let base = work_root.join(format!("{branch_slug}.gborg/{branch_slug}.gbai"));
    let src = base.join(format!("{}-test.gbdialog", bot_name));
    let dst = base.join(format!("{bot_name}.gbdialog"));
    if !src.is_dir() {
        return Err(format!("no TEST materialization for {bot_name} — run the project first"));
    }
    // Replace PROD content with the TEST release (mirrors site promote).
    if dst.is_dir() {
        std::fs::remove_dir_all(&dst).map_err(|e| format!("clear prod layout: {e}"))?;
    }
    copy_dir_recursive(&src, &dst).map_err(|e| format!("promote: {e}"))?;

    let mut paths = Vec::new();
    for entry in std::fs::read_dir(&dst).map_err(|e| format!("read prod layout: {e}"))? {
        let entry = entry.map_err(|e| format!("entry: {e}"))?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".bas") {
            let tool = name.trim_end_matches(".bas").to_string();
            paths.push(format!("{branch_slug}.gbai/{bot_name}.gbdialog/{tool}.bas"));
        }
    }
    let branch_id = resolve_branch_id(pool, branch_slug);
    mark_for_compile(pool, branch_id, &paths, etag);
    log::info!("[git_monitor] {bot_name}: TEST → PROD promotion queued");
    Ok(())
}

fn find_project(pool: &DbPool, project_id: Uuid) -> Result<MonitoredBot, String> {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        branch_slug: Option<String>,
    }
    let mut conn = pool.get().map_err(|e| format!("db pool: {e}"))?;
    diesel::sql_query(
        "SELECT vp.id, vp.name, br.slug AS branch_slug \
         FROM vibe_projects vp \
         LEFT JOIN branches br ON br.id = vp.branch_id \
         WHERE vp.id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(project_id)
    .get_result::<Row>(&mut conn)
    .optional()
    .map_err(|e| format!("project lookup: {e}"))?
    .map(|row| MonitoredBot {
        repo_slug: alm_repo(&row.name),
        project_id: row.id,
        branch_slug: row.branch_slug,
        name: row.name,
    })
    .ok_or_else(|| format!("project {project_id} not found"))
}

/// Re-materialize the TEST twin from the agent workspace right now (#1504
/// Run): the workspace IS the dev state; the checkout only provides history.
fn run_test_from_workspace(pool: &DbPool, project: &MonitoredBot) -> Result<(), String> {
    let branch_slug = project
        .branch_slug
        .clone()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "project has no branch slug".to_string())?;
    let workspace = botvibe::harness::workspace_root().join(&project.repo_slug);
    if !workspace.is_dir() {
        return Err(format!(
            "workspace '{}' missing — open the project in Vibe first",
            workspace.display()
        ));
    }
    let work_root = PathBuf::from(get_work_path());
    let test_bot = format!("{}-test", project.name);
    let commit = run_git(&workspace, &["rev-parse", "HEAD"]).unwrap_or_else(|_| "work".to_string());
    let paths = materialize_checkout(&workspace, &work_root, branch_slug.trim(), &test_bot)?;
    mark_for_compile(pool, resolve_branch_id(pool, &branch_slug), &paths, commit.trim());
    log::info!("[git_monitor] {} RUN → TEST twin recompiled", project.name);
    Ok(())
}

/// Dispatch table for the #1504 hooks ("run-test" | "deploy-prod").
fn dispatch_bot_op(pool: &DbPool, op: &str, project_id: Uuid) -> Result<(), String> {
    let project = find_project(pool, project_id)?;
    match op {
        "run-test" => run_test_from_workspace(pool, &project),
        "deploy-prod" => {
            let branch_slug = project
                .branch_slug
                .clone()
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| "project has no branch slug".to_string())?;
            // A deploy always ships the committed state: push any uncommitted
            // workspace edits first so PROD == git HEAD.
            let workspace = botvibe::harness::workspace_root().join(&project.repo_slug);
            let _ = run_git(&workspace, &["add", "-A"]);
            let _ = run_git(
                &workspace,
                &[
                    "-c", "user.name=Vibe Agent",
                    "-c", "user.email=vibe@gbo.local",
                    "commit", "-m", &format!("Deploy {}", project.name),
                ],
            );
            let _ = run_git(&workspace, &["push", "origin", "HEAD:main"]);
            let etag =
                run_git(&workspace, &["rev-parse", "HEAD"]).unwrap_or_else(|_| "deploy".to_string());
            promote_test_to_prod(pool, branch_slug.trim(), &project.name, etag.trim())
        }
        other => Err(format!("unknown bot project op '{other}'")),
    }
}

/// Background loop: every tick syncs every monitored project.
pub async fn start(app_state: Arc<AppState>, pool: DbPool) {
    let secs: u64 = std::env::var("GB_VIBE_GIT_MONITOR_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|v| *v >= 5)
        .unwrap_or(DEFAULT_MONITOR_SECS);
    register_hooks(pool.clone(), app_state.clone());
    tokio::spawn(async move {
        log::info!(
            "[git_monitor] started (interval {secs}s) — bot sources come from git, not Drive"
        );
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
            let work_root = PathBuf::from(get_work_path());
            let projects = list_monitored_bots(&pool);
            let mut synced: HashSet<Uuid> = HashSet::new();
            for project in &projects {
                if !synced.insert(project.project_id) {
                    continue;
                }
                let pool_for_task = pool.clone();
                let work_root_for_task = work_root.clone();
                let project_for_task = project.clone();
                let _ = tokio::task::spawn_blocking(move || {
                    sync_one(&pool_for_task, &project_for_task, &work_root_for_task)
                })
                .await;
            }
        }
    });
}

/// Register the #1504/#1500 hooks on the main binary.
fn register_hooks(pool: DbPool, _state: Arc<AppState>) {
    let ops_pool = pool.clone();
    let ops: vibe_hooks::BotProjectOpsHook =
        Arc::new(move |op, project_id| dispatch_bot_op(&ops_pool, op, project_id));
    vibe_hooks::register_bot_project_ops_hook(ops);
    log::info!("[git_monitor] hooks registered (run-test / deploy-prod)");
}
