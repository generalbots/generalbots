//! Login status reporting and OAuth URL extraction for Antigravity.

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::vm;
use crate::AgentService;

use super::paths::{CREDENTIALS_PATH, LOGIN_LOG, LOGIN_STATUS};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AntigravityStatus {
    pub state: String,
    pub url: Option<String>,
    pub message: String,
}

pub fn extract_oauth_url(log: &str) -> Option<String> {
    let marker = "https://";
    let start = log.find(marker)?;
    let rest = &log[start..];
    let end = rest
        .char_indices()
        .find(|(_, c)| {
            matches!(
                *c,
                '\u{1b}' | '\u{7}' | '\0' | '\n' | '\r' | ' ' | '\t' | '"' | '\'' | '<' | '>'
            )
        })
        .map(|(i, _)| i)
        .unwrap_or(rest.len());
    let url = rest[..end].trim();
    if url.len() > marker.len() {
        Some(url.to_string())
    } else {
        None
    }
}

pub(super) async fn run_script(
    state: &AgentService,
    vm_name: &str,
    script: &str,
) -> Result<(bool, String, String), (StatusCode, String)> {
    vm::exec_in_vm(
        state,
        vm_name,
        &["sh".into(), "-c".into(), script.to_string()],
    )
    .await
}

pub(super) async fn read_status_file(state: &AgentService, vm_name: &str) -> String {
    match run_script(
        state,
        vm_name,
        &format!("cat {LOGIN_STATUS} 2>/dev/null || true"),
    )
    .await
    {
        Ok((_, stdout, _)) => stdout.trim().to_string(),
        Err(_) => String::new(),
    }
}

pub async fn status(
    state: &AgentService,
    vm_name: &str,
) -> Result<AntigravityStatus, (StatusCode, String)> {
    let script = format!(
        "test -s {credentials} && echo LINKED || echo UNLINKED; \
         echo ---; command -v antigravity >/dev/null 2>&1 && antigravity --version 2>/dev/null || true; \
         echo ---; cat {log} 2>/dev/null | tail -c 12000 || true",
        credentials = CREDENTIALS_PATH,
        log = LOGIN_LOG
    );
    let (_, stdout, stderr) = run_script(state, vm_name, &script).await?;
    if stdout.trim().is_empty() && !stderr.trim().is_empty() {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("antigravity status: {stderr}")));
    }
    let linked = stdout
        .lines()
        .next()
        .map(|line| line.trim() == "LINKED")
        .unwrap_or(false);
    let url = extract_oauth_url(&stdout);
    let phase = read_status_file(state, vm_name).await;
    let state_name = if linked {
        "linked".to_string()
    } else if !phase.is_empty() {
        phase
    } else if url.is_some() {
        "waiting".to_string()
    } else {
        "unlinked".to_string()
    };
    let message = match state_name.as_str() {
        "linked" => "Antigravity is connected inside your workspace.",
        "waiting" => "Open the Antigravity sign-in page and paste the code back here.",
        "verifying" => "Antigravity is verifying the authorization code…",
        "starting" => "Starting the Antigravity sign-in helper…",
        "error" => "Antigravity rejected the code. Start the sign-in again.",
        _ => "Antigravity is not connected.",
    };
    Ok(AntigravityStatus {
        state: state_name,
        url,
        message: message.to_string(),
    })
}
