use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::LazyLock;

static FORBIDDEN_SHELL_CHARS: LazyLock<HashSet<char>> = LazyLock::new(|| {
    HashSet::from([
        ';', '|', '&', '$', '`', '<', '>', '\n', '\r', '\0',
    ])
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandGuardError {
    CommandNotAllowed(String),
    InvalidArgument(String),
    PathTraversal(String),
    ExecutionFailed(String),
    ShellInjectionAttempt(String),
}

impl std::fmt::Display for CommandGuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandNotAllowed(cmd) => write!(f, "Command not in allowlist: {cmd}"),
            Self::InvalidArgument(arg) => write!(f, "Invalid argument: {arg}"),
            Self::PathTraversal(path) => write!(f, "Path traversal detected: {path}"),
            Self::ExecutionFailed(msg) => write!(f, "Command execution failed: {msg}"),
            Self::ShellInjectionAttempt(input) => {
                write!(f, "Shell injection attempt detected: {input}")
            }
        }
    }
}

impl From<CommandGuardError> for String {
    fn from(val: CommandGuardError) -> Self {
        val.to_string()
    }
}

impl std::error::Error for CommandGuardError {}

pub fn validate_argument(arg: &str) -> Result<(), CommandGuardError> {
    if arg.is_empty() {
        return Err(CommandGuardError::InvalidArgument(
            "Empty argument".to_string(),
        ));
    }

    if arg.len() > 4096 {
        return Err(CommandGuardError::InvalidArgument(
            "Argument too long".to_string(),
        ));
    }

    let is_url = arg.starts_with("http://") || arg.starts_with("https://");

    for c in arg.chars() {
        if FORBIDDEN_SHELL_CHARS.contains(&c) {
            if is_url && (c == '&' || c == '?' || c == '=') {
                continue;
            }
            return Err(CommandGuardError::ShellInjectionAttempt(format!(
                "Forbidden character '{}' in argument",
                c.escape_default()
            )));
        }
    }

    let dangerous_patterns = [
        "$(", "`", "&&", "||", ">>", "<<", "..", "//", "\\\\",
    ];

    for pattern in dangerous_patterns {
        if arg.contains(pattern) {
            if is_url && pattern == "//" {
                continue;
            }
            return Err(CommandGuardError::ShellInjectionAttempt(format!(
                "Dangerous pattern '{}' detected",
                pattern
            )));
        }
    }

    Ok(())
}

fn normalize_path_for_compare(p: &std::path::Path) -> String {
    let s = p.to_string_lossy();
    let s = if let Some(stripped) = s.strip_prefix("\\\\?\\") {
        stripped.to_string()
    } else {
        s.to_string()
    };
    let s = if s.starts_with("Z:\\") || s.starts_with("Z:/")
        || s.starts_with("z:\\") || s.starts_with("z:/")
    {
        s.replacen("Z:\\", "/", 1)
            .replacen("Z:/", "/", 1)
            .replacen("z:\\", "/", 1)
            .replacen("z:/", "/", 1)
    } else {
        s
    };
    s.replace('\\', "/")
}

pub fn validate_path(
    path: &std::path::Path,
    allowed_roots: &[PathBuf],
) -> Result<PathBuf, CommandGuardError> {
    let canonical = match path.canonicalize() {
        Ok(c) => c,
        Err(_) => match path.parent() {
            Some(parent) => match parent.canonicalize() {
                Ok(p) => p.join(path.file_name().unwrap_or_default()),
                Err(_) => path.to_path_buf(),
            },
            None => path.to_path_buf(),
        },
    };

    let path_str = canonical.to_string_lossy();
    if path_str.contains("..") {
        return Err(CommandGuardError::PathTraversal(format!(
            "Path contains traversal: {}",
            path.display()
        )));
    }

    let is_allowed = allowed_roots.iter().any(|root| {
        if canonical.starts_with(root) {
            return true;
        }
        let np = normalize_path_for_compare(&canonical);
        let nr = normalize_path_for_compare(root);
        np.starts_with(&nr)
    });

    if !is_allowed {
        return Err(CommandGuardError::PathTraversal(format!(
            "Path outside allowed directories: {}",
            path.display()
        )));
    }

    Ok(canonical)
}

pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect::<String>()
        .trim_start_matches('.')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_argument_valid() {
        assert!(validate_argument("hello").is_ok());
        assert!(validate_argument("-f").is_ok());
        assert!(validate_argument("--format=csv").is_ok());
        assert!(validate_argument("/path/to/file.txt").is_ok());
    }

    #[test]
    fn test_validate_argument_invalid() {
        assert!(validate_argument("hello; rm -rf /").is_err());
        assert!(validate_argument("$(whoami)").is_err());
        assert!(validate_argument("file | cat").is_err());
        assert!(validate_argument("test && echo").is_err());
        assert!(validate_argument("`id`").is_err());
        assert!(validate_argument("").is_err());
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test.pdf"), "test.pdf");
        assert_eq!(sanitize_filename("my-file_v1.txt"), "my-file_v1.txt");
        assert_eq!(sanitize_filename("../../../etc/passwd"), "etcpasswd");
        assert_eq!(sanitize_filename(".hidden"), "hidden");
        assert_eq!(
            sanitize_filename("file;rm -rf.txt"),
            "filerm-rf.txt"
        );
    }

    #[test]
    fn test_path_traversal_detection() {
        let result = validate_argument("../../../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_command_guard_error_display() {
        let err = CommandGuardError::CommandNotAllowed("bash".to_string());
        assert!(err.to_string().contains("bash"));

        let err2 = CommandGuardError::ShellInjectionAttempt("$(id)".to_string());
        assert!(err2.to_string().contains("injection"));
    }
}
