//! Telegram inbound media ingestion.
//!
//! Photos, documents, voice notes and videos are fetched through the Bot API
//! and stored in the bot's Drive `inbox/` directory. The conversation then
//! carries an `[image] inbox/...` or `[document] inbox/...` marker so a BASIC
//! script can hand the path to `CLASSIFY IMAGE`, `GET` or a filing tool.

use crate::adapter::TelegramAdapter;
use crate::media_names::stored_file_name;
use crate::state::ChannelState;
use crate::webhook::{extract_message_content, TelegramMessage};
use log::{info, warn};
use std::sync::Arc;
use uuid::Uuid;

/// Telegram refuses to serve files larger than 20 MB through `getFile`.
pub const MAX_DOWNLOAD_BYTES: i64 = 20 * 1024 * 1024;

/// Directory inside `{bot}.gbdrive` where inbound media is staged before a
/// filing tool moves it into its final folder.
const INBOX_PREFIX: &str = "inbox";

/// Describes the media carried by one Telegram update.
pub(crate) struct InboundMedia {
    pub(crate) kind: &'static str,
    pub(crate) file_id: String,
    pub(crate) file_name: Option<String>,
    pub(crate) mime_type: Option<String>,
    pub(crate) file_size: Option<i64>,
}

/// Builds the conversation content for an update, fetching any media into Drive
/// first. Falls back to the plain text/caption extraction when the update
/// carries no attachment.
pub async fn message_content(
    state: &Arc<ChannelState>,
    adapter: &TelegramAdapter,
    message: &TelegramMessage,
    bot_id: Uuid,
) -> String {
    let Some(media) = select_media(message) else {
        return extract_message_content(message);
    };

    let caption = message.caption.as_deref().unwrap_or("").trim();

    let stored = match media.file_size {
        Some(size) if size > MAX_DOWNLOAD_BYTES => Err(format!(
            "larger than the {MAX_DOWNLOAD_BYTES} byte Telegram download limit"
        )),
        _ => store_media(state, adapter, bot_id, &media).await,
    };

    match stored {
        Ok(path) => {
            info!("Stored Telegram {} for bot {bot_id} at {path}", media.kind);
            marker(media.kind, &path, caption)
        }
        Err(reason) => {
            warn!("Telegram {} for bot {bot_id} not stored: {reason}", media.kind);
            marker(media.kind, &format!("(not stored: {reason})"), caption)
        }
    }
}

fn select_media(message: &TelegramMessage) -> Option<InboundMedia> {
    if let Some(photos) = message.photo.as_ref() {
        // Telegram orders the sizes ascending, so the last entry is the largest.
        if let Some(largest) = photos.last() {
            return Some(InboundMedia {
                kind: "image",
                file_id: largest.file_id.clone(),
                file_name: None,
                mime_type: Some("image/jpeg".to_string()),
                file_size: largest.file_size,
            });
        }
    }

    if let Some(document) = message.document.as_ref() {
        return Some(InboundMedia {
            kind: "document",
            file_id: document.file_id.clone(),
            file_name: document.file_name.clone(),
            mime_type: document.mime_type.clone(),
            file_size: document.file_size,
        });
    }

    if let Some(voice) = message.voice.as_ref() {
        return Some(InboundMedia {
            kind: "voice",
            file_id: voice.file_id.clone(),
            file_name: None,
            mime_type: voice.mime_type.clone(),
            file_size: voice.file_size,
        });
    }

    if let Some(audio) = message.audio.as_ref() {
        return Some(InboundMedia {
            kind: "audio",
            file_id: audio.file_id.clone(),
            file_name: audio.title.clone(),
            mime_type: audio.mime_type.clone(),
            file_size: audio.file_size,
        });
    }

    if let Some(video) = message.video.as_ref() {
        return Some(InboundMedia {
            kind: "video",
            file_id: video.file_id.clone(),
            file_name: None,
            mime_type: video.mime_type.clone(),
            file_size: video.file_size,
        });
    }

    None
}

async fn store_media(
    state: &Arc<ChannelState>,
    adapter: &TelegramAdapter,
    bot_id: Uuid,
    media: &InboundMedia,
) -> Result<String, String> {
    let file_path = adapter
        .get_file(&media.file_id)
        .await
        .map_err(|e| format!("getFile failed: {e}"))?;

    let bytes = adapter
        .download_file(&file_path)
        .await
        .map_err(|e| format!("download failed: {e}"))?;

    if bytes.is_empty() {
        return Err("Telegram returned an empty file".to_string());
    }

    let relative_path = format!("{INBOX_PREFIX}/{}", stored_file_name(media));
    let content_type = media
        .mime_type
        .clone()
        .unwrap_or_else(|| "application/octet-stream".to_string());

    (state.put_media)(bot_id, relative_path, bytes, Some(content_type)).await
}

fn marker(kind: &str, target: &str, caption: &str) -> String {
    if caption.is_empty() {
        format!("[{kind}] {target}")
    } else {
        format!("[{kind}] {target}\n{caption}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webhook::{TelegramChat, TelegramPhotoSize};

    fn message_with_photo(photos: Vec<TelegramPhotoSize>) -> TelegramMessage {
        TelegramMessage {
            message_id: 1,
            from: None,
            chat: TelegramChat {
                id: -100,
                chat_type: "private".to_string(),
                title: None,
                username: None,
                first_name: None,
                last_name: None,
            },
            date: 0,
            text: None,
            photo: Some(photos),
            document: None,
            voice: None,
            audio: None,
            video: None,
            location: None,
            contact: None,
            caption: None,
        }
    }

    fn photo(file_id: &str, width: i32) -> TelegramPhotoSize {
        TelegramPhotoSize {
            file_id: file_id.to_string(),
            file_unique_id: format!("{file_id}-unique"),
            width,
            height: width,
            file_size: Some(i64::from(width)),
        }
    }

    #[test]
    fn picks_the_largest_photo_size() {
        let message = message_with_photo(vec![photo("small", 90), photo("large", 1280)]);
        let selected = select_media(&message);

        assert_eq!(selected.map(|media| media.file_id), Some("large".to_string()));
    }

    #[test]
    fn text_only_message_selects_no_media() {
        let mut message = message_with_photo(Vec::new());
        message.photo = None;
        message.text = Some("bom dia".to_string());

        assert!(select_media(&message).is_none());
        assert_eq!(extract_message_content(&message), "bom dia");
    }

    #[test]
    fn marker_without_caption_is_a_single_line() {
        assert_eq!(marker("image", "inbox/a1.jpg", ""), "[image] inbox/a1.jpg");
    }

    #[test]
    fn marker_appends_the_caption() {
        assert_eq!(
            marker("document", "inbox/b2-contrato.pdf", "nota fiscal"),
            "[document] inbox/b2-contrato.pdf\nnota fiscal"
        );
    }

    #[test]
    fn marker_reports_media_that_could_not_be_stored() {
        assert_eq!(
            marker("image", "(not stored: download failed)", ""),
            "[image] (not stored: download failed)"
        );
    }
}
