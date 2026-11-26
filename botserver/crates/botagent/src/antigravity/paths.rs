//! Stable VM paths for the Antigravity login helper (#2576).

pub const ANTIGRAVITY_USER: &str = "antigravity";
pub const ANTIGRAVITY_HOME: &str = "/home/antigravity";
/// Where `agy` keeps its state. Without a desktop keyring (a container, WSL)
/// it falls back to file-based token storage under these folders; the file
/// name is internal to the CLI, so sign-in is detected by a non-empty file
/// whose name marks a token or credential (MCP server tokens excluded).
pub const CREDENTIALS_DIRS: [&str; 2] = ["/home/antigravity/.gemini", "/home/antigravity/.config/antigravity"];
pub(super) const PROMPT_TXT: &str = "/tmp/gb-antigravity-prompt.txt";

/// Root shell test that succeeds once the CLI has stored a sign-in.
pub fn credentials_test() -> String {
    format!(
        "find {} -maxdepth 4 -type f -size +0c \\( -iname '*token*' -o -iname '*cred*' -o -iname '*oauth*' -o -iname 'auth*' \\) ! -iname 'mcp_oauth_tokens.json' 2>/dev/null | grep -q .",
        CREDENTIALS_DIRS.join(" ")
    )
}

#[cfg(test)]
mod tests {
    use super::credentials_test;

    #[test]
    fn the_sign_in_test_looks_for_token_files_in_both_state_folders() {
        let test = credentials_test();
        assert!(test.starts_with("find /home/antigravity/.gemini /home/antigravity/.config/antigravity "));
        assert!(test.contains("\\( -iname '*token*'"));
        assert!(test.contains("! -iname 'mcp_oauth_tokens.json'"));
        assert!(test.ends_with("| grep -q ."));
    }
}
pub const LOGIN_SCRIPT: &str = "/tmp/gb-antigravity-login.py";
pub const LOGIN_LOG: &str = "/tmp/gb-antigravity-login.log";
pub const LOGIN_STATUS: &str = "/tmp/gb-antigravity-login.status";
pub const LOGIN_CODE: &str = "/tmp/gb-antigravity-code";
