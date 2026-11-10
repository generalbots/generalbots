pub(crate) use std::path::{Component, Path, PathBuf};
pub(crate) use tracing::warn;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathGuardError {
    PathTraversal,
    AbsolutePath,
    InvalidComponent,
    EmptyPath,
    OutsideAllowedRoot,
    SymlinkNotAllowed,
    HiddenFileNotAllowed,
    InvalidExtension,
    PathTooLong,
    NullByte,
}

#[derive(Debug, Clone)]
pub struct PathGuardConfig {
    pub allowed_roots: Vec<PathBuf>,
    pub allow_symlinks: bool,
    pub allow_hidden_files: bool,
    pub allowed_extensions: Option<Vec<String>>,
    pub denied_extensions: Vec<String>,
    pub max_path_length: usize,
    pub max_depth: usize,
}

pub struct PathGuard {
    config: PathGuardConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_path() {
        let guard = PathGuard::default();
        assert!(guard.validate(Path::new("foo/bar/file.txt")).is_ok());
    }

    #[test]
    fn test_path_traversal_simple() {
        let guard = PathGuard::default();
        assert_eq!(
            guard.validate(Path::new("../secret")).unwrap_err(),
            PathGuardError::PathTraversal
        );
    }

    #[test]
    fn test_path_traversal_embedded() {
        let guard = PathGuard::default();
        assert_eq!(
            guard.validate(Path::new("foo/../../secret")).unwrap_err(),
            PathGuardError::PathTraversal
        );
    }

