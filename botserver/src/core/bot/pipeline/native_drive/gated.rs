//! Drive tools that run only behind the VM consent gate.
//!
//! `vm_consent::tools::run_drive_tool` decides whether the turn may read or
//! write the workspace; these helpers do the work and report what really
//! happened, so a refusal never looks like a successful save.

use super::super::native_tools::{persist, send};
use super::super::types::SessionIdentity;
use super::super::vm_consent::reason;
use super::outcome::refuse;
use super::{human_size, parse, sanitize_name, DriveOutcome};
use crate::core::bot::pipeline::vm_consent::VmConsent;
use crate::core::bot::{agent_vm_hook, drive_bridge, person_drive};

/// [`super::drive_link`] for a file of the person's own Drive area; `path` is
/// the path the person's Drive shows (`Conversas/<folder>/<name>`).
fn person_file_link(name: &str, path: &str) -> String {
    let path = urlencoding::encode(path);
    format!("[{name}](app://drive?path={path}&scope=user)")
}

/// Copies a Drive file into the conversation folder; the consent gate decided
/// first (`vm_consent::tools::run_drive_tool`). The ticket is the proof that
/// lets the guarded file write happen.
pub(crate) async fn run_drive_fetch(
    identity: &SessionIdentity<'_>,
    args: &str,
    ticket: VmConsent,
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
                .unwrap_or_else(|| {
                    log::warn!("drive fetch: data/ listing unavailable");
                    Vec::new()
                });
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

/// Copies generated `output/` files into the person's own Drive area; the
/// consent gate decided first (`vm_consent::tools::run_drive_tool`).
pub(crate) async fn run_drive_save(identity: &SessionIdentity<'_>, args: &str) -> DriveOutcome {
    let parsed = parse(args);
    let SessionIdentity {
        state,
        session_id,
        user_id,
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
        .unwrap_or_else(Vec::new);
    let folder = parsed
        .get("folder")
        .and_then(|v| v.as_str())
        .map(drive_bridge::slug)
        .filter(|f| !f.is_empty())
        .unwrap_or_else(String::new);

    let files = agent_vm_hook::list_session_folder(user_id, session_id, "output")
        .await
        .unwrap_or_else(|| {
            log::warn!("drive save: output/ listing unavailable");
            Vec::new()
        });
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
