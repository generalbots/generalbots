//! Stable VM paths for the Codex CLI login helper (#2574).

pub const CODEX_USER: &str = "codex";
pub const CODEX_HOME: &str = "/home/codex";
pub const CREDENTIALS_PATH: &str = "/home/codex/.codex/credentials.json"; // adjust if Codex stores elsewhere
pub const LOGIN_SCRIPT: &str = "/tmp/gb-codex-login.py";
pub const LOGIN_LOG: &str = "/tmp/gb-codex-login.log";
pub const LOGIN_STATUS: &str = "/tmp/gb-codex-login.status";
pub const LOGIN_CODE: &str = "/tmp/gb-codex-code";
