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
            run_drive_fetch(identity, args).await;
            true
        }
        "drive_save" => {
            run_drive_save(identity, args).await;
            true
        }
        "drive_list" => {
            run_drive_list(identity, args).await;
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

async fn run_drive_fetch(identity: &SessionIdentity<'_>, args: &str) {
    let parsed = parse(args);
    let reference = parsed
        .get("path")
        .or_else(|| parsed.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let SessionIdentity {
        state,
        session_id,
        user_id,
        bot_uuid,
        ..
    } = *identity;
    if reference.is_empty() {
        send(identity, "Tell me which Drive file to fetch (`path`).").await;
        return;
    }
    if !identity.agent_vm_enabled {
        send(identity, "Agent/VM is disabled for this conversation").await;
        return;
    }
    agent_vm_hook::ensure_agent_vm(session_id, user_id, bot_uuid).await;

    let scope = match drive_bridge::scope(state, &bot_uuid, user_id) {
        Some(scope) => scope,
        None => {
            send(identity, "Drive is not available for this workspace.").await;
            return;
        }
    };
    let found = drive_bridge::find_object(state, &scope, &reference).await;
    let Some((key, relative)) = found else {
        send(
            identity,
            &format!("I could not find `{reference}` in your Drive."),
        )
        .await;
        return;
    };
    let Some(bytes) = drive_bridge::read_object(state, &scope, &key).await else {
        send(
            identity,
            &format!("Could not download `{relative}` from Drive."),
        )
        .await;
        return;
    };
    if bytes.len() as u64 > drive_bridge::MAX_FETCH_BYTES {
        send(
            identity,
            &format!(
                "`{relative}` is {} and exceeds the 100 MB transfer limit.",
                human_size(bytes.len() as u64)
            ),
        )
        .await;
        return;
    }
    let name = parsed
        .get("as")
        .and_then(|v| v.as_str())
        .map(sanitize_name)
        .unwrap_or_else(|| sanitize_name(&relative));
    let target = format!("data/{name}");
    let written = agent_vm_hook::write_session_file(user_id, session_id, &target, &bytes).await;
    let message = match written {
        Ok(size) => {
            let listing = agent_vm_hook::list_session_folder(user_id, session_id, "data")
                .await
                .unwrap_or_default();
            let names: Vec<String> = listing
                .iter()
                .map(|(name, size)| format!("{name} ({})", human_size(*size)))
                .collect();
            format!(
                "Fetched `{relative}` from Drive into `{target}` ({}).\n\n**data/** {}",
                human_size(size as u64),
                names.join(", ")
            )
        }
        Err(e) => format!("Could not copy `{relative}` into the workspace: {e}"),
    };
    send(identity, &message).await;
    persist(state, session_id, user_id, &message).await;
}

const DRIVE_LIST_DEFAULT_LIMIT: usize = 20;
const DRIVE_LIST_MAX_LIMIT: usize = 100;
const DRIVE_LIST_SEARCH_SCAN_LIMIT: usize = 200;

/// #2164 — lists files in the bot's `.gbdrive` so the model can answer
/// "open my last document about X" without the user naming a file.
async fn run_drive_list(identity: &SessionIdentity<'_>, args: &str) {
    let parsed = parse(args);
    let limit = parsed
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .unwrap_or(DRIVE_LIST_DEFAULT_LIMIT)
        .clamp(1, DRIVE_LIST_MAX_LIMIT);
    let query = parsed
        .get("query")
        .or_else(|| parsed.get("q"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let SessionIdentity {
        state,
        session_id,
        user_id,
        bot_uuid,
        ..
    } = *identity;
    let Some(scope) = drive_bridge::scope(state, &bot_uuid, user_id) else {
        send(identity, "Drive is not available for this workspace.").await;
        return;
    };
    let scan_limit = if query.is_some() {
        DRIVE_LIST_SEARCH_SCAN_LIMIT
    } else {
        limit
    };
    let Some(items) = drive_bridge::list_bot_drive_objects(state, &scope, scan_limit).await else {
        send(identity, "Could not list files in your bot Drive.").await;
        return;
    };
    let filtered: Vec<_> = if let Some(ref q) = query {
        items
            .into_iter()
            .filter(|item| matches_drive_query(item, q))
            .take(limit)
            .collect()
    } else {
        items.into_iter().take(limit).collect()
    };
    if filtered.is_empty() {
        let message = if let Some(ref q) = query {
            format!("No files matching \"{q}\" were found in the bot Drive (`.gbdrive`).")
        } else {
            "No files found in the bot Drive (`.gbdrive`).".to_string()
        };
        send(identity, &message).await;
        persist(state, session_id, user_id, &message).await;
        return;
    }
    let heading = if let Some(ref q) = query {
        format!("Bot Drive files matching \"{q}\" (newest first, up to {limit}):")
    } else {
        format!("Bot Drive files (newest first, up to {limit}):")
    };
    let mut lines = vec![heading];
    for item in &filtered {
        let name = sanitize_name(&item.relative);
        let entry = if is_document_file(&item.relative) {
            format!(
                "- {} · [Open in Drive]({}) ({})",
                docs_link(&name, &scope.bucket, &item.key),
                drive_href(&scope.bucket, &item.key),
                human_size(item.size)
            )
        } else {
            format!(
                "- {} ({})",
                drive_link(&name, &scope.bucket, &item.key),
                human_size(item.size)
            )
        };
        lines.push(entry);
    }
    if query.is_some() {
        lines.push(String::new());
        lines.push(
            "To open a document in the editor, use its Docs link (`app://docs?file=<key>&bucket=<bucket>`)."
                .to_string(),
        );
    }
    let message = lines.join("\n");
    send(identity, &message).await;
    persist(state, session_id, user_id, &message).await;
}

async fn run_drive_save(identity: &SessionIdentity<'_>, args: &str) {
    let parsed = parse(args);
    let SessionIdentity {
        state,
        session_id,
        user_id,
        bot_uuid,
        ..
    } = *identity;
    // #2239 — workspace output access must be refused when Agent/VM is off.
    if !identity.agent_vm_enabled {
        send(identity, "Agent/VM is disabled for this conversation").await;
        return;
    }
    let requested: Vec<String> = parsed
        .get("files")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(sanitize_name))
                .collect()
        })
        .unwrap_or_default();
    let folder = parsed
        .get("folder")
        .and_then(|v| v.as_str())
        .map(drive_bridge::slug)
        .filter(|f| !f.is_empty())
        .unwrap_or_else(|| String::new());

    let files = agent_vm_hook::list_session_folder(user_id, session_id, "output")
        .await
        .unwrap_or_default();
    if files.is_empty() {
        send(
            identity,
            "There are no generated files in `output/` to save.",
        )
        .await;
        return;
    }
    let selected: Vec<(String, u64)> = files
        .into_iter()
        .filter(|(name, _)| requested.is_empty() || requested.contains(name))
        .collect();
    if selected.is_empty() {
        send(identity, "None of the named files exist in `output/`.").await;
        return;
    }
    let folder = if folder.is_empty() {
        drive_bridge::conversation_title(state, session_id).await
    } else {
        folder
    };

    let mut saved = Vec::new();
    let mut failed = Vec::new();
    let mut total: u64 = 0;
    for (name, _) in &selected {
        match agent_vm_hook::read_session_file(user_id, session_id, &format!("output/{name}")).await
        {
            Ok(bytes) => {
                total += bytes.len() as u64;
                if total > botagent::transfer::MAX_TURN_BYTES as u64 {
                    failed.push(format!("{name} (turn limit)"));
                    continue;
                }
                match drive_bridge::save_object(state, &bot_uuid, &folder, name, bytes).await {
                    Some(path) => {
                        let (bucket, key) = split_bucket_path(&path);
                        saved.push(drive_link(name, bucket, key));
                    }
                    None => failed.push(name.to_string()),
                }
            }
            Err(e) => failed.push(format!("{name} ({e})")),
        }
    }
    let mut message = String::new();
    if !saved.is_empty() {
        message.push_str(&format!(
            "Saved to Drive under `conversations/{folder}/` ({}):\n",
            human_size(total)
        ));
        for item in &saved {
            message.push_str(&format!("- {item}\n"));
        }
    }
    if !failed.is_empty() {
        message.push_str(&format!("\nNot saved: {}", failed.join(", ")));
    }
    if message.is_empty() {
        message.push_str("Nothing was saved to Drive.");
    }
    send(identity, &message).await;
    persist(state, session_id, user_id, &message).await;
}

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
mod tests {
    use super::*;

    /// #2143 gap A — the chat renderer resolves only `vmfile:` and `app://`
    /// links, so a `drive:` URL rendered as dead text and the artifact was
    /// unreachable. The emitted string must be one of those two prefixes.
    /// #2283 — the bucket is included so the Drive can resolve the path in the
    /// correct store instead of the currently selected one.
    #[test]
    fn drive_link_uses_a_prefix_the_chat_renderer_resolves() {
        let link = drive_link(
            "report.pdf",
            "beiner.gbai",
            "beiner.gbdrive/conversations/quarterly-report",
        );
        assert!(
            link.starts_with("[report.pdf](app://drive?path="),
            "unexpected link: {link}"
        );
        assert!(
            link.contains("&scope=bot&bucket="),
            "unexpected link: {link}"
        );
        assert!(!link.contains("](drive:"), "dead drive: scheme: {link}");
        assert!(
            link.starts_with("[report.pdf](app://") || link.starts_with("[report.pdf](vmfile:"),
            "not a renderer prefix: {link}"
        );
    }

    /// The path must survive percent-encoding, because the renderer decodes it
    /// into the Drive app's `path` parameter.
    #[test]
    fn drive_link_encodes_the_folder_key() {
        let link = drive_link(
            "report.pdf",
            "beiner.gbai",
            "beiner.gbdrive/conversations/q3 2026",
        );
        let encoded = link
            .split("path=")
            .nth(1)
            .and_then(|rest| rest.split('&').next())
            .unwrap_or_default();
        let decoded = urlencoding::decode(encoded)
            .ok()
            .map(|value| value.into_owned())
            .unwrap_or_default();
        assert_eq!(decoded, "beiner.gbdrive/conversations/q3 2026");
        assert!(encoded.contains("%2F"), "slashes must be encoded: {link}");
        let bucket = link.split("bucket=").nth(1).unwrap_or_default();
        assert_eq!(bucket, "beiner.gbai");
    }

    #[test]
    fn split_bucket_path_extracts_bucket_and_key() {
        let (bucket, key) =
            split_bucket_path("beiner.gbai/beiner.gbdrive/conversations/report.pdf");
        assert_eq!(bucket, "beiner.gbai");
        assert_eq!(key, "beiner.gbdrive/conversations/report.pdf");
    }

    /// `save_object` returns `bucket/key`; the Drive app addresses the key.
    #[test]
    fn relative_key_drops_the_bucket() {
        assert_eq!(
            relative_key("beiner.gbai/beiner.gbdrive/conversations/x/report.pdf"),
            "beiner.gbdrive/conversations/x/report.pdf"
        );
        assert_eq!(relative_key("no-slash"), "no-slash");
    }
}
