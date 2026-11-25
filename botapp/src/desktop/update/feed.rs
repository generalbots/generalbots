//! Which releases the update check reads.
//!
//! The default build follows the project's `releases/latest`. An edition
//! publishes its own tag line in the same repository (for example
//! `gbgov-v1.2.0`), so its build lists the releases, keeps only the tags with
//! the edition's prefix and offers the newest one. Edition releases are
//! published as pre-releases, so the prefixed channel accepts them.
//!
//! The prefix is fixed at build time: `GB_RELEASE_TAG_PREFIX` when it is set,
//! otherwise the edition's default.

use super::{parse_version, GitHubRelease, ParsedVersion};

const REPOSITORY_API: &str = "https://api.github.com/repos/generalbots/generalbots";

const EDITION_TAG_PREFIX: Option<&str> = if cfg!(feature = "gbgov") {
    Some("gbgov-v")
} else {
    None
};

/// Tag prefix of this build's release channel, if it has one.
pub(super) fn tag_prefix() -> Option<&'static str> {
    match option_env!("GB_RELEASE_TAG_PREFIX").map(str::trim) {
        Some(prefix) if !prefix.is_empty() => Some(prefix),
        _ => EDITION_TAG_PREFIX,
    }
}

/// GitHub endpoint for this build's channel.
pub(super) fn releases_endpoint(prefix: Option<&str>) -> String {
    match prefix {
        Some(_) => format!("{REPOSITORY_API}/releases?per_page=100"),
        None => format!("{REPOSITORY_API}/releases/latest"),
    }
}

/// Version carried by `tag` when it belongs to the channel with `prefix`.
fn channel_version(tag: &str, prefix: &str) -> Option<ParsedVersion> {
    let version = tag.trim().strip_prefix(prefix)?;
    if !version.starts_with(|character: char| character.is_ascii_digit()) {
        return None;
    }
    parse_version(version)
}

/// Version text shown to the person for a release tag.
pub(super) fn tag_version(tag: &str, prefix: Option<&str>) -> String {
    let tag = tag.trim();
    let tag = prefix
        .and_then(|prefix| tag.strip_prefix(prefix))
        .unwrap_or(tag);
    tag.trim_start_matches(['v', 'V']).to_string()
}

/// Newest published release of the channel, ignoring drafts and other tags.
pub(super) fn newest_in_channel(
    releases: Vec<GitHubRelease>,
    prefix: &str,
) -> Option<GitHubRelease> {
    releases
        .into_iter()
        .filter(|release| !release.draft)
        .filter_map(|release| channel_version(&release.tag_name, prefix).map(|v| (v, release)))
        .max_by(|(left, _), (right, _)| left.cmp(right))
        .map(|(_, release)| release)
}

#[cfg(test)]
mod tests {
    use super::{channel_version, newest_in_channel, releases_endpoint, tag_version};
    use crate::desktop::update::{update_available, GitHubRelease};

    fn release(tag: &str, draft: bool, prerelease: bool) -> GitHubRelease {
        GitHubRelease {
            tag_name: tag.to_string(),
            name: None,
            html_url: String::new(),
            body: None,
            published_at: None,
            prerelease,
            draft,
            assets: Vec::new(),
        }
    }

    #[test]
    fn only_tags_with_the_channel_prefix_are_read() {
        assert!(channel_version("gbgov-v1.2.0", "gbgov-v").is_some());
        assert!(channel_version("v7.0.40", "gbgov-v").is_none());
        assert!(channel_version("gbgov-vnext", "gbgov-v").is_none());
        assert!(channel_version("other-gbgov-v1.0.0", "gbgov-v").is_none());
    }

    #[test]
    fn the_newest_channel_release_wins_over_newer_platform_tags() {
        let releases = vec![
            release("v7.0.40", false, false),
            release("gbgov-v1.0.0", false, true),
            release("gbgov-v1.10.0", false, true),
            release("gbgov-v1.9.3", false, true),
            release("gbgov-v2.0.0", true, true),
        ];
        let newest = newest_in_channel(releases, "gbgov-v").map(|release| release.tag_name);
        assert_eq!(newest.as_deref(), Some("gbgov-v1.10.0"));
    }

    #[test]
    fn a_channel_without_releases_offers_nothing() {
        let releases = vec![release("v7.0.40", false, false), release("gbgov-v9.0.0", true, true)];
        assert!(newest_in_channel(releases, "gbgov-v").is_none());
    }

    #[test]
    fn channel_versions_compare_without_the_prefix() {
        let latest = tag_version("gbgov-v1.2.0", Some("gbgov-v"));
        assert_eq!(latest, "1.2.0");
        assert!(update_available("1.1.9", &latest));
        assert!(!update_available("1.2.0", &latest));
        assert!(update_available("1.2.0-1", &latest));
        assert_eq!(tag_version("v7.0.40", None), "7.0.40");
    }

    #[test]
    fn the_endpoint_follows_the_channel() {
        assert!(releases_endpoint(Some("gbgov-v")).ends_with("/releases?per_page=100"));
        assert!(releases_endpoint(None).ends_with("/releases/latest"));
    }
}
