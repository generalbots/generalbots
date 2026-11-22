//! The in-VM command of a bridged session.
//!
//! As root it first (re)writes the session's settings file, then runs the
//! CLI as the `claude` user (`su`, never `su -`, so the duplex session tag
//! survives). The flags are the ones the live spike proved on 2.1.330:
//! - `--permission-prompt-tool stdio`: every permission prompt reaches the
//!   bridge as a `can_use_tool` control request;
//! - `--restricted`: user, project and local settings files are ignored (so
//!   the agent cannot pre-approve itself), file tools stay in the working
//!   directory, bypass mode is refused;
//! - `--tools`: only the tools the bridge can describe;
//! - `--settings`: `ask` rules for each of them, so reads and read-only
//!   commands raise a card too (in `manual` mode alone they run unasked).

use base64::Engine;

use crate::claude::{CLAUDE_HOME, CLAUDE_USER};
use crate::cli_turn::cli_model;

use super::actions::TOOLS;

/// The Claude Code release the bridge was proven against (provisioning pins it).
pub const CLAUDE_CODE_VERSION: &str = "2.1.330";
/// Root-owned settings of every bridged session.
pub(super) const SETTINGS_PATH: &str = "/etc/gb/claude-settings.json";

fn settings_json() -> String {
    let tools: Vec<&str> = TOOLS.split(',').collect();
    serde_json::json!({
        "permissions": { "ask": tools, "disableBypassPermissionsMode": "disable" }
    })
    .to_string()
}

fn b64(text: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(text.as_bytes())
}

/// Root part: writes `content` to `path` (0644, atomically).
pub(super) fn write_root_file(path: &str, content: &str) -> String {
    format!(
        "mkdir -p \"$(dirname {path})\" && printf '%s' '{}' | base64 -d > {path}.$$ && chmod 644 {path}.$$ && mv -f {path}.$$ {path}",
        b64(content)
    )
}

/// A session id the CLI gave earlier: a UUID, nothing else reaches the shell.
pub(super) fn clean_session_id(id: &str) -> Option<&str> {
    let ok = !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
    ok.then_some(id)
}

/// `claude -p …` with the model and the session to resume, if any.
pub(super) fn claude_invocation(model: &str, resume: Option<&str>) -> String {
    let model = cli_model(model);
    let mut line = format!(
        "claude -p --input-format stream-json --output-format stream-json --verbose \
         --permission-prompt-tool stdio --permission-mode manual --restricted \
         --tools {TOOLS} --settings {SETTINGS_PATH}"
    );
    if !model.is_empty() {
        line.push_str(&format!(" --model {model}"));
    }
    if let Some(id) = resume.and_then(clean_session_id) {
        line.push_str(&format!(" --resume {id}"));
    }
    line
}

/// The whole `sh -c` script: `root` (extra root steps, may be empty), the
/// settings file, then the CLI as `claude` with `env` (extra `NAME=value`
/// words) and `before` (steps run as `claude` before the CLI).
pub(super) fn session_script(root: &str, env: &str, before: &str, invocation: &str) -> String {
    let settings = write_root_file(SETTINGS_PATH, &settings_json());
    let root = if root.is_empty() { String::new() } else { format!("{root} && ") };
    format!(
        "{root}{settings} && su {CLAUDE_USER} -s /bin/sh -c 'cd {CLAUDE_HOME} && export HOME={CLAUDE_HOME} \
         DISABLE_AUTOUPDATER=1{env} && {before}exec {invocation}'"
    )
}

/// The script of one chat turn.
pub(super) fn turn_script(model: &str, resume: Option<&str>) -> String {
    session_script("", "", "", &claude_invocation(model, resume))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_script_runs_the_cli_as_claude_with_every_tool_asking() {
        let script = turn_script("claude-sonnet-4-5", Some("2f25aa59-31f0-40d5-82a5-b2e43226ab98"));
        assert!(script.contains("su claude -s /bin/sh -c '"));
        assert!(!script.contains("su -"));
        assert!(script.contains("--permission-prompt-tool stdio --permission-mode manual --restricted"));
        assert!(script.contains("--tools Bash,Read,Write,Edit,NotebookEdit,WebFetch"));
        assert!(script.contains("--model claude-sonnet-4-5"));
        assert!(script.contains("--resume 2f25aa59-31f0-40d5-82a5-b2e43226ab98"));
        assert!(script.contains(&format!("> {SETTINGS_PATH}.$$")));
        let settings: serde_json::Value = serde_json::from_str(&settings_json()).expect("json");
        assert_eq!(settings["permissions"]["ask"].as_array().map(Vec::len), Some(6));
    }

    #[test]
    fn nothing_but_a_uuid_is_resumed_and_placeholder_models_are_dropped() {
        let script = turn_script("default", Some("x; rm -rf /"));
        assert!(!script.contains("--resume"));
        assert!(!script.contains("--model"));
        assert_eq!(clean_session_id("abc-123"), Some("abc-123"));
        assert_eq!(clean_session_id("'$(id)'"), None);
    }
}
