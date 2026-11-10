//! Legacy namespace expectations (#2135).

use super::*;
use std::path::{Path, PathBuf};

#[test]
fn the_shared_owner_is_recognised_in_any_case() {
    assert!(is_legacy_owner("default-user"));
    assert!(is_legacy_owner("Default-User"));
    assert!(is_legacy_owner("  default-user  "));
    assert!(!is_legacy_owner("default-user-2"));
    assert!(!is_legacy_owner(
        "11111111-1111-1111-1111-111111111111"
    ));
    assert!(!is_legacy_owner("local-11111111-1111-1111-1111-111111111111"));
    assert!(!is_legacy_owner(""));
}

#[test]
fn the_legacy_prefix_is_the_shared_namespace() {
    assert_eq!(legacy_docs_prefix(), "users/default-user/docs/");
}

#[test]
fn a_verified_owner_never_resolves_into_the_legacy_prefix() {
    let owner = "11111111-1111-1111-1111-111111111111";
    let path = crate::storage_core::get_user_docs_path(owner);
    assert_eq!(path, format!("users/{owner}/docs"));
    assert!(!path.starts_with(&legacy_docs_prefix()));
}

/// Regression guard for the actual defect: the constant owner must not come
/// back into the crate. `legacy.rs` is the only place allowed to name it, and
/// test modules are excluded because they assert on the literal.
#[test]
fn no_module_but_the_legacy_one_names_the_shared_owner() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rust_files(&root, &mut files);
    assert!(!files.is_empty(), "no sources found under {}", root.display());

    let mut offenders = Vec::new();
    for path in files {
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
        if name == "legacy.rs" || name.ends_with("_tests.rs") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if text.contains(LEGACY_OWNER) {
            offenders.push(format!("{} names the legacy owner", path.display()));
        }
        if text.contains("get_current_user_id") {
            offenders.push(format!("{} uses the removed constant owner", path.display()));
        }
    }

    assert!(
        offenders.is_empty(),
        "docs ownership must come from the verified caller:\n{}",
        offenders.join("\n")
    );
}

fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}
