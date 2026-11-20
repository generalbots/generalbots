//! Shared shape of one non-interactive subscription CLI turn inside the VM.
//!
//! Every CLI runs as its own Unix user with that user's HOME, so the
//! credentials it wrote at sign-in are the ones it reads. The prompt is
//! base64 encoded on the host and reaches the CLI through stdin (or a 0600
//! file owned by that user), never through shell interpolation.

/// Upper bound for one turn inside the VM, in seconds.
pub const TURN_TIMEOUT_SECS: u64 = 300;

use axum::http::StatusCode;
use base64::Engine;

use crate::vm;
use crate::AgentService;

/// The model argument a CLI accepts: an empty string means "the CLI's own
/// default". The placeholder names the account layer uses for "no specific
/// model" (`codex-default`, `antigravity-default`, `auto`) are not real
/// models, so they map to the CLI default instead of a rejected `--model`.
pub fn cli_model(model: &str) -> String {
    let clean: String = model
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_'))
        .collect();
    let lower = clean.to_ascii_lowercase();
    if lower.is_empty() || lower == "auto" || lower == "default" || lower.ends_with("-default") {
        String::new()
    } else {
        clean
    }
}

/// Base64 characters per write: the command guard refuses a `sh -c` script
/// over 8 KB, and a whole conversation easily passes that.
const CHUNK: usize = 6_000;

/// Writes the prompt to `prompt_path` inside the VM in base64 pieces small
/// enough for the command guard, then decodes it into a 0600 file owned by
/// `user`.
async fn stage_prompt(
    state: &AgentService,
    vm_name: &str,
    user: &str,
    prompt_path: &str,
    prompt: &str,
) -> Result<(), (StatusCode, String)> {
    let encoded = base64::engine::general_purpose::STANDARD.encode(prompt.as_bytes());
    let tmp = format!("{prompt_path}.b64");
    let mut first = true;
    for piece in encoded.as_bytes().chunks(CHUNK) {
        let piece = std::str::from_utf8(piece).unwrap_or_default();
        let redirect = if first { ">" } else { ">>" };
        first = false;
        run(state, vm_name, format!("umask 077; printf '%s' '{piece}' {redirect} {tmp}")).await?;
    }
    if first {
        run(state, vm_name, format!("umask 077; : > {tmp}")).await?;
    }
    run(
        state,
        vm_name,
        format!("umask 077; base64 -d {tmp} > {prompt_path} && rm -f {tmp} && chown {user}:{user} {prompt_path}"),
    )
    .await
}

async fn run(state: &AgentService, vm_name: &str, script: String) -> Result<(), (StatusCode, String)> {
    let (ok, stdout, stderr) = vm::exec_in_vm(state, vm_name, &["sh".into(), "-c".into(), script]).await?;
    if ok {
        Ok(())
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not hand the prompt to the agent: {}", failure_reason(&stdout, &stderr)),
        ))
    }
}

/// The shell script that runs `invocation` as `user` from `home` with `HOME`
/// set, under a hard timeout, then removes the prompt file. `invocation` may
/// read the prompt through `$GB_PROMPT`; it must not contain single quotes,
/// `$(`, backticks or `..` (the command guard refuses them).
pub fn turn_script(user: &str, home: &str, prompt_path: &str, invocation: &str) -> String {
    format!(
        "timeout {timeout} su {user} -s /bin/sh -c 'cd {home} && export HOME={home} GB_PROMPT={prompt_path} && {invocation}' </dev/null; \
         rc=$?; rm -f {prompt_path}; exit $rc",
        timeout = TURN_TIMEOUT_SECS,
    )
}

/// One CLI turn: stage the prompt, then run the CLI as its own user.
/// Returns `(ok, stdout, stderr)` of the CLI run.
pub async fn run_turn(
    state: &AgentService,
    vm_name: &str,
    user: &str,
    home: &str,
    prompt_path: &str,
    prompt: &str,
    invocation: &str,
) -> Result<(bool, String, String), (StatusCode, String)> {
    stage_prompt(state, vm_name, user, prompt_path, prompt).await?;
    let script = turn_script(user, home, prompt_path, invocation);
    vm::exec_in_vm(state, vm_name, &["sh".into(), "-c".into(), script]).await
}

/// The reason to log when a turn fails: stderr when there is any, else stdout.
pub fn failure_reason<'a>(stdout: &'a str, stderr: &'a str) -> &'a str {
    if stderr.trim().is_empty() {
        stdout.trim()
    } else {
        stderr.trim()
    }
}

#[cfg(test)]
mod tests {
    use super::{cli_model, turn_script};

    #[test]
    fn placeholder_models_fall_back_to_the_cli_default() {
        assert_eq!(cli_model("codex-default"), "");
        assert_eq!(cli_model("antigravity-default"), "");
        assert_eq!(cli_model(" auto "), "");
        assert_eq!(cli_model(""), "");
        assert_eq!(cli_model("gpt-5-codex"), "gpt-5-codex");
    }

    #[test]
    fn model_is_restricted_to_a_safe_alphabet() {
        assert_eq!(cli_model("o3; rm -rf /"), "o3rm-rf");
        assert_eq!(cli_model("gemini-3-pro'$(id)"), "gemini-3-proid");
    }

    #[test]
    fn the_turn_runs_as_the_cli_user_with_its_home() {
        let script = turn_script("codex", "/home/codex", "/tmp/p.txt", "codex exec - < \"$GB_PROMPT\"");
        assert!(script.contains("su codex -s /bin/sh -c 'cd /home/codex && export HOME=/home/codex GB_PROMPT=/tmp/p.txt && codex exec - < \"$GB_PROMPT\"'"));
        assert!(script.contains("timeout 300 su codex"));
        assert!(script.ends_with("rm -f /tmp/p.txt; exit $rc"));
        assert!(!script.contains("$("), "the command guard refuses $(");
    }

    #[test]
    fn a_staging_piece_fits_the_command_guard() {
        let piece = "A".repeat(super::CHUNK);
        let cmd = format!("umask 077; printf '%s' '{piece}' >> /tmp/gb-codex-prompt.txt.b64");
        assert!(cmd.len() < 8192);
    }
}
