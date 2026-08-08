use crate::harness::cmd::run;
use crate::harness::ensure_workspace;
use crate::tool_executor::{ToolHandler, ToolSchema};
use crate::types::{VibeState, VibeToolResult};
use serde_json::{json, Value};
use std::sync::Arc;

fn ok(data: Value) -> VibeToolResult {
    VibeToolResult { success: true, data, error: None, latency_ms: 0 }
}

fn err(msg: String) -> VibeToolResult {
    VibeToolResult { success: false, data: Value::Null, error: Some(msg), latency_ms: 0 }
}

fn str_arg(args: &Value, key: &str) -> String {
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

fn git_ok(project: &str, git_args: &[String]) -> VibeToolResult {
    git_run(project, git_args, 60)
}

pub fn gitflow_tools() -> Vec<(String, ToolSchema, ToolHandler)> {
    vec![
        ("git/branch".into(), ToolSchema::new("git/branch", "List branches of the project repository").with_parameters(json!({
            "type": "object",
            "properties": {
                "project": {"type": "string", "description": "Vibe project id (name)"}
            },
            "required": ["project"]
        })), git_branch()),
        ("git/push".into(), ToolSchema::new("git/push", "Push the current branch to the remote").with_parameters(json!({
            "type": "object",
            "properties": {
                "project": {"type": "string", "description": "Vibe project id (name)"},
                "remote": {"type": "string", "description": "Remote name (default origin)"},
                "branch": {"type": "string", "description": "Branch to push (default current)"}
            },
            "required": ["project"]
        })).with_approval(), git_push()),
        ("git/pr".into(), ToolSchema::new("git/pr", "Create a pull request against the project remote via Forgejo").with_parameters(json!({
            "type": "object",
            "properties": {
                "project": {"type": "string", "description": "Vibe project id (name)"},
                "title": {"type": "string", "description": "PR title"},
                "head": {"type": "string", "description": "Source branch"},
                "base": {"type": "string", "description": "Target branch (default main)"},
                "body": {"type": "string", "description": "PR body"}
            },
            "required": ["project", "title", "head"]
        })).with_approval(), git_pr()),
    ]
}

fn git_branch() -> ToolHandler {
    Arc::new(move |args: Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = str_arg(&args, "project");
            git_ok(&project, &["branch".to_string(), "-a".to_string()])
        })
    })
}

fn git_push() -> ToolHandler {
    Arc::new(move |args: Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = str_arg(&args, "project");
            let remote = str_arg(&args, "remote");
            let branch = str_arg(&args, "branch");
            let remote = if remote.is_empty() { "origin".to_string() } else { remote };
            if branch.is_empty() {
                git_ok(&project, &["push".to_string(), remote])
            } else {
                git_ok(&project, &["push".to_string(), remote, branch])
            }
        })
    })
}

fn git_pr() -> ToolHandler {
    Arc::new(move |args: Value, _state: &dyn VibeState| {
        let args = args.clone();
        Box::pin(async move {
            let project = str_arg(&args, "project");
            let title = str_arg(&args, "title");
            let head = str_arg(&args, "head");
            let base = str_arg(&args, "base");
            let body = str_arg(&args, "body");
            if title.is_empty() || head.is_empty() {
                return err("title and head are required".into());
            }
            let base = if base.is_empty() { "main".to_string() } else { base };

            let cwd = match ensure_workspace(&project) {
                Ok(p) => p,
                Err(e) => return err(e),
            };
            let remote_out = match run("git", &["remote".to_string(), "get-url".to_string(), "origin".to_string()], &cwd, 30) {
                Ok(out) if out.exit_code == Some(0) => out.stdout.trim().to_string(),
                _ => return err("no origin remote configured".into()),
            };
            let (owner, repo) = match parse_forgejo_repo(&remote_out) {
                Some(pair) => pair,
                None => return err(format!("cannot determine owner/repo from remote: {remote_out}")),
            };

            let forgejo_url = std::env::var("FORGEJO_URL")
                .unwrap_or_else(|_| "https://alm.pragmatismo.com.br".to_string());
            let token = match std::env::var("FORGEJO_TOKEN").ok() {
                Some(t) if !t.is_empty() => t,
                _ => return err("FORGEJO_TOKEN is not configured".into()),
            };

            let endpoint = format!(
                "{}/api/v1/repos/{}/{}/pulls",
                forgejo_url.trim_end_matches('/'),
                owner,
                repo
            );
            let payload = json!({
                "title": title,
                "head": head,
                "base": base,
                "body": body,
            });

            let client = reqwest::Client::new();
            match client
                .post(&endpoint)
                .header("Authorization", format!("token {token}"))
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() => {
                    let json_resp = resp.json::<Value>().await.unwrap_or(json!({}));
                    ok(json!({
                        "pr_number": json_resp.get("number").cloned().unwrap_or(Value::Null),
                        "url": json_resp.get("html_url").cloned().unwrap_or(Value::Null),
                    }))
                }
                Ok(resp) => err(format!("Forgejo returned status {}", resp.status())),
                Err(e) => err(format!("Forgejo request failed: {e}")),
            }
        })
    })
}

fn parse_forgejo_repo(remote: &str) -> Option<(String, String)> {
    let normalized = remote.trim();
    let path = if let Some(rest) = normalized.strip_prefix("https://") {
        rest
    } else if let Some(rest) = normalized.strip_prefix("http://") {
        rest
    } else if let Some(rest) = normalized.strip_prefix("git@") {
        rest.splitn(2, ':').nth(1)?
    } else {
        normalized
    };
    let mut parts = path.trim_end_matches(".git").split('/');
    let owner = parts.next()?.to_string();
    let repo = parts.next()?.to_string();
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner, repo))
}
