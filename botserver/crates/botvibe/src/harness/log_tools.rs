//! #747 — Log tools: inspect runtime logs inside the project workspace.

use crate::harness::read_rel_file;
use crate::tool_executor::ToolHandler;
use crate::types::{VibeState, VibeToolResult};
use serde_json::json;
use std::sync::Arc;

fn ok(data: serde_json::Value) -> VibeToolResult {
    VibeToolResult { success: true, data, error: None, latency_ms: 0 }
}

fn err(msg: String) -> VibeToolResult {
    VibeToolResult { success: false, data: serde_json::Value::Null, error: Some(msg), latency_ms: 0 }
}

/// `logs/read` — read a log file inside the workspace (capped), returning
/// the last N lines (default 200).
pub fn logs_read() -> ToolHandler {
    Arc::new(move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = args.get("project").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let max_lines = args.get("lines").and_then(|v| v.as_u64()).unwrap_or(200).clamp(1, 2000) as usize;
            match read_rel_file(&project, &path, 512 * 1024) {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes).into_owned();
                    let lines: Vec<&str> = text.lines().collect();
                    let total = lines.len();
                    let start = total.saturating_sub(max_lines);
                    ok(json!({
                        "path": path,
                        "total_lines": total,
                        "lines": lines[start..].to_vec(),
                    }))
                }
                Err(e) => err(e),
            }
        })
    })
}

/// `logs/list` — list log files available under `logs/`.
pub fn logs_list() -> ToolHandler {
    Arc::new(move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = args.get("project").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            match crate::harness::list_rel(&project, "logs", 0) {
                Ok(mut entries) => {
                    entries.sort();
                    ok(json!({"path": "logs", "count": entries.len(), "entries": entries}))
                }
                Err(e) => err(e),
            }
        })
    })
}