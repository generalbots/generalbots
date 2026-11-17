//! Drive ↔ workspace native tools (#1584).
//!
//! `drive_fetch` copies a Drive object into the conversation's `data/` before
//! the program runs; `drive_save` publishes `output/` into
//! `{bot}.gbdrive/conversations/{title}/`, keyed by the conversation title so
//! saved files outlive the session folder.
//!
//! The runtime also pre-fetches file names mentioned in the message
//! ([`fetch_referenced`]) so "summarise sales.csv from my Drive" works without
//! an explicit tool call.

use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

use super::native_tools::{persist, send};
use super::types::SessionIdentity;
use crate::core::bot::{agent_vm_hook, drive_bridge};

const TEXT_EXTENSIONS: &[&str] = &[
    "csv", "tsv", "xlsx", "xls", "docx", "doc", "pdf", "txt", "md", "json", "pptx", "ppt", "png",
    "jpg", "jpeg", "gif", "webp", "svg", "zip", "py", "js", "html", "xml", "yaml", "yml", "log",
    "sql",
];

pub async fn try_run(identity: &SessionIdentity<'_>, name: &str, args: &str) -> bool {
    match name {
        "drive_fetch" => {
            handlers::run_drive_fetch(identity, args).await;
            true
        }
        "drive_save" => {
            handlers::run_drive_save(identity, args).await;
            true
        }
        "drive_list" => {
            handlers::run_drive_list(identity, args).await;
            true
        }
        // #2047 — same capability the UI button uses, reachable from chat.
        "save_to_external_drive" => {
            super::native_external_save::run(identity, args).await;
            true
        }
        _ => false,
    }
}

fn parse(args: &str) -> serde_json::Value {
    super::native_args::parse_args(args).unwrap_or_else(|_| serde_json::json!({}))
}

fn sanitize_name(raw: &str) -> String {
    let base = raw.rsplit('/').next().unwrap_or(raw).trim();
    let clean: String = base
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ' '))
        .collect();
    if clean.trim().is_empty() {
        "file".to_string()
    } else {
        clean
    }
}

