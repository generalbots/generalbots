//! Reform #1505 — AutoTask source persistence into the bot's git repository.
//!
//! The create/edit paths used to upload generated BASIC straight to the Drive
//! bucket `{bot}.gbai/{bot}.gbdialog/`. For a git-owned bot (#1501) the
//! repository is the canonical `.gbdialog`, so the next `git_bot_monitor` pull
//! overwrote that object and the task left no artifact behind. This module
//! routes a generated tool (and its MCP manifest) into the repository, commits
//! it and pushes it to ALM, and attaches any `BEGIN TABLE … END TABLE` blocks
//! the generation declared to the bot's `.gbdialog/tables.bas`.

use uuid::Uuid;

use crate::tables::split_table_blocks;
use crate::types::{AutoTaskState, BotInfo, BotSourceOps};

/// Drive-layout description of a git source write, used to build API responses.
pub struct PersistedSources {
    pub bucket: String,
    pub tool_key: String,
    pub manifest_key: Option<String>,
    /// Table names appended to `tables.bas` (empty when none were needed).
    pub tables: Vec<String>,
}

/// One generated source set to commit.
struct WritePlan {
    /// `(path relative to .gbdialog, content)` — tool first, then manifests.
    files: Vec<(String, String)>,
    /// Bare manifest file name for the response, when one was written.
    manifest_name: Option<String>,
    /// Requested tool path, used when the write reports no file back.
    requested_tool_path: String,
    /// `tables.bas` material split out of the generated tool.
    tables: String,
    commit_message: String,
}

/// Commit generated sources into the bot's repository (ALM).
///
/// Returns `None` when the bot has no git project — the caller then keeps the
/// legacy Drive path. `Some(Err(..))` means the git write itself failed, so the
/// caller surfaces the reason instead of silently writing to a source of truth
/// the monitor will overwrite.
pub fn persist_to_git(
    state: &dyn AutoTaskState,
    info: &BotInfo,
    bot_id: Uuid,
    tool_relative_path: &str,
    tool_source: &str,
    manifest: Option<(&str, &str)>,
    commit_message: &str,
) -> Option<Result<PersistedSources, String>> {
    let sources = state.source_ops()?;
    let (tool, tables) = split_table_blocks(tool_source);
    let mut files: Vec<(String, String)> = vec![(tool_relative_path.to_string(), tool)];
    if let Some((rel, src)) = manifest {
        files.push((rel.to_string(), src.to_string()));
        // Materialization is non-recursive and the compiler regenerates an
        // empty-schema manifest on every `.bas` compile, so the manifest is
        // also written under its bare file name at the dialog root.
        let root = rel.trim_start_matches("tools/");
        if root != rel {
            files.push((root.to_string(), src.to_string()));
        }
    }
    let plan = WritePlan {
        files,
        manifest_name: manifest.map(|(rel, _)| rel.trim_start_matches("tools/").to_string()),
        requested_tool_path: tool_relative_path.to_string(),
        tables,
        commit_message: commit_message.to_string(),
    };
    Some(persist_impl(sources, info, bot_id, &plan))
}

/// Commit a shipped template's `.gbot` configuration (channel prompts, styles)
/// into the bot's repository.
///
/// Returns `None` when the bot has no git project — the Drive fallback writes
/// nothing here, and `load_system_prompt_for_channel` then keeps using the
/// generic prompt, which is the pre-#1505 behaviour for those bots. On success
/// the returned names are the drive-layout keys the monitor materializes them
/// to, so the API response can report where the channel prompts landed.
pub fn persist_config_to_git(
    state: &dyn AutoTaskState,
    info: &BotInfo,
    bot_id: Uuid,
    config_files: &[(&str, &str)],
    commit_message: &str,
) -> Option<Result<Vec<String>, String>> {
    if config_files.is_empty() {
        return Some(Ok(Vec::new()));
    }
    let sources = state.source_ops()?;
    let files: Vec<(String, String)> = config_files
        .iter()
        .map(|(name, body)| (name.to_string(), body.to_string()))
        .collect();
    match sources.write_bot_config(bot_id, &files, commit_message) {
        Ok(written) => Some(
            Ok(written
                .into_iter()
                .map(|name| format!("{}/{}.gbot/{name}", info.bucket_name(), info.name))
                .collect()),
        ),
        Err(e) => Some(Err(format!("git config write failed: {e}"))),
    }
}

fn persist_impl(
    sources: &dyn BotSourceOps,
    info: &BotInfo,
    bot_id: Uuid,
    plan: &WritePlan,
) -> Result<PersistedSources, String> {
    let written = sources
        .write_sources(bot_id, &plan.files, &plan.commit_message)
        .map_err(|e| format!("git source write failed: {e}"))?;
    let tool_name = written
        .first()
        .cloned()
        .unwrap_or_else(|| plan.requested_tool_path.clone());
    let dialog = info.dialog_folder();
    let mut attached = Vec::new();
    if !plan.tables.trim().is_empty() {
        match sources.merge_tables(bot_id, &plan.tables) {
            Ok(names) => attached = names,
            Err(e) => log::warn!("[autotask] tables.bas merge failed: {e}"),
        }
    }
    Ok(PersistedSources {
        bucket: info.bucket_name(),
        tool_key: format!("{dialog}/{tool_name}"),
        manifest_key: plan
            .manifest_name
            .clone()
            .map(|name| format!("{dialog}/{name}")),
        tables: attached,
    })
}
