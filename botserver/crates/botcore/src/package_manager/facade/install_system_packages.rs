use super::*;

/// #2629 — true when a relative symlink target resolves inside `base`.
/// Absolute targets and any `..` that escapes the entry's directory are
/// rejected; llama.cpp's in-tree `.so` chains (e.g. `libggml.so.0`,
/// `../lib/libggml.so`) remain allowed when they stay within the base.
fn symlink_target_is_contained(base: &std::path::Path, entry_dir: &std::path::Path, link: &str) -> bool {
    use std::path::Component;
    let link = std::path::Path::new(link);
    if link.is_absolute() {
        return false;
    }
    let mut parts: Vec<std::ffi::OsString> = Vec::new();
    if let Ok(rel) = entry_dir.strip_prefix(base) {
        for c in rel.components() {
            if let Component::Normal(s) = c {
                parts.push(s.to_os_string());
            }
        }
    }
    for c in link.components() {
        match c {
            Component::Normal(s) => parts.push(s.to_os_string()),
            Component::ParentDir => {
                if parts.pop().is_none() {
                    return false;
                }
            }
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => return false,
        }
    }
    true
}

impl PackageManager {
    pub fn install_system_packages(&self, component: &ComponentConfig) -> Result<()> {
        let packages = match self.os_type {
            OsType::Linux => &component.linux_packages,
            OsType::MacOS => &component.macos_packages,
            OsType::Windows => &component.windows_packages,
        };
        if packages.is_empty() {
            return Ok(());
        }
        trace!(
            "Installing {} system packages for component '{}'",
            packages.len(),
            component.name
        );
        match self.os_type {
            OsType::Linux => {
                if let Some(output) = safe_apt_get(&["update"]) {
                    if !output.status.success() {
                        warn!("apt-get update had issues");
                    }
                }
                let mut args = vec!["install", "-y"];
                for pkg in packages {
                    args.push(pkg.as_str());
                }
                if let Some(output) = safe_apt_get(&args) {
                    if !output.status.success() {
                        warn!("Some packages may have failed to install");
                    }
                }
            }
            OsType::MacOS => {
                let mut args = vec!["install"];
                for pkg in packages {
                    args.push(pkg.as_str());
                }
                if let Some(output) = safe_brew(&args) {
                    if !output.status.success() {
                        warn!("Homebrew installation had warnings");
                    }
                }
            }
            OsType::Windows => {
                trace!("Windows packages managed via GB portable binaries, skipping system package manager");
            }
        }
        Ok(())
    }
    pub async fn download_and_install(
        &self,
        url: &str,
        component: &str,
        binary_name: Option<&str>,
    ) -> Result<()> {
        let bin_path = self.base_path.join("bin").join(component);
        std::fs::create_dir_all(&bin_path)?;

        let cache_base = self.base_path.parent().unwrap_or(&self.base_path);
        let cache = match DownloadCache::new(cache_base) {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to initialize download cache: {}", e);
                match DownloadCache::new(&self.base_path) {
                    Ok(c) => c,
                    Err(e) => {
                        log::error!("Failed to create fallback cache: {}", e);
                        return Err(anyhow::anyhow!("Failed to create download cache"));
                    }
                }
            }
        };

        let cache_result = cache.resolve_component_url(component, url);
        // A file already present in the cache (bundled installers or a
        // previous download) must never be moved or deleted: the directory
        // may be read-only (Program Files) and removing it breaks repair
        // installs.  Only a freshly-downloaded file may be cleaned up.
        let preserve_source = cache_result.is_cached();

        let source_file = match cache_result {
            CacheResult::Cached(cached_path) => {
                info!(
                    "Using cached file for {}: {}",
                    component,
                    cached_path.display()
                );
                cached_path
            }
            CacheResult::Download {
                url: download_url,
                cache_path,
            } => {
                if cfg!(feature = "desktop-local") && !desktop_download_allowed() {
                    return Err(anyhow::anyhow!(
                        "Offline mode: component '{}' not found in bundled installers. \
                         Expected file at {}. Desktop-local builds must not download at runtime.",
                        component,
                        cache_path.display()
                    ));
                }
                info!("Downloading {} from {}", component, download_url);
                println!("Downloading {}", download_url);

                self.download_with_reqwest(&download_url, &cache_path, component)
                    .await?;

                info!("Cached {} to {}", component, cache_path.display());
                cache_path
            }
        };

