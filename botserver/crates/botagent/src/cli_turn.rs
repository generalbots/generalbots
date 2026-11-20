//! Shared shape of one non-interactive subscription CLI turn inside the VM.
//!
//! Every CLI runs as its own Unix user with that user's HOME, so the
//! credentials it wrote at sign-in are the ones it reads. The prompt is
//! base64 encoded on the host and reaches the CLI through stdin (or a 0600
//! file owned by that user), never through shell interpolation.

/// Upper bound for one turn inside the VM, in seconds.
pub const TURN_TIMEOUT_SECS: u64 = 300;

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

/// Builds the root shell script that writes the prompt to `prompt_path`
/// (owned by `user`, mode 0600) and runs `invocation` as `user` from `home`
/// with `HOME` set, under a hard timeout. `invocation` may read the prompt
/// file through `$GB_PROMPT`; it must not contain single quotes.
pub fn turn_script(user: &str, home: &str, prompt_path: &str, prompt_b64: &str, invocation: &str) -> String {
    format!(
        "umask 077; printf '%s' '{prompt_b64}' | base64 -d > {prompt_path} && chown {user}:{user} {prompt_path} || exit 4; \
         timeout {timeout} su {user} -s /bin/sh -c 'cd {home} && export HOME={home} GB_PROMPT={prompt_path} && {invocation}' </dev/null; \
         rc=$?; rm -f {prompt_path}; exit $rc",
        timeout = TURN_TIMEOUT_SECS,
    )
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
        let script = turn_script("codex", "/home/codex", "/tmp/p.txt", "aGk=", "codex exec - < \"$GB_PROMPT\"");
        assert!(script.contains("su codex -s /bin/sh -c 'cd /home/codex && export HOME=/home/codex GB_PROMPT=/tmp/p.txt && codex exec - < \"$GB_PROMPT\"'"));
        assert!(script.contains("chown codex:codex /tmp/p.txt"));
        assert!(script.contains("timeout 300 su codex"));
        assert!(script.ends_with("rm -f /tmp/p.txt; exit $rc"));
    }
}
