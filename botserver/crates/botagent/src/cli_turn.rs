//! Shared shape of one non-interactive subscription CLI turn inside the VM.
//!
//! Every CLI runs as its own Unix user with that user's HOME, so the
//! credentials it wrote at sign-in are the ones it reads. The prompt reaches
//! the VM on the standard input of one exec and the CLI through stdin (or a
//! 0600 file owned by that user), never through shell interpolation.

/// Upper bound for one turn inside the VM, in seconds.
pub const TURN_TIMEOUT_SECS: u64 = 300;

use axum::http::StatusCode;

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

/// Longest the prompt staging may take: one exec, the prompt on its stdin.
const STAGE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

/// The script that stores standard input as a 0600 file owned by `user`.
fn stage_script(user: &str, path: &str) -> String {
    format!("umask 077; cat > {path}.part && mv -f {path}.part {path} && chown {user}:{user} {path}")
}

/// Writes `bytes` to `path` inside the VM as a 0600 file owned by `user`, in
/// one exec with the bytes on standard input: a whole conversation or a long
/// paste is far beyond what a shell argument may carry, and the old pieces
/// cost one WSL round trip each.
pub async fn stage_file(
    state: &AgentService,
    vm_name: &str,
    user: &str,
    path: &str,
    bytes: Vec<u8>,
) -> Result<(), (StatusCode, String)> {
    let (ok, stdout, stderr) =
        vm::exec_in_vm_with_input(state, vm_name, &stage_script(user, path), bytes, STAGE_TIMEOUT).await?;
    if ok {
        Ok(())
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("could not hand the prompt to the agent: {}", failure_reason(&stdout, &stderr)),
        ))
    }
}

async fn stage_prompt(
    state: &AgentService,
    vm_name: &str,
    user: &str,
    prompt_path: &str,
    prompt: &str,
) -> Result<(), (StatusCode, String)> {
    stage_file(state, vm_name, user, prompt_path, prompt.as_bytes().to_vec()).await
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
    vm::exec_in_vm_within(state, vm_name, &["sh".into(), "-c".into(), script], turn_host_timeout()).await
}

/// Host-side limit of one turn: the in-VM `timeout` plus a minute for the
/// Incus/WSL round trip, so a wedged exec channel cannot hold the turn.
fn turn_host_timeout() -> std::time::Duration {
    std::time::Duration::from_secs(TURN_TIMEOUT_SECS + 60)
}

/// A limit the CLI reported. A spent plan carries
/// [`crate::plan_limit::MARKER`] so the chat ends the turn with its own card
/// instead of retrying; a short-lived rate limit stays an ordinary 429.
pub fn plan_limit(stdout: &str, stderr: &str) -> Option<(StatusCode, String)> {
    if let Some(line) = crate::plan_limit::spent_line(stdout, stderr) {
        tracing::warn!("subscription plan limit: {line}");
        return Some((StatusCode::TOO_MANY_REQUESTS, format!("{} {line}", crate::plan_limit::MARKER)));
    }
    let line = stderr
        .lines()
        .chain(stdout.lines())
        .map(|l| l.trim().trim_start_matches("ERROR:").trim())
        .find(|l| {
            let lower = l.to_ascii_lowercase();
            lower.contains("rate limit") || lower.contains("too many requests")
        })?;
    tracing::warn!("subscription rate limit: {line}");
    Some((StatusCode::TOO_MANY_REQUESTS, format!("429 rate limit: {line}")))
}

/// A clean exit whose whole answer is the CLI's spent-plan line.
pub fn spent_answer(answer: &str) -> Option<(StatusCode, String)> {
    let line = crate::plan_limit::spent_answer(answer)?;
    tracing::warn!("subscription plan limit: {line}");
    Some((StatusCode::TOO_MANY_REQUESTS, format!("{} {line}", crate::plan_limit::MARKER)))
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
    use axum::http::StatusCode;

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
    fn the_prompt_is_staged_from_stdin_not_from_the_script() {
        let script = super::stage_script("antigravity", "/home/antigravity/.gb-prompt.txt");
        assert_eq!(
            script,
            "umask 077; cat > /home/antigravity/.gb-prompt.txt.part && mv -f /home/antigravity/.gb-prompt.txt.part /home/antigravity/.gb-prompt.txt && chown antigravity:antigravity /home/antigravity/.gb-prompt.txt"
        );
        assert!(script.len() < 1024);
    }

    #[test]
    fn a_spent_plan_carries_the_marker() {
        let out = "OpenAI Codex v0.170.0
ERROR: You've hit your usage limit. Try again at Nov 22nd.";
        let (code, msg) = super::plan_limit(out, "").unwrap_or((StatusCode::OK, String::new()));
        assert_eq!(code, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(msg, "subscription plan limit: You've hit your usage limit. Try again at Nov 22nd.");
        assert!(super::plan_limit("all fine", "").is_none());
    }

    #[test]
    fn a_short_lived_rate_limit_stays_a_plain_429() {
        let err = "API Error: 429 rate limit exceeded, retry in 20s";
        let (code, msg) = super::plan_limit("", err).unwrap_or((StatusCode::OK, String::new()));
        assert_eq!(code, StatusCode::TOO_MANY_REQUESTS);
        assert!(msg.starts_with("429 rate limit: "));
    }

    #[test]
    fn a_clean_exit_with_only_the_limit_line_is_a_spent_plan() {
        let answer = "Claude AI usage limit reached|1795400000";
        let msg = super::spent_answer(answer).map(|(_, m)| m).unwrap_or_default();
        assert_eq!(msg, "subscription plan limit: Claude AI usage limit reached|1795400000");
        assert!(super::spent_answer("Here is your summary.").is_none());
    }
}
