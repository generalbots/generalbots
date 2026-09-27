//! Reform #1501/#1505 — persist AutoTask-generated sources into the bot's git
//! repository, the canonical `.gbdialog` of a git-owned bot.
//!
//! AutoTask (`create-and-execute`, the shipped-template fast path, tool
//! generation) used to write generated tools straight to the Drive bucket
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

use botcore::shared::utils::DbPool;
use diesel::RunQueryDsl;

use super::core::{dialog_source_dir, ensure_checkout_with_heal, run_git, MonitoredBot};
use super::loop_ops::{list_monitored_bots, resolve_branch_id};

/// Committer identity used for AutoTask-authored source commits.
const COMMITTER_NAME: &str = "GB-Dev";
const COMMITTER_EMAIL: &str = "dev@gbo.local";

/// Resolved git target for a bot's generated sources.
struct SourceTarget {
    bot_name: String,
    org: String,
    project: MonitoredBot,
}

/// Resolve `bot_name` to its git-owned project, ALM org and branch.
fn find_source_target(pool: &DbPool, bot_name: &str) -> Result<SourceTarget, String> {
    let project = list_monitored_bots(pool)
        .into_iter()
        .find(|p| p.name == bot_name)
        .ok_or_else(|| format!("bot '{bot_name}' has no git project — sources cannot be committed"))?;
    let branch_slug = project.branch_slug.clone().unwrap_or_default();
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
fn prepare_checkout(pool: &DbPool, target: &SourceTarget) -> Result<PathBuf, String> {
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
fn dialog_dir(checkout: &Path, bot_name: &str) -> Result<PathBuf, String> {
    if let Some(existing) = dialog_source_dir(checkout, bot_name) {
        return Ok(existing);
    }
    let dir = checkout.join(".gbdialog");
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
    Ok(dir)
}

/// Generated files always land at the dialog root, because only files directly
/// inside `.gbdialog/` are materialized and compiled.
fn flatten(relative_path: &str) -> Result<String, String> {
    Path::new(relative_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .ok_or_else(|| format!("invalid generated source path: '{relative_path}'"))
}

/// Stage, commit and push the generated sources.
fn commit_and_push(
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
    Ok(())
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
    commit_and_push(&checkout, &target, message, &written)?;
    Ok(written)
}

/// Split a `tables.bas` body into `(table_name, block)` pairs.
fn split_table_blocks(tables_bas: &str) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let mut current: Option<(String, Vec<&str>)> = None;
    for line in tables_bas.lines() {
        let trimmed = line.trim();
        let upper = trimmed.to_uppercase();
        if upper.starts_with("BEGIN TABLE ") {
            let name = trimmed[12..].trim().to_string();
            if !name.is_empty() {
                current = Some((name, vec![line]));
            }
            continue;
        }
        if upper.starts_with("END TABLE") {
            if let Some((name, mut lines)) = current.take() {
                lines.push(line);
                blocks.push((name, lines.join("\n")));
            }
            continue;
        }
        if let Some((_, lines)) = current.as_mut() {
            lines.push(line);
        }
    }
    blocks
}

/// Merge the `BEGIN TABLE … END TABLE` blocks of `tables_bas` into the bot's
/// `.gbdialog/tables.bas`, appending only tables that are not declared yet.
/// Returns the table names appended (empty when the schema already covered
/// them), so the caller can report what changed.
pub(crate) fn merge_tables(
    pool: &DbPool,
    bot_name: &str,
    tables_bas: &str,
) -> Result<Vec<String>, String> {
    let blocks = split_table_blocks(tables_bas);
    if blocks.is_empty() {
        return Ok(Vec::new());
    }
    let target = find_source_target(pool, bot_name)?;
    let checkout = prepare_checkout(pool, &target)?;
    let dialog = dialog_dir(&checkout, bot_name)?;
    let path = dialog.join("tables.bas");
    let existing = match std::fs::read_to_string(&path) {
        Ok(body) => body,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("read {}: {e}", path.display())),
    };
    let mut body = existing.clone();
    let mut appended = Vec::new();
    for (name, block) in blocks {
        if existing.contains(&format!("BEGIN TABLE {name}")) {
            continue;
        }
        if !body.is_empty() && !body.ends_with('\n') {
            body.push('\n');
        }
        body.push_str(&block);
        if !body.ends_with('\n') {
            body.push('\n');
        }
        appended.push(name);
    }
    if appended.is_empty() {
        return Ok(Vec::new());
    }
    std::fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))?;
    commit_and_push(
        &checkout,
        &target,
        &format!("autotask: attach tables ({})", appended.join(", ")),
        &["tables.bas".to_string()],
    )?;
    Ok(appended)
}

