//! Resolution of a bot's materialized files inside the work tree.
//!
//! The runtime writes each org into `<org-id>.gborg/`, while several lookups
//! assumed the directory carried the bot name (`<bot>.gborg/`). Every tool call
//! therefore read a missing path and was dropped as "no compiled script".

use std::path::{Path, PathBuf};

/// #2685 — a single, safe path component.
///
/// Bot names, trees and file names are interpolated into work-tree paths. A
/// name that is not a plain component is rejected rather than falling back to
/// the original string: `bots.name` is not unique and a traversal/odd name
/// must never select another tenant's file.
fn is_safe_component(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 255
        && !name.starts_with('.')
        && !name.contains("..")
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Returns the path of `file` inside the bot's `.gbdialog`, accepting both the
/// historical `<bot>.gborg` layout and the real `<org-id>.gborg` one.
pub fn resolve_bot_file(work_root: &str, bot_name: &str, file: &str) -> Option<PathBuf> {
    resolve_bot_file_in(work_root, bot_name, "gbdialog", file)
}

/// Same lookup for any of the bot's sibling trees (`gbdialog`, `gbot`, `gbkb`,
/// `gbdrive`). The prompt lives in `.gbot`, so reusing the dialog-only resolver
/// made `PROMPT-WEB.md` invisible and the model ran without instructions.
pub fn resolve_bot_file_in(
    work_root: &str,
    bot_name: &str,
    tree: &str,
    file: &str,
) -> Option<PathBuf> {
    if !is_safe_component(bot_name) || !is_safe_component(tree) || !is_safe_component(file) {
        log::warn!("Refusing to resolve a bot file with an unsafe path component");
        return None;
    }

    let root = Path::new(work_root);
    let direct = root.join(format!(
        "{bot_name}.gborg/{bot_name}.gbai/{bot_name}.{tree}/{file}"
    ));
    if direct.exists() {
        return Some(direct);
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let dir = entry.path();
            let is_org = dir
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.ends_with(".gborg"))
                .unwrap_or(false);
            if !is_org || !dir.is_dir() {
                continue;
            }
            let candidate = dir.join(format!("{bot_name}.gbai/{bot_name}.{tree}/{file}"));
            if candidate.exists() {
                candidates.push(candidate);
            }
        }
    }

    // `read_dir` order is arbitrary and `bots.name` is not unique. Sorting
    // makes the choice deterministic; more than one answer means the name
    // alone cannot identify the tenant, so fail closed instead of picking an
    // arbitrary (possibly another org's) script.
    candidates.sort();
    candidates.dedup();
    match candidates.len() {
        1 => candidates.into_iter().next(),
        0 => None,
        _ => {
            log::warn!(
                "Ambiguous bot file '{file}' for bot '{bot_name}': {} candidates, refusing to choose",
                candidates.len()
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{is_safe_component, resolve_bot_file_in};

    #[test]
    fn accepts_plain_components() {
        assert!(is_safe_component("default"));
        assert!(is_safe_component("classify_media"));
        assert!(is_safe_component("social-media-post"));
        assert!(is_safe_component("PROMPT-WEB.md"));
        assert!(is_safe_component("foo.ast"));
    }

    #[test]
    fn rejects_traversal_and_odd_names() {
        let too_long = "x".repeat(256);
        for raw in [
            "",
            ".",
            "..",
            ".hidden",
            "foo..bar",
            "../secret",
            "bot/../../secret",
            "..\\secret",
            "foo\\bar",
            "with space",
            "foo\nbas",
            &too_long,
        ] {
            assert!(!is_safe_component(raw), "accepted {raw:?}");
        }
    }

    #[test]
    fn rejects_names_before_building_any_path() {
        let root = std::env::temp_dir().join("gb-tool-paths-2685");
        assert_eq!(resolve_bot_file_in(root.to_str().unwrap_or(""), "../secret", "gbdialog", "x.ast"), None);
        assert_eq!(resolve_bot_file_in(root.to_str().unwrap_or(""), "bot", "gbdialog", "../x.ast"), None);
    }

    #[test]
    fn issue_2664_rejects_full_path_tool_names_in_resolver() {
        // #2664: a USE TOOL operand that still carried separators after a
        // broken normalization was joined into manifest/script paths here.
        // The resolver must fail closed on every such shape.
        let root = std::env::temp_dir().join("gb-tool-paths-2664");
        let root = root.to_str().unwrap_or("");
        for file in [
            "bot.gbdialog/foo.bas",
            "work/bot.gbai/bot.gbdialog/foo",
            "..\\secret.bas",
            "foo/bar.ast",
        ] {
            assert_eq!(
                resolve_bot_file_in(root, "bot", "gbdialog", file),
                None,
                "accepted {file:?}"
            );
        }
    }
}
