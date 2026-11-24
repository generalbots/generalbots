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
    /// When set, the attachment was stored but its media content could not be
    /// read; the value is a short reason code (e.g. `image_reader_unavailable`).
    pub media_fail_reason: Option<&'static str>,
    /// Images and PDFs for a model that can see them (`botllm::media_parts`).
    pub media: Vec<Value>,
}

/// Largest image a vision model receives (Anthropic's limit per image).
const MAX_VISION_IMAGE: usize = 5 * 1024 * 1024;
/// Largest PDF Claude receives as a document.
const MAX_VISION_PDF: usize = 32 * 1024 * 1024;

fn image_media_type(name: &str) -> Option<&'static str> {
    let lower = name.to_lowercase();
    if lower.ends_with(".png") {
        Some("image/png")
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        Some("image/jpeg")
    } else if lower.ends_with(".gif") {
        Some("image/gif")
    } else if lower.ends_with(".webp") {
        Some("image/webp")
    } else {
        None
    }
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
    vision: bool,
) -> AttachmentResult {
    let files: Vec<Value> = parsed
        .get("files")
        .and_then(Value::as_array)
        .cloned()
        .map_or_else(Default::default, |v| v);
    if files.is_empty() {
        return ingest_single_attachment(state, bot_uuid, bot_name, parsed, user_text, vision).await;
    }
    let mut text = user_text;
    let mut notes: Vec<String> = Vec::new();
    let mut media: Vec<Value> = Vec::new();
    for file in files {
        let single = serde_json::json!({ "file": file });
        let result = ingest_single_attachment(state, bot_uuid, bot_name, &single, text, vision).await;
        text = result.user_text;
        media.extend(result.media);
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
        media_fail_reason: None,
        media,
    }
}

