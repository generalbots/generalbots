use std::path::Path;

/// #1870 — the bundle identifier moved from `br.com.pragmatismo.botapp` to
/// `org.generalbots.botapp`, and Tauri derives the app data folders from the
/// identifier (`app_data_dir` / `app_local_data_dir` + dot notation). The
/// WebKit profile that holds localStorage — login sessions, settings — lives
/// inside those folders, so without migration every existing install would
/// come up logged out on the first launch of the renamed build.
const LEGACY_IDENTIFIER: &str = "br.com.pragmatismo.botapp";

/// Carries the legacy app data over to the renamed identifier's folders.
///
/// Runs before the Tauri builder creates any webview (main.rs), so the WebKit
/// profile is never opened from both folders at once. Both roots are checked
/// because they differ per platform: on Linux both resolve to `~/.local/share`,
/// on Windows to Roaming vs Local AppData, on macOS to Application Support vs
/// Caches. The move is a plain rename when the source sits on the same volume;
/// otherwise it falls back to a full copy that keeps the legacy folder intact,
/// so a partial copy cannot destroy anything.
pub fn migrate_legacy_data_dir() {
    let mut roots: Vec<std::path::PathBuf> = Vec::new();
    match dirs::data_dir() {
        Some(dir) => roots.push(dir),
        None => log::error!("could not resolve the system data dir for identifier migration"),
    }
    if let Some(dir) = dirs::data_local_dir() {
        if !roots.contains(&dir) {
            roots.push(dir);
        }
    }
    for root in roots {
        migrate_root(&root);
    }
}

/// What the migration does for one data root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Migration {
    /// The renamed build already owns its folder; nothing to carry over.
    SkipTargetExists,
    /// Fresh install: there is no legacy folder.
    SkipNoLegacy,
    /// Move (or copy) the legacy folder into place.
    Migrate,
}

/// Decides the action for one root from the two folder facts.
fn plan_migration(new_dir_exists: bool, legacy_is_dir: bool) -> Migration {
    if new_dir_exists {
        Migration::SkipTargetExists
    } else if legacy_is_dir {
        Migration::Migrate
    } else {
        Migration::SkipNoLegacy
    }
}

fn migrate_root(new_dir: &Path) {
    let legacy_dir = match new_dir.parent() {
        Some(parent) => parent.join(LEGACY_IDENTIFIER),
        None => return,
    };
    match plan_migration(new_dir.exists(), legacy_dir.is_dir()) {
        Migration::SkipTargetExists => return,
        Migration::SkipNoLegacy => return,
        Migration::Migrate => {}
    }
    if std::fs::rename(&legacy_dir, new_dir).is_ok() {
        log::info!(
            "legacy app data moved from {} to {}",
            legacy_dir.display(),
            new_dir.display()
        );
        return;
    }
    match copy_dir(&legacy_dir, new_dir) {
        Ok(()) => log::info!(
            "legacy app data copied from {} to {} (legacy folder kept)",
            legacy_dir.display(),
            new_dir.display()
        ),
        Err(e) => log::error!(
            "could not migrate legacy app data from {} to {}: {e}",
            legacy_dir.display(),
            new_dir.display()
        ),
    }
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{plan_migration, Migration, LEGACY_IDENTIFIER};

    #[test]
    fn the_legacy_identifier_is_the_old_bundle_id() {
        assert_eq!(LEGACY_IDENTIFIER, "br.com.pragmatismo.botapp");
    }

    #[test]
    fn an_existing_target_is_never_touched() {
        // The renamed build already owns its data, even if a legacy folder
        // lingers next to it.
        assert_eq!(plan_migration(true, true), Migration::SkipTargetExists);
        assert_eq!(plan_migration(true, false), Migration::SkipTargetExists);
    }

    #[test]
    fn only_a_real_legacy_folder_is_migrated() {
        assert_eq!(plan_migration(false, true), Migration::Migrate);
        // A fresh install has nothing to carry over.
        assert_eq!(plan_migration(false, false), Migration::SkipNoLegacy);
    }
}
