use super::*;

/// Release tags are immutable in the publishing repository, so a build cannot
/// overwrite a stable "latest" tag: every build publishes its own release.
/// A component therefore declares `.../releases/download/{tag}/<file>` and the
/// newest release that actually carries the file is resolved here, at install
/// time.
pub(crate) async fn resolve_release_tag(url: &str, asset_name: &str) -> Result<String> {
    let (owner, repo) = github_owner_repo(url)
        .with_context(|| format!("cannot resolve {{tag}} in {url}: not a GitHub release URL"))?;
    let api = format!("https://api.github.com/repos/{owner}/{repo}/releases?per_page=50");
    let client = Client::builder()
        .user_agent("botserver-package-manager/1.0")
        .timeout(std::time::Duration::from_secs(20))
        .build()?;
    let releases: Vec<serde_json::Value> = client
        .get(&api)
        .send()
        .await
        .with_context(|| format!("listing releases of {owner}/{repo}"))?
        .error_for_status()
        .with_context(|| format!("listing releases of {owner}/{repo}"))?
        .json()
        .await
        .with_context(|| format!("parsing the release list of {owner}/{repo}"))?;

    // The API returns newest first; drafts and pre-releases never carry a
    // deployable artifact.
    for release in releases {
        if release["draft"].as_bool().unwrap_or(false)
            || release["prerelease"].as_bool().unwrap_or(false)
        {
            continue;
        }
        let Some(tag) = release["tag_name"].as_str() else {
            continue;
        };
        let publishes_asset = release["assets"]
            .as_array()
            .map(|assets| {
                assets
                    .iter()
                    .any(|asset| asset["name"].as_str() == Some(asset_name))
            })
            .unwrap_or(false);
        if publishes_asset {
            info!("Resolved {{tag}} to {tag} for {asset_name}");
            return Ok(tag.to_string());
        }
    }

    Err(anyhow::anyhow!(
        "no published release of {owner}/{repo} carries {asset_name}"
    ))
}

fn github_owner_repo(url: &str) -> Option<(&str, &str)> {
    let index = url.find("github.com/")? + "github.com/".len();
    let mut parts = url[index..].split('/');
    let owner = parts.next()?;
    let repo = parts.next()?;
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner, repo))
}


pub async fn download_and_install(base_path: &std::path::Path, url: &str, component: &str, binary_name: Option<&str>) -> Result<()> {
    let bin_path = base_path.join("bin").join(component);
    std::fs::create_dir_all(&bin_path)?;
    let cache_base = base_path.parent().unwrap_or(base_path);
    let cache = match DownloadCache::new(cache_base) {
        Ok(c) => c,
        Err(e) => {
            warn!("Failed to initialize download cache: {}", e);
            match DownloadCache::new(base_path) {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to create fallback cache: {}", e);
                    return Err(anyhow::anyhow!("Failed to create download cache"));
                }
            }
        }
    };
    let cache_result = cache.resolve_component_url(component, url);
    let source_file = match cache_result {
        CacheResult::Cached(cached_path) => {
            info!("Using cached file for {}: {}", component, cached_path.display());
            cached_path
        }
        CacheResult::Download { url: download_url, cache_path } => {
            let asset_name = cache_path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default();
            let download_url = if download_url.contains("{tag}") {
                let tag = resolve_release_tag(&download_url, &asset_name).await?;
                download_url.replace("{tag}", &tag)
            } else {
                download_url
            };
            info!("Downloading {} from {}", component, download_url);
            println!("Downloading {}", download_url);
            download_with_reqwest(&download_url, &cache_path, component).await?;
            info!("Cached {} to {}", component, cache_path.display());
            cache_path
        }
    };
    handle_downloaded_file(&source_file, &bin_path, binary_name)?;
    Ok(())
}

pub async fn download_with_reqwest(url: &str, target_file: &std::path::Path, component: &str) -> Result<()> {
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(2);
    if let Some(parent) = target_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("botserver-package-manager/1.0")
        .build()?;
    let mut last_error = None;
    for attempt in 0..=MAX_RETRIES {
        if attempt > 0 {
            trace!("Retry attempt {}/{} for {}", attempt, MAX_RETRIES, component);
            std::thread::sleep(RETRY_DELAY * attempt);
        }
        match attempt_reqwest_download(&client, url, target_file).await {
            Ok(_size) => {
                if attempt > 0 {
                    trace!("Download succeeded on retry attempt {}", attempt);
                }
                return Ok(());
            }
            Err(e) => {
                warn!("Download attempt {} failed: {}", attempt + 1, e);
                last_error = Some(e);
                let _ = std::fs::remove_file(target_file);
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

pub async fn attempt_reqwest_download(_client: &Client, url: &str, temp_file: &std::path::Path) -> Result<u64> {
    let output_path = temp_file.to_str().context("Invalid temp file path")?;
    download_file(url, output_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to download file using shared utility: {}", e))?;
    let metadata = std::fs::metadata(temp_file).context("Failed to get file metadata")?;
    let size = metadata.len();
    Ok(size)
}
