//! Workspace manifest invariants (#2074).
//!
//! The unit suite must know which crates exist before it can assert anything
//! about them. This module parses the root `Cargo.toml` `members`/`exclude`
//! arrays with no extra dependency and checks that every member is a real
//! crate directory, so a typo or a deleted crate fails the offline suite
//! instead of a release build.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| manifest.to_path_buf())
}

/// Reads the quoted strings of a top-level `key = [ ... ]` array.
///
/// The root manifest is hand-maintained and small, so a full TOML parser would
/// add a dependency for one array. Entries are one per line (the convention in
/// this file); inline entries are supported as well.
fn manifest_array(text: &str, key: &str) -> Vec<String> {
    let mut entries = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if !inside {
            if trimmed.starts_with(&format!("{key} = [")) {
                inside = true;
                collect_quoted(trimmed, &mut entries);
                if trimmed.ends_with(']') {
                    break;
                }
            }
            continue;
        }
        collect_quoted(trimmed, &mut entries);
        if trimmed.contains(']') {
            break;
        }
    }
    entries
}

fn collect_quoted(line: &str, out: &mut Vec<String>) {
    // Odd indices are the quoted spans: `"a", "b"` splits into
    // ["", "a", ", ", "b", ""].
    for (index, part) in line.split('"').enumerate() {
        if index % 2 == 1 {
            out.push(part.to_string());
        }
    }
}

fn manifest_text(root: &Path) -> String {
    let path = root.join("Cargo.toml");
    std::fs::read_to_string(&path).expect("read the root Cargo.toml")
}

fn members(root: &Path) -> Vec<String> {
    manifest_array(&manifest_text(root), "members")
}

fn excluded(root: &Path) -> Vec<String> {
    manifest_array(&manifest_text(root), "exclude")
}

#[test]
fn every_workspace_member_is_a_real_crate() {
    let root = workspace_root();
    let members = members(&root);
    assert!(
        members.len() > 10,
        "root Cargo.toml members array did not parse: {members:?}"
    );
    for member in &members {
        let dir = root.join(member);
        assert!(
            dir.join("Cargo.toml").is_file(),
            "workspace member `{member}` has no Cargo.toml at {}",
            dir.display()
        );
    }
}

#[test]
fn workspace_members_are_listed_once() {
    let root = workspace_root();
    let members = members(&root);
    let unique: BTreeSet<&String> = members.iter().collect();
    assert_eq!(
        unique.len(),
        members.len(),
        "duplicate member in the root Cargo.toml: {members:?}"
    );
}

#[test]
fn vendored_office_crates_stay_outside_the_workspace() {
    let root = workspace_root();
    let excluded = excluded(&root);
    for vendored in ["3rdparty/docx-rs", "3rdparty/umya-spreadsheet"] {
        assert!(
            excluded.iter().any(|entry| entry == vendored),
            "`{vendored}` must stay in the workspace `exclude` list: it is \
             third-party code, not a workspace member"
        );
        assert!(
            !members(&root).iter().any(|entry| entry == vendored),
            "`{vendored}` is vendored third-party code and must not be a member"
        );
    }
    assert!(
        !members(&root).iter().any(|entry| entry.starts_with("3rdparty/")),
        "no vendored crate may be a workspace member"
    );
}
