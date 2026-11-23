//! Offline unit tests for the Source Control path guard (#2862).
//!
//! `resolve_repo` decides which checkout a git command runs in and
//! `validate_git_path` / `ensure_git_path_contained` guard every pathspec, so
//! every rejection branch is a security boundary. The tests below use hostile
//! payloads only — absolute paths, traversal, NUL, separators and overlong
//! names — and never touch the filesystem or require a running server.

use super::{resolve_repo, validate_git_path, validate_ref_name, validate_repo_name, DEFAULT_REPO};

#[test]
fn a_missing_repo_falls_back_to_the_default_checkout() {
    assert_eq!(resolve_repo(None), DEFAULT_REPO);
}

#[test]
fn a_blank_repo_falls_back_to_the_default_checkout() {
    for blank in ["", " ", "   ", "\t", "\n"] {
        assert_eq!(resolve_repo(Some(blank)), DEFAULT_REPO, "{blank:?}");
    }
}

#[test]
fn absolute_repo_paths_are_rejected_not_used_verbatim() {
    // Regression for #2862: these used to be handed to `git -C` unchanged.
    for hostile in [
        "/tmp/repo",
        "/var/tmp/repo",
        "/opt/gbo/data/vibe-workspaces/x",
        "/srv/git",
        "/etc",
        "/",
        "//server/share/repo",
        "C:\\repo",
        "\\\\server\\share\\repo",
    ] {
        assert_eq!(
            resolve_repo(Some(hostile)),
            DEFAULT_REPO,
            "{hostile:?} must never be used as a checkout"
        );
        assert!(
            validate_repo_name(hostile).is_err(),
            "{hostile:?} must fail project-id validation"
        );
    }
}

#[test]
fn surrounding_whitespace_does_not_rescue_an_absolute_path() {
    assert_eq!(resolve_repo(Some("  /tmp/repo  ")), DEFAULT_REPO);
    assert!(validate_repo_name("  /tmp/repo  ").is_err());
}

#[test]
fn relative_names_with_separators_or_traversal_are_rejected() {
    for hostile in [
        "..",
        ".",
        "../etc",
        "../../etc/passwd",
        "a/b",
        "a\\b",
        "repo/../../etc",
        "./repo",
        "repo/./x",
        "repo//x",
    ] {
        assert_eq!(
            resolve_repo(Some(hostile)),
            DEFAULT_REPO,
            "{hostile:?} must not escape the workspace"
        );
    }
}

#[test]
fn nul_and_control_characters_are_rejected_everywhere() {
    for hostile in ["repo\0name", "repo\nname", "repo\tname", "\0", "a\u{0007}b"] {
        assert_eq!(resolve_repo(Some(hostile)), DEFAULT_REPO, "{hostile:?}");
        assert!(validate_repo_name(hostile).is_err(), "{hostile:?}");
        assert!(validate_git_path(hostile, false).is_err(), "{hostile:?}");
    }
}

#[test]
fn overlong_project_names_are_rejected() {
    let long = "a".repeat(129);
    assert_eq!(resolve_repo(Some(&long)), DEFAULT_REPO);
    assert!(validate_repo_name(&long).is_err());
}

#[test]
fn valid_project_names_pass_validation() {
    for name in ["e2e-gh-4404", "my_project", "project.v2", "A-1_b.2", "My Project"] {
        assert_eq!(validate_repo_name(name), Ok(name));
    }
}

#[test]
fn diff_and_commit_pathspecs_reject_traversal_and_absolute_paths() {
    for hostile in [
        "../secret",
        "a/../../secret",
        "/etc/passwd",
        "C:\\secret",
        "a\\..\\b",
        "repo\0file",
        "a\nfile",
    ] {
        assert!(
            validate_git_path(hostile, false).is_err(),
            "{hostile:?} must be rejected as a pathspec"
        );
    }
}

#[test]
fn tree_and_diff_accept_plain_relative_paths() {
    assert_eq!(validate_git_path("src/main.rs", false), Ok("src/main.rs"));
    assert_eq!(validate_git_path("docs", false), Ok("docs"));
    assert_eq!(validate_git_path("", true), Ok(""));
    assert!(validate_git_path("", false).is_err());
}

#[test]
fn ref_names_cannot_be_reinterpreted_as_git_options() {
    assert_eq!(validate_ref_name("feature/login"), Ok("feature/login"));
    for hostile in ["-B", "--upload-pack=x", "ref\0name", "ref\nname", ""] {
        assert!(validate_ref_name(hostile).is_err(), "{hostile:?}");
    }
}

#[cfg(unix)]
#[test]
fn a_symlink_pointing_outside_the_root_is_rejected() {
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let unique = format!(
        "gb-git-guard-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    let base = std::env::temp_dir().join(&unique);
    let outside = std::env::temp_dir().join(format!("{unique}-outside"));
    let _ = std::fs::create_dir_all(&base);
    let _ = std::fs::create_dir_all(&outside);
    let link = base.join("escape");
    if std::os::unix::fs::symlink(&outside, &link).is_ok() {
        assert!(
            super::ensure_contained(&link.join("secret.txt"), &base).is_err(),
            "a symlink must not resolve outside the workspace root"
        );
        let root = base.to_string_lossy();
        assert!(
            super::ensure_git_path_contained(&root, "escape/secret.txt").is_err(),
            "a git pathspec must not follow a symlink out of the repo"
        );
    }
    let _ = std::fs::remove_dir_all(&base);
    let _ = std::fs::remove_dir_all(&outside);
}

#[test]
fn paths_with_parent_segments_are_never_accepted_even_when_normalized() {
    // `/tmp/../etc` is absolute; `a/../b` looks normalized but still contains
    // a parent segment and must never reach git as a pathspec.
    assert!(validate_git_path("/tmp/../etc", false).is_err());
    assert!(validate_git_path("a/../b", false).is_err());
    assert_eq!(resolve_repo(Some("/tmp/../etc")), DEFAULT_REPO);
}
