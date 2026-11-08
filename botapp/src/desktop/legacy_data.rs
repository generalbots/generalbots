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

fn migrate_root(new_dir: &Path) {
    if new_dir.exists() {
        return; // the renamed build already owns its data; nothing to carry over
    }
    let Some(parent) = new_dir.parent() else {
        return;
    };
    let legacy_dir = parent.join(LEGACY_IDENTIFIER);
    if !legacy_dir.is_dir() {
        return; // fresh install under the new identifier
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
