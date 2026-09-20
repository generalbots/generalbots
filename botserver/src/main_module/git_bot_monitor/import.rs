//! Reform #1501 — one-shot import of bot sources from Drive into git.
//!
//! For every bot-kind Vibe project of a branch, list the bot's
//! `{branch}.gbai/{bot}.gbdialog/*` (and `.gbot`) objects in MinIO, copy them
//! into the project's git workspace, and commit+push `main`. Idempotent via
//! `payload.source_imported_at`. Drive keeps `.gbkb`/`.gbdrive` — only bot
//! **sources** move to git.

use std::path::Path;
use std::sync::Arc;

use botcore::shared::utils::DbPool;
use diesel::prelude::*;
use uuid::Uuid;

use super::core::{alm_repo, run_git};

#[derive(Debug, Clone)]
struct ImportTarget {
    project_id: Uuid,
    name: String,
    repo_slug: String,
    branch_slug: String,
}

fn list_import_targets(pool: &DbPool) -> Vec<ImportTarget> {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        branch_slug: String,
    }
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            log::error!("[git_import] pool: {e}");
            return Vec::new();
        }
    };
    diesel::sql_query(
        "SELECT vp.id, vp.name, br.slug AS branch_slug \
         FROM vibe_projects vp \
         JOIN branches br ON br.id = vp.branch_id \
         WHERE vp.project_type = 'bot' AND vp.source_control = 'git' \
           AND (vp.payload->>'source_imported_at') IS NULL",
    )
    .load::<Row>(&mut conn)
    .unwrap_or_default()
    .into_iter()
    .map(|row| ImportTarget {
        repo_slug: alm_repo(&row.name),
        project_id: row.id,
        branch_slug: row.branch_slug,
        name: row.name,
    })
    .collect()
}

fn mark_imported(pool: &DbPool, project_id: Uuid, files: usize) {
    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("[git_import] mark pool: {e}");
            return;
        }
    };
    let patch = serde_json::json!({
        "source_imported_at": chrono::Utc::now().to_rfc3339(),
        "source_imported_files": files as i64,
    });
    let _ = diesel::sql_query(
        "UPDATE vibe_projects SET payload = payload || $2::jsonb, updated_at = NOW() WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(project_id)
    .bind::<diesel::sql_types::Jsonb, _>(&patch)
    .execute(&mut conn);
}

/// Copy the Drive objects of one bot into the workspace's `.gbdialog/` dir.
async fn import_dialog_from_drive(
    state: &Arc<botcore::shared::state::AppState>,
    bucket: &str,
    prefix: &str,
    bot_name: &str,
    dialog_dir: &Path,
) -> Result<usize, String> {
    let s3 = state
        .drive
        .as_ref()
        .ok_or_else(|| "drive (S3) not available in AppState".to_string())?;
    let bot_prefix = format!("{prefix}{bot_name}.gbdialog/");
    let objects = match s3.list_objects(bucket, Some(&bot_prefix)).await {
        Ok(o) => o,
        // A branch with no Drive bucket (fresh/demo) has zero sources — not
        // an error, otherwise the import warn-loops on every boot.
        Err(e) if e.to_string().to_lowercase().contains("nosuchbucket")
            || e.to_string().to_lowercase().contains("no such bucket")
            || e.to_string().to_lowercase().contains("404") =>
        {
            log::info!(
                "[git_import] {bot_name}: bucket {bucket} absent on Drive — treating as fresh branch"
            );
            Vec::new()
        }
        Err(e) => {
            // botserver's log layer caps line width — emit the error in short
            // chunks so the journal always carries the full cause.
            let text = format!("list {bot_prefix}: {e}");
            for (i, chunk) in text.as_bytes().chunks(80).enumerate() {
                log::warn!(
                    "[git_import] {} list-err part{}: {}",
                    bot_name,
                    i,
                    String::from_utf8_lossy(chunk)
                );
            }
            return Err(text);
        }
    };
    std::fs::create_dir_all(dialog_dir)
        .map_err(|e| format!("mkdir {}: {e}", dialog_dir.display()))?;
    let mut copied = 0usize;
    for key in objects {
        let file_name = match key.rsplit('/').next() {
            Some(n) if !n.is_empty() => n.to_string(),
            _ => continue,
        };
        if !file_name.ends_with(".bas")
            && !file_name.ends_with(".ast")
            && !file_name.ends_with(".json")
        {
            continue;
        }
        let content = s3
            .get_object(bucket, &key)
            .await
            .map_err(|e| format!("get {key}: {e}"))?;
        std::fs::write(dialog_dir.join(&file_name), content)
            .map_err(|e| format!("write {file_name}: {e}"))?;
        copied += 1;
    }
    Ok(copied)
}

