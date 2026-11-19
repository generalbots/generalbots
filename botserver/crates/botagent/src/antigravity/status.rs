//! Login status reporting and auth-code extraction for Antigravity.

use axum::http::StatusCode;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::vm;
use crate::AgentService;

use super::paths::{CREDENTIALS_PATH, LOGIN_LOG, LOGIN_STATUS};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AntigravityStatus {
    /// unlinked | waiting | verifying | linked | error | starting
    pub state: String,
    pub url: Option<String>,
    pub auth_code: Option<String>,
    pub message: String,
}

/// Extracts the authorization URL from the Antigravity CLI transcript.
/// In SSH/remote mode, agy prints: "Visit https://... to authenticate"
pub fn extract_auth_url(log: &str) -> Option<String> {
    let re = Regex::new(r#"https://[^\s\x1b\x07<>'\"`]+"#).ok()?;
    for cap in re.captures_iter(log) {
        let url = cap.get(0)?.as_str().trim_end_matches(['.', ',', ';', ':']);
        if url.contains("google") || url.contains("antigravity") || url.contains("auth") || url.contains("oauth") {
            if url.len() > 20 {
                return Some(url.to_string());
            }
        }
    }
    // Fallback: first https URL
    let re2 = Regex::new(r#"https://[^\s\x1b\x07<>'\"`]+"#).ok()?;
    for cap in re2.captures_iter(log) {
        let url = cap.get(0)?.as_str().trim_end_matches(['.', ',', ';', ':']);
        if url.len() > 20 {
            return Some(url.to_string());
        }
    }
    None
}

/// Extracts the authorization code from the Antigravity CLI transcript.
/// In SSH mode, agy prints: "Enter the code: XXXXXXXX" or "Code: XXXXXXXX"
pub fn extract_auth_code(log: &str) -> Option<String> {
    let re = Regex::new(r"(?i)(?:code|enter)[:\s]+([A-Za-z0-9_-]{6,})").ok()?;
    if let Some(cap) = re.captures(log) {
        let code = cap.get(1)?.as_str().to_string();
        if code.len() >= 6 {
            return Some(code);
        }
    }
    None
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
         echo ---; command -v agy >/dev/null 2>&1 && agy --version 2>/dev/null || true; \
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
    let url = extract_auth_url(&stdout);
    let code = extract_auth_code(&stdout);
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
        "waiting" => "Open the URL in your browser, sign in, and paste the code here.",
        "verifying" => "Antigravity is verifying the authorization code…",
        "starting" => "Starting the Antigravity sign-in helper…",
        "error" => "Antigravity rejected the code. Start the sign-in again.",
        _ => "Antigravity is not connected.",
    };
    Ok(AntigravityStatus {
        state: state_name,
        url,
        auth_code: code,
        message: message.to_string(),
    })
}
