//! Files the person attached to this message, copied into the conversation's
//! workspace for the subscription engines (#2691).
//!
//! The attachment hook stores each upload at `inbox/<name>` and leaves a
//! marker line in the message. The names are read from those markers whole:
//! guessing them from the words of the message split "Tusks Poster.jpg" into
//! "Poster.jpg", which was not found, so the file never reached the engine.
//! Each copy keeps the person-request consent (`kind=file_write
//! source=person`), and the engines are told the exact path in the VM.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use botcore::shared::state::AppState;
use uuid::Uuid;

use crate::core::bot::{agent_vm_hook, drive_bridge};

const MARKER: &str = "[User attached a file stored at inbox/";
const MARKER_END: &str = "; size=";
/// Most attachments copied for one message.
const MAX_FILES: usize = 10;
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp"];

/// One attachment now inside the VM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedFile {
    /// Absolute path inside the VM.
    pub vm_path: String,
    pub image: bool,
}

fn staged() -> &'static Mutex<HashMap<Uuid, Vec<StagedFile>>> {
    static STAGED: OnceLock<Mutex<HashMap<Uuid, Vec<StagedFile>>>> = OnceLock::new();
    STAGED.get_or_init(|| Mutex::new(HashMap::new()))
}

fn record(session_id: Uuid, files: Vec<StagedFile>) {
    if let Ok(mut map) = staged().lock() {
        if files.is_empty() {
            map.remove(&session_id);
        } else {
            map.insert(session_id, files);
        }
    }
}

/// The attachments staged for this session's current turn. Reading does not
/// clear them, so a retried attempt of the same turn still has them; the next
/// message replaces them ([`clear`]).
pub fn staged_for(session_id: Uuid) -> Vec<StagedFile> {
    staged()
        .lock()
        .ok()
        .and_then(|map| map.get(&session_id).cloned())
        .unwrap_or_default()
}

/// Forgets the previous message's attachments.
pub(super) fn clear(session_id: Uuid) {
    record(session_id, Vec::new());
}

/// The attachment paths for an engine that receives only the latest message.
pub fn resumed_note(files: &[StagedFile]) -> String {
    if files.is_empty() {
        return String::new();
    }
    let paths: Vec<&str> = files.iter().map(|file| file.vm_path.as_str()).collect();
    format!(
        "[Attached to this message and copied whole into the workspace: {}. Open each file at that          exact path (pictures included) and read the whole file before answering about it.]",
        paths.join(", ")
    )
}

/// The attachment names the hook recorded in `text`, exactly as stored.
pub(super) fn attached_names(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix(MARKER) else {
            continue;
        };
        let Some(end) = rest.rfind(MARKER_END) else {
            continue;
        };
        let name = rest[..end].trim();
        if !name.is_empty() && !names.iter().any(|known: &String| known == name) {
            names.push(name.to_string());
        }
    }
    names
}

/// `text` without the hook's attachment lines, so the Drive name guess does
/// not pick pieces of attachment names.
pub(super) fn without_attachment_lines(text: &str) -> String {
    text.lines()
        .filter(|line| {
            let line = line.trim_start();
            !line.starts_with(MARKER) && !line.starts_with('\u{1F4CE}')
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn fold(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
        'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'A',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'É' | 'È' | 'Ê' | 'Ë' => 'E',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
        'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'O',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
        'ç' => 'c',
        'Ç' => 'C',
        'ñ' => 'n',
        'Ñ' => 'N',
        c if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ' ') => c,
        _ => '_',
    }
}

/// The file name used inside the VM: accents folded, any other character
/// the workspace path check refuses becomes `_`, never empty or hidden.
pub(super) fn vm_file_name(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name).trim();
    let clean: String = base.chars().map(fold).collect();
    let clean = clean.trim().trim_start_matches('.').to_string();
    if clean.is_empty() || clean.chars().all(|c| c == '_' || c == '.') {
        "anexo".to_string()
    } else {
        clean.chars().take(120).collect()
    }
}

