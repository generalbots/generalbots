//! Claude Code subscription provider (#1652).
//!
//! The CLI and its credentials live inside the user's agent VM:
//! `claude setup-token` runs under the `claude` Unix user, prints an OAuth
//! URL, and accepts the authorization code through a small PTY bridge. The
//! botserver never sees the token itself — it only reads the status of
//! `/home/claude/.claude/.credentials.json`.
//!
//! Every VM command goes through `vm::exec_in_vm`, which uses `SafeCommand`.
//! User-supplied text (the authorization code and the prompt) is base64
//! encoded before it reaches a shell.

use axum::http::StatusCode;
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::vm;
use crate::AgentService;

pub const CLAUDE_USER: &str = "claude";
pub const CLAUDE_HOME: &str = "/home/claude";
pub const CREDENTIALS_PATH: &str = "/home/claude/.claude/.credentials.json";
const LOGIN_LOG: &str = "/tmp/gb-claude-login.log";
const LOGIN_STATUS: &str = "/tmp/gb-claude-login.status";
const LOGIN_CODE: &str = "/tmp/gb-claude-code";
const LOGIN_SCRIPT: &str = "/tmp/gb-claude-login.py";
const PROMPT_B64: &str = "/tmp/gb-claude-prompt.b64";
const PROMPT_TXT: &str = "/tmp/gb-claude-prompt.txt";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeStatus {
    /// unlinked | waiting | verifying | linked | error
    pub state: String,
    pub url: Option<String>,
    pub message: String,
}

fn internal(msg: String) -> (StatusCode, String) {
    tracing::error!("{msg}");
    (StatusCode::INTERNAL_SERVER_ERROR, "Claude account operation failed".to_string())
}

fn b64(value: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(value.as_bytes())
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

/// The PTY bridge: runs `claude setup-token` as the `claude` user, mirrors its
/// output to a log, and forwards an authorization code dropped in
/// `/tmp/gb-claude-code`. Credentials stay in `/home/claude/.claude`.
const PTY_SCRIPT: &str = r#"import os, pty, select, time
HOME = os.environ.get("GB_CLAUDE_HOME", "/home/claude")
TMP = os.environ.get("GB_CLAUDE_TMP", "/tmp")
LOG = TMP + "/gb-claude-login.log"
STATUS = TMP + "/gb-claude-login.status"
CODE = TMP + "/gb-claude-code"
CRED = HOME + "/.claude/.credentials.json"

def set_status(value):
    try:
        with open(STATUS, "w") as handle:
            handle.write(value)
    except Exception:
        pass

os.environ["HOME"] = HOME
try:
    os.chdir(HOME)
except Exception:
    pass
for path in (LOG, CODE):
    try:
        os.unlink(path)
    except FileNotFoundError:
        pass
    except Exception:
        pass
set_status("starting")
pid, fd = pty.fork()
if pid == 0:
    # npm can still be flushing the freshly installed binary when the PTY
    # starts; retry the transient "Text file busy" instead of losing the login.
    for attempt in range(30):
        try:
            os.execvp("claude", ["claude", "setup-token"])
        except OSError as error:
            if getattr(error, "errno", None) == 26 and attempt < 29:
                time.sleep(0.5)
                continue
            raise
set_status("waiting")
log = open(LOG, "wb", 0)
sent = False
deadline = time.time() + 1800
while time.time() < deadline:
    ready, _, _ = select.select([fd], [], [], 0.5)
    if ready:
        try:
            data = os.read(fd, 4096)
        except OSError:
            break
        if not data:
            break
        log.write(data)
        log.flush()
        text = data.decode("utf-8", "replace")
        if "Invalid code" in text or "OAuth error" in text:
            # The CLI rejected a partial/incorrect paste. Return to the code
            # prompt instead of leaving the status stuck on "verifying".
            sent = False
            set_status("waiting")
            try:
                os.write(fd, b"\r")
            except OSError:
                pass
    if not sent and os.path.exists(CODE):
        code = ""
        try:
            with open(CODE, "r") as handle:
                code = handle.read().strip()
        except Exception:
            code = ""
        if code:
            os.write(fd, (code + "\r").encode())
            sent = True
            set_status("verifying")
            try:
                os.unlink(CODE)
            except OSError:
                pass
    if os.path.exists(CRED):
        set_status("linked")
        time.sleep(2)
        break
if os.path.exists(CRED):
    set_status("linked")
elif sent:
    set_status("error")
else:
    set_status("waiting")
"#;

async fn run_script(
    state: &AgentService,
    vm_name: &str,
    script: &str,
) -> Result<(bool, String, String), (StatusCode, String)> {
    vm::exec_in_vm(state, vm_name, &["sh".into(), "-c".into(), script.to_string()]).await
}

async fn read_status_file(state: &AgentService, vm_name: &str) -> String {
    match run_script(state, vm_name, &format!("cat {LOGIN_STATUS} 2>/dev/null || true")).await {
        Ok((_, stdout, _)) => stdout.trim().to_string(),
        Err(_) => String::new(),
    }
}

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

/// Runs one non-interactive turn with the subscription. The prompt is base64
/// encoded on the host, so quotes, newlines and shell metacharacters are data.
pub async fn run_prompt(
    state: &AgentService,
    vm_name: &str,
    prompt: &str,
    model: &str,
) -> Result<String, (StatusCode, String)> {
    if prompt.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Empty prompt".to_string()));
    }
    let model = sanitize_model(model);
    let model_arg = if model.is_empty() {
        String::new()
    } else {
        format!(" --model {model}")
    };
    let script = format!(
        "printf '%s' '{payload}' | base64 -d > {b64_path}; \
         chown {user}:{user} {b64_path}; chmod 600 {b64_path}; \
         base64 -d {b64_path} > {txt_path}; chown {user}:{user} {txt_path}; chmod 600 {txt_path}; \
         timeout 300 su {user} -s /bin/sh -c 'cd {home} && claude -p \"$(cat {txt_path})\" --output-format text{model_arg}'",
        payload = b64(prompt),
        b64_path = PROMPT_B64,
        txt_path = PROMPT_TXT,
        user = CLAUDE_USER,
        home = CLAUDE_HOME,
        model_arg = model_arg
    );
    let (ok, stdout, stderr) = run_script(state, vm_name, &script).await?;
    if !ok {
        let reason = if stderr.trim().is_empty() { stdout.trim() } else { stderr.trim() };
        return Err(internal(format!("Claude Code failed: {reason}")));
    }
    Ok(stdout.trim().to_string())
}