        // The component id and the archive differ: "llm" downloads a
        // platform-specific build (Vulkan where available, CPU zip otherwise),
        // so the checksum must follow the URL in use, not the component id.
        // Using the id broke Vulkan hosts with "Checksum verification failed for 'llm'".
        let pin = cache.checksum_for_url(url).or_else(|| {
            cache
                .get_component(component)
                .map(|info| info.effective_sha256().to_string())
        });
        let expected = botcorepkg::checksum::pin_or_empty(pin);
        // #2001 — hash the file actually unpacked (bundled installers use stable
        // names such as `otterio`) and apply the shared policy: mismatches are fatal,
        // a missing pin warns or aborts under GB_REQUIRE_CHECKSUMS.
        botcorepkg::checksum::verify_download(&source_file, &expected, component)?;

        self.handle_downloaded_file(&source_file, &bin_path, binary_name, preserve_source)?;
        Ok(())
    }
    pub async fn download_with_reqwest(
        &self,
        url: &str,
        target_file: &std::path::Path,
        component: &str,
    ) -> Result<()> {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(2);

        if let Some(parent) = target_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let client = Client::builder()
            .connect_timeout(std::time::Duration::from_secs(15))
            .read_timeout(std::time::Duration::from_secs(120))
            .user_agent("botserver-package-manager/1.0")
            .build()?;
        let mut last_error = None;
        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                trace!(
                    "Retry attempt {}/{} for {}",
                    attempt,
                    MAX_RETRIES,
                    component
                );
                std::thread::sleep(RETRY_DELAY * attempt);
            }
            match self
                .attempt_reqwest_download(&client, url, target_file)
                .await
            {
                Ok(_size) => {
                    if attempt > 0 {
                        trace!("Download succeeded on retry attempt {}", attempt);
                    }
                    return Ok(());
                }
                Err(e) => {
                    warn!("Download attempt {} failed: {}", attempt + 1, e);
                    last_error = Some(e);
                    if let Err(e) = std::fs::remove_file(target_file) {
                        warn!("Could not remove partial download {}: {e}", target_file.display());
                    }
                }
            }
        }
        Err(anyhow::anyhow!(
            "Failed to download {} after {} attempts. Last error: {}",
            component,
            MAX_RETRIES + 1,
            last_error.unwrap_or_else(|| anyhow::anyhow!("unknown error"))
        ))
    }
    pub async fn attempt_reqwest_download(
        &self,
        _client: &Client,
        url: &str,
        temp_file: &std::path::Path,
    ) -> Result<u64> {
        let output_path = temp_file.to_str().context("Invalid temp file path")?;
        utils::download_file(url, output_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to download file using shared utility: {}", e))?;
        let metadata = std::fs::metadata(temp_file).context("Failed to get file metadata")?;
        let size = metadata.len();
        Ok(size)
    }
    /// Installs a downloaded or bundled file (binary or archive) into the
    /// component's `bin_path`.
    ///
    /// When `preserve_source` is true the source file is never moved or
    /// deleted, so a read-only bundled-installers directory stays intact.
    pub fn handle_downloaded_file(
        &self,
        temp_file: &std::path::Path,
        bin_path: &std::path::Path,
        binary_name: Option<&str>,
        preserve_source: bool,
    ) -> Result<()> {
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
                self.extract_tar_gz(temp_file, bin_path, preserve_source)?;
            }
            "zip" => {
                self.extract_zip(temp_file, bin_path, preserve_source)?;
            }
            _ => {
                if let Some(name) = binary_name {
                    self.install_binary(temp_file, bin_path, name, preserve_source)?;
                } else {
                let final_path = bin_path.join(temp_file.file_name().unwrap_or_default());
                if preserve_source || temp_file.to_string_lossy().contains("botserver-installers") {
                        std::fs::copy(temp_file, &final_path)?;
                    } else {
                        std::fs::rename(temp_file, &final_path)?;
                    }
                    self.make_executable(&final_path)?;
                }
            }
        }
        Ok(())
    }
    pub fn extract_tar_gz(
        &self,
        temp_file: &std::path::Path,
        bin_path: &std::path::Path,
        preserve_source: bool,
    ) -> Result<()> {
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
    pub fn extract_zip(
        &self,
        temp_file: &std::path::Path,
        bin_path: &std::path::Path,
        preserve_source: bool,
    ) -> Result<()> {
        trace!("extract_zip: opening {}", temp_file.display());
        let file = std::fs::File::open(temp_file)
            .with_context(|| format!("Failed to open zip: {}", temp_file.display()))?;
        trace!("extract_zip: file opened, size={}", file.metadata().map(|m| m.len()).unwrap_or(0));
        let mut archive = zip::ZipArchive::new(file)
            .with_context(|| format!("Failed to read zip: {}", temp_file.display()))?;
        trace!("extract_zip: archive read, {} entries", archive.len());
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)
                .with_context(|| format!("Failed to read entry {i} from zip"))?;
            // #2629 — enclosed_name() rejects absolute paths and `..`
            // traversal. Defence in depth even though the archive is
            // SHA-256 verified before extraction.
            let Some(enclosed) = entry.enclosed_name() else {
                warn!("skipping zip entry with unsafe path at index {i}");
                continue;
            };
            let name = match entry.name() {
                Ok(n) => n.to_string(),
                Err(_) => String::new(),
            };
            // Skip unnecessary directories to speed up extraction
            // (e.g. pgAdmin 4 in EDB PostgreSQL distribution has 22k+ files)
            if name.contains("pgAdmin 4") || name.contains("StackBuilder") {
                continue;
            }
            let target = bin_path.join(&enclosed);
            if !target.starts_with(bin_path) {
                warn!("skipping zip entry escaping destination: {name}");
                continue;
            }
            trace!("extracting[{}]: {}", i, name);
            // Unix symlinks are stored as regular entries whose content is the
            // link target. Writing them as files leaves a 20-byte "library"
            // that the loader rejects with "file too short"; llama.cpp ships its
            // whole .so chain that way.
            let is_symlink = entry
                .unix_mode()
                .map(|mode| mode & 0o170000 == 0o120000)
                .unwrap_or(false);
            if entry.is_dir() {
                std::fs::create_dir_all(&target)
                    .with_context(|| format!("Failed to create dir: {}", target.display()))?;
            } else if is_symlink {
                #[cfg(unix)]
                {
                    let mut link_target = String::new();
                    if let Err(e) = std::io::Read::read_to_string(&mut entry, &mut link_target) {
                        warn!("could not read symlink entry {}: {e}", name);
                    }
                    // #2629 — a symlink is only written when its target also
                    // resolves inside the destination directory.
                    let link = link_target.trim();
                    let entry_dir = target.parent().unwrap_or(bin_path);
                    if !symlink_target_is_contained(bin_path, entry_dir, link) {
                        warn!("skipping symlink escaping destination: {name} -> {link}");
                    } else {
                        if let Err(e) = std::fs::remove_file(&target) {
                            warn!("Could not replace symlink target {}: {e}", target.display());
                        }
                        if let Err(e) = std::os::unix::fs::symlink(link, &target) {
                            warn!(
                                "could not create symlink {} -> {}: {e}",
                                target.display(),
                                link
                            );
                        }
                    }
                }
                #[cfg(not(unix))]
                {
                    let mut outfile = std::fs::File::create(&target)
                        .with_context(|| format!("Failed to create file: {}", target.display()))?;
                    std::io::copy(&mut entry, &mut outfile)
                        .with_context(|| format!("Failed to write: {}", target.display()))?;
                }
            } else {
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create parent dir for: {}", target.display()))?;
                }
                let mut outfile = std::fs::File::create(&target)
                    .with_context(|| format!("Failed to create file: {}", target.display()))?;
                std::io::copy(&mut entry, &mut outfile)
                    .with_context(|| format!("Failed to write: {}", target.display()))?;
            }
            #[cfg(unix)]
            {
                if let Some(mode) = entry.unix_mode() {
                    if mode & 0o111 != 0 {
                        if let Err(e) = botlib::os::fs::get_permissions_manager()
                            .set_executable(&target)
                        {
                            warn!("Could not make {} executable: {e}", target.display());
                        }
                    }
                }
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
                                if let Err(e) = botlib::os::fs::get_permissions_manager().set_executable(&path) {
                                    warn!("Could not make {} executable: {e}", path.display());
                                } else {
                                    trace!("Made executable: {}", path.display());
                                }
                            }
                        }
                    }
                }
            }
        }

        collapse_single_subdirectory(bin_path)?;

        if !preserve_source && !temp_file.to_string_lossy().contains("botserver-installers") {
            if let Err(e) = std::fs::remove_file(temp_file) {
                warn!("Could not remove archive {}: {e}", temp_file.display());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod containment_tests;
