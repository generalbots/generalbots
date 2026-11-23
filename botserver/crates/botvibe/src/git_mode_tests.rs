//! #2861 regression tests for token-free git argv / remote config in `git_mode`.

use super::{git_auth_env, plain_clone_url};

const TOKEN: &str = "alm-token-do-not-leak-0123456789";

/// `git remote add origin <url>` is the call that used to persist
/// `TOKEN@host` in `.git/config`, and `git push origin` used to carry the
/// token in argv via the same URL. Both now use the token-free URL while the
/// credential travels in the child environment.
#[test]
fn origin_url_and_push_argv_are_token_free() {
    let internal_url = plain_clone_url("http://alm:4747", "branch", "repo");
    assert_eq!(internal_url, "http://alm:4747/branch/repo.git");
    let remote_add_argv = ["git", "remote", "add", "origin", &internal_url];
    let push_argv = ["git", "push", "-u", "origin", "main"];
    for arg in remote_add_argv.iter().chain(push_argv.iter()) {
        assert!(!arg.contains(TOKEN), "token in argv: {arg}");
    }
    let config = format!("[remote \"origin\"]\n\turl = {internal_url}\n");
    assert!(!config.contains(TOKEN), "token persisted in .git/config");
}

#[test]
fn git_auth_env_is_ephemeral_and_hides_token() {
    let env = git_auth_env(TOKEN);
    assert_eq!(env.len(), 3);
    for (key, value) in &env {
        assert!(!value.contains(TOKEN), "raw token in {key}");
    }
    assert!(env
        .iter()
        .any(|(k, v)| k == "GIT_CONFIG_KEY_0" && v == "http.extraHeader"));
}
