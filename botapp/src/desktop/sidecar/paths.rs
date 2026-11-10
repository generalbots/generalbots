//! Sidecar binary and stack-directory resolution.
//!
//! Kept pure so the precedence rules are testable without touching the process
//! environment: an explicit override wins, then a binary shipped beside the app,
//! then the payload extracted into the per-user stack.

use std::path::{Path, PathBuf};

/// The sidecar executable name on this platform.
pub(super) const BINARY_NAME: &str = if cfg!(target_os = "windows") {
    "botserver.exe"
} else {
    "botserver"
};

/// Resolves the stack directory.
///
/// `BOTSERVER_STACK_PATH` wins; otherwise the per-user local data directory
/// holds `generalbots/stack`.
pub(super) fn resolve_stack(explicit: Option<&str>, data_local: Option<PathBuf>) -> PathBuf {
    if let Some(value) = explicit {
        if !value.trim().is_empty() {
            return PathBuf::from(value);
        }
    }
    data_local
        .unwrap_or_else(|| PathBuf::from("."))
        .join("generalbots")
        .join("stack")
}

/// Resolves the sidecar executable.
///
/// `BOTSERVER_BIN` wins, then the binary beside the running app (packaged
/// installs), then the copy the self-extractor wrote into the stack (#1633).
pub(super) fn resolve_binary(
    explicit: Option<&str>,
    beside: Option<PathBuf>,
    stack: &Path,
) -> PathBuf {
    if let Some(value) = explicit {
        if !value.trim().is_empty() {
            return PathBuf::from(value);
        }
    }
    beside.unwrap_or_else(|| stack.join(BINARY_NAME))
}

#[cfg(test)]
mod tests {
    use super::{resolve_binary, resolve_stack, BINARY_NAME};
    use std::path::{Path, PathBuf};

    #[test]
    fn the_binary_name_matches_the_platform() {
        if cfg!(target_os = "windows") {
            assert_eq!(BINARY_NAME, "botserver.exe");
        } else {
            assert_eq!(BINARY_NAME, "botserver");
        }
    }

    #[test]
    fn an_explicit_stack_path_wins() {
        assert_eq!(
            resolve_stack(Some("/opt/gb/stack"), Some(PathBuf::from("/home/u/.local/share"))),
            PathBuf::from("/opt/gb/stack")
        );
        // A blank override is not a path.
        assert_eq!(
            resolve_stack(Some("  "), Some(PathBuf::from("/data"))),
            PathBuf::from("/data/generalbots/stack")
        );
    }

    #[test]
    fn the_stack_falls_back_to_the_user_data_directory() {
        assert_eq!(
            resolve_stack(None, Some(PathBuf::from("/home/u/.local/share"))),
            PathBuf::from("/home/u/.local/share/generalbots/stack")
        );
        // No resolvable data directory keeps the path relative.
        assert_eq!(resolve_stack(None, None), PathBuf::from("./generalbots/stack"));
    }

    #[test]
    fn an_explicit_binary_wins_over_the_bundled_one() {
        let stack = Path::new("/home/u/stack");
        assert_eq!(
            resolve_binary(Some("/opt/gb/bin/botserver"), Some(PathBuf::from("/app/botserver")), stack),
            PathBuf::from("/opt/gb/bin/botserver")
        );
        assert_eq!(
            resolve_binary(Some(""), Some(PathBuf::from("/app/botserver")), stack),
            PathBuf::from("/app/botserver")
        );
    }

    #[test]
    fn the_beside_binary_beats_the_extracted_payload() {
        let stack = Path::new("/home/u/stack");
        assert_eq!(
            resolve_binary(None, Some(PathBuf::from("/app/botserver")), stack),
            PathBuf::from("/app/botserver")
        );
        assert_eq!(
            resolve_binary(None, None, stack),
            stack.join(BINARY_NAME)
        );
    }
}
