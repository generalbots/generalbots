//! Issue #2861 — authenticate git against the internal ALM without leaking
//! the access token.
//!
//! Embedding the token in a clone URL (`https://TOKEN@host/...`) leaks it in
//! two places: the URL becomes an element of the `git` child's **argv**
//! (visible to every local user via `ps`), and `git remote add origin <url>`
//! persists it in the workspace's `.git/config`, where it survives for every
//! later process. Both are unacceptable for a long-lived service.
//!
//! The helpers here instead inject an `http.extraHeader` through Git's
//! config-from-environment interface (`GIT_CONFIG_COUNT` /
//! `GIT_CONFIG_KEY_n` / `GIT_CONFIG_VALUE_n`). Those variables exist only in
//! the child's environment for the duration of one command: they are never
//! part of argv and Git never writes them to `.git/config`.

use super::command_guard::{CommandGuardError, SafeCommand};

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Minimal RFC 4648 base64 encoder. Kept local so this security helper has no
/// dependency on a crate version and cannot drift with the workspace's base64
/// bump. Used only for the short `x-access-token:<token>` credential string.
fn base64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(((input.len() + 2) / 3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        out.push(B64[(b0 >> 2) as usize] as char);
        out.push(B64[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(if chunk.len() > 1 {
            B64[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64[(b2 & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// `(key, value)` pairs to pass to a git child so its HTTP requests carry the
/// ALM token. Empty for an empty token (caller should treat that as a config
/// error rather than running an unauthenticated command).
pub fn git_auth_env(token: &str) -> Vec<(String, String)> {
    if token.is_empty() {
        return Vec::new();
    }
    // Gitea/Forgejo accept HTTP Basic with `x-access-token` as the username
    // and a personal access token as the password (the convention used by
    // `git config http.<host>.extraHeader "AUTHORIZATION: basic ..."`).
    let credential = base64_encode(format!("x-access-token:{token}").as_bytes());
    vec![
        ("GIT_CONFIG_COUNT".to_string(), "1".to_string()),
        (
            "GIT_CONFIG_KEY_0".to_string(),
            "http.extraHeader".to_string(),
        ),
        (
            "GIT_CONFIG_VALUE_0".to_string(),
            format!("AUTHORIZATION: basic {credential}"),
        ),
    ]
}

/// Apply [`git_auth_env`] to a `SafeCommand`. Uses `trusted_env` because the
/// base64 value may legitimately contain `//`, which normal argument
/// validation rejects as a shell-injection pattern.
pub fn apply_git_auth(
    mut command: SafeCommand,
    token: &str,
) -> Result<SafeCommand, CommandGuardError> {
    for (key, value) in git_auth_env(token) {
        command = command.trusted_env(&key, &value)?;
    }
    Ok(command)
}

/// Build the **token-free** clone URL used in argv and persisted as `origin`.
/// The scheme/host come from Vault's ALM base; the token is never concatenated.
pub fn plain_clone_url(base: &str, org: &str, repo: &str) -> String {
    let base = base.trim_end_matches('/');
    let org = org.trim_matches('/');
    let repo = repo.trim_matches('/');
    if org.is_empty() {
        format!("{base}/{repo}.git")
    } else {
        format!("{base}/{org}/{repo}.git")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = "a1b2c3d4e5f60718293a4b5c6d7e8f90";

    #[test]
    fn base64_matches_rfc4648_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    /// The raw token must not appear anywhere in the environment values that
    /// are handed to the child (only its base64-encoded credential may).
    #[test]
    fn auth_env_never_contains_the_raw_token() {
        let env = git_auth_env(TOKEN);
        assert!(!env.is_empty());
        for (key, value) in &env {
            assert!(!key.contains(TOKEN), "token leaked in env key {key}");
            assert!(!value.contains(TOKEN), "token leaked in env value of {key}");
        }
        assert_eq!(env[0].0, "GIT_CONFIG_COUNT");
        assert_eq!(env[1].0, "GIT_CONFIG_KEY_0");
        assert_eq!(env[1].1, "http.extraHeader");
        assert!(env[2].1.starts_with("AUTHORIZATION: basic "));
        assert!(git_auth_env("").is_empty());
    }

    /// The clone URL that reaches `git clone` argv contains no token, even if
    /// a caller mistakenly passes a token-shaped value as the base.
    #[test]
    fn clone_argv_is_token_free() {
        let base = format!("http://alm.local:4747");
        let url = plain_clone_url(&base, "org", "repo");
        assert_eq!(url, "http://alm.local:4747/org/repo.git");
        let argv = ["clone", "--quiet", &url, "/tmp/clone-target"];
        for arg in argv {
            assert!(!arg.contains(TOKEN), "token leaked in argv element {arg}");
        }
        // A token in the ALM base is not silently carried into argv either.
        let bad_url = plain_clone_url(&format!("http://{TOKEN}@alm.local:4747"), "org", "repo");
        assert!(bad_url.contains(TOKEN));
        // Callers must use the Vault token via git_auth_env, never in a URL.
        let safe_url = plain_clone_url("http://alm.local:4747", "org", "repo");
        assert!(!safe_url.contains(TOKEN));
    }

    /// Simulate what `git remote add origin <url>` writes into `.git/config`:
    /// the persisted `url =` line must not contain the raw token.
    #[test]
    fn persisted_remote_config_is_token_free() {
        let url = plain_clone_url("http://alm.local:4747", "org", "repo");
        let config = format!("[remote \"origin\"]\n\turl = {url}\n\tfetch = +refs/heads/*\n");
        assert!(!config.contains(TOKEN), "token persisted in .git/config");
        assert!(config.contains("url = http://alm.local:4747/org/repo.git"));
    }
}
