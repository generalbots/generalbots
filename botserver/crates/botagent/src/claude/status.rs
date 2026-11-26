//! Login status reporting and OAuth URL extraction.

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::vm;
use crate::AgentService;

use super::internal;
use super::paths::{CLAUDE_HOME, CLAUDE_USER, CREDENTIALS_PATH, LOGIN_LOG, LOGIN_STATUS};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeStatus {
    /// unlinked | waiting | verifying | linked | error
    pub state: String,
    pub url: Option<String>,
    pub message: String,
}

/// Sign-in page prefixes the CLI has printed: older releases used the
/// `claude.com/cai` path, 2.1.330 prints `claude.ai/oauth`.
const OAUTH_PREFIXES: [&str; 2] = ["https://claude.ai/oauth/authorize?", "https://claude.com/cai/oauth/authorize?"];

/// Extracts the first OAuth URL from the PTY transcript. The CLI wraps the
/// link in ANSI/OSC-8 sequences (`ESC ] 8 ;; <url> BEL`), so the scan stops
/// at an escape, a control character, whitespace, a quote or an angle bracket.
pub fn extract_oauth_url(log: &str) -> Option<String> {
    let (start, marker) = OAUTH_PREFIXES
        .iter()
        .filter_map(|marker| log.find(marker).map(|at| (at, *marker)))
        .min_by_key(|(at, _)| *at)?;
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

/// The reported phase. The helper writes `waiting` as soon as the CLI starts,
/// before it printed the link; until a link is captured the sign-in is still
/// `starting`, so the shell never shows a waiting state without a link.
fn phase_name(linked: bool, phase: String, has_url: bool) -> String {
    if linked {
        "linked".to_string()
    } else if phase == "waiting" && !has_url {
        "starting".to_string()
    } else if !phase.is_empty() {
        phase
    } else if has_url {
        "waiting".to_string()
    } else {
        "unlinked".to_string()
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

/// Reads the CLI transcript and credential marker without changing anything.
pub async fn status(
    state: &AgentService,
    vm_name: &str,
) -> Result<ClaudeStatus, (StatusCode, String)> {
    let script = format!(
        "{{ test -s {credentials} || su {user} -s /bin/sh -c 'cd {home} && HOME={home} timeout 20 claude auth status --json' 2>/dev/null | grep -q '\"loggedIn\": *true'; }} && echo LINKED || echo UNLINKED; \
         echo ---; command -v claude >/dev/null 2>&1 && claude --version 2>/dev/null || true; \
         echo ---; cat {log} 2>/dev/null | tail -c 12000 || true",
        credentials = CREDENTIALS_PATH,
        user = CLAUDE_USER,
        home = CLAUDE_HOME,
        log = LOGIN_LOG
    );
    let (_, stdout, stderr) = run_script(state, vm_name, &script).await?;
    if stdout.trim().is_empty() && !stderr.trim().is_empty() {
        return Err(internal(format!("claude status: {stderr}")));
    }
    let linked = stdout
        .lines()
        .next()
        .map(|line| line.trim() == "LINKED")
        .unwrap_or(false);
    let url = extract_oauth_url(&stdout);
    let phase = read_status_file(state, vm_name).await;
    let state_name = phase_name(linked, phase, url.is_some());
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
    use super::{extract_oauth_url, phase_name};

    /// The line Claude Code 2.1.330 prints, inside its OSC-8 hyperlink.
    #[test]
    fn extracts_the_claude_ai_url_from_an_osc8_hyperlink() {
        let log = "Browser didn't open? Use the url below to sign in (c to copy)\r\n\r\n\
                   \u{1b}]8;;https://claude.ai/oauth/authorize?code=true&client_id=x&state=y\u{7}\
                   \u{1b}[94mhttps://claude.ai/oauth/authorize?code=true&client_id=x&state=y\u{1b}[39m\
                   \u{1b}]8;;\u{7}\r\n\r\nPaste code here if prompted > ";
        assert_eq!(
            extract_oauth_url(log).as_deref(),
            Some("https://claude.ai/oauth/authorize?code=true&client_id=x&state=y")
        );
        assert!(extract_oauth_url("https://claude.ai/oauth/authorize?\u{7}").is_none());
    }

    #[test]
    fn waiting_without_a_link_is_still_starting() {
        assert_eq!(phase_name(false, "waiting".into(), false), "starting");
        assert_eq!(phase_name(false, "waiting".into(), true), "waiting");
        assert_eq!(phase_name(false, "verifying".into(), false), "verifying");
        assert_eq!(phase_name(true, "waiting".into(), false), "linked");
        assert_eq!(phase_name(false, String::new(), true), "waiting");
        assert_eq!(phase_name(false, String::new(), false), "unlinked");
    }

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