fn human_size(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

/// Splits a [`drive_bridge::save_object`] result (`bucket/key`) into bucket and
/// bucket-relative key.
///
/// S3 bucket names never contain `/`, so the first segment is always the
/// bucket and the remainder is the key the Drive app addresses.
fn split_bucket_path(bucket_path: &str) -> (&str, &str) {
    bucket_path
        .split_once('/')
        .unwrap_or((bucket_path, bucket_path))
}

/// Bucket-relative key of a [`drive_bridge::save_object`] result (`bucket/key`).
fn relative_key(bucket_path: &str) -> &str {
    split_bucket_path(bucket_path).1
}

/// Markdown link the chat renderer turns into an "Open in Drive" button.
///
/// #2143 gap A — `drive:` is not a scheme any client resolves, so every saved
/// artifact was a dead link. The two forms the renderer understands are
/// `vmfile:` (workspace files only, `chat/modules/vm_attachments/02_links.js`)
/// and `app://` (`chat/chat-render.js`). Drive files therefore use the
/// registered deep link `app://drive?path={path}`
/// (`apps/commands/deep_link_params_for_app.rs`), where `path` is the
/// bucket-relative key, percent-encoded. `scope=bot` makes the Drive app open
/// the bot workspace `drive_save` publishes into instead of the caller's
/// personal bucket (`drive/modules/99_init.js`). #2283 — the bucket is carried
/// explicitly so the Drive resolves the path in the right store instead of the
/// currently selected one.
fn drive_link(name: &str, bucket: &str, key: &str) -> String {
    let path = urlencoding::encode(key);
    let bucket = urlencoding::encode(bucket);
    format!("[{name}](app://drive?path={path}&scope=bot&bucket={bucket})")
}

/// Deep link that opens a Drive file in the Docs editor (DOC-05 retrieval path).
fn docs_href(bucket: &str, key: &str) -> String {
    let path = urlencoding::encode(key);
    let bucket = urlencoding::encode(bucket);
    format!("app://docs?file={path}&bucket={bucket}")
}

fn docs_link(name: &str, bucket: &str, key: &str) -> String {
    format!("[{name}]({})", docs_href(bucket, key))
}

fn drive_href(bucket: &str, key: &str) -> String {
    let path = urlencoding::encode(key);
    let bucket = urlencoding::encode(bucket);
    format!("app://drive?path={path}&scope=bot&bucket={bucket}")
}

fn is_document_file(name: &str) -> bool {
    matches!(
        name.rsplit('.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str(),
        "doc" | "docx" | "odt" | "rtf" | "txt" | "md" | "markdown" | "pdf" | "html" | "htm"
    )
}

fn matches_drive_query(item: &crate::core::bot::drive_bridge::DriveListItem, query: &str) -> bool {
    let needle = query.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return true;
    }
    let hay = format!(
        "{} {}",
        item.relative.to_ascii_lowercase(),
        item.relative
            .rsplit('/')
            .next()
            .unwrap_or(&item.relative)
            .to_ascii_lowercase()
    );
    needle
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .all(|token| hay.contains(token))
}

mod handlers;

/// File names a user typed that are plausible Drive references.
fn referenced_names(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    for raw in text.split(|c: char| {
        c.is_whitespace()
            || matches!(
                c,
                ',' | ';' | '(' | ')' | '[' | ']' | '"' | '\'' | '<' | '>'
            )
    }) {
        let token = raw.trim_matches(|c: char| matches!(c, '.' | ':' | '!' | '?' | '*' | '`'));
        let Some((_, ext)) = token.rsplit_once('.') else {
            continue;
        };
        if !TEXT_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
            continue;
        }
        if token.len() > 200 || token.starts_with('/') {
            continue;
        }
        let name = sanitize_name(token);
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names.truncate(drive_bridge::MAX_FETCH_FILES);
    names
}

/// Copies Drive files named in the message into `data/` and tells the model
/// their local paths. Best-effort: a Drive outage must not fail the turn.
pub async fn fetch_referenced(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    user_id: Uuid,
    session_id: Uuid,
    text: &str,
    messages: &mut Vec<serde_json::Value>,
) {
    // #2239 — never provision/invoke the workspace when Agent/VM is off.
    if !super::agent_turn::is_enabled(session_id) {
        return;
    }
    let names = referenced_names(text);
    log::info!("drive bridge: referenced names {:?}", names);
    if names.is_empty() {
        return;
    }
    let Some(scope) = drive_bridge::scope(state, &bot_uuid, user_id) else {
        botlog::safe_warn!("drive bridge: no Drive scope for bot {bot_uuid}");
        return;
    };
    botlog::safe_info!(
        "drive bridge: scope bucket={} bot_prefix={} user_prefix={:?}",
        scope.bucket,
        scope.bot_prefix,
        scope.user_prefix
    );
    let Some(_) = agent_vm_hook::ensure_agent_vm(session_id, user_id, bot_uuid).await else {
        return;
    };
    let mut fetched = Vec::new();
    for name in names {
        let Some((key, _)) = drive_bridge::find_object(state, &scope, &name).await else {
            log::info!("drive bridge: '{name}' not found in Drive");
            continue;
        };
        let Some(bytes) = drive_bridge::read_object(state, &scope, &key).await else {
            continue;
        };
        if bytes.len() as u64 > drive_bridge::MAX_FETCH_BYTES {
            continue;
        }
        let local = format!("data/{}", sanitize_name(&name));
        if agent_vm_hook::write_session_file(user_id, session_id, &local, &bytes)
            .await
            .is_ok()
        {
            fetched.push(format!("{local} ({})", human_size(bytes.len() as u64)));
        }
    }
    if fetched.is_empty() {
        return;
    }
    messages.push(serde_json::json!({
        "role": "system",
        "content": format!(
            "These Drive files were copied into the conversation workspace: {}. \
             Read them with the program at those relative paths (e.g. open(\"data/x.csv\")). \
             Do not fetch them again with a network call.",
            fetched.join(", ")
        )
    }));
    botlog::safe_info!(
        "drive bridge: pre-fetched {} files for {session_id}",
        fetched.len()
    );
}

#[cfg(test)]
#[path = "../native_drive_tests.rs"]
mod tests;
