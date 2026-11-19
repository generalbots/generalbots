//! Contract guard for the endpoint inventory (#2546).
//!
//! `api.find` advertises every route in `endpoint_inventory` plus the source
//! file that registers it. Nothing used to check either claim, so a module
//! split or a copy-pasted entry could rot the catalogue invisibly and the model
//! would keep being told about routes it cannot call.

use super::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Every entry that names a source file must still resolve to one.
#[test]
fn documented_source_files_still_exist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut broken = Vec::new();
    for endpoint in all_routes() {
        let Some(pointer) = source_pointer(endpoint.summary) else {
            continue;
        };
        if !resolves(root, pointer) {
            broken.push(format!(
                "{} {} -> {}",
                endpoint.method, endpoint.path, pointer
            ));
        }
    }
    assert!(
        broken.is_empty(),
        "{} documented endpoint(s) point at a source file that no longer exists:\n{}",
        broken.len(),
        broken.join("\n")
    );
}

/// `registered in crates/x/src/y.rs;` → `crates/x/src/y.rs`
fn source_pointer(summary: &str) -> Option<&str> {
    let rest = summary.split("registered in ").nth(1)?;
    let token = rest.split_whitespace().next()?;
    let cleaned = token.trim_end_matches([';', ',', '.']);
    cleaned.ends_with(".rs").then_some(cleaned)
}

/// A split module keeps the old pointer valid through its `mod.rs`.
fn resolves(root: &Path, pointer: &str) -> bool {
    let direct = root.join(pointer);
    if direct.exists() {
        return true;
    }
    let stem = PathBuf::from(pointer);
    match (stem.parent(), stem.file_stem()) {
        (Some(parent), Some(name)) => {
            let mut dir_module = parent.to_path_buf();
            dir_module.push(name);
            dir_module.push("mod.rs");
            root.join(dir_module).exists()
        }
        _ => false,
    }
}

/// Two entries for one method+path would make `api.find` contradictory and the
/// router ambiguous, so the catalogue must stay unique.
#[test]
fn no_duplicate_method_and_path() {
    let mut seen: HashSet<(&str, &str)> = HashSet::new();
    let mut duplicates = Vec::new();
    for endpoint in all_routes() {
        if !seen.insert((endpoint.method, endpoint.path)) {
            duplicates.push(format!("{} {}", endpoint.method, endpoint.path));
        }
    }
    assert!(
        duplicates.is_empty(),
        "duplicate endpoint(s) advertised to the LLM:\n{}",
        duplicates.join("\n")
    );
}

/// A path the LLM is told exists must look like a mounted route.
#[test]
fn every_path_is_absolute_and_every_method_is_known() {
    const METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE"];
    for endpoint in all_routes() {
        assert!(
            endpoint.path.starts_with('/'),
            "{} is not an absolute path",
            endpoint.path
        );
        assert!(
            METHODS.contains(&endpoint.method),
            "{} {} uses an unknown method",
            endpoint.method,
            endpoint.path
        );
    }
}
