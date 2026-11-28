//! Build metadata for the desktop shell (issue #2103).
//!
//! Mirrors `botcore::product::build_info`: the release workflow exports
//! `BOTSERVER_BUILD_VERSION` / `BOTSERVER_COMMIT` / `BOTSERVER_BUILD_DATE`
//! before every build, so the shell, the sidecar and `/api/product` all report
//! the same release tag instead of the workspace `Cargo.toml` version.

/// Product version of this build (the release tag, or the manifest version).
pub fn version() -> String {
    let raw = option_env!("BOTSERVER_BUILD_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
    raw.trim().trim_start_matches('v').to_string()
}

/// Short commit hash the binary was built from.
pub fn commit() -> String {
    text(option_env!("BOTSERVER_COMMIT"))
}

/// UTC date of the build.
pub fn build_date() -> String {
    text(option_env!("BOTSERVER_BUILD_DATE"))
}

fn text(value: Option<&'static str>) -> String {
    match value.map(str::trim) {
        Some(value) if !value.is_empty() => value.to_string(),
        _ => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_follows_the_injected_build_version() {
        let expected = match option_env!("BOTSERVER_BUILD_VERSION") {
            Some(injected) => injected.trim().trim_start_matches('v').to_string(),
            None => env!("CARGO_PKG_VERSION").to_string(),
        };
        assert_eq!(version(), expected);
    }

    #[test]
    fn commit_and_date_are_never_empty() {
        assert!(!commit().is_empty());
        assert!(!build_date().is_empty());
    }

    /// The build script stamps a date on every build, with or without the
    /// release workflow's variable, so About never shows it as unknown.
    #[test]
    fn the_build_date_is_stamped_on_every_build() {
        let date = build_date();
        let parts: Vec<&str> = date.split(['-', 'T']).collect();
        assert!(parts.len() >= 3, "{date}");
        assert_eq!(parts[0].len(), 4, "{date}");
        assert!(parts[..3].iter().all(|part| part.parse::<u32>().is_ok()), "{date}");
    }
}
