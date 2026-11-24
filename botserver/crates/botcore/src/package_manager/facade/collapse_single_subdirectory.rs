use super::*;

/// After extracting a zip/tar.gz, some archives (e.g. EDB PostgreSQL for Windows)
/// place files inside a subdirectory (e.g. `pgsql/`). This function collapses
/// a single top-level subdirectory by moving its contents up and removing it.
pub(super) fn collapse_single_subdirectory(bin_path: &std::path::Path) -> Result<()> {
    let entries: Vec<_> = match std::fs::read_dir(bin_path) {
        Ok(iter) => iter.flatten().collect(),
        Err(_) => return Ok(()),
    };
    let dirs: Vec<_> = entries.iter().filter(|e| e.path().is_dir()).collect();
    if dirs.len() == 1 && entries.len() == 1 {
        let sub = dirs[0].path();
        for entry in std::fs::read_dir(&sub).into_iter().flatten().flatten() {
            let src = entry.path();
            let name = match src.file_name() {
                Some(name) => name,
                None => {
                    log::warn!("Skipping entry without a file name: {}", src.display());
                    std::ffi::OsStr::new("")
                }
            };
            let dst = bin_path.join(name);
            if let Err(e) = std::fs::rename(&src, &dst) {
                log::warn!("Could not move {} to {}: {e}", src.display(), dst.display());
            }
        }
        if let Err(e) = std::fs::remove_dir(&sub) {
            log::warn!("Could not remove collapsed subdirectory {}: {e}", sub.display());
        }
        trace!("Collapsed single subdirectory {:?} in {:?}", sub, bin_path);
    }
    Ok(())
}
