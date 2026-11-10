//! Offline unit tests for the repository path resolver (#2071).
//!
//! The git app takes a `?repo=` parameter from the UI. `resolve_repo` decides
//! which checkout a command runs in, so every branch that rejects input is a
//! security boundary: no traversal, no separators, no unbounded length. The
//! tests below cover the branches that never touch the filesystem — the "bare
//! project name" branch probes `vibe-workspaces/<name>/.git` and is left to the
//! integration suite.

use super::{resolve_repo, DEFAULT_REPO};

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
fn temporary_and_absolute_paths_are_used_verbatim() {
    for path in ["/tmp/repo", "/var/tmp/repo", "/opt/gbo/data/vibe-workspaces/x", "/srv/git"] {
        assert_eq!(resolve_repo(Some(path)), path, "{path}");
    }
}

#[test]
fn surrounding_whitespace_is_trimmed_from_a_path() {
    assert_eq!(resolve_repo(Some("  /tmp/repo  ")), "/tmp/repo");
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
    ] {
        assert_eq!(
            resolve_repo(Some(hostile)),
            DEFAULT_REPO,
            "{hostile:?} must not escape the workspace"
        );
    }
}

#[test]
fn an_absolute_path_is_never_rewritten_into_a_project_lookup() {
    // `/tmp/../etc` is still an explicit absolute request: the resolver hands it
    // to git, which rejects it, instead of silently mapping it to a project.
    assert_eq!(resolve_repo(Some("/tmp/../etc")), "/tmp/../etc");
}

#[test]
fn overlong_project_names_are_rejected() {
    let long = "a".repeat(129);
    assert_eq!(resolve_repo(Some(&long)), DEFAULT_REPO);
}
