use super::*;

/// Installs a downloaded or bundled file (binary or archive) into the
/// component's `bin_path`.
///
/// When `preserve_source` is true the source file is never moved or
/// deleted, so a read-only bundled-installers directory stays intact.
pub fn handle_downloaded_file(temp_file: &std::path::Path, bin_path: &std::path::Path, binary_name: Option<&str>, preserve_source: bool) -> Result<()> {
    let metadata = std::fs::metadata(temp_file)?;
    if metadata.len() == 0 {
        return Err(anyhow::anyhow!("Downloaded file is empty"));
    }
    let file_extension = temp_file
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    match file_extension {
        "gz" | "tgz" => {
            extract_tar_gz(temp_file, bin_path, preserve_source)?;
        }
        "zip" => {
            extract_zip(temp_file, bin_path, preserve_source)?;
        }
        _ => {
            if let Some(name) = binary_name {
                install_binary(temp_file, bin_path, name, preserve_source)?;
            } else {
                let final_path = bin_path.join(temp_file.file_name().unwrap_or_default());
                if preserve_source || temp_file.to_string_lossy().contains("botserver-installers") {
                    std::fs::copy(temp_file, &final_path)?;
                } else {
                    std::fs::rename(temp_file, &final_path)?;
                }
                make_executable(&final_path)?;
            }
        }
    }
    Ok(())
}

pub fn extract_tar_gz(temp_file: &std::path::Path, bin_path: &std::path::Path, preserve_source: bool) -> Result<()> {
    let tar_gz = std::fs::File::open(temp_file)?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(bin_path)?;

    collapse_single_subdirectory(bin_path)?;

    if !preserve_source {
        std::fs::remove_file(temp_file)?;
    }
    Ok(())
}

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
            let name = src.file_name().unwrap_or_else(|| std::ffi::OsStr::new(""));
            let dst = bin_path.join(name);
            if let Err(e) = std::fs::rename(&src, &dst) {
                log::warn!("Failed to move {} to {}: {e}", src.display(), dst.display());
            }
        }
        if let Err(e) = std::fs::remove_dir(&sub) {
            log::warn!("Failed to remove collapsed subdirectory {}: {e}", sub.display());
        }
        trace!("Collapsed single subdirectory {:?} in {:?}", sub, bin_path);
    }
    Ok(())
}

pub fn extract_zip(temp_file: &std::path::Path, bin_path: &std::path::Path, preserve_source: bool) -> Result<()> {
    let file = std::fs::File::open(temp_file)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| anyhow::anyhow!("Failed to open zip archive: {}", e))?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| anyhow::anyhow!("Failed to read zip entry {}: {}", i, e))?;
        let Some(path) = entry.enclosed_name() else { continue };
        let target = bin_path.join(path);
        if entry.is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut output = std::fs::File::create(&target)?;
            std::io::copy(&mut entry, &mut output)?;
        }
    }
    #[cfg(unix)]
    {
        if let Ok(entries) = std::fs::read_dir(bin_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(_metadata) = std::fs::metadata(&path) {
                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                        if ext.is_empty() || ext == "sh" || ext == "bash" {
                            if let Err(e) =
                                botlib::os::fs::get_permissions_manager().set_executable(&path)
                            {
                                log::warn!("Could not make {} executable: {e}", path.display());
                            }
                            trace!("Made executable: {}", path.display());
                        }
                    }
                }
            }
        }
    }

    collapse_single_subdirectory(bin_path)?;

    if !preserve_source {
        std::fs::remove_file(temp_file)?;
    }
    Ok(())
}
