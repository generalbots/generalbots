//! Reform #1505 — read, edit and LLM-rephrase an AutoTask-produced `.bas`.
//!
//! An AutoTask item leaves a `.bas` in the bot's repository (the canonical
//! `.gbdialog`). These endpoints let the suite editor open that file, save an
//! edited revision, or ask the LLM to rephrase it while preserving the
//! declared MCP manifest contract. Every write goes through
//! [`crate::types::BotSourceOps`], so it is committed and pushed to ALM and the
//! pull monitor compiles it.

use std::sync::Arc;

use axum::{extract::Query, extract::State, Json};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::AutoTaskApi;
use crate::handlers::{canonical_bot_id, err_msg};

/// Query naming only a bot (listing endpoints).
#[derive(Debug, Deserialize)]
pub struct BotQuery {
    pub bot_id: Option<String>,
}

/// Query for reading a source file.
#[derive(Debug, Deserialize)]
pub struct SourceQuery {
    pub bot_id: Option<String>,
    /// File name inside `.gbdialog` (e.g. `classify_media.bas`).
    pub name: String,
}

/// Body for saving an edited revision.
#[derive(Debug, Deserialize)]
pub struct UpdateSourceRequest {
    pub bot_id: Option<String>,
    pub name: String,
    pub content: String,
    /// Commit message override (defaults to a generic edit message).
    pub message: Option<String>,
}

/// Body for the LLM rephrase flow.
#[derive(Debug, Deserialize)]
pub struct RephraseSourceRequest {
    pub bot_id: Option<String>,
    pub name: String,
    /// What the user wants changed (e.g. "file receipts under finance/").
    pub instruction: String,
    /// When true the rewritten source is returned without being committed.
    pub dry_run: Option<bool>,
}

/// Body for attaching a database schema to a task.
#[derive(Debug, Deserialize)]
pub struct AttachTablesRequest {
    pub bot_id: Option<String>,
    /// Raw BASIC `BEGIN TABLE … END TABLE` blocks the task needs.
    pub tables: String,
}

/// Response shared by the three endpoints.
#[derive(Debug, Serialize)]
pub struct SourceResponse {
    pub success: bool,
    pub name: String,
    pub content: Option<String>,
    pub committed: bool,
    pub message: String,
    pub error: Option<String>,
}

fn fail(name: &str, message: &str, error: &str) -> Json<SourceResponse> {
    Json(SourceResponse {
        success: false,
        name: name.to_string(),
        content: None,
        committed: false,
        message: message.to_string(),
        error: Some(error.to_string()),
    })
}

/// `GET /api/autotask/source` — the committed content of a bot source file.
pub async fn get_source(
    State(api): State<Arc<AutoTaskApi>>,
    Query(query): Query<SourceQuery>,
) -> Json<SourceResponse> {
    let bot_id = canonical_bot_id(query.bot_id.clone());
    let sources = match api.state().source_ops() {
        Some(sources) => sources,
        None => return fail(&query.name, "No source repository for this bot", "git sources unavailable"),
    };
    match sources.read_source(bot_id, &query.name) {
        Ok(Some(content)) => Json(SourceResponse {
            success: true,
            name: query.name,
            content: Some(content),
            committed: true,
            message: "Source loaded from the bot repository".to_string(),
            error: None,
        }),
        Ok(None) => fail(&query.name, "Source file not found", "not found"),
        Err(e) => {
            warn!("[autotask] read source {} failed: {e}", query.name);
            let error = err_msg("read_source", &*e);
            fail(&query.name, "Could not read the source file", &error)
        }
    }
}

/// `PUT /api/autotask/source` — commit an edited revision.
pub async fn put_source(
    State(api): State<Arc<AutoTaskApi>>,
    Json(req): Json<UpdateSourceRequest>,
) -> Json<SourceResponse> {
    let bot_id = canonical_bot_id(req.bot_id.clone());
    write_source(&api, bot_id, &req.name, &req.content, req.message.as_deref(), "edit")
}

