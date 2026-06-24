use std::path::{Component, Path, PathBuf};
use tracing::warn;

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

impl std::fmt::Display for PathGuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PathTraversal => write!(f, "Path traversal attempt detected"),
            Self::AbsolutePath => write!(f, "Absolute paths are not allowed"),
            Self::InvalidComponent => write!(f, "Invalid path component"),
            Self::EmptyPath => write!(f, "Empty path is not allowed"),
            Self::OutsideAllowedRoot => write!(f, "Path is outside allowed root directory"),
            Self::SymlinkNotAllowed => write!(f, "Symbolic links are not allowed"),
            Self::HiddenFileNotAllowed => write!(f, "Hidden files are not allowed"),
            Self::InvalidExtension => write!(f, "File extension is not allowed"),
            Self::PathTooLong => write!(f, "Path exceeds maximum length"),
            Self::NullByte => write!(f, "Path contains null byte"),
        }
    }
}

impl std::error::Error for PathGuardError {}

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

impl Default for PathGuardConfig {
    fn default() -> Self {
        Self {
            allowed_roots: vec![],
            allow_symlinks: false,
            allow_hidden_files: false,
            allowed_extensions: None,
            denied_extensions: vec![
                "exe".to_string(),
                "bat".to_string(),
                "cmd".to_string(),
                "sh".to_string(),
                "ps1".to_string(),
                "vbs".to_string(),
                "js".to_string(),
                "jar".to_string(),
                "msi".to_string(),
                "dll".to_string(),
                "so".to_string(),
            ],
            max_path_length: 4096,
            max_depth: 20,
        }
    }
}

impl PathGuardConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn permissive() -> Self {
        Self {
            allowed_roots: vec![],
            allow_symlinks: true,
            allow_hidden_files: true,
            allowed_extensions: None,
            denied_extensions: vec![],
            max_path_length: 8192,
            max_depth: 50,
        }
    }

    pub fn strict() -> Self {
        Self {
            allowed_roots: vec![],
            allow_symlinks: false,
            allow_hidden_files: false,
            allowed_extensions: Some(vec![
                "txt".to_string(),
                "pdf".to_string(),
                "doc".to_string(),
                "docx".to_string(),
                "xls".to_string(),
                "xlsx".to_string(),
                "csv".to_string(),
                "json".to_string(),
                "xml".to_string(),
                "png".to_string(),
                "jpg".to_string(),
                "jpeg".to_string(),
                "gif".to_string(),
                "svg".to_string(),
                "mp3".to_string(),
                "mp4".to_string(),
                "wav".to_string(),
                "zip".to_string(),
            ]),
            denied_extensions: vec![],
            max_path_length: 2048,
            max_depth: 10,
        }
    }

    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.allowed_roots.push(root.into());
        self
    }

    pub fn with_roots(mut self, roots: Vec<PathBuf>) -> Self {
        self.allowed_roots = roots;
        self
    }

    pub fn allow_symlinks(mut self, allow: bool) -> Self {
        self.allow_symlinks = allow;
        self
    }

    pub fn allow_hidden(mut self, allow: bool) -> Self {
        self.allow_hidden_files = allow;
        self
    }

    pub fn with_allowed_extensions(mut self, extensions: Vec<String>) -> Self {
        self.allowed_extensions = Some(extensions);
        self
    }

    pub fn with_denied_extensions(mut self, extensions: Vec<String>) -> Self {
        self.denied_extensions = extensions;
        self
    }

    pub fn with_max_length(mut self, length: usize) -> Self {
        self.max_path_length = length;
        self
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }
}

pub struct PathGuard {
    config: PathGuardConfig,
}

impl Default for PathGuard {
    fn default() -> Self {
        Self::new(PathGuardConfig::default())
    }
}

impl PathGuard {
    pub fn new(config: PathGuardConfig) -> Self {
        Self { config }
    }

