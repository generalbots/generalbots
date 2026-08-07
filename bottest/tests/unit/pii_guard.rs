use regex::bytes::Regex;
use std::path::{Path, PathBuf};

// PII safeguard regression test (issue #739). The production botserver must
// never reference the internal staging clean-room identifier `stg_rob` (a
// gborg tenant used only during data staging operations) from any handler.
//
// This is a static source scan — it requires no server and runs offline as a
// unit test. It scans the Rust, TOML, and YAML sources under `botserver/`
// (excluding the binary `target/` directory and `migrations/`) and asserts
// that none of them reference the forbidden token or its base64 variant.

const FORBIDDEN: &[&str] = &[
    "stg_rob",
    // Base64 variant of `stg_rob`.
    "c3RnX3JvYg",
];

/// Resolves the workspace root that holds the `botserver` member. The bottest
/// crate sits directly under the workspace root, so its manifest's parent is
/// the scan target.
fn workspace_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| manifest.to_path_buf())
}

/// Recursively lists source files, skipping the compiled target directory.
fn source_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "target" || name == ".git" || name == "node_modules" {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else if is_scanned_extension(&path) {
                out.push(path);
            }
        }
    }
    out
}

fn is_scanned_extension(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs" | "toml" | "yaml" | "yml")
    )
}

/// Collects paths whose content contains any forbidden token.
fn collect_forbidden(root: &Path, hits: &mut Vec<String>) {
    let pattern = FORBIDDEN
        .iter()
        .map(|t| regex::escape(t))
        .collect::<Vec<_>>()
        .join("|");
    let re = Regex::new(&format!("(?i)(?:{pattern})"))
        .expect("valid forbidden token regex");

    for path in source_files(root) {
        if let Ok(content) = std::fs::read(&path) {
            if re.is_match(&content) {
                hits.push(path.display().to_string());
            }
        }
    }
}

#[test]
fn no_stg_rob_pii_in_production_sources() {
    let root = workspace_root();
    assert!(root.exists(), "workspace root not found: {root:?}");

    let mut hits = Vec::new();
    collect_forbidden(&root, &mut hits);

    assert!(
        hits.is_empty(),
        "Forbidden PII token referenced in {} file(s):\n{}",
        hits.len(),
        hits.join("\n")
    );
}