/// `POST /api/autotask/source/rephrase` — let the LLM rewrite the source.
pub async fn rephrase_source(
    State(api): State<Arc<AutoTaskApi>>,
    Json(req): Json<RephraseSourceRequest>,
) -> Json<SourceResponse> {
    let bot_id = canonical_bot_id(req.bot_id.clone());
    let sources = match api.state().source_ops() {
        Some(sources) => sources,
        None => return fail(&req.name, "No source repository for this bot", "git sources unavailable"),
    };
    let current = match sources.read_source(bot_id, &req.name) {
        Ok(Some(content)) => content,
        Ok(None) => return fail(&req.name, "Source file not found", "not found"),
        Err(e) => {
            warn!("[autotask] rephrase read {} failed: {e}", req.name);
            let error = err_msg("rephrase_read", &*e);
            return fail(&req.name, "Could not read the source file", &error);
        }
    };
    let rewritten = match call_llm(&api, bot_id, &rephrase_prompt(&req.name, &current, &req.instruction)).await {
        Ok(text) => strip_code_fence(&text),
        Err(e) => return fail(&req.name, "The model could not rewrite the automation", &e),
    };
    if rewritten.trim().is_empty() {
        return fail(&req.name, "The model returned an empty program", "empty completion");
    }
    if req.dry_run.unwrap_or(false) {
        return Json(SourceResponse {
            success: true,
            name: req.name,
            content: Some(rewritten),
            committed: false,
            message: "Rephrased source (not committed)".to_string(),
            error: None,
        });
    }
    write_source(&api, bot_id, &req.name, &rewritten, None, "rephrase")
}

/// Response for the source listing.
#[derive(Debug, Serialize)]
pub struct SourceListResponse {
    pub success: bool,
    pub project_id: Option<Uuid>,
    pub files: Vec<crate::types::SourceFile>,
    pub message: String,
    pub error: Option<String>,
}

/// `GET /api/autotask/sources` — the bot's `.gbdialog` sources and the project
/// that owns the repository, so a row action can open one in the editor.
pub async fn list_sources(
    State(api): State<Arc<AutoTaskApi>>,
    Query(query): Query<BotQuery>,
) -> Json<SourceListResponse> {
    let bot_id = canonical_bot_id(query.bot_id.clone());
    let fail = |message: &str, error: String| {
        Json(SourceListResponse {
            success: false,
            project_id: None,
            files: Vec::new(),
            message: message.to_string(),
            error: Some(error),
        })
    };
    let sources = match api.state().source_ops() {
        Some(sources) => sources,
        None => return fail("No source repository for this bot", "git sources unavailable".to_string()),
    };
    match sources.list_sources(bot_id) {
        Ok(listing) => Json(SourceListResponse {
            success: true,
            project_id: listing.project_id,
            message: format!("{} source file(s)", listing.files.len()),
            files: listing.files,
            error: None,
        }),
        Err(e) => {
            warn!("[autotask] list sources failed: {e}");
            let error = err_msg("list_sources", &*e);
            fail("Could not list the bot sources", error)
        }
    }
}

/// `POST /api/autotask/tables` — attach `BEGIN TABLE … END TABLE` blocks to the
/// bot's `.gbdialog/tables.bas`, so an automation that needs storage declares
/// its schema instead of embedding DDL in the tool script.
pub async fn attach_tables(
    State(api): State<Arc<AutoTaskApi>>,
    Json(req): Json<AttachTablesRequest>,
) -> Json<SourceResponse> {
    const FILE: &str = "tables.bas";
    let bot_id = canonical_bot_id(req.bot_id.clone());
    let sources = match api.state().source_ops() {
        Some(sources) => sources,
        None => return fail(FILE, "No source repository for this bot", "git sources unavailable"),
    };
    let attached = match sources.merge_tables(bot_id, &req.tables) {
        Ok(names) => names,
        Err(e) => {
            warn!("[autotask] attach tables failed: {e}");
            let error = err_msg("attach_tables", &*e);
            return fail(FILE, "Could not attach the schema", &error);
        }
    };
    let content = sources.read_source(bot_id, FILE).ok().flatten();
    if attached.is_empty() {
        return Json(SourceResponse {
            success: true,
            name: FILE.to_string(),
            content,
            committed: false,
            message: "Schema already declared every table".to_string(),
            error: None,
        });
    }
    info!("[autotask] attached tables to {FILE}: {attached:?}");
    Json(SourceResponse {
        success: true,
        name: FILE.to_string(),
        content,
        committed: true,
        message: format!("Attached to {FILE}: {}", attached.join(", ")),
        error: None,
    })
}

