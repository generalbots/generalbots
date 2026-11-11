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