fn is_image(name: &str) -> bool {
    name.rsplit_once('.')
        .is_some_and(|(_, ext)| IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
}

fn conversation_path(session_id: Uuid, local: &str) -> String {
    format!("/opt/vibe/conversations/{session_id}/{local}")
}

/// What the engine is told about the copies (and about what failed).
pub(super) fn workspace_note(copied: &[(String, String, u64)], failed: &[String]) -> String {
    let mut note = String::new();
    if !copied.is_empty() {
        let list: Vec<String> = copied
            .iter()
            .map(|(original, path, size)| format!("- \"{original}\" -> {path} ({} bytes)", size))
            .collect();
        note.push_str(&format!(
            "The person attached these files to this message. Each was copied whole into the \
             conversation workspace at the exact path shown (relative to the conversation folder: \
             data/<name>):\n{}\n\
             Use these exact paths. A picture is given to you as an image when your interface \
             supports it; otherwise open the file at its path to look at it, and never describe a \
             picture you did not open. Any excerpt of an attachment quoted in the message may be \
             cut short: the file holds the whole content, so read the whole file before answering \
             about it.",
            list.join("\n")
        ));
    }
    if !failed.is_empty() {
        if !note.is_empty() {
            note.push('\n');
        }
        note.push_str(&format!(
            "These attachments could not be copied into the workspace: {}. Tell the person, in \
             their language, that you could not open them, instead of guessing their content.",
            failed.join(", ")
        ));
    }
    note
}

async fn read_inbox(state: &Arc<AppState>, scope: &drive_bridge::DriveScope, name: &str) -> Option<Vec<u8>> {
    let key = format!("{}inbox/{name}", scope.bot_prefix);
    if let Some(bytes) = drive_bridge::read_object(state, scope, &key).await {
        return Some(bytes);
    }
    let (key, _) = drive_bridge::find_object(state, scope, &format!("inbox/{name}")).await?;
    drive_bridge::read_object(state, scope, &key).await
}

/// Copies every attachment of this message into `data/` of the conversation
/// folder and tells the engine their exact paths. Returns how many arrived.
pub(super) async fn stage_attachments(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    user_id: Uuid,
    session_id: Uuid,
    text: &str,
    messages: &mut Vec<serde_json::Value>,
) -> usize {
    let names = attached_names(text);
    if names.is_empty() {
        return 0;
    }
    log::info!("drive bridge: {} attachments for the workspace", names.len());
    let Some(scope) = drive_bridge::scope(state, &bot_uuid, user_id) else {
        botlog::safe_warn!("drive bridge: no Drive scope for bot {bot_uuid}");
        return 0;
    };
    if agent_vm_hook::ensure_agent_vm(session_id, user_id, bot_uuid).await.is_none() {
        return 0;
    }
    let mut copied: Vec<(String, String, u64)> = Vec::new();
    let mut failed: Vec<String> = Vec::new();
    let mut staged_files = Vec::new();
    let mut total: u64 = 0;
    let mut used_names: Vec<String> = Vec::new();
    for name in names.into_iter().take(MAX_FILES) {
        let Some(bytes) = read_inbox(state, &scope, &name).await else {
            failed.push(format!("\"{name}\""));
            continue;
        };
        let size = bytes.len() as u64;
        total += size;
        if size > drive_bridge::MAX_FETCH_BYTES || total > botagent::transfer::MAX_TURN_BYTES as u64 {
            failed.push(format!("\"{name}\" (too large)"));
            continue;
        }
        let mut file_name = vm_file_name(&name);
        if used_names.contains(&file_name) {
            file_name = format!("{}-{file_name}", used_names.len() + 1);
        }
        used_names.push(file_name.clone());
        let local = format!("data/{file_name}");
        let ticket = crate::core::bot::pipeline::vm_consent::VmConsent::for_person_request(session_id, &local);
        match agent_vm_hook::write_session_file(ticket, user_id, session_id, &local, &bytes).await {
            Ok(_) => {
                let vm_path = conversation_path(session_id, &local);
                staged_files.push(StagedFile { vm_path: vm_path.clone(), image: is_image(&file_name) });
                copied.push((name, vm_path, size));
            }
            Err(error) => {
                botlog::safe_warn!("drive bridge: attachment copy failed: {error}");
                failed.push(format!("\"{name}\""));
            }
        }
    }
    let note = workspace_note(&copied, &failed);
    if !note.is_empty() {
        messages.push(serde_json::json!({ "role": "system", "content": note }));
    }
    botlog::safe_info!(
        "drive bridge: {} attachments copied, {} failed for {session_id}",
        copied.len(),
        failed.len()
    );
    let arrived = staged_files.len();
    record(session_id, staged_files);
    arrived
}

#[cfg(test)]
#[path = "attached_tests.rs"]
mod tests;
