//! Build metadata reported by `/api/product` (issue #2103).
//!
//! The product version used to be whatever `Cargo.toml` declared at build
//! time, so a `v7.0.1` tag still shipped a binary that answered `7.0.0` on
//! every surface. The release workflow now exports `BOTSERVER_BUILD_VERSION`
//! (the tag without its `v` prefix) together with `BOTSERVER_COMMIT` and
//! `BOTSERVER_BUILD_DATE` before `cargo build`; the workspace version stays as
//! the fallback for local builds, where no tag exists.

/// Version of the running build: the release tag when the build injected one,
/// otherwise the workspace `Cargo.toml` version.
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

/// One-line diagnostic shown next to the version when the UI has room.
pub fn label() -> String {
    format!("{} ({}, {})", version(), commit(), build_date())
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

    /// The manifest version is the documented fallback: a local build must
    /// report exactly what `Cargo.toml` declares, and a tagged CI build must
    /// report the tag instead.
    #[test]
    fn version_follows_the_injected_build_version() {
        let expected = match option_env!("BOTSERVER_BUILD_VERSION") {
            Some(injected) => injected.trim().trim_start_matches('v').to_string(),
            None => env!("CARGO_PKG_VERSION").to_string(),
        };
        assert_eq!(version(), expected);
    }

    #[test]
    fn version_is_a_numeric_major_minor_patch() {
        let value = version();
        let parts: Vec<&str> = value.split('.').collect();
        assert_eq!(parts.len(), 3, "version must be MAJOR.MINOR.PATCH");
        for part in parts {
            assert!(
                part.parse::<u64>().is_ok(),
                "version part {part:?} is not numeric"
            );
        }
    }

    #[test]
    fn commit_and_date_are_never_empty() {
        assert!(!commit().is_empty());
        assert!(!build_date().is_empty());
        assert!(!label().is_empty());
    }

    #[test]
    fn blank_values_fall_back_to_unknown() {
        assert_eq!(text(None), "unknown");
        assert_eq!(text(Some("   ")), "unknown");
        assert_eq!(text(Some(" abc123 ")), "abc123");
    }
}
