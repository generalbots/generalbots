//! #747 — Git tooling for the Vibe agent: status, log, diff and commit
//! against the project workspace checkout.

use crate::harness::cmd::run;
use crate::harness::ensure_workspace;
use crate::tool_executor::ToolHandler;
use crate::types::{VibeState, VibeToolResult};
use serde_json::json;
use std::sync::Arc;

const GIT_USER_NAME: &str = "Vibe Agent";
const GIT_USER_EMAIL: &str = "vibe@gbo.local";

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
            // Auto-initialize the repository when absent so the deploy
            // pipeline's commit_push stage (which runs git/commit directly
            // without a prior git/init) works on fresh projects instead of
            // failing with "not a git repository".
            if !cwd.join(".git").exists() {
                // Branch naming standard: `main`, never `master` (git >= 2.28
                // supports -b; dev/prod containers ship git 2.4x).
                if let Ok(out) = run("git", &["init".to_string(), "-b".to_string(), "main".to_string()], &cwd, 30) {
                    if out.exit_code != Some(0) {
                        return err(format!("git init failed: {}", out.stderr.trim()));
                    }
                }
            }
            match run("git", &["add".to_string(), "-A".to_string()], &cwd, 30) {
                Ok(out) if out.exit_code == Some(0) => {}
                Ok(out) => return err(format!("git add failed: {}", out.stderr.trim())),
                Err(e) => return err(e.to_string()),
            }
            // `-c` config must precede the subcommand; the harness runs
            // processes with env_clear(), so git has no user identity and the
            // commit would otherwise fail with "Please tell me who you are".
            let commit_args = vec![
                format!("-c"), format!("user.name={GIT_USER_NAME}"),
                format!("-c"), format!("user.email={GIT_USER_EMAIL}"),
                "commit".to_string(),
                "-m".to_string(),
                message.clone(),
            ];
            // Deploy pipelines re-run against the same workspace: a tree
            // that is already committed has nothing to stage, so both the
            // `git add` ("nothing to commit") and `git commit` ("nothing to
            // commit, working tree clean") no-op paths must succeed instead
            // of failing the whole deploy run. Git may print these to
            // stdout or stderr depending on version, so both are checked.
            let clean_tree = |out: &crate::harness::cmd::RunOutput| {
                let combined = format!("{} {}", out.stdout, out.stderr);
                combined.contains("nothing to commit")
                    || combined.contains("no changes added to commit")
                    || combined.contains("nothing added to commit")
            };
            match run("git", &commit_args, &cwd, 30) {
                Ok(out) if out.exit_code == Some(0) => ok(json!({
                    "message": message,
                    "hash": out.stdout.lines().next().unwrap_or("").trim(),
                    "output": out.stdout,
                })),
                Ok(out) if clean_tree(&out) => ok(json!({
                    "message": message,
                    "hash": "",
                    "output": "nothing to commit — workspace already clean",
                    "noop": true,
                })),
                Ok(out) => err(format!("git commit failed: {}", out.stderr.trim())),
                Err(e) => err(e.to_string()),
            }
        })
    })
}

/// `git/snapshot-previous` — before a deploy publishes the current state,
/// snapshot the currently deployed commit into a `release/prev-<ts>` branch so
/// the user can switch back from the toolbar branch combo and re-deploy the
/// previous version. Runs `git branch release/prev-<ts> <current-HEAD>`
/// without checking out, so the working tree (and the running app) stays put.
pub fn git_snapshot_previous_tool() -> ToolHandler {
    Arc::new(move |args: serde_json::Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = str_arg(&args, "project");
            let cwd = match ensure_workspace(&project) {
                Ok(p) => p,
                Err(e) => return err(e),
            };
            if !cwd.join(".git").exists() {
                // Branch naming standard: `main`, never `master`.
                match run("git", &["init".to_string(), "-b".to_string(), "main".to_string()], &cwd, 30) {
                    Ok(out) if out.exit_code == Some(0) => {}
                    Ok(out) => return err(format!("git init failed: {}", out.stderr.trim())),
                    Err(e) => return err(format!("git init failed: {e}")),
                }
            }
            // Current HEAD; "previous" = the commit that is about to be
            // replaced by this deploy (the currently deployed version).
            let head = match run("git", &["rev-parse".to_string(), "HEAD".to_string()], &cwd, 15) {
                Ok(out) if out.exit_code == Some(0) => out.stdout.trim().to_string(),
                _ => String::new(),
            };
            if head.is_empty() {
                return ok(json!({
                    "snapshot": false,
                    "reason": "no commits yet — nothing to snapshot",
                    "branch": null,
                }));
            }
            let ts = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
            let branch = format!("release/prev-{ts}");
            let existing = run(
                "git",
                &["rev-parse".to_string(), "--verify".to_string(), format!("refs/heads/{branch}")],
                &cwd,
                15,
            )
            .map(|out| out.exit_code == Some(0))
            .unwrap_or(false);
            let args_vec = if existing {
                vec!["branch".to_string(), "-f".to_string(), branch.clone(), head.clone()]
            } else {
                vec!["branch".to_string(), branch.clone(), head.clone()]
            };
            match run("git", &args_vec, &cwd, 30) {
                Ok(out) if out.exit_code == Some(0) => ok(json!({
                    "snapshot": true,
                    "branch": branch,
                    "from": head,
                    "output": out.stdout.trim(),
                })),
                Ok(out) => err(format!("git branch snapshot failed: {}", out.stderr.trim())),
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
                // Branch naming standard: `main`, never `master`.
                match run("git", &["init".to_string(), "-b".to_string(), "main".to_string()], &cwd, 30) {
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