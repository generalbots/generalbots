//! Reform #1501/#1505 — persist AutoTask-generated sources into the bot's git
//! repository, the canonical `.gbdialog` of a git-owned bot.
//!
//! AutoTask (`create-and-execute`, tool generation) used to write generated
//! tools straight to the Drive bucket
//! `{bot}.gbai/{bot}.gbdialog/`. Since #1501 the repository is the source of
//! truth: on the next tick `git_bot_monitor` pulls `origin/main`, materializes
//! the checkout over the work layout and compiles it — so a Drive-only write is
//! overwritten, and the task leaves no artifact behind. This module gives
//! AutoTask the same write path the monitor reads from: file into the checkout,
//! commit, push.
//!
//! Layout constraint: `materialize_dialog_dir` copies only the *files directly
//! inside* `.gbdialog/` (non-recursive), therefore every generated source is
//! written at the dialog root.

use std::path::{Path, PathBuf};

use botcore::shared::utils::{get_work_path, DbPool};
use diesel::RunQueryDsl;

use super::core::{
    dialog_source_dir, ensure_checkout_with_heal, materialize_checkout, run_git, MonitoredBot,
};
use super::loop_ops::{list_monitored_bots, mark_for_compile, resolve_branch_id};

/// Committer identity used for AutoTask-authored source commits.
const COMMITTER_NAME: &str = "GB-Dev";
const COMMITTER_EMAIL: &str = "dev@gbo.local";

/// Resolved git target for a bot's generated sources.
pub(crate) struct SourceTarget {
    pub(crate) bot_name: String,
    pub(crate) org: String,
    pub(crate) project: MonitoredBot,
}

/// Resolve `bot_name` to its git-owned project, ALM org and branch.
pub(crate) fn find_source_target(pool: &DbPool, bot_name: &str) -> Result<SourceTarget, String> {
    let project = list_monitored_bots(pool)
        .into_iter()
        .find(|p| p.name == bot_name)
        .ok_or_else(|| {
            format!("bot '{bot_name}' has no git project — sources cannot be committed")
        })?;
    let branch_slug = project.branch_slug.clone().unwrap_or(Default::default());
    if branch_slug.is_empty() {
        return Err(format!(
            "bot '{bot_name}': project {} has no branch — cannot resolve the ALM org",
            project.project_id
        ));
    }
    let branch_id = resolve_branch_id(pool, &branch_slug);
    let org = if branch_id.is_nil() {
        botvibe::bootstrap::alm_org_from_slug(&branch_slug)
    } else {
        botvibe::vm_lifecycle::VmLifecycle::alm_org(branch_id)
    };
    Ok(SourceTarget {
        bot_name: bot_name.to_string(),
        org,
        project,
    })
}

/// Checkout the bot's repository, fast-forwarding to `origin/main` before any
/// edit so the following commit contains only the generated sources.
pub(crate) fn prepare_checkout(pool: &DbPool, target: &SourceTarget) -> Result<PathBuf, String> {
    let checkout = ensure_checkout_with_heal(pool, &target.project, &target.org)?;
    if let Err(e) = run_git(&checkout, &["fetch", "origin", "main"]) {
        log::warn!("[git_monitor] autotask fetch {}: {e}", target.bot_name);
    }
    // Best effort: the push below retries once when this fast-forward fails
    // (dirty tree from an interrupted write, or a foreign local commit).
    if let Err(e) = run_git(&checkout, &["merge", "--ff-only", "origin/main"]) {
        log::debug!("[git_monitor] autotask ff {}: {e}", target.bot_name);
    }
    Ok(checkout)
}

/// Resolve (creating when absent) the checkout's `.gbdialog` source directory.
pub(super) fn dialog_dir(checkout: &Path, bot_name: &str) -> Result<PathBuf, String> {
    if let Some(existing) = dialog_source_dir(checkout, bot_name) {
        return Ok(existing);
    }
    let dir = checkout.join(".gbdialog");
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
    Ok(dir)
}

