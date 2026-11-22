//! Stable VM paths for the Codex CLI login helper (#2574).

pub const CODEX_USER: &str = "codex";
pub const CODEX_HOME: &str = "/home/codex";
/// The CLI version a new workspace installs (`@openai/codex@…`); any
/// version runs chat turns only after passing the approval self-check.
pub const CODEX_VERSION: &str = "0.171.0";
/// Where `codex login` stores the ChatGPT sign-in (file credential store).
pub const CREDENTIALS_PATH: &str = "/home/codex/.codex/auth.json";
pub(super) const PROMPT_TXT: &str = "/tmp/gb-codex-prompt.txt";
pub(super) const LAST_MESSAGE: &str = "/home/codex/.gb-codex-last-message.txt";
pub const LOGIN_SCRIPT: &str = "/tmp/gb-codex-login.py";
pub const LOGIN_LOG: &str = "/tmp/gb-codex-login.log";
pub const LOGIN_STATUS: &str = "/tmp/gb-codex-login.status";
pub const LOGIN_CODE: &str = "/tmp/gb-codex-code";
