//! Chat turn preludes split out of `exec.rs` (#1641, #1644, #1645).
//!
//! Attachments become prompt content; explicit URL/command requests become
//! computer-use proposals. Keeping them here keeps the pipeline entry point
//! readable and every file under the repository limit.

use std::sync::Arc;

use base64::Engine;
use serde_json::Value;
use uuid::Uuid;

use botcore::shared::state::AppState;

use super::types::SessionIdentity;

pub(super) struct AttachmentResult {
    pub user_text: String,
    pub note: Option<String>,
}

/// Stores a base64 attachment under `inbox/` and produces the system block
/// that carries its extracted text (or an honest "cannot read this" note).
pub(super) async fn ingest_attachment(
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
    let data = match base64::engine::general_purpose::STANDARD.decode(b64) {
        Ok(data) => data,
        Err(error) => {
            log::warn!("chat attachment decode failed: {error}");
            user_text = format!(
                "{user_text}\n\n📎 {fname} (the file could not be decoded and was not stored)"
            );
            return AttachmentResult {
                user_text,
                note: attachment_note,
            };
        }
    };

    let excerpt = super::attachment_text::extract_excerpt(&fname, &data);
    let media = super::attachment_text::is_media(&fname);
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
        match drive.put_object(&bucket, &key, data, None).await {
            Ok(()) => {
                stored = true;
                log::info!("stored chat attachment: {bucket}/{key}");
            }
            Err(error) => {
                log::error!("chat attachment store failed: {error}");
                user_text = format!(
                    "{user_text}\n\n📎 {fname} (the upload could not be stored: {error})"
                );
            }
        }
    } else {
        user_text = format!(
            "{user_text}\n\n📎 {fname} (Drive is not configured, so the file was not stored)"
        );
    }
    if stored {
        user_text = format!("{user_text}\n\n📎 {fname}");
        attachment_note = Some(match excerpt {
            Some(text) => format!(
                "The user attached '{fname}'. The file is stored at inbox/{fname} and its extracted text is reproduced below. Use it to answer; never claim content that is not in it.\n\n--- BEGIN ATTACHMENT {fname} ---\n{text}\n--- END ATTACHMENT {fname} ---"
            ),
            None if media => format!(
                "The user attached the media file '{fname}' (stored at inbox/{fname}). This build cannot read its pixels or audio. Answer only from the user's text and say plainly that you cannot see the file's content. Never invent what it shows."
            ),
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

/// True when the turn was turned into a consent card (the caller returns
/// without invoking the model).
pub(super) async fn propose_computer_action(
    identity: &SessionIdentity<'_>,
    msg_type: i64,
    user_text: &str,
) -> bool {
    let SessionIdentity {
        sink,
        bot_uuid,
        session_id,
        user_id,
        ..
    } = *identity;
    if msg_type != 1 || !sink.supports_raw_frames() {
        return false;
    }
    if let Some(action) = crate::core::computer_use::propose_from_text(user_text) {
        let proposal = crate::core::computer_use::create(
            &session_id.to_string(),
            user_id,
            "browser",
            action.clone(),
        )
        .await;
        let frame = serde_json::json!({
            "type": "browser_action",
            "action_type": "browser",
            "message_type": "browser_action",
            "bot_id": bot_uuid.to_string(),
            "session_id": session_id.to_string(),
            "browser_action": {
                "action_id": proposal.id,
                "action": "open_url",
                "url": action.get("url").cloned().unwrap_or_default(),
            },
        });
        let _ = sink.send_raw_json(&frame).await;
        return true;
    }
    if let Some(action) = crate::core::computer_use::propose_terminal_from_text(user_text) {
        let proposal = crate::core::computer_use::create(
            &session_id.to_string(),
            user_id,
            "terminal",
            action.clone(),
        )
        .await;
        let frame = serde_json::json!({
            "type": "terminal_action",
            "action_type": "terminal",
            "message_type": "terminal_action",
            "bot_id": bot_uuid.to_string(),
            "session_id": session_id.to_string(),
            "terminal_action": {
                "action_id": proposal.id,
                "command": action.get("command").cloned().unwrap_or_default(),
                "cwd": action.get("cwd").cloned().unwrap_or_default(),
                "reason": action.get("reason").cloned().unwrap_or_default(),
            },
        });
        let _ = sink.send_raw_json(&frame).await;
        return true;
    }
    false
}