    pub fn validate(&self, path: &Path) -> Result<PathBuf, PathGuardError> {
        let path_str = path.to_string_lossy();
        if path_str.contains('\0') {
            warn!(path = %path_str, "Path contains null byte");
            return Err(PathGuardError::NullByte);
        }

        if path_str.is_empty() {
            return Err(PathGuardError::EmptyPath);
        }

        if path_str.len() > self.config.max_path_length {
            warn!(path_len = path_str.len(), max = self.config.max_path_length, "Path too long");
            return Err(PathGuardError::PathTooLong);
        }

        if path.is_absolute() && !self.config.allowed_roots.is_empty() {
            let is_within_root = self.config.allowed_roots.iter().any(|root| {
                path.starts_with(root)
            });
            if !is_within_root {
                warn!(path = %path_str, "Absolute path outside allowed roots");
                return Err(PathGuardError::AbsolutePath);
            }
        }

        let mut depth: usize = 0;
        let mut normalized = PathBuf::new();

        for component in path.components() {
            match component {
                Component::ParentDir => {
                    if normalized.pop() {
                        depth = depth.saturating_sub(1);
                    } else {
                        warn!(path = %path_str, "Path traversal attempt detected");
                        return Err(PathGuardError::PathTraversal);
                    }
                }
                Component::Normal(name) => {
                    let name_str = name.to_string_lossy();

                    if !self.config.allow_hidden_files && name_str.starts_with('.') {
                        warn!(path = %path_str, component = %name_str, "Hidden file not allowed");
                        return Err(PathGuardError::HiddenFileNotAllowed);
                    }

                    if has_dangerous_patterns(&name_str) {
                        warn!(path = %path_str, component = %name_str, "Invalid path component");
                        return Err(PathGuardError::InvalidComponent);
                    }

                    normalized.push(name);
                    depth += 1;

                    if depth > self.config.max_depth {
                        warn!(path = %path_str, depth = depth, max = self.config.max_depth, "Path depth exceeded");
                        return Err(PathGuardError::PathTooLong);
                    }
                }
                Component::RootDir => {
                    normalized.push(Component::RootDir);
                }
                Component::Prefix(prefix) => {
                    normalized.push(prefix.as_os_str());
                }
                Component::CurDir => {}
            }
        }

        if let Some(ext) = normalized.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();

            if let Some(ref allowed) = self.config.allowed_extensions {
                if !allowed.iter().any(|e| e.to_lowercase() == ext_str) {
                    warn!(path = %path_str, extension = %ext_str, "Extension not in allowed list");
                    return Err(PathGuardError::InvalidExtension);
                }
            }

            if self.config.denied_extensions.iter().any(|e| e.to_lowercase() == ext_str) {
                warn!(path = %path_str, extension = %ext_str, "Extension is denied");
                return Err(PathGuardError::InvalidExtension);
            }
        }

        Ok(normalized)
    }

    pub fn validate_and_resolve(&self, base: &Path, path: &Path) -> Result<PathBuf, PathGuardError> {
        let validated = self.validate(path)?;
        let full_path = base.join(&validated);

        if !self.config.allowed_roots.is_empty() {
            let is_within_root = self.config.allowed_roots.iter().any(|root| {
                full_path.starts_with(root)
            });
            if !is_within_root {
                warn!(
                    path = %full_path.display(),
                    "Resolved path outside allowed roots"
                );
                return Err(PathGuardError::OutsideAllowedRoot);
            }
        }

        Ok(full_path)
    }

    pub fn validate_existing(&self, path: &Path) -> Result<PathBuf, PathGuardError> {
        let validated = self.validate(path)?;

        if !self.config.allow_symlinks && validated.is_symlink() {
            warn!(path = %validated.display(), "Symlink not allowed");
            return Err(PathGuardError::SymlinkNotAllowed);
        }

        if let Ok(canonical) = validated.canonicalize() {
            if !self.config.allowed_roots.is_empty() {
                let is_within_root = self.config.allowed_roots.iter().any(|root| {
                    if let Ok(root_canonical) = root.canonicalize() {
                        canonical.starts_with(&root_canonical)
                    } else {
                        canonical.starts_with(root)
                    }
                });
                if !is_within_root {
                    warn!(
                        path = %canonical.display(),
                        "Canonical path outside allowed roots"
                    );
                    return Err(PathGuardError::OutsideAllowedRoot);
                }
            }
            Ok(canonical)
        } else {
            Ok(validated)
        }
    }
}

fn has_dangerous_patterns(name: &str) -> bool {
    let dangerous = [
        "..",
        "...",
        "~",
        "$",
        "`",
        "|",
        ";",
        "&",
        "<",
        ">",
        "\\",
        "%00",
        "%2e",
        "%2f",
        "%5c",
        "\r",
        "\n",
        "\t",
    ];

    for pattern in &dangerous {
        if name.contains(pattern) {
            return true;
        }
    }

    if name.chars().any(|c| c.is_control()) {
        return true;
    }

    false
}

pub fn sanitize_filename(name: &str) -> String {
    let dangerous_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];

    let sanitized: String = name
        .chars()
        .map(|c| {
            if dangerous_chars.contains(&c) || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();

    let sanitized = sanitized.trim_matches(|c| c == '.' || c == ' ');

    if sanitized.is_empty() {
        return "unnamed".to_string();
    }

    let reserved = [
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    let upper = sanitized.to_uppercase();
    let base_name = upper.split('.').next().unwrap_or("");
    if reserved.contains(&base_name) {
        return format!("_{}", sanitized);
    }

    if sanitized.len() > 255 {
        sanitized[..255].to_string()
    } else {
        sanitized.to_string()
    }
}

pub fn sanitize_path_component(component: &str) -> String {
    sanitize_filename(component)
}

pub fn is_safe_path(path: &Path) -> bool {
    PathGuard::default().validate(path).is_ok()
}

pub fn join_safe(base: &Path, relative: &Path) -> Result<PathBuf, PathGuardError> {
    let guard = PathGuard::new(PathGuardConfig::default().with_root(base.to_path_buf()));
    guard.validate_and_resolve(base, relative)
}

pub fn canonicalize_safe(path: &Path, allowed_root: &Path) -> Result<PathBuf, PathGuardError> {
    let guard = PathGuard::new(PathGuardConfig::default().with_root(allowed_root.to_path_buf()));
    guard.validate_existing(path)
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
