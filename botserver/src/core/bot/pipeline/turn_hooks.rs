//! Chat turn preludes split out of `exec.rs` (#1641, #1644, #1645).
//!
//! Attachments become prompt content; explicit URL/command requests become
//! computer-use proposals. Keeping them here keeps the pipeline entry point
//! readable and every file under the repository limit.

use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use serde_json::Value;
use uuid::Uuid;

use botcore::shared::state::AppState;

use super::types::SessionIdentity;

/// Stable media class for an attachment name.
fn media_kind(name: &str) -> Option<&'static str> {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "heic" => Some("image"),
        "mp4" | "mov" | "webm" | "avi" | "mkv" => Some("video"),
        "mp3" | "wav" | "ogg" | "m4a" | "flac" => Some("audio"),
        _ => None,
    }
}

fn media_mime(name: &str) -> &'static str {
    match name.rsplit('.').next().unwrap_or("").to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "heic" => "image/heic",
        "mp4" => "video/mp4",
        "mov" => "video/quicktime",
        "webm" => "video/webm",
        "avi" => "video/x-msvideo",
        "mkv" => "video/x-matroska",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        "flac" => "audio/flac",
        _ => "application/octet-stream",
    }
}

/// Resolve the local inference service endpoint and credentials.
///
/// Bot config wins; the process environment is the fallback because a
/// dev/test deployment sets `BOTMODELS_HOST`/`BOTMODELS_API_KEY` without
/// rewriting every bot's `config.csv`. BotModels is considered enabled when
/// the bot says so explicitly or when the environment points at it.
fn botmodels_endpoint(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
) -> Option<(String, String)> {
    use botcore::config::ConfigManager;
    let cfg = ConfigManager::new(state.conn.clone());

    // get_config returns Ok("") for a missing key, so distinguish an explicit
    // false from no setting before falling back to the environment.
    let enabled = match cfg.get_config(&bot_uuid, "botmodels-enabled", None) {
        Ok(value) if !value.trim().is_empty() => value.eq_ignore_ascii_case("true"),
        _ => std::env::var("BOTMODELS_HOST").is_ok(),
    };
    if !enabled {
        return None;
    }

    let base = cfg
        .get_config(&bot_uuid, "botmodels-host", None)
        .ok()
        .filter(|host| !host.trim().is_empty())
        .map(|host| {
            let host = if host == "0.0.0.0" {
                "127.0.0.1".to_string()
            } else {
                host
            };
            if host.starts_with("http://") || host.starts_with("https://") {
                host
            } else {
                let port = cfg
                    .get_config(&bot_uuid, "botmodels-port", Some("8085"))
                    .ok()
                    .filter(|port| !port.trim().is_empty())
                    .unwrap_or_else(|| "8085".to_string());
                format!("http://{host}:{port}")
            }
        })
        .or_else(|| std::env::var("BOTMODELS_HOST").ok())
        .unwrap_or_else(|| "http://127.0.0.1:8085".to_string());

    let key = cfg
        .get_config(&bot_uuid, "botmodels-api-key", None)
        .ok()
        .filter(|key| !key.trim().is_empty())
        .or_else(|| std::env::var("BOTMODELS_API_KEY").ok())
        .unwrap_or_default();

    Some((base.trim_end_matches('/').to_string(), key))
}

/// Transcribe/describe a staged media attachment through BotModels.
///
/// Returns the prompt block on success and `None` when the service is
/// disabled or the request fails; the caller then keeps the honest
/// "cannot read this media" note instead of inventing content.
async fn describe_media_locally(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    fname: &str,
    data: &[u8],
) -> Option<String> {
    let kind = media_kind(fname)?;
    let (base, key) = botmodels_endpoint(state, bot_uuid)?;
    let path = match kind {
        "audio" => "/api/speech/totext",
        "video" => "/api/vision/describe-video",
        _ => "/api/vision/describe",
    };
    let url = format!("{base}{path}");

    let part = reqwest::multipart::Part::bytes(data.to_vec())
        .file_name(fname.to_string())
        .mime_str(media_mime(fname))
        .ok()?;
    let mut form = reqwest::multipart::Form::new().part("file", part);
    if kind == "video" {
        form = form.text("num_frames", "6");
    }

    let response = reqwest::Client::new()
        .post(&url)
        .header("X-API-Key", key)
        .timeout(Duration::from_secs(300))
        .multipart(form)
        .send()
        .await;

    let response = match response {
        Ok(response) => response,
        Err(error) => {
            log::warn!("botmodels media request failed for {fname}: {error}");
            return None;
        }
    };
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        log::warn!("botmodels media request for {fname} returned {status}: {body}");
        return None;
    }

    let payload: Value = response.json().await.ok()?;
    let text = payload
        .get("text")
        .or_else(|| payload.get("description"))
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if text.is_empty() {
        return None;
    }

    Some(match kind {
        "audio" => format!(
            "The user attached audio '{fname}'. A local speech model produced this transcript. Use it to answer; never invent content that is not in the transcript.\n\n--- BEGIN TRANSCRIPT {fname} ---\n{text}\n--- END TRANSCRIPT {fname} ---"
        ),
        "video" => {
            let frames = payload
                .get("frame_count")
                .and_then(|value| value.as_u64())
                .unwrap_or(0);
            format!(
                "The user attached video '{fname}'. A local vision model sampled {frames} frames and described them. Use the description to answer; never invent content that is not in it.\n\n--- BEGIN VIDEO DESCRIPTION {fname} ---\n{text}\n--- END VIDEO DESCRIPTION {fname} ---"
            )
        }
        _ => format!(
            "The user attached image '{fname}'. A local vision model described it. Use the description to answer; never invent content that is not in it.\n\n--- BEGIN IMAGE DESCRIPTION {fname} ---\n{text}\n--- END IMAGE DESCRIPTION {fname} ---"
        ),
    })
}

pub(super) struct AttachmentResult {
    pub user_text: String,
    pub note: Option<String>,
}

/// Stores a base64 attachment under `inbox/` and produces the system block
/// that carries its extracted text (or an honest "cannot read this" note).
/// Stages one attachment (`file`) or every entry of a `files` array. The
/// composer sends the array when the user selects PDF + PNG in one turn; the
/// single-file shape stays for older clients and channel adapters.
pub(super) async fn ingest_attachment(
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
        note: if notes.is_empty() { None } else { Some(notes.join("\n\n")) },
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
            None if media => media_note.unwrap_or_else(|| format!(
                "The user attached the media file '{fname}' (stored at inbox/{fname}). Local transcription is unavailable, so you cannot read its pixels or audio. Answer only from the user's text and say plainly that you cannot see the file's content. Never invent what it shows."
            )),
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

#[cfg(test)]
mod media_tests {
    use super::{media_kind, media_mime};

    #[test]
    fn classifies_audio_video_and_image_extensions() {
        assert_eq!(media_kind("nota.ogg"), Some("audio"));
        assert_eq!(media_kind("clip.MP4"), Some("video"));
        assert_eq!(media_kind("foto.jpeg"), Some("image"));
        assert_eq!(media_kind("documento.pdf"), None);
    }

    #[test]
    fn mime_types_match_the_container() {
        assert_eq!(media_mime("nota.ogg"), "audio/ogg");
        assert_eq!(media_mime("clip.mp4"), "video/mp4");
        assert_eq!(media_mime("foto.png"), "image/png");
    }
}