fn commit_and_push(workspace: &Path, message: &str) -> Result<(), String> {
    run_git(workspace, &["add", "-A"])?;
    match run_git(
        workspace,
        &[
            "-c", "user.name=Vibe Agent",
            "-c", "user.email=vibe@gbo.local",
            "commit", "-m", message,
        ],
    ) {
        Ok(_) => {}
        Err(e) if e.contains("nothing to commit") => return Ok(()),
        Err(e) => return Err(e),
    }
    match run_git(workspace, &["push", "-u", "origin", "main"]) {
        Ok(_) => Ok(()),
        Err(_) => run_git(workspace, &["push", "--force", "-u", "origin", "main"])
            .map(|_| ())
            .map_err(|e| format!("push: {e}")),
    }
}

async fn import_one(
    state: &Arc<botcore::shared::state::AppState>,
    pool: &DbPool,
    target: &ImportTarget,
) -> Result<usize, String> {
    // Bucket layout 2 (org workspace): `{org}.gborg/{branch}.gbai/...`.
    // Layout 1 (standalone): `{branch}.gbai`. Probe the org bucket first.
    let org_bucket = format!("{}.gborg", target.branch_slug);
    let branch_prefix = format!("{}.gbai/", target.branch_slug);
    let (bucket, prefix) = match state.drive.as_ref() {
        Some(s3) if s3.list_objects(&org_bucket, Some(&branch_prefix)).await.is_ok() => {
            (org_bucket, branch_prefix)
        }
        _ => (format!("{}.gbai", target.branch_slug), String::new()),
    };

    let workspace = botvibe::harness::workspace_root().join(&target.repo_slug);
    std::fs::create_dir_all(&workspace)
        .map_err(|e| format!("mkdir {}: {e}", workspace.display()))?;
    if !workspace.join(".git").exists() {
        // Repo not provisioned yet — provisioning creates it; the import is
        // retried on the next boot (only unmarked projects are touched).
        return Err(format!(
            "workspace {} has no git checkout yet — provisioning pending",
            workspace.display()
        ));
    }
    let dialog_dir = workspace.join(".gbdialog");
    let copied =
        import_dialog_from_drive(state, &bucket, &prefix, &target.name, &dialog_dir).await?;
    if copied == 0 {
        mark_imported(pool, target.project_id, 0);
        log::info!(
            "[git_import] {}: no Drive sources found (fresh branch) — marked imported",
            target.name
        );
        return Ok(0);
    }
    commit_and_push(
        &workspace,
        &format!("Import bot sources from Drive ({})", target.name),
    )?;
    mark_imported(pool, target.project_id, copied);
    log::info!(
        "[git_import] {}: {copied} file(s) imported from {bucket} → git and pushed",
        target.name
    );
    Ok(copied)
}

/// Import pass: run once at boot after the bootstrap backfill. Safe to
/// re-run — only projects without `payload.source_imported_at` are touched.
pub async fn run_import_pass(state: Arc<botcore::shared::state::AppState>, pool: DbPool) {
    let targets = {
        let pool = pool.clone();
        match tokio::task::spawn_blocking(move || list_import_targets(&pool)).await {
            Ok(t) => t,
            Err(e) => {
                log::error!("[git_import] target listing failed: {e}");
                return;
            }
        }
    };
    if targets.is_empty() {
        return;
    }
    log::info!(
        "[git_import] {} bot project(s) pending Drive import",
        targets.len()
    );
    for target in targets {
        if let Err(e) = import_one(&state, &pool, &target).await {
            log::warn!("[git_import] project {} import deferred: {e}", target.name);
        }
    }
}
