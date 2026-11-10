//! Startup update check against the latest GitHub release (#1862).
//!
//! The desktop shell asks GitHub for `releases/latest`, compares the release
//! tag with the running application version and, when a newer version exists,
//! offers an opt-in prompt that links to the release page. Nothing is
//! downloaded or installed automatically: the UI only notifies the user and
//! opens the official release page (or the platform asset when one is
//! published).
//!
//! The check is desktop-only by construction — this module is compiled into
//! `botapp` and is never used by `botui` in a browser or server deployment.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

const RELEASES_API: &str = "https://api.github.com/repos/generalbots/generalbots/releases/latest";
const RELEASES_PAGE: &str = "https://github.com/generalbots/generalbots/releases";
const RELEASE_URL_PREFIX: &str = "https://github.com/generalbots/generalbots/releases";

/// User agent for the release check, carrying the real build version (#2103).
fn user_agent() -> String {
    format!("GeneralBots-Desktop/{}", crate::build_info::version())
}

/// GitHub's release payload (only the fields the desktop needs).
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

/// Result returned to the suite UI.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReleaseInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_name: String,
    pub release_url: String,
    pub release_notes: String,
    pub published_at: Option<String>,
    pub checked_at: String,
    pub platform: String,
    pub download_url: Option<String>,
    pub download_name: Option<String>,
    pub download_size: Option<u64>,
}

/// Compares the running application version with the release tag.
///
/// A non-semver tag never enables an update: the desktop must not offer a
/// downgrade or an update it cannot reason about.
#[must_use]
pub fn update_available(current: &str, latest: &str) -> bool {
    match (parse_version(current), parse_version(latest)) {
        (Some(current), Some(latest)) => latest > current,
        _ => {
            log::warn!("could not compare desktop versions: current={current:?} latest={latest:?}");
            false
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedVersion {
    major: u64,
    minor: u64,
    patch: u64,
    prerelease: Option<String>,
}

impl Ord for ParsedVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.cmp(&other.patch))
            .then_with(|| match (&self.prerelease, &other.prerelease) {
                // A stable release is newer than any prerelease of the same
                // numeric version.
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (Some(left), Some(right)) => left.cmp(right),
            })
    }
}

impl PartialOrd for ParsedVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn parse_version(raw: &str) -> Option<ParsedVersion> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Tags may be `v6.7.0`, `generalbots-v6.7.0` or just `6.7.0`. Start at the
    // first digit so an optional product prefix does not hide the version.
    let start = trimmed.find(|character: char| character.is_ascii_digit())?;
    let release = trimmed.get(start..)?;
    let (core, prerelease) = match release.split_once('-') {
        Some((core, prerelease)) => (core, Some(prerelease.to_string())),
        None => (release, None),
    };
    let core = core.split('+').next().unwrap_or(core);
    // Drop build metadata from a prerelease such as `beta.1+build.5`.
    let prerelease = prerelease.map(|value| value.split('+').next().unwrap_or(&value).to_string());

    let mut parts = core.split('.');
    let major = parse_component(parts.next()?)?;
    let minor = parse_component(parts.next()?)?;
    let patch = match parts.next() {
        Some(patch) => parse_component(patch)?,
        None => 0,
    };
    if parts.next().is_some() {
        return None;
    }
    Some(ParsedVersion {
        major,
        minor,
        patch,
        prerelease,
    })
}

fn parse_component(component: &str) -> Option<u64> {
    let digits = component
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    digits.parse().ok()
}

fn platform_id() -> String {
    let os = match std::env::consts::OS {
        "macos" => "macos",
        "windows" => "windows",
        other => other,
    };
    format!("{os}-{}", std::env::consts::ARCH)
}

fn select_asset<'a>(assets: &'a [GitHubAsset], platform: &str) -> Option<&'a GitHubAsset> {
    let arch = platform.split_once('-').map_or("", |(_, arch)| arch);
    let mut candidates: Vec<&GitHubAsset> = assets
        .iter()
        .filter(|asset| asset.name.contains(arch))
        .filter(|asset| !asset.name.to_lowercase().contains("botserver"))
        .collect();
    if candidates.is_empty() {
        candidates = assets
            .iter()
            .filter(|asset| !asset.name.to_lowercase().contains("botserver"))
            .collect();
    }

    let preferred: &[&str] = if platform.starts_with("windows") {
        &["setup.exe", ".msi", "generalbots.exe"]
    } else if platform.starts_with("macos") {
        &[".dmg"]
    } else {
        // Prefer the tarball/single-file desktop package over the AppImage
        // while the release still publishes the AppImage best-effort.
        &["generalbots-linux", ".tar.gz", ".appimage", "generalbots"]
    };

    preferred.iter().find_map(|needle| {
        candidates
            .iter()
            .find(|asset| asset.name.to_lowercase().contains(needle))
            .copied()
    })
}

fn release_url_or_page(candidate: &str) -> String {
    let candidate = candidate.trim();
    if candidate.starts_with(RELEASE_URL_PREFIX)
        && !candidate
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        candidate.to_string()
    } else {
        RELEASES_PAGE.to_string()
    }
}

