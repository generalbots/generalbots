//! Attachment ingestion: staging under `inbox/` and the prompt block that
//! carries the extracted text (or an honest "cannot read this" note).

use std::sync::Arc;

use base64::Engine;
use serde_json::Value;
use uuid::Uuid;

use botcore::shared::state::AppState;

use super::media::describe_media_locally;

pub(crate) struct AttachmentResult {
    pub user_text: String,
    pub note: Option<String>,
}

/// Stores a base64 attachment under `inbox/` and produces the system block
/// that carries its extracted text (or an honest "cannot read this" note).
/// Stages one attachment (`file`) or every entry of a `files` array. The
/// composer sends the array when the user selects PDF + PNG in one turn; the
/// single-file shape stays for older clients and channel adapters.
pub(crate) async fn ingest_attachment(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    bot_name: &str,
    parsed: &Value,
    user_text: String,
) -> AttachmentResult {
    let files: Vec<Value> = parsed
        .get("files")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if files.is_empty() {
        return ingest_single_attachment(state, bot_uuid, bot_name, parsed, user_text).await;
    }
    let mut text = user_text;
    let mut notes: Vec<String> = Vec::new();
    for file in files {
        let single = serde_json::json!({ "file": file });
        let result = ingest_single_attachment(state, bot_uuid, bot_name, &single, text).await;
        text = result.user_text;
        if let Some(note) = result.note {
            notes.push(note);
        }
    }
    AttachmentResult {
        user_text: text,
        note: if notes.is_empty() {
            None
        } else {
            Some(notes.join("\n\n"))
        },
    }
}

async fn ingest_single_attachment(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    bot_name: &str,
    parsed: &Value,
    user_text: String,
) -> AttachmentResult {
    let mut user_text = user_text;
    let mut attachment_note: Option<String> = None;
    let Some(file) = parsed.get("file").and_then(|value| value.as_object()) else {
        return AttachmentResult {
            user_text,
            note: attachment_note,
        };
    };
    let fname = file
        .get("name")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string();
    let b64 = file
        .get("content_base64")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    if fname.is_empty() || b64.is_empty() {
        return AttachmentResult {
            user_text,
            note: attachment_note,
        };
    }
    // #1947 — the composer sends attachments inline as base64. A file that is
    // too large never reaches the server intact, and the user only saw their
    // message without the attachment. Report the limit explicitly.
    const MAX_ATTACHMENT_B64: usize = 12 * 1024 * 1024;
    if b64.len() > MAX_ATTACHMENT_B64 {
        log::warn!(
            "chat attachment too large: {fname} ({} base64 bytes, limit {MAX_ATTACHMENT_B64})",
            b64.len()
        );
        user_text = format!(
            "{user_text}\n\n📎 {fname} (larger than 8 MB and was not stored — share it from Drive or split it)"
        );
        return AttachmentResult {
            user_text,
            note: attachment_note,
        };
    }
    let data = match base64::engine::general_purpose::STANDARD.decode(b64) {
        Ok(data) => data,
        Err(error) => {
            log::warn!("chat attachment decode failed for {fname}: {error}");
            user_text = format!(
                "{user_text}\n\n📎 {fname} (the file could not be decoded and was not stored — attach it again)"
            );
            return AttachmentResult {
                user_text,
                note: attachment_note,
            };
        }
    };

    let excerpt = super::super::attachment_text::extract_excerpt(&fname, &data);
    let media = super::super::attachment_text::is_media(&fname);
    let mut stored = false;
    if let Some(drive) = state.drive.as_ref() {
        // Same resolution as the channel stagers: an org-hosted bot lives under
        // `{slug}.gborg`, so the bucket cannot be derived from the bot name.
        let loc = match state.conn.get() {
            Ok(mut conn) => botbasic_core::utils::bot_drive_location_for(&mut conn, bot_uuid),
            Err(error) => {
                log::error!("chat attachment: DB error: {error}");
                botbasic_core::utils::resolve_bot_drive_location(bot_name, None, None)
            }
        };
        let bucket = loc.bucket.clone();
        let key = loc.key_for(&format!("inbox/{fname}"));
        // Stage the upload BEFORE media analysis: describing an image can take
        // tens of seconds, and the chat already offers Preview, which answered
        // 404 while the file was not yet in the inbox (#1693).
        match drive.put_object(&bucket, &key, data.clone(), None).await {
            Ok(()) => {
                stored = true;
                log::info!("stored chat attachment: {bucket}/{key}");
            }
            Err(error) => {
                log::error!("chat attachment store failed: {error}");
                user_text =
                    format!("{user_text}\n\n📎 {fname} (the upload could not be stored: {error})");
            }
        }
    } else {
        user_text = format!(
            "{user_text}\n\n📎 {fname} (Drive is not configured, so the file was not stored)"
        );
    }
    // Media analysis runs after the file is reachable in the inbox.
    let media_note = if media && excerpt.is_none() {
        describe_media_locally(state, bot_uuid, &fname, &data).await
    } else {
        None
    };
    if stored {
        // The pipeline detects a staged upload by this marker and injects its
        // inbox path into tool calls (classify_media). The bare "📎 name" line
        // alone left every web upload without a path, so media tools never ran.
        user_text = format!(
            "{user_text}\n\n[User attached a file stored at inbox/{fname}; the upload was staged for the bot]\n📎 {fname}"
        );
        attachment_note = Some(match excerpt {
            Some(text) => format!(
                "The user attached '{fname}'. The file is stored at inbox/{fname} and its extracted text is reproduced below. Use it to answer; never claim content that is not in it.\n\n--- BEGIN ATTACHMENT {fname} ---\n{text}\n--- END ATTACHMENT {fname} ---"
            ),
            None if media => media_note.unwrap_or_else(|| {
                let agent_on = parsed
                    .get("agent_enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                if agent_on {
                    format!(
                        "The user attached the media file '{fname}' (stored at inbox/{fname}). Local transcription (BotModels) is unavailable, but the Agent/VM is enabled for this conversation: you may process the file from the VM (reachable at inbox/{fname}) to extract its content. Do not invent what it shows; only report what the VM actually extracts."
                    )
                } else {
                    format!(
                        "The user attached the media file '{fname}' (stored at inbox/{fname}). Local transcription is unavailable and the Agent/VM is disabled, so you cannot read its pixels or audio. Answer only from the user's text and say plainly that you cannot see the file's content. Never invent what it shows."
                    )
                }
            }),
            None => format!(
                "The user attached '{fname}' (stored at inbox/{fname}). No text extractor is available for this format, so you cannot read its content. Say the file is stored and ask the user to paste the relevant text (or to convert it). Never invent its content."
            ),
        });
    }
    AttachmentResult {
        user_text,
        note: attachment_note,
    }
}
