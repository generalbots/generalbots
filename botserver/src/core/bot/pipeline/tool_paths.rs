//! Resolution of a bot's materialized files inside the work tree.
//!
//! The runtime writes each org into `<org-id>.gborg/`, while several lookups
//! assumed the directory carried the bot name (`<bot>.gborg/`). Every tool call
//! therefore read a missing path and was dropped as "no compiled script".

use std::path::{Path, PathBuf};

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
    let root = Path::new(work_root);
    let direct = root.join(format!(
        "{bot_name}.gborg/{bot_name}.gbai/{bot_name}.{tree}/{file}"
    ));
    if direct.exists() {
        return Some(direct);
    }

    let entries = std::fs::read_dir(root).ok()?;
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
            return Some(candidate);
        }
    }
    None
}
