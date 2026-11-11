//! `save_to_external_drive` — write workspace artifacts to Google Drive or
//! OneDrive without the user clicking (#2047).
//!
//! The HTTP route (`POST /api/external-drives/upload`) is the surface the UI
//! uses; this tool is the same capability for the chat model. It mirrors the
//! route's rules exactly: branch-scoped connection, capability read from the
//! recorded consent (never assumed), and an honest refusal when the grant is
//! read-only. Uploads happen after the program already ran, so a save never
//! competes with the execution budget (#2045).

use super::native_tools::{persist, send};
use super::types::SessionIdentity;
use crate::core::bot::agent_vm_hook;
use botdrive::external::upload::{self, UploadRequest};
use botdrive::external::{db as external_db, engine, types::Provider};

/// Parses the tool arguments; an unreadable payload becomes an empty object so
/// the tool answers with usage instead of failing the turn.
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

fn mime_for(name: &str) -> &'static str {
    match name.rsplit_once('.').map(|(_, ext)| ext.to_ascii_lowercase()) {
        Some(ext) if ext == "pdf" => "application/pdf",
        Some(ext) if ext == "docx" => {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        }
        Some(ext) if ext == "xlsx" => {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        }
        Some(ext) if ext == "pptx" => {
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        }
        Some(ext) if matches!(ext.as_str(), "png") => "image/png",
        Some(ext) if matches!(ext.as_str(), "jpg" | "jpeg") => "image/jpeg",
        Some(ext) if ext == "csv" => "text/csv",
        Some(ext) if matches!(ext.as_str(), "txt" | "md") => "text/plain",
        Some(ext) if ext == "json" => "application/json",
        _ => "application/octet-stream",
    }
}