/// Generated files always land at the dialog root, because only files directly
/// inside `.gbdialog/` are materialized and compiled.
pub(super) fn flatten(relative_path: &str) -> Result<String, String> {
    Path::new(relative_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or_else(|| format!("invalid generated source path: '{relative_path}'"))
}

/// Stage, commit and push the generated sources, then materialize the commit
/// into the work layout (see `materialize_autotask_commit`).
pub(super) fn commit_and_push(
    pool: &DbPool,
    checkout: &Path,
    target: &SourceTarget,
    message: &str,
    written: &[String],
) -> Result<(), String> {
    run_git(checkout, &["add", "-A"])?;
    let staged = run_git(checkout, &["status", "--porcelain"])?;
    if staged.trim().is_empty() {
        log::info!(
            "[git_monitor] autotask {}: sources already current ({})",
            target.bot_name,
            written.join(", ")
        );
        return Ok(());
    }
    run_git(
        checkout,
        &[
            "-c",
            &format!("user.name={COMMITTER_NAME}"),
            "-c",
            &format!("user.email={COMMITTER_EMAIL}"),
            "commit",
            "-m",
            message,
        ],
    )?;
    match run_git(checkout, &["push", "origin", "HEAD:main"]) {
        Ok(_) => {}
        Err(first) => {
            // A monitor tick may have moved `origin/main` between our
            // fast-forward and the push — rebase onto it and retry once.
            log::warn!(
                "[git_monitor] autotask push {} failed ({first}) — retrying after rebase",
                target.bot_name
            );
            run_git(checkout, &["fetch", "origin", "main"])?;
            if let Err(e) = run_git(checkout, &["rebase", "origin/main"]) {
                let _ = run_git(checkout, &["rebase", "--abort"]);
                return Err(format!("rebase onto origin/main failed: {e}"));
            }
            run_git(checkout, &["push", "origin", "HEAD:main"])?;
        }
    }
    log::info!(
        "[git_monitor] autotask {}: committed + pushed {}",
        target.bot_name,
        written.join(", ")
    );
    materialize_autotask_commit(pool, checkout, target);
    Ok(())
}

/// Materialize the just-pushed commit into the work layout and queue the
/// compile. The pull monitor only reacts to a *remote* transition
/// (local != origin/main), which an in-place commit never creates — HEAD
/// already equals origin/main after the push — so without this the generated
/// tool would sit in the repository until some unrelated pull happened.
fn materialize_autotask_commit(pool: &DbPool, checkout: &Path, target: &SourceTarget) {
    let Some(branch_slug) = target
        .project
        .branch_slug
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return;
    };
    let branch_id = resolve_branch_id(pool, branch_slug);
    let work_root = PathBuf::from(get_work_path());
    let head = match run_git(checkout, &["rev-parse", "HEAD"]) {
        Ok(head) => head.trim().to_string(),
        Err(e) => {
            log::warn!(
                "[git_monitor] autotask {}: rev-parse HEAD: {e}",
                target.bot_name
            );
            return;
        }
    };
    match materialize_checkout(checkout, &work_root, branch_slug, &target.project.name) {
        Ok(paths) => mark_for_compile(pool, branch_id, &paths, &head),
        Err(e) => log::error!(
            "[git_monitor] autotask materialize {}: {e}",
            target.bot_name
        ),
    }
}

/// Write generated bot sources into `bot_name`'s repository and push them to
/// ALM, so the next monitor tick compiles them. `files` are
/// `(path relative to .gbdialog, content)` pairs; the returned paths are the
/// dialog-root file names actually written.
pub(crate) fn write_sources(
    pool: &DbPool,
    bot_name: &str,
    files: &[(String, String)],
    message: &str,
) -> Result<Vec<String>, String> {
    if files.is_empty() {
        return Ok(Vec::new());
    }
    let target = find_source_target(pool, bot_name)?;
    let checkout = prepare_checkout(pool, &target)?;
    let dialog = dialog_dir(&checkout, bot_name)?;
    let mut written = Vec::new();
    for (relative_path, body) in files {
        let name = flatten(relative_path)?;
        let path = dialog.join(&name);
        std::fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))?;
        written.push(name);
    }
    commit_and_push(pool, &checkout, &target, message, &written)?;
    Ok(written)
}

mod io;
mod tables;

pub(crate) use io::{list_sources, read_source, GitBotSourceOps};
pub(crate) use tables::merge_tables;

#[cfg(test)]
mod tests;
