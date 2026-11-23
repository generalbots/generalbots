//! Encodes and decodes per-message metadata (reasoning, attachment details)
//! inside the encrypted content field (#2562).
//!
//! The metadata block is appended after the visible content, separated by a
//! marker that `get_conversation_history` strips before feeding the text to
//! the LLM (the model must never see its own reasoning replayed).
//!
//! Format: `<visible content>\n\x00GB_META\x00<json>`
//!
//! The NUL bytes make an accidental match in prose impossible. The JSON object
//! carries `reasoning` and, for user messages, `attachments`.

use serde::{Deserialize, Serialize};

/// The separator between visible content and the metadata JSON.
const META_SEPARATOR: &str = "\0GB_META\0";

/// Metadata stored alongside a message.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageMeta {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reasoning: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<AttachmentInfo>,
    /// Model-only record of what this turn received or did (a page read, a
    /// command run). Replayed with the message in later turns, never shown.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub model_note: String,
}

/// Attachment detail stored with user messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub mime_type: String,
    pub size: usize,
    pub drive_path: String,
}

/// Appends metadata to content for storage. Returns the combined string.
pub fn encode(content: &str, meta: &MessageMeta) -> String {
    if meta.reasoning.is_empty() && meta.attachments.is_empty() && meta.model_note.is_empty() {
        return content.to_string();
    }
    let json = match serde_json::to_string(meta) {
        Ok(j) => j,
        Err(e) => {
            log::warn!("message_meta encode failed: {e}");
            return content.to_string();
        }
    };
    format!("{content}\n{META_SEPARATOR}{json}")
}

/// What is stored as the answer of a turn the person stopped: the text that
/// was on screen, or an ellipsis when none had arrived yet (a mark that reads
/// the same in every language).
pub fn stopped_answer(partial: &str, reasoning: &str) -> String {
    let visible = if partial.trim().is_empty() {
        "…"
    } else {
        partial.trim_end()
    };
    let meta = MessageMeta {
        reasoning: reasoning.trim().to_string(),
        ..Default::default()
    };
    encode(visible, &meta)
}

/// Splits stored content into (visible_content, metadata).
pub fn decode(stored: &str) -> (&str, Option<MessageMeta>) {
    if let Some(pos) = stored.find(META_SEPARATOR) {
        let content = stored[..pos].trim_end_matches('\n');
        let json_part = &stored[pos + META_SEPARATOR.len()..];
        let meta = serde_json::from_str(json_part).ok();
        (content, meta)
    } else {
        (stored, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stopped_turn_keeps_what_was_on_screen() {
        let stored = stopped_answer("Half an answer \n", " thinking ");
        let (content, meta) = decode(&stored);
        assert_eq!(content, "Half an answer");
        assert_eq!(meta.map(|m| m.reasoning), Some("thinking".to_string()));
    }

    #[test]
    fn a_turn_stopped_before_any_text_leaves_a_mark() {
        assert_eq!(stopped_answer("  ", ""), "…");
        let stored = stopped_answer("", "was thinking");
        let (content, meta) = decode(&stored);
        assert_eq!(content, "…");
        assert_eq!(meta.map(|m| m.reasoning), Some("was thinking".to_string()));
    }

    #[test]
    fn roundtrip_with_reasoning() {
        let meta = MessageMeta {
            reasoning: "I think this is about X".to_string(),
            ..Default::default()
        };
        let encoded = encode("Hello world", &meta);
        let (content, decoded) = decode(&encoded);
        assert_eq!(content, "Hello world");
        let m = decoded.expect("should decode");
        assert_eq!(m.reasoning, "I think this is about X");
    }

    #[test]
    fn roundtrip_with_attachments() {
        let meta = MessageMeta {
            reasoning: String::new(),
            attachments: vec![AttachmentInfo {
                name: "doc.pdf".to_string(),
                mime_type: "application/pdf".to_string(),
                size: 1024,
                drive_path: "inbox/doc.pdf".to_string(),
            }],
            ..Default::default()
        };
        let encoded = encode("check this file", &meta);
        let (content, decoded) = decode(&encoded);
        assert_eq!(content, "check this file");
        let m = decoded.expect("should decode");
        assert_eq!(m.attachments.len(), 1);
        assert_eq!(m.attachments[0].name, "doc.pdf");
        assert_eq!(m.attachments[0].size, 1024);
    }

    #[test]
    fn no_meta_passthrough() {
        let meta = MessageMeta::default();
        let encoded = encode("plain text", &meta);
        assert_eq!(encoded, "plain text");
        let (content, decoded) = decode(&encoded);
        assert_eq!(content, "plain text");
        assert!(decoded.is_none());
    }

    #[test]
    fn decode_legacy_content() {
        let (content, meta) = decode("old message without metadata");
        assert_eq!(content, "old message without metadata");
        assert!(meta.is_none());
    }
}
