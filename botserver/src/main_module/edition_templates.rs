//! Edition-specific bot templates embedded in the binary.
//!
//! The `gbgov` edition ships its own default assistant instead of cloning the
//! upstream `templates.git` repository. The template files live under
//! `bottemplates/gbgov/` in the source tree and are compiled into the binary
//! via `rust-embed` so the product works offline from its first start.
//!
//! At boot the embedded tree is extracted to `work/templates/bots/gbgov/` — the
//! path `BootstrapManager::templates_source_dir` already falls back to when the
//! git clone is absent — and a cleanup pass removes files the edition no longer
//! ships (e.g. tools dropped between versions).

#[cfg(feature = "gbgov")]
use rust_embed::RustEmbed;

/// Embedded gbgov templates — `bottemplates/gbgov/` relative to botserver's
/// `Cargo.toml`.  When the directory does not yet exist at compile time
/// (another agent is still writing it), the embed is empty and
/// `extract_edition_templates` logs a warning and returns early.
#[cfg(feature = "gbgov")]
#[derive(RustEmbed)]
#[folder = "../bottemplates/gbgov"]
struct GbgovTemplates;

/// Target directory where the embedded templates are extracted to.
#[cfg(feature = "gbgov")]
const EDITION_TEMPLATES_DIR: &str = "work/templates/bots/gbgov";

/// Extracts the edition's embedded templates to `work/templates/bots/gbgov/`
/// and removes files that no longer ship with the edition.
///
/// Returns `true` when at least one file was written.
#[cfg(feature = "gbgov")]
pub fn extract_edition_templates() -> bool {
    use log::{debug, info, warn};
    use std::path::Path;

    let files: Vec<_> = GbgovTemplates::iter().collect();
    if files.is_empty() {
        warn!(
            "gbgov edition templates are empty at compile time; \
             the default assistant will use the generic fallback"
        );
        return false;
    }

    let target = Path::new(EDITION_TEMPLATES_DIR);
    if let Err(e) = std::fs::create_dir_all(target) {
        warn!("Failed to create edition templates dir {}: {e}", target.display());
        return false;
    }

    // Collect paths the edition ships so we can remove stale ones afterwards.
    let mut shipped: std::collections::HashSet<std::path::PathBuf> =
        std::collections::HashSet::new();

    let mut written = 0usize;
    for rel in &files {
        let Some(content) = GbgovTemplates::get(rel) else {
            continue;
        };
        let dest = target.join(rel.as_ref());
        shipped.insert(dest.clone());

        // Skip if the file on disk is identical (avoids unnecessary writes and
        // preserves timestamps for the drive monitor's change detection).
        if dest.is_file() {
            if let Ok(existing) = std::fs::read(&dest) {
                if existing == content.data.as_ref() {
                    debug!("edition template unchanged: {}", rel);
                    continue;
                }
            }
        }

        if let Some(parent) = dest.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                warn!("cannot create dir for {}: {e}", dest.display());
                continue;
            }
        }
        if let Err(e) = std::fs::write(&dest, content.data.as_ref()) {
            warn!("cannot write edition template {}: {e}", dest.display());
            continue;
        }
        written += 1;
    }

    // Remove files the edition no longer ships. Walk the target tree and delete
    // anything not in `shipped`. This handles dropped tools (e.g. `weather.bas`)
    // and renamed prompts.
    if let Ok(walker) = walk_dir_files(target) {
        for existing in walker {
            if !shipped.contains(&existing) {
                info!("removing stale edition template: {}", existing.display());
                let _ = std::fs::remove_file(&existing);
            }
        }
    }

    if written > 0 {
        info!(
            "Extracted {written} edition template files to {}",
            target.display()
        );
    } else {
        debug!("Edition templates are up to date");
    }
    written > 0
}

#[cfg(not(feature = "gbgov"))]
pub fn extract_edition_templates() -> bool {
    false
}

/// Returns `true` when the build carries edition-specific templates that
/// replace the upstream git repository.
pub fn has_edition_templates() -> bool {
    cfg!(feature = "gbgov")
}

/// Recursively collects all file paths under `dir`.
#[cfg(any(feature = "gbgov", test))]
fn walk_dir_files(dir: &std::path::Path) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut result = Vec::new();
    if !dir.is_dir() {
        return Ok(result);
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            result.extend(walk_dir_files(&path)?);
        } else {
            result.push(path);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_edition_templates_matches_feature() {
        let expected = cfg!(feature = "gbgov");
        assert_eq!(has_edition_templates(), expected);
    }

    #[test]
    fn walk_dir_files_empty_on_missing_dir() {
        let files = walk_dir_files(std::path::Path::new("nonexistent-dir-2562"));
        assert!(files.is_ok());
        assert!(files.map(|v| v.is_empty()).unwrap_or(false));
    }
}
