//! VM-local paths and identities used by the Claude provider.

pub const CLAUDE_USER: &str = "claude";
pub const CLAUDE_HOME: &str = "/home/claude";
pub const CREDENTIALS_PATH: &str = "/home/claude/.claude/.credentials.json";
pub(super) const LOGIN_LOG: &str = "/tmp/gb-claude-login.log";
pub(super) const LOGIN_STATUS: &str = "/tmp/gb-claude-login.status";
pub(super) const LOGIN_CODE: &str = "/tmp/gb-claude-code";
pub(super) const LOGIN_SCRIPT: &str = "/tmp/gb-claude-login.py";
pub(super) const PROMPT_B64: &str = "/tmp/gb-claude-prompt.b64";
pub(super) const PROMPT_TXT: &str = "/tmp/gb-claude-prompt.txt";