fn sanitize_model(model: &str) -> String {
    model
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{extract_oauth_url, sanitize_model};

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

    #[test]
    fn model_is_restricted_to_a_safe_alphabet() {
        assert_eq!(sanitize_model("claude-sonnet-4.5; rm -rf /"), "claude-sonnet-4.5rm-rf");
        assert_eq!(sanitize_model(""), "");
    }

    /// Exercises the real PTY bridge with a fake `claude` binary: it prints an
    /// OAuth URL, waits for the code, and writes the credentials file. This is
    /// the same code path the VM runs, without a Claude subscription.
    #[cfg(unix)]
    #[test]
    fn the_pty_bridge_reaches_linked_without_a_real_subscription() {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;

        let root = std::env::temp_dir().join(format!("gb-claude-test-{}", uuid::Uuid::new_v4()));
        let home = root.join("home");
        let bin = root.join("bin");
        std::fs::create_dir_all(home.join(".claude")).expect("home");
        std::fs::create_dir_all(&bin).expect("bin");
        let fake = bin.join("claude");
        std::fs::write(
            &fake,
            "#!/bin/sh\n\
             echo 'https://claude.com/cai/oauth/authorize?code=true&client_id=test&state=abc'\n\
             printf 'Paste code here if prompted >'\n\
             read code\n\
             mkdir -p \"$HOME/.claude\"\n\
             printf '{\"token\":\"%s\"}' \"$code\" > \"$HOME/.claude/.credentials.json\"\n",
        )
        .expect("fake claude");
        let mut permissions = std::fs::metadata(&fake).expect("metadata").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&fake, permissions).expect("chmod");

        let script = root.join("bridge.py");
        std::fs::write(&script, super::PTY_SCRIPT).expect("script");
        let path = format!(
            "{}:{}",
            bin.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let mut child = std::process::Command::new("python3")
            .arg(&script)
            .env("HOME", &home)
            .env("GB_CLAUDE_HOME", &home)
            .env("GB_CLAUDE_TMP", &root)
            .env("PATH", path)
            .spawn()
            .expect("spawn bridge");

        let log_path = root.join("gb-claude-login.log");
        let mut saw_url = false;
        for _ in 0..100 {
            if let Ok(log) = std::fs::read_to_string(&log_path) {
                if log.contains("https://claude.com/cai/oauth/authorize?") {
                    saw_url = true;
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert!(saw_url, "the bridge must expose the OAuth URL");

        let mut code = std::fs::File::create(root.join("gb-claude-code")).expect("code file");
        code.write_all(b"test-code\n").expect("write code");
        drop(code);

        let credential = home.join(".claude/.credentials.json");
        let mut linked = false;
        for _ in 0..100 {
            if credential.exists() {
                linked = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        let _ = child.kill();
        let _ = child.wait();
        let _ = std::fs::remove_dir_all(&root);
        assert!(linked, "the bridge must write the credential after the code");
    }
}
