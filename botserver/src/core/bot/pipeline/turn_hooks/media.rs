//! Attachment classification, local inference calls and prompt blocks.

use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use serde_json::Value;
use uuid::Uuid;

use botcore::shared::state::AppState;

/// Stable media class for an attachment name.
pub(crate) fn media_kind(name: &str) -> Option<&'static str> {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "heic" => Some("image"),
        "mp4" | "mov" | "webm" | "avi" | "mkv" => Some("video"),
        "mp3" | "wav" | "ogg" | "m4a" | "flac" => Some("audio"),
        _ => None,
    }
}

pub(crate) fn media_mime(name: &str) -> &'static str {
    match name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
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
fn botmodels_endpoint(state: &Arc<AppState>, bot_uuid: Uuid) -> Option<(String, String)> {
    use botcore::config::ConfigManager;
    let cfg = ConfigManager::new(state.conn.clone());

    // get_config returns Ok("") for a missing key, so distinguish an explicit
    // false from no setting before falling back to the environment.
    // #1911 — default to enabled so STT/vision work when BotModels is reachable,
    // even if the bot has no explicit botmodels-enabled config entry. The
    // endpoint resolution below falls back to localhost:8085.
    let enabled = match cfg.get_config(&bot_uuid, "botmodels-enabled", None) {
        Ok(value) if !value.trim().is_empty() => value.eq_ignore_ascii_case("true"),
        _ => true,
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
pub(crate) async fn describe_media_locally(
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
