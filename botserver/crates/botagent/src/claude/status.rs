//! Login status reporting and OAuth URL extraction.

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::vm;
use crate::AgentService;

use super::internal;
use super::paths::{CREDENTIALS_PATH, LOGIN_LOG, LOGIN_STATUS};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeStatus {
    /// unlinked | waiting | verifying | linked | error
    pub state: String,
    pub url: Option<String>,
    pub message: String,
}

/// Extracts the first OAuth URL from the PTY transcript. The CLI wraps the
/// link in ANSI/OSC sequences, so the scan stops at an escape, a control
/// character, whitespace, a quote or an angle bracket.
pub fn extract_oauth_url(log: &str) -> Option<String> {
    const MARKER: &str = "https://claude.com/cai/oauth/authorize?";
    let start = log.find(MARKER)?;
    let rest = &log[start..];
    let end = rest
        .char_indices()
        .find(|(_, c)| {
            matches!(*c, '\u{1b}' | '\u{7}' | '\0' | '\n' | '\r' | ' ' | '\t' | '"' | '\'' | '<' | '>')
        })
        .map(|(i, _)| i)
        .unwrap_or(rest.len());
    let url = rest[..end].trim();
    if url.len() > MARKER.len() {
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
    vm::exec_in_vm(state, vm_name, &["sh".into(), "-c".into(), script.to_string()]).await
}

pub(super) async fn read_status_file(state: &AgentService, vm_name: &str) -> String {
    match run_script(state, vm_name, &format!("cat {LOGIN_STATUS} 2>/dev/null || true")).await {
        Ok((_, stdout, _)) => stdout.trim().to_string(),
        Err(_) => String::new(),
    }
}

/// Reads the CLI transcript and credential marker without changing anything.
pub async fn status(
    state: &AgentService,
    vm_name: &str,
) -> Result<ClaudeStatus, (StatusCode, String)> {
    let script = format!(
        "test -s {credentials} && echo LINKED || echo UNLINKED; \
         echo ---; command -v claude >/dev/null 2>&1 && claude --version 2>/dev/null || true; \
         echo ---; cat {log} 2>/dev/null | tail -c 12000 || true",
        credentials = CREDENTIALS_PATH,
        log = LOGIN_LOG
    );
    let (_, stdout, stderr) = run_script(state, vm_name, &script).await?;
    if stdout.trim().is_empty() && !stderr.trim().is_empty() {
        return Err(internal(format!("claude status: {stderr}")));
    }
    let linked = stdout.lines().next().map(|line| line.trim() == "LINKED").unwrap_or(false);
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
        "linked" => "Claude Code is connected inside your workspace.",
        "waiting" => "Open the Claude sign-in page and paste the code back here.",
        "verifying" => "Claude is verifying the authorization code…",
        "starting" => "Starting the Claude sign-in helper…",
        "error" => "Claude rejected the code. Start the sign-in again.",
        _ => "Claude Code is not connected.",
    };
    Ok(ClaudeStatus {
        state: state_name,
        url,
        message: message.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::extract_oauth_url;

    #[test]
    fn extracts_the_oauth_url_from_an_ansi_transcript() {
        let log = "\u{1b}[38;5;246mhttps://claude.com/cai/oauth/authorize?code=true&client_id=abc&state=xyz\u{1b}]8;;\u{7}\nPaste code";
        let url = extract_oauth_url(log).expect("url");
        assert!(url.starts_with("https://claude.com/cai/oauth/authorize?"));
        assert!(url.contains("client_id=abc"));
        assert!(!url.contains('\u{1b}'));
    }

    #[test]
    fn missing_url_is_none() {
        assert!(extract_oauth_url("no link here").is_none());
    }
}
