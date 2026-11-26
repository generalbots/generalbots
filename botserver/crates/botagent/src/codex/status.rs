//! Login status reporting and device-code extraction for Codex.

use axum::http::StatusCode;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::vm;
use crate::AgentService;

use super::paths::{CODEX_HOME, CODEX_USER, CREDENTIALS_PATH, LOGIN_LOG, LOGIN_STATUS};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexStatus {
    /// unlinked | waiting | verifying | linked | error | starting
    pub state: String,
    pub url: Option<String>,
    pub device_code: Option<String>,
    pub message: String,
}

/// Extracts the device authorization URL from the Codex CLI transcript.
/// Codex prints: "Open https://auth.openai.com/... to authenticate"
/// or "Visit https://..." during `codex login --device-auth`.
pub fn extract_device_url(log: &str) -> Option<String> {
    // Codex device auth prints a URL containing "device" or "auth.openai.com"
    let re = Regex::new(r#"https://[^\s\x1b\x07<>'\"`]+"#).ok()?;
    for cap in re.captures_iter(log) {
        let url = cap.get(0)?.as_str().trim_end_matches(['.', ',', ';', ':']);
        if url.contains("auth.openai.com") || url.contains("device") || url.contains("oauth") {
            if url.len() > 20 {
                return Some(url.to_string());
            }
        }
    }
    // Fallback: first https URL that looks like an auth URL
    let re2 = Regex::new(r#"https://[^\s\x1b\x07<>'\"`]+"#).ok()?;
    for cap in re2.captures_iter(log) {
        let url = cap.get(0)?.as_str().trim_end_matches(['.', ',', ';', ':']);
        if url.contains("openai") || url.contains("auth") {
            if url.len() > 20 {
                return Some(url.to_string());
            }
        }
    }
    None
}

/// The transcript without terminal colour/cursor codes, which the CLI wraps
/// around the code (an escape before `ABCD-12345` defeats a word boundary).
fn strip_ansi(log: &str) -> String {
    Regex::new(r"\x1b\[[0-9;?]*[ -/]*[@-~]|\x1b\][^\x07]*\x07")
        .map(|re| re.replace_all(log, "").into_owned())
        .unwrap_or_else(|_| log.to_string())
}

/// Extracts the one-time device code from the Codex CLI transcript.
/// Codex prints: "Enter this one-time code ... \n   XXXX-XXXXX" in colour.
pub fn extract_device_code(log: &str) -> Option<String> {
    let log = &strip_ansi(log);
    // Pattern: "code: XXXX-XXXX" or "enter code XXXX-XXXX" or "XXXX-XXXX"
    let re = Regex::new(r"(?i)(?:code|enter)[:\s]+([A-Z0-9]{4,8}-[A-Z0-9]{4,8})").ok()?;
    if let Some(cap) = re.captures(log) {
        return Some(cap.get(1)?.as_str().to_string());
    }
    // Fallback: look for XXXX-XXXX pattern anywhere
    let re2 = Regex::new(r"\b([A-Z0-9]{4,8}-[A-Z0-9]{4,8})\b").ok()?;
    if let Some(cap) = re2.captures(log) {
        return Some(cap.get(1)?.as_str().to_string());
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

/// Reads the CLI transcript and credential marker without changing anything.
pub async fn status(
    state: &AgentService,
    vm_name: &str,
) -> Result<CodexStatus, (StatusCode, String)> {
    let script = format!(
        "{{ test -s {credentials} || su {user} -s /bin/sh -c 'cd {home} && HOME={home} timeout 20 codex login status' >/dev/null 2>&1; }} && echo LINKED || echo UNLINKED; \
         echo ---; command -v codex >/dev/null 2>&1 && codex --version 2>/dev/null || true; \
         echo ---; cat {log} 2>/dev/null | tail -c 12000 || true",
        credentials = CREDENTIALS_PATH,
        user = CODEX_USER,
        home = CODEX_HOME,
        log = LOGIN_LOG
    );
    let (_, stdout, stderr) = run_script(state, vm_name, &script).await?;
    if stdout.trim().is_empty() && !stderr.trim().is_empty() {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("codex status: {stderr}")));
    }
    let linked = stdout
        .lines()
        .next()
        .map(|line| line.trim() == "LINKED")
        .unwrap_or(false);
    let url = extract_device_url(&stdout);
    let code = extract_device_code(&stdout);
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
        "linked" => "Codex is connected inside your workspace.",
        "waiting" => "Open the URL and enter the code in your browser to connect Codex.",
        "verifying" => "Codex is verifying the authorization code…",
        "starting" => "Starting the Codex sign-in helper…",
        "error" => "Codex rejected the code. Start the sign-in again.",
        _ => "Codex is not connected.",
    };
    Ok(CodexStatus {
        state: state_name,
        url,
        device_code: code,
        message: message.to_string(),
    })
}

#[cfg(test)]
mod device_code_tests {
    use super::extract_device_code;

    #[test]
    fn the_code_is_found_inside_terminal_colours() {
        let log = "2. Enter this one-time code (expires in 15 minutes)\n   \x1b[94mABCD-12EFG\x1b[0m\n";
        assert_eq!(extract_device_code(log).as_deref(), Some("ABCD-12EFG"));
    }
}