/// Runs the tool. `provider` is `gdrive` or `onedrive`; `files` selects names
/// from `output/` (all when omitted); `folder` is the destination folder.
pub async fn run(identity: &SessionIdentity<'_>, args: &str) {
    let parsed = parse(args);
    let SessionIdentity {
        state,
        session_id,
        user_id,
        ..
    } = *identity;
    // #2239 — reads artifacts from the conversation workspace (output/).
    if !identity.agent_vm_enabled {
        send(identity, "Agent/VM is disabled for this conversation").await;
        return;
    }

    let provider = match parsed
        .get("provider")
        .and_then(|v| v.as_str())
        .and_then(Provider::parse)
    {
        Some(provider) => provider,
        None => {
            let message = "Choose a save target: `provider` must be `gdrive` or `onedrive`.";
            send(identity, message).await;
            persist(state, session_id, user_id, message).await;
            return;
        }
    };
    let folder = parsed
        .get("folder")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim_matches('/')
        .to_string();
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

    let files = agent_vm_hook::list_session_folder(user_id, session_id, "output")
        .await
        .unwrap_or_default();
    let selected: Vec<(String, u64)> = files
        .into_iter()
        .filter(|(name, _)| requested.is_empty() || requested.contains(name))
        .collect();
    if selected.is_empty() {
        let message = if requested.is_empty() {
            "There are no generated files in `output/` to save.".to_string()
        } else {
            "None of the named files exist in `output/`.".to_string()
        };
        send(identity, &message).await;
        persist(state, session_id, user_id, &message).await;
        return;
    }

    let (branch_id, connection, granted) = {
        let Ok(mut conn) = external_db::conn(&state.conn) else {
            let message = "Could not reach the database to look up the drive connection.";
            send(identity, message).await;
            persist(state, session_id, user_id, message).await;
            return;
        };
        let branch_id = match external_db::branch_for_user(&mut conn, user_id) {
            Ok(branch) => branch,
            Err(error) => {
                botlog::safe_warn!("external save: branch lookup failed: {error}");
                uuid::Uuid::nil()
            }
        };
        let connection = external_db::get_connection(&mut conn, branch_id, provider)
            .ok()
            .flatten();
        let granted = connection
            .as_ref()
            .and_then(|c| c.scopes.clone())
            .unwrap_or_default();
        (branch_id, connection, granted)
    };
    let Some(connection) = connection.filter(|c| c.status != "disconnected") else {
        let message = format!(
            "{} is not connected for this workspace. Open Drive → External and connect it first.",
            provider.display_name()
        );
        send(identity, &message).await;
        persist(state, session_id, user_id, &message).await;
        return;
    };
    if !upload::allows_write(provider, &granted) {
        let message = format!(
            "The {} connection only allows reading. Grant write access in Drive → External \
             (reconnect the account) and ask again — nothing was uploaded.",
            provider.display_name()
        );
        send(identity, &message).await;
        persist(state, session_id, user_id, &message).await;
        return;
    }

    let http = match engine::http_client() {
        Ok(http) => http,
        Err(error) => {
            botlog::safe_warn!("external save: http client unavailable: {error}");
            let message = "The drive client is unavailable right now.";
            send(identity, message).await;
            persist(state, session_id, user_id, message).await;
            return;
        }
    };
    let token = match engine::connection_access_token(&http, &state.conn, branch_id, &connection)
        .await
    {
        Ok(token) => token,
        Err(error) => {
            botlog::safe_warn!("external save: token unavailable: {error}");
            let message = format!(
                "Could not authorise the {} upload; reconnect the account and try again.",
                provider.display_name()
            );
            send(identity, &message).await;
            persist(state, session_id, user_id, &message).await;
            return;
        }
    };

    let mut saved = Vec::new();
    let mut failed = Vec::new();
    for (name, _) in &selected {
        let relative = format!("output/{name}");
        let bytes = match agent_vm_hook::read_session_file(user_id, session_id, &relative).await {
            Ok(bytes) => bytes,
            Err(error) => {
                failed.push(format!("{name} ({error})"));
                continue;
            }
        };
        let request = UploadRequest {
            provider,
            path: folder.clone(),
            name: name.clone(),
            mime_type: mime_for(name).to_string(),
            content: bytes,
            replace_remote_id: None,
            parent_id: None,
        };
        let prepared = match upload::build_target(&request, &granted) {
            Ok(prepared) => prepared,
            Err(error) => {
                failed.push(format!("{name} ({error})"));
                continue;
            }
        };
        match upload::send(&http, provider, &token, name, prepared).await {
            Ok(uploaded) => {
                let link = uploaded
                    .web_view_link
                    .clone()
                    .map(|url| format!("[{name}]({url})"))
                    .unwrap_or_else(|| format!("{name} (`{}`)", uploaded.remote_id));
                if let Ok(mut conn) = external_db::conn(&state.conn) {
                    let mirror = botdrive::external::types::ExternalFile {
                        remote_id: uploaded.remote_id.clone(),
                        name: uploaded.name.clone(),
                        path: if folder.is_empty() {
                            uploaded.name.clone()
                        } else {
                            format!("{folder}/{}", uploaded.name)
                        },
                        mime_type: Some(mime_for(name).to_string()),
                        size_bytes: uploaded.bytes as i64,
                        modified_at: Some(chrono::Utc::now()),
                        is_folder: false,
                        revision: None,
                        parent_remote_id: None,
                    };
                    if let Err(error) = botdrive::external::files::upsert_file(
                        &mut conn,
                        branch_id,
                        provider,
                        &mirror,
                    ) {
                        botlog::safe_warn!("external save: mirror failed for {name}: {error}");
                    }
                }
                saved.push(link);
            }
            Err(error) => failed.push(format!("{name} ({error})")),
        }
    }

    let mut message = String::new();
    if !saved.is_empty() {
        message.push_str(&format!(
            "Saved to {} ({}):\n",
            provider.display_name(),
            if folder.is_empty() {
                "default folder".to_string()
            } else {
                folder.clone()
            }
        ));
        for item in &saved {
            message.push_str(&format!("- {item}\n"));
        }
    }
    if !failed.is_empty() {
        message.push_str(&format!("\nNot saved: {}", failed.join(", ")));
    }
    if message.is_empty() {
        message.push_str("Nothing was saved.");
    }
    send(identity, &message).await;
    persist(state, session_id, user_id, &message).await;
}
