//! Claude CLI login: install, start the PTY, submit the code and log out.

use axum::http::StatusCode;

use crate::AgentService;

use super::b64;
use super::internal;
use super::paths::{CLAUDE_HOME, CLAUDE_USER, LOGIN_CODE, LOGIN_LOG, LOGIN_SCRIPT, LOGIN_STATUS};
use super::pty::PTY_SCRIPT;
use super::status::{run_script, status, ClaudeStatus};

/// Guarantees the CLI and its `claude` identity exist before a login starts.
/// Workspace preparation runs in the background after the VM is created, so a
/// Connect click can race it; installing here makes the clean-install flow
/// deterministic instead of returning "claude CLI is not installed".
async fn ensure_cli(
    state: &AgentService,
    vm_name: &str,
) -> Result<(), (StatusCode, String)> {
    let probe = "command -v claude >/dev/null 2>&1 && id -u claude >/dev/null 2>&1 && echo READY || echo MISSING";
    let (_, stdout, _) = run_script(state, vm_name, probe).await?;
    if stdout.trim() == "READY" {
        return Ok(());
    }
    let install = format!(
        "id -u {user} >/dev/null 2>&1 || useradd -m -s /bin/bash {user}; \
         mkdir -p {home}/.claude; chown -R {user}:{user} {home}; chmod 700 {home} {home}/.claude; \
         command -v npm >/dev/null 2>&1 || (apt-get -o DPkg::Lock::Timeout=300 update -q && apt-get -o DPkg::Lock::Timeout=300 install -y -q nodejs npm); \
         command -v claude >/dev/null 2>&1 || timeout 420 npm install -g @anthropic-ai/claude-code >/tmp/claude-install.log 2>&1; \
         command -v claude >/dev/null 2>&1 && echo INSTALLED || {{ tail -20 /tmp/claude-install.log 2>/dev/null; exit 3; }}",
        user = CLAUDE_USER,
        home = CLAUDE_HOME
    );
    let (ok, stdout, stderr) = run_script(state, vm_name, &install).await?;
    if ok && stdout.contains("INSTALLED") {
        return Ok(());
    }
    Err(internal(format!(
        "could not install the Claude CLI in the workspace: {}",
        if stderr.trim().is_empty() { stdout.trim() } else { stderr.trim() }
    )))
}

/// Starts `claude setup-token` under a PTY inside the VM. Idempotent: a second
/// call while a process is already waiting returns the existing URL.
pub async fn start_login(
    state: &AgentService,
    vm_name: &str,
) -> Result<ClaudeStatus, (StatusCode, String)> {
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
         su {user} -s /bin/sh -c 'nohup python3 {path} >/tmp/gb-claude-login.out 2>&1 & echo $!'",
        payload = b64(PTY_SCRIPT),
        path = LOGIN_SCRIPT,
        user = CLAUDE_USER
    );
    let (ok, stdout, stderr) = run_script(state, vm_name, &script).await?;
    if !ok {
        return Err(internal(format!(
            "could not start the Claude login helper: {}",
            if stderr.trim().is_empty() { stdout.trim() } else { stderr.trim() }
        )));
    }
    // The CLI needs a moment to print the URL; poll briefly so the caller can
    // render the link on the first response instead of polling blindly.
    for _ in 0..30 {
        let snapshot = status(state, vm_name).await?;
        if snapshot.url.is_some() || snapshot.state == "linked" || snapshot.state == "error" {
            return Ok(snapshot);
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    status(state, vm_name).await
}

/// Forwards the authorization code the user copied from the Claude page.
pub async fn submit_code(
    state: &AgentService,
    vm_name: &str,
    code: &str,
) -> Result<ClaudeStatus, (StatusCode, String)> {
    let clean = code.trim();
    if clean.is_empty() || clean.len() > 4096 {
        return Err((StatusCode::BAD_REQUEST, "Invalid authorization code".to_string()));
    }
    let script = format!(
        "printf '%s' '{payload}' | base64 -d > {path}; \
         chown {user}:{user} {path}; chmod 600 {path}",
        payload = b64(clean),
        path = LOGIN_CODE,
        user = CLAUDE_USER
    );
    let (ok, _, stderr) = run_script(state, vm_name, &script).await?;
    if !ok {
        return Err(internal(format!("could not deliver the Claude code: {}", stderr.trim())));
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

/// Removes the VM-local credentials. The account row is updated by the API.
pub async fn logout(state: &AgentService, vm_name: &str) -> Result<(), (StatusCode, String)> {
    let script = format!(
        "su {user} -s /bin/sh -c 'rm -rf {home}/.claude' 2>/dev/null || true; \
         rm -f {log} {status} {code}",
        user = CLAUDE_USER,
        home = CLAUDE_HOME,
        log = LOGIN_LOG,
        status = LOGIN_STATUS,
        code = LOGIN_CODE
    );
    let (ok, _, stderr) = run_script(state, vm_name, &script).await?;
    if !ok {
        return Err(internal(format!("could not clear Claude credentials: {}", stderr.trim())));
    }
    Ok(())
}
