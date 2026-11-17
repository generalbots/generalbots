//! Typed activity events for the chat client (#2562).
//!
//! Each activity the assistant performs during a turn (reading an attachment,
//! searching the knowledge base, running an API command, executing a dialog
//! tool, requesting consent) emits a start/end pair through the existing
//! `event_sink::emit_custom` mechanism (`gb_event` frames with `"type":
//! "CUSTOM"`). The client renders each as it arrives; the history endpoint
//! returns persisted events alongside messages so a reopened conversation
//! shows them again.
//!
//! Activities use the `gb.activity` custom-event name. The `data` payload
//! carries a stable `kind`, a `status` ("started" or "completed" / "failed"),
//! a unique `id` that pairs start with end, and a machine-readable `summary`
//! the client can render without parsing prose.
//!
//! **Caps:** list sizes max 20 entries, excerpts max 200 chars, file paths
//! max 256 chars. Nothing secret (tokens, passwords, full file contents)
//! ever appears in an activity.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::event_sink::{can_emit, emit_custom, RunContext};
use super::sink::ChannelSink;

/// Maximum entries in a list summary (Drive entries, KB results, etc.).
pub const MAX_LIST_ENTRIES: usize = 20;
/// Maximum characters in an excerpt field.
pub const MAX_EXCERPT_CHARS: usize = 200;
/// Maximum characters in a path field.
pub const MAX_PATH_CHARS: usize = 256;

/// The kinds of activity the assistant performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityKind {
    AttachmentRead,
    DriveList,
    DriveSearch,
    DriveRead,
    KbSearch,
    ToolRun,
    ApiCommand,
    Permission,
}

/// Status of an activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityStatus {
    Started,
    Completed,
    Failed,
    /// Consent: waiting for the user to approve or deny.
    NeedsInput,
}

/// A typed activity event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub kind: ActivityKind,
    pub status: ActivityStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Machine-readable summary: shape depends on `kind`.
    pub summary: Value,
    /// The run this activity belongs to so the client can group it with a turn.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    /// The message index this activity precedes (set by history merge).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_index: Option<i32>,
}

impl Activity {
    /// Creates a started activity with the given kind and summary.
    pub fn started(kind: ActivityKind, summary: Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            kind,
            status: ActivityStatus::Started,
            error: None,
            summary,
            run_id: None,
            message_index: None,
        }
    }

    /// Returns a completed copy of this activity with an updated summary.
    pub fn completed(&self, summary: Value) -> Self {
        Self {
            id: self.id.clone(),
            kind: self.kind,
            status: ActivityStatus::Completed,
            error: None,
            summary,
            run_id: self.run_id.clone(),
            message_index: self.message_index,
        }
    }

    /// Returns a failed copy with a sanitised error message.
    pub fn failed(&self, error: &str) -> Self {
        Self {
            id: self.id.clone(),
            kind: self.kind,
            status: ActivityStatus::Failed,
            error: Some(truncate(error, MAX_EXCERPT_CHARS)),
            summary: self.summary.clone(),
            run_id: self.run_id.clone(),
            message_index: self.message_index,
        }
    }

    /// Returns a `needs_input` copy (consent pending).
    pub fn needs_input(&self, summary: Value) -> Self {
        Self {
            id: self.id.clone(),
            kind: self.kind,
            status: ActivityStatus::NeedsInput,
            error: None,
            summary,
            run_id: self.run_id.clone(),
            message_index: self.message_index,
        }
    }

    /// Sets the run_id on this activity.
    pub fn with_run_id(mut self, run_id: &str) -> Self {
        self.run_id = Some(run_id.to_string());
        self
    }
}

/// Emits an activity as a `gb.activity` custom event.
pub async fn emit_activity(sink: &dyn ChannelSink, context: &RunContext, activity: &Activity) {
    if !can_emit(sink) {
        return;
    }
    let payload = match serde_json::to_value(activity) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("activity serialization failed: {e}");
            return;
        }
    };
    emit_custom(sink, context, "gb.activity", payload).await;
}

// --- Summary builders (cap-enforcing) ---

/// Attachment summary for `attachment_read`.
pub fn attachment_summary(
    name: &str,
    mime: &str,
    size_bytes: usize,
    chars_read: usize,
    truncated: bool,
    drive_path: &str,
) -> Value {
    serde_json::json!({
        "name": truncate(name, MAX_PATH_CHARS),
        "type": truncate(mime, 64),
        "size": size_bytes,
        "chars_read": chars_read,
        "truncated": truncated,
        "drive_path": truncate(drive_path, MAX_PATH_CHARS),
    })
}

/// KB search summary for `kb_search`.
pub fn kb_search_summary(query: &str, sources: &[KbSource]) -> Value {
    let capped: Vec<Value> = sources
        .iter()
        .take(MAX_LIST_ENTRIES)
        .map(|s| {
            serde_json::json!({
                "document": truncate(&s.document, MAX_PATH_CHARS),
                "excerpt": truncate(&s.excerpt, MAX_EXCERPT_CHARS),
                "score": s.score,
            })
        })
        .collect();
    serde_json::json!({
        "query": truncate(query, MAX_EXCERPT_CHARS),
        "results": capped,
        "total": sources.len(),
    })
}