/// Commit `content` as `name` through the bot's source repository.
fn write_source(
    api: &Arc<AutoTaskApi>,
    bot_id: Uuid,
    name: &str,
    content: &str,
    message: Option<&str>,
    verb: &str,
) -> Json<SourceResponse> {
    let sources = match api.state().source_ops() {
        Some(sources) => sources,
        None => return fail(name, "No source repository for this bot", "git sources unavailable"),
    };
    let commit_message = message
        .map(|m| m.to_string())
        .unwrap_or_else(|| format!("autotask: {verb} {name}"));
    let files = vec![(name.to_string(), content.to_string())];
    match sources.write_sources(bot_id, &files, &commit_message) {
        Ok(written) => {
            info!("[autotask] committed source {name} ({written:?})");
            Json(SourceResponse {
                success: true,
                name: name.to_string(),
                content: Some(content.to_string()),
                committed: true,
                message: format!("Committed to the bot repository: {name}"),
                error: None,
            })
        }
        Err(e) => {
            warn!("[autotask] commit source {name} failed: {e}");
            let error = err_msg("write_source", &*e);
            fail(name, "Could not commit the source file", &error)
        }
    }
}

/// Instruction mixing the current program, the requested change and the
/// response contract (program only, no prose, no code fences).
fn rephrase_prompt(name: &str, current: &str, instruction: &str) -> String {
    format!(
        "You maintain a General Bots automation written in the BASIC dialect.\n\
         The file is '{name}' inside the bot's .gbdialog directory.\n\n\
         Requested change:\n{instruction}\n\n\
         Current program:\n```basic\n{current}\n```\n\n\
         Rewrite the whole program applying the change. Keep every declared\n\
         keyword, tool name and output channel working exactly as before unless\n\
         the change explicitly requires otherwise. Reply with the program only —\n\
         no explanation, no markdown fences."
    )
}

/// Run a one-shot completion against the bot's configured model.
async fn call_llm(api: &Arc<AutoTaskApi>, bot_id: Uuid, prompt: &str) -> Result<String, String> {
    let model = api
        .config_ops()
        .get_config(&bot_id, "llm-model", None)
        .unwrap_or_else(|_| "gpt-4".to_string());
    let key = api
        .config_ops()
        .get_config(&bot_id, "llm-key", None)
        .unwrap_or_default();
    let config = serde_json::json!({ "temperature": 0.2, "max_tokens": 4000 });
    crate::types::collect_llm_stream(api.llm_ops().as_ref(), prompt, &config, &model, &key, None)
        .await
        .map_err(|e| format!("llm call failed: {e}"))
}

/// Drop a surrounding markdown code fence from a completion.
fn strip_code_fence(text: &str) -> String {
    let trimmed = text.trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }
    let without_open = match trimmed.find('\n') {
        Some(idx) => &trimmed[idx + 1..],
        None => return String::new(),
    };
    match without_open.rfind("```") {
        Some(idx) => without_open[..idx].trim_end().to_string(),
        None => without_open.trim_end().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::strip_code_fence;

    #[test]
    fn strips_fenced_completion() {
        let out = strip_code_fence("```basic\nTALK \"hi\"\n```");
        assert_eq!(out, "TALK \"hi\"");
    }

    #[test]
    fn keeps_unfenced_completion() {
        assert_eq!(strip_code_fence("TALK \"hi\""), "TALK \"hi\"");
    }
}