async fn ingest_single_attachment(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    bot_name: &str,
    parsed: &Value,
    user_text: String,
    vision: bool,
) -> AttachmentResult {
    let mut user_text = user_text;
    let mut attachment_note: Option<String> = None;
    let Some(file) = parsed.get("file").and_then(|value| value.as_object()) else {
        return AttachmentResult {
            user_text,
            note: attachment_note,
            media_fail_reason: None,
            media: Vec::new(),
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
            media_fail_reason: None,
            media: Vec::new(),
        };
    }
    // #1947 — the composer sends attachments inline as base64. A file that is
    // too large never reaches the server intact, and the user only saw their
    // message without the attachment. Report the limit explicitly.
    const MAX_ATTACHMENT_B64: usize = 68 * 1024 * 1024; // 50 MB of file as base64
    if b64.len() > MAX_ATTACHMENT_B64 {
        log::warn!(
            "chat attachment too large: {fname} ({} base64 bytes, limit {MAX_ATTACHMENT_B64})",
            b64.len()
        );
        user_text = format!(
            "{user_text}\n\n📎 {fname} (larger than 50 MB and was not stored — share it from Drive or split it)"
        );
        return AttachmentResult {
            user_text,
            note: attachment_note,
            media_fail_reason: None,
            media: Vec::new(),
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
                media_fail_reason: None,
                media: Vec::new(),
            };
        }
    };

    let mut excerpt = super::super::attachment_text::extract_excerpt(&fname, &data);
    let media = super::super::attachment_text::is_media(&fname);
    // An image, or a PDF that is only scanned pages, has no text layer: read
    // the text off the pixels on this machine.
    let image = super::media::media_kind(&fname) == Some("image");
    let pdf = fname.to_lowercase().ends_with(".pdf");
    let recognised = if excerpt.is_none() && (image || pdf) {
        super::text_recognition::recognize(&fname, data.clone()).await
    } else {
        None
    };
    let read_by_ocr = recognised.is_some();
    // A model that can see gets the file itself too: symbols, stamps,
    // signatures, layout and pictures are lost in plain text.
    let mut vision_media: Vec<Value> = Vec::new();
    if vision {
        if let Some(mt) = image_media_type(&fname).filter(|_| data.len() <= MAX_VISION_IMAGE) {
            vision_media.push(serde_json::json!({ "type": "image", "media_type": mt, "data": b64 }));
        } else if pdf && data.len() <= MAX_VISION_PDF {
            vision_media.push(serde_json::json!({ "type": "pdf", "media_type": "application/pdf", "data": b64 }));
        }
    }
    let sees_file = !vision_media.is_empty();
    let seen = if sees_file {
        " The file itself is also attached to this message: look at it and use what you see (symbols, stamps, signatures, layout, colors, pictures), not only the text."
    } else {
        ""
    };
    if recognised.is_some() {
        excerpt = recognised;
    }
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
                // A raw storage error is S3 XML with bucket details: never put
                // it in the prompt. Use a fixed reason code instead.
                let reason = if error.to_string().contains("StorageFull") {
                    "the disk of this computer is almost full".to_string()
                } else {
                    "storage_error".to_string()
                };
                user_text =
                    format!("{user_text}\n\n📎 {fname} (the upload could not be stored: {reason})");
            }
        }
    } else {
        user_text = format!(
            "{user_text}\n\n📎 {fname} (Drive is not configured, so the file was not stored)"
        );
    }
    // Media analysis runs after the file is reachable in the inbox.
    // #2561 — the gbgov edition ships without the BotModels media service;
    // skip the request entirely to avoid a connection-refused delay.
    let media_note = if media && excerpt.is_none() && !cfg!(feature = "gbgov") {
        describe_media_locally(state, bot_uuid, &fname, &data).await
    } else {
        None
    };
    let mut media_fail_reason: Option<&'static str> = None;
    if stored {
        // The pipeline detects a staged upload by this marker and injects its
        // inbox path into tool calls (classify_media). The bare "📎 name" line
        // alone left every web upload without a path, so media tools never ran.
        user_text = format!(
            "{user_text}\n\n[User attached a file stored at inbox/{fname}; size={}; the upload was staged for the bot]\n📎 {fname}",
            data.len()
        );
        attachment_note = Some(match excerpt {
            Some(text) if read_by_ocr && sees_file => format!(
                "The user attached '{fname}'. The file is stored at inbox/{fname}.{seen} Text recognition (OCR) on this computer also read the text below from its pixels; it may contain recognition errors, so trust what you see where they differ.\n\n--- BEGIN ATTACHMENT {fname} ---\n{text}\n--- END ATTACHMENT {fname} ---"
            ),
            Some(text) if read_by_ocr => format!(
                "The user attached '{fname}'. The file is stored at inbox/{fname}. It has no text layer, so text recognition (OCR) on this computer read the text below from its pixels: it may contain recognition errors, and pictures, stamps, signatures and layout are not described. Use it to answer; never claim content that is not in it.\n\n--- BEGIN ATTACHMENT {fname} ---\n{text}\n--- END ATTACHMENT {fname} ---"
            ),
            Some(text) => format!(
                "The user attached '{fname}'. The file is stored at inbox/{fname} and its extracted text is reproduced below.{seen} Use it to answer; never claim content that is not in it.\n\n--- BEGIN ATTACHMENT {fname} ---\n{text}\n--- END ATTACHMENT {fname} ---"
            ),
            None if sees_file => format!(
                "The user attached '{fname}' (stored at inbox/{fname}). No text was found in it, but the file itself is attached to this message: look at it and answer from what you see. Never invent details you cannot see."
            ),
            None if media => media_note.unwrap_or_else(|| {
                // #2561 — determine whether any image-processing path is
                // actually available before telling the model it can use one.
                let has_vm = cfg!(feature = "agent-vm");
                let agent_on = has_vm
                    && parsed
                        .get("agent_enabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                if agent_on {
                    format!(
                        "The user attached the media file '{fname}' (stored at inbox/{fname}). Local transcription (BotModels) is unavailable, but the Agent/VM is enabled for this conversation: you may process the file from the VM (reachable at inbox/{fname}) to extract its content. Do not invent what it shows; only report what the VM actually extracts."
                    )
                } else {
                    media_fail_reason = Some("image_reader_unavailable");
                    let kind = super::media::media_kind(&fname).unwrap_or("media");
                    if cfg!(windows) && kind == "image" {
                        return format!(
                            "The user attached the image '{fname}' (stored at inbox/{fname}). \
                             Text recognition on this computer found no readable text in it, and \
                             describing pictures is not available in this installation. Tell the \
                             user in one or two sentences that you found no text in the image and \
                             cannot describe what it shows, and suggest they describe it if they \
                             need help with it. Do NOT call any tool or command to look for the \
                             file. Never invent what it shows."
                        );
                    }
                    format!(
                        "The user attached the {kind} file '{fname}' (stored at inbox/{fname}). \
                         Image reading is not available in this installation and the current model \
                         does not receive images. Tell the user in one or two sentences that you \
                         cannot see the file's visual content because image reading is not installed, \
                         and suggest they describe what is in the {kind} if they need help with it. \
                         Do NOT call any tool or command to look for the file. \
                         Never invent what it shows."
                    )
                }
            }),
            None => format!(
                "The user attached '{fname}' (stored at inbox/{fname}). No text extractor is available for this format, so you cannot read its content. Say the file is stored and ask the user to paste the relevant text (or to convert it). Never invent its content."
            ),
        });
    } else if let Some(text) = excerpt {
        // The file could not be kept (Drive full or missing) but its text was
        // already read: the person still gets an answer about it.
        let how = if read_by_ocr {
            "It has no text layer, so text recognition (OCR) on this computer read the text below from its pixels: it may contain recognition errors."
        } else {
            "Its extracted text is reproduced below."
        };
        let how = format!("{how}{seen}");
        attachment_note = Some(format!(
            "The user attached '{fname}'. The file could not be stored, so it is not in Drive and no tool can open it; tell the user in one short sentence that the file was read but not saved. {how} Use it to answer; never claim content that is not in it.\n\n--- BEGIN ATTACHMENT {fname} ---\n{text}\n--- END ATTACHMENT {fname} ---"
        ));
    } else if sees_file {
        attachment_note = Some(format!(
            "The user attached '{fname}'. The file could not be stored, so it is not in Drive; tell the user in one short sentence that it was not saved. No text was found in it, but the file itself is attached to this message: look at it and answer from what you see. Never invent details you cannot see."
        ));
    }
    AttachmentResult {
        user_text,
        note: attachment_note,
        media_fail_reason,
        media: vision_media,
    }
}