    #[test]
    fn test_valid_parent_traversal() {
        let guard = PathGuard::default();
        let result = guard.validate(Path::new("foo/bar/../baz/file.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("foo/baz/file.txt"));
    }

    #[test]
    fn test_hidden_file_blocked() {
        let guard = PathGuard::default();
        assert_eq!(
            guard.validate(Path::new("foo/.secret")).unwrap_err(),
            PathGuardError::HiddenFileNotAllowed
        );
    }

    #[test]
    fn test_hidden_file_allowed() {
        let guard = PathGuard::new(PathGuardConfig::default().allow_hidden(true));
        assert!(guard.validate(Path::new("foo/.gitignore")).is_ok());
    }

    #[test]
    fn test_denied_extension() {
        let guard = PathGuard::default();
        assert_eq!(
            guard.validate(Path::new("script.exe")).unwrap_err(),
            PathGuardError::InvalidExtension
        );
    }

    #[test]
    fn test_allowed_extension() {
        let guard = PathGuard::new(
            PathGuardConfig::default().with_allowed_extensions(vec!["txt".to_string()])
        );
        assert!(guard.validate(Path::new("file.txt")).is_ok());
        assert_eq!(
            guard.validate(Path::new("file.pdf")).unwrap_err(),
            PathGuardError::InvalidExtension
        );
    }

    #[test]
    fn test_empty_path() {
        let guard = PathGuard::default();
        assert_eq!(
            guard.validate(Path::new("")).unwrap_err(),
            PathGuardError::EmptyPath
        );
    }

    #[test]
    fn test_max_depth() {
        let guard = PathGuard::new(PathGuardConfig::default().with_max_depth(3));
        assert!(guard.validate(Path::new("a/b/c")).is_ok());
        assert_eq!(
            guard.validate(Path::new("a/b/c/d")).unwrap_err(),
            PathGuardError::PathTooLong
        );
    }

    #[test]
    fn test_max_length() {
        let guard = PathGuard::new(PathGuardConfig::default().with_max_length(10));
        assert!(guard.validate(Path::new("short.txt")).is_ok());
        assert_eq!(
            guard.validate(Path::new("very_long_filename.txt")).unwrap_err(),
            PathGuardError::PathTooLong
        );
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("normal.txt"), "normal.txt");
        assert_eq!(sanitize_filename("file/with\\slashes"), "file_with_slashes");
        assert_eq!(sanitize_filename("file:name"), "file_name");
        assert_eq!(sanitize_filename("..."), "unnamed");
        assert_eq!(sanitize_filename("   "), "unnamed");
        assert_eq!(sanitize_filename("CON"), "_CON");
        assert_eq!(sanitize_filename("CON.txt"), "_CON.txt");
    }

    #[test]
    fn test_sanitize_filename_long() {
        let long_name = "a".repeat(300);
        let sanitized = sanitize_filename(&long_name);
        assert_eq!(sanitized.len(), 255);
    }

    #[test]
    fn test_dangerous_patterns() {
        assert!(has_dangerous_patterns(".."));
        assert!(has_dangerous_patterns("file%2f"));
        assert!(has_dangerous_patterns("file;cmd"));
        assert!(has_dangerous_patterns("file`inject`"));
        assert!(!has_dangerous_patterns("normal_file.txt"));
    }

    #[test]
    fn test_is_safe_path() {
        assert!(is_safe_path(Path::new("documents/file.txt")));
        assert!(!is_safe_path(Path::new("../secret")));
    }

    #[test]
    fn test_join_safe() {
        let base = Path::new("/data/uploads");
        assert!(join_safe(base, Path::new("user/file.txt")).is_ok());
    }

    #[test]
    fn test_config_builder() {
        let config = PathGuardConfig::new()
            .with_root("/data")
            .allow_hidden(true)
            .allow_symlinks(true)
            .with_max_depth(5)
            .with_max_length(1000);

        assert_eq!(config.allowed_roots.len(), 1);
        assert!(config.allow_hidden_files);
        assert!(config.allow_symlinks);
        assert_eq!(config.max_depth, 5);
        assert_eq!(config.max_path_length, 1000);
    }

    #[test]
    fn test_strict_config() {
        let config = PathGuardConfig::strict();
        assert!(!config.allow_symlinks);
        assert!(!config.allow_hidden_files);
        assert!(config.allowed_extensions.is_some());
    }

    #[test]
    fn test_permissive_config() {
        let config = PathGuardConfig::permissive();
        assert!(config.allow_symlinks);
        assert!(config.allow_hidden_files);
        assert!(config.denied_extensions.is_empty());
    }

    #[test]
    fn test_null_byte() {
        let guard = PathGuard::default();
        let path = Path::new("file\0.txt");
        assert_eq!(
            guard.validate(path).unwrap_err(),
            PathGuardError::NullByte
        );
    }

    #[test]
    fn test_path_guard_error_display() {
        assert_eq!(
            PathGuardError::PathTraversal.to_string(),
            "Path traversal attempt detected"
        );
        assert_eq!(
            PathGuardError::EmptyPath.to_string(),
            "Empty path is not allowed"
        );
    }

    #[test]
    fn test_current_dir_component() {
        let guard = PathGuard::default();
        let result = guard.validate(Path::new("foo/./bar/./file.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("foo/bar/file.txt"));
    }

    #[test]
    fn test_case_insensitive_extension() {
        let guard = PathGuard::default();
        assert_eq!(
            guard.validate(Path::new("script.EXE")).unwrap_err(),
            PathGuardError::InvalidExtension
        );
    }
}

mod path_guard_error;
mod permissive;
mod validate;

pub use validate::*;

#[cfg(test)]
mod adversarial_tests {
    use super::*;

    #[test]
    fn absolute_paths_outside_the_roots_are_refused() {
        let guard = PathGuard::new(PathGuardConfig::new().with_root("/data/uploads"));
        assert!(guard.validate(Path::new("/data/uploads/a.txt")).is_ok());
        assert_eq!(
            guard.validate(Path::new("/etc/passwd")).unwrap_err(),
            PathGuardError::AbsolutePath
        );
    }

    #[test]
    fn encoded_traversal_is_still_rejected() {
        let guard = PathGuard::default();
        // %2f is decoded by the caller; the guard refuses the literal form too.
        // The leading dot makes it a hidden component, which is also refused.
        assert!(guard.validate(Path::new("..%2fsecret")).is_err());
        assert!(guard.validate(Path::new("foo/%2e%2e/secret")).is_err());
        // A leading dot trips the hidden-file rule first; either way it is
        // refused, which is what matters.
        assert!(guard.validate(Path::new("foo/..%2fsecret")).is_err());
    }

    #[test]
    fn shell_metacharacters_in_a_component_are_rejected() {
        let guard = PathGuard::default();
        for hostile in [
            "file;rm -rf.txt",
            "file`id`.txt",
            "file$(whoami).txt",
            "file|cat.txt",
        ] {
            assert!(
                guard.validate(Path::new(hostile)).is_err(),
                "{hostile} must be rejected"
            );
        }
    }

    #[test]
    fn traversal_above_the_base_is_rejected_at_every_depth() {
        let guard = PathGuard::default();
        assert_eq!(
            guard.validate(Path::new("a/b/../../../etc/passwd")).unwrap_err(),
            PathGuardError::PathTraversal
        );
        assert_eq!(
            guard.validate(Path::new("./../x")).unwrap_err(),
            PathGuardError::PathTraversal
        );
    }

    #[test]
    fn resolve_rejects_a_path_that_escapes_the_base() {
        let guard = PathGuard::new(PathGuardConfig::new().with_root("/data/uploads"));
        assert!(guard
            .validate_and_resolve(Path::new("/data/uploads"), Path::new("a/b.txt"))
            .is_ok());
        assert!(guard
            .validate_and_resolve(Path::new("/etc"), Path::new("passwd"))
            .is_err());
    }

    #[test]
    fn join_safe_refuses_an_escaping_relative_path() {
        let base = Path::new("/data/uploads");
        assert!(join_safe(base, Path::new("../etc/passwd")).is_err());
    }

    #[test]
    fn extension_checks_are_case_insensitive() {
        let guard = PathGuard::new(
            PathGuardConfig::default().with_allowed_extensions(vec!["TXT".to_string()]),
        );
        assert!(guard.validate(Path::new("report.txt")).is_ok());
        assert_eq!(
            guard.validate(Path::new("report.pdf")).unwrap_err(),
            PathGuardError::InvalidExtension
        );
    }

    #[test]
    fn denied_extensions_win_over_allowed_ones() {
        let guard = PathGuard::new(PathGuardConfig::default().with_allowed_extensions(vec![
            "exe".to_string(),
        ]));
        assert_eq!(
            guard.validate(Path::new("payload.exe")).unwrap_err(),
            PathGuardError::InvalidExtension
        );
    }

    #[test]
    fn a_path_without_extension_is_unaffected_by_the_extension_rules() {
        let guard = PathGuard::new(
            PathGuardConfig::default().with_allowed_extensions(vec!["txt".to_string()]),
        );
        assert!(guard.validate(Path::new("README")).is_ok());
    }

    #[test]
    fn sanitize_filename_strips_traversal_and_reserved_names() {
        // Slashes become underscores and dots are kept, so the result can never
        // climb out of its directory even though it is not prettified.
        let sanitized = sanitize_filename("../../etc/passwd");
        assert!(!sanitized.contains('/'));
        assert!(!sanitized.contains('\\'));
        assert_eq!(sanitize_filename("NUL"), "_NUL");
        // AUX is a reserved Windows device name, so it is prefixed.
        assert_eq!(sanitize_filename("aux.tar.gz"), "_aux.tar.gz");
        assert_eq!(sanitize_filename("report.tar.gz"), "report.tar.gz");
        assert!(!sanitize_filename("a\u{0}b").contains('\u{0}'));
    }
}
