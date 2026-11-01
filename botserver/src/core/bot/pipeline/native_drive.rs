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
    "csv", "tsv", "xlsx", "xls", "docx", "doc", "pdf", "txt", "md", "json", "pptx", "ppt",
    "png", "jpg", "jpeg", "gif", "webp", "svg", "zip", "py", "js", "html", "xml", "yaml",
    "yml", "log", "sql",
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
        _ => false,
    }
}

fn parse(args: &str) -> serde_json::Value {
    serde_json::from_str(args).unwrap_or_else(|_| serde_json::json!({}))
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
        send(identity, &format!("Could not download `{relative}` from Drive."))
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

async fn run_drive_save(identity: &SessionIdentity<'_>, args: &str) {
    let parsed = parse(args);
    let SessionIdentity {
        state,
        session_id,
        user_id,
        bot_uuid,
        ..
    } = *identity;
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
        send(identity, "There are no generated files in `output/` to save.").await;
        return;
    }
    let selected: Vec<(String, u64)> = files
        .into_iter()
        .filter(|(name, _)| requested.is_empty() || requested.contains(name))
        .collect();
    if selected.is_empty() {
        send(
            identity,
            "None of the named files exist in `output/`.",
        )
        .await;
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
        match agent_vm_hook::read_session_file(
            user_id,
            session_id,
            &format!("output/{name}"),
        )
        .await
        {
            Ok(bytes) => {
                total += bytes.len() as u64;
                if total > botagent::transfer::MAX_TURN_BYTES as u64 {
                    failed.push(format!("{name} (turn limit)"));
                    continue;
                }
                match drive_bridge::save_object(state, &bot_uuid, &folder, name, bytes).await {
                    Some(path) => saved.push(format!("[{name}](drive:{path})")),
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
        c.is_whitespace() || matches!(c, ',' | ';' | '(' | ')' | '[' | ']' | '"' | '\'' | '<' | '>')
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
    let names = referenced_names(text);
    if names.is_empty() {
        return;
    }
    let Some(scope) = drive_bridge::scope(state, &bot_uuid, user_id) else {
        return;
    };
    let Some(_) = agent_vm_hook::ensure_agent_vm(session_id, user_id, bot_uuid).await else {
        return;
    };
    let mut fetched = Vec::new();
    for name in names {
        let Some((key, _)) = drive_bridge::find_object(state, &scope, &name).await else {
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
    log::info!("drive bridge: pre-fetched {} files for {session_id}", fetched.len());
}
