//! #747 — File tools for the Vibe agent. Writes are confined to the
//! project workspace (`VIBE_WORKSPACE_ROOT/{project}/`).

use crate::harness::{list_rel, write_rel_file};
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

fn str_arg(args: &serde_json::Value, key: &str) -> String {
    args.get(key).and_then(|v| v.as_str()).unwrap_or_default().to_string()
}

/// `file/read` — read a workspace file (1 MiB cap) as text.
pub fn file_read() -> ToolHandler {
    Arc::new(move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = str_arg(&args, "project");
            let path = str_arg(&args, "path");
            match crate::harness::read_rel_file(&project, &path, 1024 * 1024) {
                Ok(bytes) => ok(json!({
                    "project": project,
                    "path": path,
                    "size": bytes.len(),
                    "content": String::from_utf8_lossy(&bytes),
                })),
                Err(e) => err(e),
            }
        })
    })
}

/// `file/write` — write a workspace file (4 MiB cap, path confined).
pub fn file_write() -> ToolHandler {
    Arc::new(move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = str_arg(&args, "project");
            let path = str_arg(&args, "path");
            let content = str_arg(&args, "content");
            match write_rel_file(&project, &path, content.as_bytes()) {
                Ok(()) => ok(json!({"project": project, "path": path, "bytes": content.len()})),
                Err(e) => err(e),
            }
        })
    })
}

/// `file/list` — list entries under a workspace path.
pub fn file_list() -> ToolHandler {
    Arc::new(move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = str_arg(&args, "project");
            let path = str_arg(&args, "path");
            match list_rel(&project, &path, 0) {
                Ok(mut entries) => {
                    entries.sort();
                    ok(json!({"project": project, "path": path, "count": entries.len(), "entries": entries}))
                }
                Err(e) => err(e),
            }
        })
    })
}

/// `file/delete` — delete a workspace file.
pub fn file_delete() -> ToolHandler {
    Arc::new(move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = str_arg(&args, "project");
            let path = str_arg(&args, "path");
            match crate::harness::resolve_workspace_path(&project, &path) {
                Ok(p) => match std::fs::remove_file(&p) {
                    Ok(()) => ok(json!({"project": project, "path": path, "deleted": true})),
                    Err(e) => err(format!("delete {}: {}", path, e)),
                },
                Err(e) => err(e),
            }
        })
    })
}

/// `file/exists` — probe a workspace path.
pub fn file_exists() -> ToolHandler {
    Arc::new(move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = str_arg(&args, "project");
            let path = str_arg(&args, "path");
            match crate::harness::resolve_workspace_path(&project, &path) {
                Ok(p) => ok(json!({"exists": p.exists(), "is_dir": p.is_dir()})),
                Err(e) => err(e),
            }
        })
    })
}