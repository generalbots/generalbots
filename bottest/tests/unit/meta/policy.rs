//! Rust mirror of `scripts/test_guards.py` (#2074).
//!
//! The Python guard runs in CI, but a local `cargo test -p bottest --test unit`
//! should fail too when someone adds an unregistered module or a file that
//! crosses the 450-line limit. Both checks are filesystem-only and need no
//! service, so they are legitimate unit tests.

use std::path::{Path, PathBuf};

/// AGENTS.md: split a file at 350 lines, never exceed 450.
const MAX_LINES: usize = 450;

fn unit_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("unit")
}

fn rust_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir).expect("read a unit test directory");
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

#[test]
fn every_unit_module_is_declared_in_its_parent_mod() {
    let root = unit_root();
    let files = rust_files(&root);
    assert!(files.len() > 1, "no unit files found under {}", root.display());

    let mut undeclared = Vec::new();
    for path in &files {
        if path.file_name().and_then(|name| name.to_str()) == Some("mod.rs") {
            continue;
        }
        let parent_mod = path.with_file_name("mod.rs");
        let declaration = format!(
            "mod {};",
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or_default()
        );
        let declared = std::fs::read_to_string(&parent_mod)
            .map(|text| text.lines().any(|line| line.trim_end().ends_with(&declaration)))
            .unwrap_or(false);
        if !declared {
            undeclared.push(path.display().to_string());
        }
    }

    assert!(
        undeclared.is_empty(),
        "unit modules missing from their parent mod.rs:\n{}",
        undeclared.join("\n")
    );
}

#[test]
fn unit_files_stay_under_the_line_limit() {
    let root = unit_root();
    let mut over = Vec::new();
    for path in rust_files(&root) {
        let text = std::fs::read_to_string(&path).expect("read a unit test file");
        let lines = text.lines().count();
        if lines > MAX_LINES {
            over.push(format!("{}: {lines} lines", path.display()));
        }
    }
    assert!(
        over.is_empty(),
        "unit files over {MAX_LINES} lines (split by responsibility):\n{}",
        over.join("\n")
    );
}

#[test]
fn the_policy_guard_script_is_checked_in() {
    // The CI step calls this script by path; a rename would silently disable
    // the guard instead of failing the build.
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("bottest is a workspace member");
    let script = repo_root.join("scripts").join("test_guards.py");
    assert!(
        script.is_file(),
        "scripts/test_guards.py is missing at {}",
        script.display()
    );
}
