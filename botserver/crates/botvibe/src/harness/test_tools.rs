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
                .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            let timeout = args.get("timeout_secs").and_then(|v| v.as_u64()).unwrap_or(300);
            let cwd = match ensure_workspace(&project) {
                Ok(p) => p,
                Err(e) => return err(e),
            };
            // Auto-detect the test command when none is provided (the deploy
            // pipeline's BuildTest stage injects only project context). If the
            // project has no tests at all, report a clean pass so the pipeline
            // can continue to commit/publish instead of failing on an empty
            // command (#8xx).
            let (command, argv) = if command.is_empty() {
                if cwd.join("package.json").exists() {
                    let has_test = std::fs::read_to_string(cwd.join("package.json"))
                        .ok()
                        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                        .and_then(|v| {
                            v.get("scripts")
                                .and_then(|s| s.get("test"))
                                .and_then(|t| t.as_str())
                                .map(|t| !t.is_empty())
                        })
                        .unwrap_or(false);
                    if !has_test {
                        return ok(json!({
                            "passed": true,
                            "skipped": true,
                            "message": "No test script in package.json — nothing to run",
                        }));
                    }
                    ("npm".to_string(), vec!["test".to_string()])
                } else if cwd.join("Cargo.toml").exists() {
                    ("cargo".to_string(), vec!["test".to_string()])
                } else if cwd.join("pyproject.toml").exists() {
                    ("python".to_string(), vec!["-m".to_string(), "pytest".to_string()])
                } else {
                    return ok(json!({
                        "passed": true,
                        "skipped": true,
                        "message": "No test framework detected — nothing to run",
                    }));
                }
            } else {
                (command.clone(), argv.clone())
            };
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