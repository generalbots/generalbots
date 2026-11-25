//! Drive ↔ workspace native tools (#1584).
//!
//! `drive_fetch` copies a Drive object into the conversation's `data/` before
//! the program runs; `drive_save` publishes `output/` into the person's own
//! Drive area, `Conversas/{title}-{id8}/` (`person_drive`), so saved files
//! outlive the session folder and are the person's to rename, move or delete.
//!
//! The runtime also pre-fetches file names mentioned in the message
//! ([`fetch_referenced`]) so "summarise sales.csv from my Drive" works without
//! an explicit tool call.

use std::sync::Arc;

use botcore::shared::state::AppState;
use uuid::Uuid;

use super::native_tools::{persist, send};
use super::types::SessionIdentity;
use super::vm_consent::reason;
pub mod attached;
mod list;
mod outcome;
pub(crate) use outcome::DriveOutcome;
use outcome::refuse;
use crate::core::bot::{agent_vm_hook, drive_bridge, person_drive};

const TEXT_EXTENSIONS: &[&str] = &[
    "csv", "tsv", "xlsx", "xls", "docx", "doc", "pdf", "txt", "md", "json", "pptx", "ppt", "png",
    "jpg", "jpeg", "gif", "webp", "svg", "zip", "py", "js", "html", "xml", "yaml", "yml", "log",
    "sql",
];

/// `drive_save` and `save_to_external_drive` never reach here: the consent
/// gate (`vm_consent::tools::run_drive_tool`) runs first and handles both.
pub async fn try_run(identity: &SessionIdentity<'_>, name: &str, args: &str) -> bool {
    if name != "drive_list" {
        return false;
    }
    list::run_drive_list(identity, args).await;
    true
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

/// [`drive_link`] for a file of the person's own Drive area; `path` is the
/// path the person's Drive shows (`Conversas/<folder>/<name>`).
fn person_file_link(name: &str, path: &str) -> String {
    let path = urlencoding::encode(path);
    format!("[{name}](app://drive?path={path}&scope=user)")
}

/// Copies a Drive file into the conversation folder; the gate decided first
/// (`vm_consent::tools::run_drive_tool`).
pub(crate) async fn run_drive_fetch(
    identity: &SessionIdentity<'_>,
    args: &str,
    ticket: crate::core::bot::pipeline::vm_consent::VmConsent,
) -> DriveOutcome {
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
        return refuse(identity, "Tell me which Drive file to fetch (`path`).", reason::NOT_FOUND).await;
    }
    if !identity.agent_vm_enabled {
        return refuse(identity, "Agent/VM is disabled for this conversation", reason::VM_OFF).await;
    }
    agent_vm_hook::ensure_agent_vm(session_id, user_id, bot_uuid).await;

    let scope = match drive_bridge::scope(state, &bot_uuid, user_id) {
        Some(scope) => scope,
        None => {
            return refuse(identity, "Drive is not available for this workspace.", reason::DRIVE_UNAVAILABLE).await;
        }
    };
    let found = drive_bridge::find_object(state, &scope, &reference).await;
    let Some((key, relative)) = found else {
        return refuse(identity, &format!("I could not find `{reference}` in your Drive."), reason::NOT_FOUND).await;
    };
    let Some(bytes) = drive_bridge::read_object(state, &scope, &key).await else {
        return refuse(identity, &format!("Could not download `{relative}` from Drive."), reason::FETCH_FAILED).await;
    };
    if bytes.len() as u64 > drive_bridge::MAX_FETCH_BYTES {
        let size = human_size(bytes.len() as u64);
        let text = format!("`{relative}` is {size} and exceeds the 100 MB transfer limit.");
        return refuse(identity, &text, reason::TOO_LARGE).await;
    }
    let name = parsed
        .get("as")
        .and_then(|v| v.as_str())
        .map(sanitize_name)
        .unwrap_or_else(|| sanitize_name(&relative));
    let target = format!("data/{name}");
    let written = agent_vm_hook::write_session_file(ticket, user_id, session_id, &target, &bytes).await;
    let outcome = match &written {
        Ok(_) => DriveOutcome::done(vec![target.clone()]),
        Err(_) => DriveOutcome::refused(reason::FETCH_FAILED),
    };
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
    outcome
}

pub(crate) async fn run_drive_save(identity: &SessionIdentity<'_>, args: &str) -> DriveOutcome {
    let parsed = parse(args);
    let SessionIdentity {
        state,
        session_id,
        user_id,
        ..
    } = *identity;
    // Only reached through the consent gate, which refuses the turn when the
    // Agent (VM) switch is off.
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
        return refuse(identity, "There are no generated files in `output/` to save.", reason::NOTHING_TO_SAVE).await;
    }
    let selected: Vec<(String, u64)> = files
        .into_iter()
        .filter(|(name, _)| requested.is_empty() || requested.contains(name))
        .collect();
    if selected.is_empty() {
        return refuse(identity, "None of the named files exist in `output/`.", reason::NOTHING_TO_SAVE).await;
    }
    let folder = if folder.is_empty() {
        drive_bridge::conversation_title(state, session_id).await
    } else {
        folder
    };

    let mut saved = Vec::new();
    let mut stored = Vec::new();
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
                match person_drive::save_conversation_file(state, user_id, &folder, name, bytes).await {
                    Some(path) => {
                        saved.push(person_file_link(name, &path));
                        stored.push(format!("output/{name}"));
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
            "Saved to the person's Drive (Meus arquivos) under `Conversas/{folder}/` ({}):\n",
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
    DriveOutcome::from_save(stored, failed.len())
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
    attached::clear(session_id);
    // #2239 — never provision/invoke the workspace when Agent/VM is off.
    if !super::agent_turn::is_enabled(session_id) {
        return;
    }
    attached::stage_attachments(state, bot_uuid, user_id, session_id, text, messages).await;
    let names = referenced_names(&attached::without_attachment_lines(text));
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
        let ticket = crate::core::bot::pipeline::vm_consent::VmConsent::for_person_request(session_id, &local);
        if agent_vm_hook::write_session_file(ticket, user_id, session_id, &local, &bytes)
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
        // The link is Markdown: the bucket is the last parameter, followed
        // by the closing parenthesis of the link target.
        let bucket = link
            .split("bucket=")
            .nth(1)
            .unwrap_or_default()
            .trim_end_matches(')');
        assert_eq!(bucket, "beiner.gbai");
    }

    #[test]
    fn a_saved_file_links_to_the_persons_own_drive() {
        let link = person_file_link("relatório.pdf", "Conversas/Relatório-1a2b3c4d/relatório.pdf");
        assert!(link.starts_with("[relatório.pdf](app://drive?path=Conversas%2F"), "unexpected link: {link}");
        assert!(link.ends_with("&scope=user)"), "unexpected link: {link}");
        assert!(!link.contains("bucket="), "the person's area needs no bucket: {link}");
    }
}
