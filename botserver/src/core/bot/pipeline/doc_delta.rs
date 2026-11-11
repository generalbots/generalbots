//! gb-doc-delta/v1 protocol types and emit helpers (#2172).
//! The VM runtime emits incremental document ops over the chat session WS so
//! the side-by-side app renders generation live (SVG-style).

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocDeltaEnvelope {
    pub event: String, // "gb-doc-delta"
    pub v: u32,        // 1
    pub file: String,
    pub app: String,
    pub rev: u64,
    pub ops: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bucket: Option<String>,
}

impl DocDeltaEnvelope {
    pub fn new(file: impl Into<String>, app: impl Into<String>, rev: u64, ops: Vec<Value>) -> Self {
        Self {
            event: "gb-doc-delta".to_string(),
            v: 1,
            file: file.into(),
            app: app.into(),
            rev,
            ops,
            bucket: None,
        }
    }
    pub fn finalize(file: impl Into<String>, app: impl Into<String>, rev: u64) -> Self {
        Self::new(file, app, rev, vec![serde_json::json!({"op":"finalize"})])
    }
    pub fn error(file: impl Into<String>, app: impl Into<String>, rev: u64, message: impl Into<String>) -> Self {
        Self::new(file, app, rev, vec![serde_json::json!({"op":"error","message": message.into()})])
    }
}

/// Best-effort broadcast of a delta envelope to all WS subscribers of a session.
/// Implementations may route through the session bus/sink; if unavailable this
/// is a no-op (fallback: finalize-only file link in chat).
pub async fn emit(
    state: &std::sync::Arc<botcore::shared::state::AppState>,
    session_id: uuid::Uuid,
    envelope: &DocDeltaEnvelope,
) {
    // #2179: publish the envelope to the session bus so every subscribed
    // socket (and the side-by-side app window) receives the live ops.
    if let Ok(json) = serde_json::to_value(envelope) {
        let thread_id = crate::core::bot::runs::bus::thread_id(session_id);
        let delivered = crate::core::bot::runs::bus::bus().publish(&thread_id, json);
        log::debug!(
            "gb-doc-delta published app={} file={} rev={} delivered={}",
            envelope.app,
            envelope.file,
            envelope.rev,
            delivered
        );
        let _ = state;
    }
}

/// Infer target app id from a generated filename.
pub fn app_for_filename(name: &str) -> &'static str {
    match name.rsplit('.').next().unwrap_or("").to_lowercase().as_str() {
        "docx" | "doc" | "odt" | "rtf" => "docs",
        "xlsx" | "xls" | "ods" | "csv" => "sheet",
        "pptx" | "ppt" | "odp" => "slides",
        "svg" => "canvas",
        "md" | "txt" => "paper",
        _ => "preview",
    }
}

/// Ceiling on the bytes read back for a content delta (128 KiB) and on the
/// blocks emitted from them, so a huge markdown artifact cannot flood the bus.
const MAX_CONTENT_BYTES: usize = 128 * 1024;
const MAX_CONTENT_BLOCKS: usize = 400;

/// True when the artifact has a renderer that takes block ops straight from
/// text (`paper` renders markdown/plain text as blocks). Binary office formats
/// need a document parser and stay finalize-only for now.
pub fn is_text_artifact(name: &str) -> bool {
    matches!(
        name.rsplit('.').next().unwrap_or("").to_lowercase().as_str(),
        "md" | "markdown" | "txt"
    )
}

/// Builds `clear` + one `append_block` per paragraph for a text artifact.
///
/// #2235 F1 — the transport (session bus → WS → `GBDocDelta`) and the paper
/// renderer existed without a producer building anything but `finalize`, so
/// the app window opened empty. This is the first real op producer; an empty
/// or unparsable artifact yields no ops and the caller falls back to finalize.
pub fn text_block_ops(bytes: &[u8]) -> Vec<Value> {
    let slice = &bytes[..bytes.len().min(MAX_CONTENT_BYTES)];
    let text = String::from_utf8_lossy(slice);
    if text.trim().is_empty() {
        return Vec::new();
    }
    let mut ops = vec![serde_json::json!({"op":"clear"})];
    for (index, block) in text
        .split("\n\n")
        .map(str::trim)
        .filter(|block| !block.is_empty())
        .take(MAX_CONTENT_BLOCKS)
        .enumerate()
    {
        ops.push(serde_json::json!({
            "op": "append_block",
            "id": format!("gbd-{index}"),
            "type": "paragraph",
            "text": block,
        }));
    }
    if ops.len() == 1 {
        return Vec::new();
    }
    ops
}

#[cfg(test)]
mod tests {
    use super::text_block_ops;

    #[test]
    fn text_artifact_becomes_block_ops() {
        let ops = text_block_ops(b"First paragraph.\n\nSecond paragraph.");
        assert_eq!(ops.len(), 3, "clear + one append_block per paragraph");
        assert_eq!(ops[0]["op"], "clear");
        assert_eq!(ops[1]["op"], "append_block");
        assert_eq!(ops[1]["text"], "First paragraph.");
        assert_eq!(ops[2]["text"], "Second paragraph.");
    }

    #[test]
    fn empty_artifact_emits_finalize_only() {
        assert!(text_block_ops(b"   \n\n\t").is_empty());
    }

    #[test]
    fn text_ops_are_capped() {
        let body = "para\n\n".repeat(1_000);
        let ops = text_block_ops(body.as_bytes());
        assert_eq!(ops.len(), 401, "clear + the 400-block ceiling");
    }
}