/// Read a source file from the bot's repository. `None` when the file (or the
/// `.gbdialog` directory) does not exist yet.
pub(crate) fn read_source(
    pool: &DbPool,
    bot_name: &str,
    name: &str,
) -> Result<Option<String>, String> {
    let target = find_source_target(pool, bot_name)?;
    let checkout = ensure_checkout_with_heal(pool, &target.project, &target.org)?;
    let dialog = match dialog_source_dir(&checkout, bot_name) {
        Some(dir) => dir,
        None => return Ok(None),
    };
    let file = flatten(name)?;
    match std::fs::read_to_string(dialog.join(&file)) {
        Ok(body) => Ok(Some(body)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("read {file}: {e}")),
    }
}

/// Bot name for a bot id, used to resolve the git target from AutoTask.
fn bot_name_for_id(pool: &DbPool, bot_id: uuid::Uuid) -> Result<String, String> {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    let mut conn = pool.get().map_err(|e| format!("pool: {e}"))?;
    diesel::sql_query("SELECT name FROM bots WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .get_result::<Row>(&mut conn)
        .map(|r| r.name)
        .map_err(|e| format!("bot {bot_id}: {e}"))
}

/// `botautotask::types::BotSourceOps` adapter over this module, so the AutoTask
/// crate can persist generated sources without depending on botserver.
pub struct GitBotSourceOps {
    pool: DbPool,
}

impl GitBotSourceOps {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl botautotask::types::BotSourceOps for GitBotSourceOps {
    fn write_sources(
        &self,
        bot_id: uuid::Uuid,
        files: &[(String, String)],
        message: &str,
    ) -> Result<Vec<String>, botautotask::types::BoxError> {
        let name = bot_name_for_id(&self.pool, bot_id)?;
        write_sources(&self.pool, &name, files, message).map_err(Into::into)
    }

    fn merge_tables(
        &self,
        bot_id: uuid::Uuid,
        tables_bas: &str,
    ) -> Result<Vec<String>, botautotask::types::BoxError> {
        let name = bot_name_for_id(&self.pool, bot_id)?;
        merge_tables(&self.pool, &name, tables_bas).map_err(Into::into)
    }

    fn read_source(
        &self,
        bot_id: uuid::Uuid,
        name: &str,
    ) -> Result<Option<String>, botautotask::types::BoxError> {
        let bot_name = bot_name_for_id(&self.pool, bot_id)?;
        read_source(&self.pool, &bot_name, name).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::{flatten, split_table_blocks};

    #[test]
    fn flatten_strips_generated_subdirectories() {
        assert_eq!(flatten("tools/classify_media.bas").unwrap_or_default(), "classify_media.bas");
        assert_eq!(flatten("classify_media.bas").unwrap_or_default(), "classify_media.bas");
    }

    #[test]
    fn split_table_blocks_reads_named_blocks() {
        let src = "BEGIN TABLE orders\n    id UUID PRIMARY KEY\nEND TABLE\n";
        let blocks = split_table_blocks(src);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].0, "orders");
        assert!(blocks[0].1.contains("END TABLE"));
    }

    #[test]
    fn split_table_blocks_ignores_prose() {
        assert!(split_table_blocks("' just a comment\n").is_empty());
    }
}
