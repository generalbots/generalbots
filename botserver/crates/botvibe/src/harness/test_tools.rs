//! #747 — Test tooling: run the project test suite inside the workspace.

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

/// `test/run` — run the project tests. `command` and `args` are required
/// (e.g. `cargo test`, `npm test`); runs in the project workspace with a
/// generous timeout.
pub fn test_run() -> ToolHandler {
    Arc::new(move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = args.get("project").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let command = args.get("command").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let argv: Vec<String> = args.get("args")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string).map(String::from)).collect())
                .unwrap_or_default();
            let timeout = args.get("timeout_secs").and_then(|v| v.as_u64()).unwrap_or(300);
            if command.is_empty() {
                return err("test command is required".into());
            }
            // Tolerant parsing: models often send `command: "npm test"` as a
            // single string instead of `command: "npm", args: ["test"]`.
            // Split the program from its arguments so the allowlist guard
            // sees `npm` (allowed) instead of rejecting "npm test" outright.
            let (program, extra): (String, Vec<String>) = if argv.is_empty()
                && command.split_whitespace().count() > 1
            {
                let mut parts = command.split_whitespace();
                (
                    parts.next().unwrap_or_default().to_string(),
                    parts.map(str::to_string).collect(),
                )
            } else {
                (command.clone(), argv.clone())
            };
            let cwd = match ensure_workspace(&project) {
                Ok(p) => p,
                Err(e) => return err(e),
            };
            match run(&program, &extra, &cwd, timeout) {
                Ok(out) if out.exit_code == Some(0) => ok(json!({
                    "passed": true,
                    "exit_code": out.exit_code,
                    "output": out.stdout,
                    "stderr": out.stderr,
                })),
                Ok(out) => err(json!({
                    "passed": false,
                    "exit_code": out.exit_code,
                    "output": out.stdout,
                    "stderr": out.stderr,
                }).to_string()),
                Err(e) => err(e.to_string()),
            }
        })
    })
}

/// `test/list` — detect the test framework present in the workspace.
pub fn test_list() -> ToolHandler {
    Arc::new(move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = args.get("project").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let cwd = match ensure_workspace(&project) {
                Ok(p) => p,
                Err(e) => return err(e),
            };
            let mut frameworks: Vec<String> = Vec::new();
            if cwd.join("Cargo.toml").exists() {
                frameworks.push("cargo".into());
            }
            if cwd.join("package.json").exists() {
                frameworks.push("npm".into());
            }
            if cwd.join("pyproject.toml").exists() || cwd.join("requirements.txt").exists() {
                frameworks.push("python".into());
            }
            ok(json!({"frameworks": frameworks, "detected": !frameworks.is_empty()}))
        })
    })
}