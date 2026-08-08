//! #747 — Run tool: sandboxed command execution inside the project
//! workspace through the command guard.

use crate::harness::cmd::run;
use crate::harness::ensure_workspace;
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

/// `shell/run` — execute a program (from the allowlist) with args inside
/// the project workspace. Arguments are passed verbatim — no shell string
/// is composed.
pub fn run_command() -> ToolHandler {
    Arc::new(move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = args.get("project").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let command = args.get("command").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let argv: Vec<String> = args.get("args")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            let timeout = args.get("timeout_secs").and_then(|v| v.as_u64()).unwrap_or(30);
            if command.is_empty() {
                return err("command is required".into());
            }
            let cwd = match ensure_workspace(&project) {
                Ok(p) => p,
                Err(e) => return err(e),
            };
            match run(&command, &argv, &cwd, timeout) {
                Ok(out) if out.exit_code == Some(0) => ok(json!({
                    "exit_code": out.exit_code,
                    "stdout": out.stdout,
                    "stderr": out.stderr,
                })),
                Ok(out) => err(json!({
                    "exit_code": out.exit_code,
                    "stdout": out.stdout,
                    "stderr": out.stderr,
                }).to_string()),
                Err(e) => err(e.to_string()),
            }
        })
    })
}