/// One knowledge-base source returned by a search.
pub struct KbSource {
    pub document: String,
    pub excerpt: String,
    pub score: f32,
}

/// API command summary for `api_command`.
pub fn api_command_summary(name: &str, status: &str) -> Value {
    serde_json::json!({
        "command": truncate(name, 128),
        "status": status,
    })
}

/// Tool run summary for `tool_run`.
pub fn tool_run_summary(name: &str, outcome: &str) -> Value {
    serde_json::json!({
        "tool": truncate(name, 128),
        "outcome": truncate(outcome, MAX_EXCERPT_CHARS),
    })
}

/// Consent/permission summary for `permission`.
pub fn permission_summary(action: &str, decision: &str, request_id: Option<&str>) -> Value {
    serde_json::json!({
        "action": truncate(action, MAX_EXCERPT_CHARS),
        "decision": decision,
        "request_id": request_id,
    })
}

/// Merges start/end activity pairs into a single object per id, keeping the
/// final status, both timestamps and the duration. Returns the merged list
/// ordered by first-seen timestamp.
pub fn merge_activity_pairs(events: Vec<(Activity, String)>) -> Vec<Value> {
    use std::collections::HashMap;
    let mut map: HashMap<String, MergedActivity> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    for (act, ts) in events {
        let entry = map.entry(act.id.clone()).or_insert_with(|| {
            order.push(act.id.clone());
            MergedActivity {
                id: act.id.clone(),
                kind: act.kind,
                status: act.status,
                error: act.error.clone(),
                summary: act.summary.clone(),
                run_id: act.run_id.clone(),
                message_index: act.message_index,
                started_at: ts.clone(),
                ended_at: ts.clone(),
            }
        });
        // Later events (completed/failed) update the entry.
        if act.status != ActivityStatus::Started {
            entry.status = act.status;
            entry.summary = act.summary;
            entry.error = act.error;
            entry.ended_at = ts;
        }
    }
    order
        .iter()
        .filter_map(|id| map.remove(id))
        .map(|m| {
            let started =
                chrono::NaiveDateTime::parse_from_str(&m.started_at, "%Y-%m-%d %H:%M:%S%.f")
                    .or_else(|_| {
                        chrono::NaiveDateTime::parse_from_str(&m.started_at, "%Y-%m-%dT%H:%M:%S%.f")
                    });
            let ended = chrono::NaiveDateTime::parse_from_str(&m.ended_at, "%Y-%m-%d %H:%M:%S%.f")
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(&m.ended_at, "%Y-%m-%dT%H:%M:%S%.f")
                });
            let duration_ms = match (started.ok(), ended.ok()) {
                (Some(s), Some(e)) => {
                    let d = e.signed_duration_since(s);
                    Some(d.num_milliseconds().max(0))
                }
                _ => None,
            };
            serde_json::json!({
                "id": m.id,
                "kind": m.kind,
                "status": m.status,
                "error": m.error,
                "summary": m.summary,
                "run_id": m.run_id,
                "message_index": m.message_index,
                "started_at": m.started_at,
                "ended_at": m.ended_at,
                "duration_ms": duration_ms,
            })
        })
        .collect()
}

struct MergedActivity {
    id: String,
    kind: ActivityKind,
    status: ActivityStatus,
    error: Option<String>,
    summary: Value,
    run_id: Option<String>,
    message_index: Option<i32>,
    started_at: String,
    ended_at: String,
}

/// Drive listing summary for `drive_list` / `drive_search`.
pub fn drive_list_summary(path: &str, entries: &[DriveEntry]) -> Value {
    let capped: Vec<Value> = entries
        .iter()
        .take(MAX_LIST_ENTRIES)
        .map(|e| {
            serde_json::json!({
                "name": truncate(&e.name, MAX_PATH_CHARS),
                "size": e.size,
                "kind": e.kind,
            })
        })
        .collect();
    serde_json::json!({
        "path": truncate(path, MAX_PATH_CHARS),
        "entries": capped,
        "total": entries.len(),
    })
}

/// One Drive entry in a listing.
pub struct DriveEntry {
    pub name: String,
    pub size: u64,
    pub kind: String,
}

/// Attachment metadata stored with a user message so history can show it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentMeta {
    pub name: String,
    #[serde(rename = "type")]
    pub mime_type: String,
    pub size: usize,
    pub drive_path: String,
    pub chars_read: usize,
    pub truncated: bool,
}

/// Guesses a MIME type from a file name extension.
pub fn mime_from_name(name: &str) -> String {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "txt" | "md" | "csv" | "log" => "text/plain",
        "json" => "application/json",
        "html" | "htm" => "text/html",
        "xml" => "application/xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "mp4" => "video/mp4",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        _ => "application/octet-stream",
    }
    .to_string()
}

/// Truncates a string to `max` characters, appending an ellipsis marker.
pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{truncated}\u{2026}")
    }
}

#[cfg(test)]
#[path = "activity_tests.rs"]
mod tests;
