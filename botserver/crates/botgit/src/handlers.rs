use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use botlib::security::SafeCommand;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use botcore::shared::state::AppState;

const DEFAULT_REPO: &str = "/tmp/git-repo";

#[derive(Deserialize)]
pub struct GitQuery {
    pub repo: Option<String>,
}

#[derive(Deserialize)]
pub struct GitStatusQuery {
    pub repo: Option<String>,
}

#[derive(Serialize)]
pub struct GitStatusResponse {
    pub repo: String,
    pub branch: Option<String>,
    pub files: Vec<GitFileStatus>,
}

#[derive(Serialize)]
pub struct GitFileStatus {
    pub file: String,
    pub status: String,
}

#[derive(Deserialize)]
pub struct CommitRequest {
    pub message: String,
    pub repo: Option<String>,
}

#[derive(Serialize)]
pub struct GitLogResponse {
    pub repo: String,
    pub commits: Vec<GitLogEntry>,
}

#[derive(Serialize)]
pub struct GitLogEntry {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub date: String,
}

#[derive(Serialize)]
pub struct GitBranch {
    pub name: String,
    pub current: bool,
}

fn resolve_repo(query_repo: Option<&str>) -> String {
    let repo = query_repo
        .map(str::trim)
        .filter(|r| !r.is_empty())
        .unwrap_or(DEFAULT_REPO);
    if repo.starts_with("/tmp") || repo.starts_with("/var/tmp") {
        repo.to_string()
    } else {
        DEFAULT_REPO.to_string()
    }
}

fn run_git(repo: &str, args: &[&str]) -> Result<String, String> {
    let mut cmd = SafeCommand::new("git").map_err(|e| e.to_string())?;
    cmd = cmd.arg("-C")?;
    cmd = cmd.trusted_arg(repo)?;
    for a in args {
        cmd = cmd.trusted_arg(a)?;
    }
    let out = cmd.execute().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    if !out.status.success() {
        return Err(if stderr.is_empty() { stdout } else { stderr });
    }
    Ok(stdout)
}

pub async fn git_status(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<GitStatusQuery>,
) -> Result<Json<GitStatusResponse>, axum::http::StatusCode> {
    let repo = resolve_repo(params.repo.as_deref());
    let branch_out = run_git(&repo, &["rev-parse", "--abbrev-ref", "HEAD"]);
    let branch = branch_out.ok();
    let status_out = match run_git(&repo, &["status", "--porcelain=v1"]) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("git status failed for {repo}: {e}");
            String::new()
        }
    };
    let files = status_out.lines().filter_map(|line| {
        if line.len() < 3 {
            return None;
        }
        let code = &line[..2];
        let file = line[3..].trim();
        let status = match code {
            " M" | "M " | "MM" => "modified",
            "??" => "untracked",
            "A" => "added",
            "D" | " D" => "deleted",
            "R" => "renamed",
            _ => "changed",
        };
        Some(GitFileStatus {
            file: file.to_string(),
            status: status.to_string(),
        })
    }).collect();
    Ok(Json(GitStatusResponse {
        repo,
        branch,
        files,
    }))
}

pub async fn git_diff(
    State(_state): State<Arc<AppState>>,
    Path(file): Path<String>,
    Query(params): Query<GitQuery>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let repo = resolve_repo(params.repo.as_deref());
    let out = run_git(&repo, &["diff", "HEAD", "--", &file]);
    let diff = match out {
        Ok(d) => d,
        Err(_) => run_git(&repo, &["diff", "--cached", "--", &file]).unwrap_or_default(),
    };
    Ok(Json(serde_json::json!({ "file": file, "diff": diff })))
}

pub async fn git_commit(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<CommitRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let repo = resolve_repo(payload.repo.as_deref());
    // Persist pending changes, then commit with the provided message.
    let add = run_git(&repo, &["add", "-A"]);
    let mut results = serde_json::Map::new();
    results.insert("add".into(), serde_json::Value::String(add.unwrap_or_else(|e| e)));
    let message = if payload.message.trim().is_empty() {
        "wip".to_string()
    } else {
        payload.message.trim().to_string()
    };
    let commit = run_git(&repo, &["commit", "-m", &message]);
    match commit {
        Ok(out) => {
            results.insert("status".into(), serde_json::Value::String("success".into()));
            results.insert("output".into(), serde_json::Value::String(out.trim().to_string()));
        }
        Err(e) => {
            results.insert("status".into(), serde_json::Value::String("failure".into()));
            results.insert("error".into(), serde_json::Value::String(e));
        }
    }
    Ok(Json(serde_json::Value::Object(results)))
}

pub async fn git_push(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<GitQuery>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let repo = resolve_repo(params.repo.as_deref());
    let out = run_git(&repo, &["push", "origin", "HEAD"]);
    match out {
        Ok(_) => Ok(Json(serde_json::json!({ "status": "success", "message": "Pushed to origin/HEAD" }))),
        Err(e) => Ok(Json(serde_json::json!({ "status": "failure", "error": e }))),
    }
}

pub async fn git_branches(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<GitQuery>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let repo = resolve_repo(params.repo.as_deref());
    let out = run_git(&repo, &["branch", "--format=%(refname:short)"]);
    let current = run_git(&repo, &["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_default();
    let branches: Vec<GitBranch> = out
        .unwrap_or_default()
        .lines()
        .map(|b| b.trim().to_string())
        .filter(|b| !b.is_empty())
        .map(|b| GitBranch {
            name: b.clone(),
            current: b == current.trim(),
        })
        .collect();
    Ok(Json(serde_json::json!({ "branches": branches })))
}

pub async fn git_create_or_switch_branch(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Query(params): Query<GitQuery>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let repo = resolve_repo(params.repo.as_deref());
    let exists = run_git(&repo, &["rev-parse", "--verify", &format!("refs/heads/{name}")]);
    let cmd: &[&str] = if exists.is_ok() {
        &["checkout", &name]
    } else {
        &["checkout", "-B", &name]
    };
    let out = run_git(&repo, cmd);
    match out {
        Ok(msg) => Ok(Json(serde_json::json!({ "status": "success", "branch": name, "output": msg.trim() }))),
        Err(e) => Ok(Json(serde_json::json!({ "status": "failure", "error": e }))),
    }
}

pub async fn git_log(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<GitQuery>,
) -> Result<Json<GitLogResponse>, axum::http::StatusCode> {
    let repo = resolve_repo(params.repo.as_deref());
    let out = run_git(
        &repo,
        &["log", "-20", "--pretty=format:%H%x00%s%x00%an%x00%ad", "--date=short"],
    );
    let commits: Vec<GitLogEntry> = out
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(4, '\x00');
            let hash = parts.next()?.to_string();
            let message = parts.next().unwrap_or("").to_string();
            let author = parts.next().unwrap_or("").to_string();
            let date = parts.next().unwrap_or("").to_string();
            Some(GitLogEntry { hash, message, author, date })
        })
        .collect();
    Ok(Json(GitLogResponse { repo, commits }))
}