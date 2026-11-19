//! Codex CLI login: install/start PTY, submit code, logout.

use axum::http::StatusCode;

use crate::AgentService;

use super::b64;
use super::internal;
use super::paths::{CODEX_HOME, CODEX_USER, LOGIN_CODE, LOGIN_LOG, LOGIN_SCRIPT, LOGIN_STATUS};
use super::pty::PTY_SCRIPT;
use super::status::{run_script, status, CodexStatus};

async fn ensure_cli(state: &AgentService, vm_name: &str) -> Result<(), (StatusCode, String)> {
    let probe = "command -v codex >/dev/null 2>&1 && id -u codex >/dev/null 2>&1 && echo READY || echo MISSING";
    let (_, stdout, _) = run_script(state, vm_name, probe).await?;
    if stdout.trim() == "READY" {
        return Ok(());
    }
    // Generic install placeholder: adjust to real Codex CLI install steps
    let install = format!(
        "id -u {user} >/dev/null 2>&1 || useradd -m -s /bin/bash {user}; \
         mkdir -p {home}/.codex; chown -R {user}:{user} {home}; chmod 700 {home} {home}/.codex; \
         echo 'Codex CLI install not configured yet' >&2; exit 3",
        user = CODEX_USER,
        home = CODEX_HOME
    );
    let (ok, stdout, stderr) = run_script(state, vm_name, &install).await?;
    if ok && stdout.contains("READY") {
        return Ok(());
    }
    Err(internal(format!(
        "could not prepare Codex CLI in the workspace: {}",
        if stderr.trim().is_empty() { stdout.trim() } else { stderr.trim() }
    )))
}

pub async fn start_login(
    state: &AgentService,
    vm_name: &str,
) -> Result<CodexStatus, (StatusCode, String)> {
    let current = status(state, vm_name).await?;
    if current.state == "linked" {
        return Ok(current);
    }
    if current.state == "waiting" && current.url.is_some() {
        return Ok(current);
    }
    ensure_cli(state, vm_name).await?;
    let script = format!(
        "printf '%s' '{payload}' | base64 -d > {path}; \
         chown {user}:{user} {path}; chmod 600 {path}; \
         su {user} -s /bin/sh -c 'nohup python3 {path} >/tmp/gb-codex-login.out 2>&1 & echo $!'",
        payload = b64(PTY_SCRIPT),
        path = LOGIN_SCRIPT,
        user = CODEX_USER
    );
    let (ok, stdout, stderr) = run_script(state, vm_name, &script).await?;
    if !ok {
        return Err(internal(format!(
            "could not start the Codex login helper: {}",
            if stderr.trim().is_empty() { stdout.trim() } else { stderr.trim() }
        )));
    }
    for _ in 0..30 {
        let snapshot = status(state, vm_name).await?;
        if snapshot.url.is_some() || snapshot.state == "linked" || snapshot.state == "error" {
            return Ok(snapshot);
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    status(state, vm_name).await
}

pub async fn submit_code(
    state: &AgentService,
    vm_name: &str,
    code: &str,
) -> Result<CodexStatus, (StatusCode, String)> {
    let clean = code.trim();
    if clean.is_empty() || clean.len() > 4096 {
        return Err((StatusCode::BAD_REQUEST, "Invalid authorization code".to_string()));
    }
    let script = format!(
        "printf '%s' '{payload}' | base64 -d > {path}; \
         chown {user}:{user} {path}; chmod 600 {path}",
        payload = b64(clean),
        path = LOGIN_CODE,
        user = CODEX_USER
    );
    let (ok, _, stderr) = run_script(state, vm_name, &script).await?;
    if !ok {
        return Err(internal(format!("could not deliver the Codex code: {}", stderr.trim())));
    }
    for _ in 0..40 {
        let snapshot = status(state, vm_name).await?;
        if snapshot.state == "linked" || snapshot.state == "error" {
            return Ok(snapshot);
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    status(state, vm_name).await
}

pub async fn logout(state: &AgentService, vm_name: &str) -> Result<(), (StatusCode, String)> {
    let script = format!(
        "su {user} -s /bin/sh -c 'rm -rf {home}/.codex' 2>/dev/null || true; \
         rm -f {log} {status} {code}",
        user = CODEX_USER,
        home = CODEX_HOME,
        log = LOGIN_LOG,
        status = LOGIN_STATUS,
        code = LOGIN_CODE
    );
    let (ok, _, stderr) = run_script(state, vm_name, &script).await?;
    if !ok {
        return Err(internal(format!("could not clear Codex credentials: {}", stderr.trim())));
    }
    Ok(())
}