fn release_from_github(release: GitHubRelease) -> Result<ReleaseInfo, String> {
    if release.draft {
        return Err("GitHub returned a draft release.".to_string());
    }
    if release.prerelease {
        return Err("GitHub returned a prerelease for the stable channel.".to_string());
    }
    if release.tag_name.trim().is_empty() {
        return Err("The latest GitHub release has no tag.".to_string());
    }

    let current_version = crate::build_info::version();
    let latest_version = release
        .tag_name
        .trim()
        .trim_start_matches(|character| character == 'v' || character == 'V')
        .to_string();
    let platform = platform_id();
    let asset = select_asset(&release.assets, &platform);
    let release_name = release
        .name
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| format!("v{latest_version}"));
    let release_notes = release.body.unwrap_or_default();
    let release_url = release_url_or_page(&release.html_url);

    Ok(ReleaseInfo {
        update_available: update_available(&current_version, &latest_version),
        current_version,
        latest_version,
        release_name,
        release_url,
        release_notes,
        published_at: release.published_at,
        checked_at: chrono::Utc::now().to_rfc3339(),
        platform,
        download_url: asset.map(|asset| asset.browser_download_url.clone()),
        download_name: asset.map(|asset| asset.name.clone()),
        download_size: asset.map(|asset| asset.size),
    })
}

fn parse_github_release(raw: &str) -> Result<GitHubRelease, String> {
    serde_json::from_str(raw)
        .map_err(|error| format!("could not parse the GitHub release: {error}"))
}

/// Fetches `releases/latest` and reports whether the desktop should offer an
/// update. Network and HTTP errors are returned to the caller; the UI treats
/// them as a silent failure so the desktop still starts offline.
pub async fn check_latest_release() -> Result<ReleaseInfo, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|error| format!("could not create the update HTTP client: {error}"))?;

    let mut request = client
        .get(RELEASES_API)
        .header(reqwest::header::USER_AGENT, user_agent())
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28");

    // Optional, never required: deployments that hit the anonymous rate limit
    // can provide a token through the environment.
    if let Ok(token) = std::env::var("GB_GITHUB_TOKEN") {
        if !token.trim().is_empty() {
            request = request.bearer_auth(token.trim());
        }
    }

    let response = request
        .send()
        .await
        .map_err(|error| format!("could not reach GitHub releases: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub releases returned HTTP {}.",
            response.status().as_u16()
        ));
    }

    let body = response
        .text()
        .await
        .map_err(|error| format!("could not read the GitHub release: {error}"))?;
    let release = parse_github_release(&body)?;
    let info = release_from_github(release)?;
    log::info!(
        "update check: current={} latest={} available={}",
        info.current_version,
        info.latest_version,
        info.update_available
    );
    Ok(info)
}

/// Tauri command: check GitHub releases now (the "Check now" action and the
/// opt-in startup check both use it).
#[tauri::command]
pub async fn check_release_update() -> Result<ReleaseInfo, String> {
    check_latest_release().await
}

/// Tauri command: opens the official release page in the system browser.
/// Only URLs under the project's `/releases` path are accepted so a malformed
/// release payload can never make the shell open an arbitrary address.
#[tauri::command]
pub fn open_release_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;

    let url = release_url_or_page(&url);
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| format!("could not open the release page: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{parse_version, release_url_or_page, select_asset, update_available, GitHubAsset};

    fn asset(name: &str) -> GitHubAsset {
        GitHubAsset {
            name: name.to_string(),
            browser_download_url: format!(
                "https://github.com/generalbots/generalbots/releases/download/v1/{name}"
            ),
            size: 1,
        }
    }

    #[test]
    fn newer_versions_are_detected() {
        assert!(update_available("6.7.0", "6.7.1"));
        assert!(update_available("6.7.0", "6.8.0"));
        assert!(update_available("6.7.0", "7.0.0"));
        assert!(update_available("v6.7.0", "v6.7.1"));
    }

    #[test]
    fn equal_and_older_versions_are_not_offered() {
        assert!(!update_available("6.7.0", "6.7.0"));
        assert!(!update_available("6.7.1", "6.7.0"));
        assert!(!update_available("generalbots-v6.7.1", "v6.7.0"));
    }

    #[test]
    fn prereleases_rank_below_the_stable_release() {
        assert!(update_available("6.7.0-beta.1", "6.7.0"));
        assert!(!update_available("6.7.0", "6.7.0-beta.1"));
        assert!(parse_version("v6.7.0-beta.1").is_some());
    }

    #[test]
    fn unparseable_versions_never_offer_an_update() {
        assert!(!update_available("dev", "6.7.0"));
        assert!(!update_available("6.7.0", "latest"));
        assert!(!update_available("", "6.7.0"));
    }

    #[test]
    fn release_urls_are_restricted_to_the_project_releases() {
        assert_eq!(
            release_url_or_page("https://github.com/generalbots/generalbots/releases/tag/v6.8.0"),
            "https://github.com/generalbots/generalbots/releases/tag/v6.8.0"
        );
        assert_eq!(
            release_url_or_page("https://example.com/evil"),
            "https://github.com/generalbots/generalbots/releases"
        );
    }

    #[test]
    fn platform_assets_are_preferred() {
        let assets = vec![
            asset("botserver-linux-x86_64"),
            asset("generalbots-linux-x86_64.tar.gz"),
            asset("generalbots-linux-x86_64.AppImage"),
        ];
        let selected = select_asset(&assets, "linux-x86_64").map(|asset| asset.name.as_str());
        assert_eq!(selected, Some("generalbots-linux-x86_64.tar.gz"));

        let windows = vec![
            asset("generalbots.exe"),
            asset("generalbots-x64-setup.exe"),
            asset("generalbots-x64.msi"),
        ];
        let selected = select_asset(&windows, "windows-x86_64").map(|asset| asset.name.as_str());
        assert_eq!(selected, Some("generalbots-x64-setup.exe"));
    }
}
