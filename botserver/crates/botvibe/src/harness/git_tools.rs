//! #747 — Git tooling for the Vibe agent: status, log, diff and commit
//! against the project workspace checkout.

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

fn str_arg(args: &serde_json::Value, key: &str) -> String {
    args.get(key).and_then(|v| v.as_str()).unwrap_or_default().to_string()
}

fn git_run(project: &str, git_args: &[String], timeout: u64) -> VibeToolResult {
    let cwd = match ensure_workspace(project) {
        Ok(p) => p,
        Err(e) => return err(e),
    };
    match run("git", git_args, &cwd, timeout) {
        Ok(out) if out.exit_code == Some(0) => ok(json!({
            "exit_code": out.exit_code,
            "output": out.stdout,
            "stderr": out.stderr,
        })),
        Ok(out) => err(format!("git {} failed (exit {}): {}", git_args.join(" "),
            out.exit_code.unwrap_or(-1), out.stderr.trim())),
        Err(e) => err(e.to_string()),
    }
}

/// `git/status` — porcelain status of the project workspace.
pub fn git_status() -> ToolHandler {
    Arc::new(move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = str_arg(&args, "project");
            git_run(&project, &["status".to_string(), "--porcelain=v1".to_string()], 30)
        })
    })
}

/// `git/log` — last N commits (default 20).
pub fn git_log_tool() -> ToolHandler {
    Arc::new(move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = str_arg(&args, "project");
            let n = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(20).clamp(1, 100);
            git_run(&project, &["log".to_string(), "--oneline".to_string(), format!("-n{n}")], 30)
        })
    })
}

/// `git/diff` — working tree diff, optionally cached (staged).
pub fn git_diff_tool() -> ToolHandler {
    Arc::new(move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = str_arg(&args, "project");
            let cached = args.get("cached").and_then(|v| v.as_bool()).unwrap_or(false);
            let flags = if cached {
                vec!["diff".to_string(), "--cached".to_string()]
            } else {
                vec!["diff".to_string()]
            };
            git_run(&project, &flags, 300)
        })
    })
}

/// `git/commit` — stage all changes and commit with the given message.
pub fn git_commit_tool() -> ToolHandler {
    Arc::new(move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = str_arg(&args, "project");
            let message = str_arg(&args, "message");
            if message.trim().is_empty() {
                return err("commit message is required".into());
            }
            let cwd = match ensure_workspace(&project) {
                Ok(p) => p,
                Err(e) => return err(e),
            };
            match run("git", &["add".to_string(), "-A".to_string()], &cwd, 30) {
                Ok(out) if out.exit_code == Some(0) => {}
                Ok(out) => return err(format!("git add failed: {}", out.stderr.trim())),
                Err(e) => return err(e.to_string()),
            }
            let commit_args = vec!["commit".to_string(), "-m".to_string(), message.clone()];
            match run("git", &commit_args, &cwd, 30) {
                Ok(out) if out.exit_code == Some(0) => ok(json!({
                    "message": message,
                    "hash": out.stdout.lines().next().unwrap_or("").trim(),
                    "output": out.stdout,
                })),
                Ok(out) => err(format!("git commit failed: {}", out.stderr.trim())),
                Err(e) => err(e.to_string()),
            }
        })
    })
}

/// `git/init` — initialize a git repository if absent, and clone support.
pub fn git_init_tool() -> ToolHandler {
    Arc::new(move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = str_arg(&args, "project");
            let cwd = match ensure_workspace(&project) {
                Ok(p) => p,
                Err(e) => return err(e),
            };
            if cwd.join(".git").exists() {
                return ok(json!({"already_initialized": true}));
            }
            let clone_url = str_arg(&args, "clone_url");
            if clone_url.is_empty() {
                match run("git", &["init".to_string()], &cwd, 30) {
                    Ok(out) if out.exit_code == Some(0) => {
                        ok(json!({"initialized": true, "output": out.stdout}))
                    }
                    Ok(out) => err(format!("git init failed: {}", out.stderr.trim())),
                    Err(e) => err(e.to_string()),
                }
            } else {
                match run("git", &["clone".to_string(), clone_url, ".".to_string()], &cwd, 300) {
                    Ok(out) if out.exit_code == Some(0) => {
                        ok(json!({"cloned": true, "output": out.stdout}))
                    }
                    Ok(out) => err(format!("git clone failed: {}", out.stderr.trim())),
                    Err(e) => err(e.to_string()),
                }
            }
        })
    })
}