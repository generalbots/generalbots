//! Working-directory validation and default-shell selection for the terminal.

/// Validates a working directory that lives **inside** a container. The host
/// cannot stat it, so only the shape is checked: absolute, no traversal, no
/// shell-quoting hazards. Returns `None` when the value is unusable.
pub fn sanitize_container_cwd(cwd: &str) -> Option<String> {
    let trimmed = cwd.trim();
    if trimmed.is_empty() || !trimmed.starts_with('/') || trimmed.len() > 300 {
        return None;
    }
    if trimmed.contains("..") || trimmed.contains('\\') || trimmed.contains('\n') {
        return None;
    }
    let allowed = trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '_' | '.' | ' '));
    allowed.then(|| trimmed.to_string())
}

pub fn sanitize_cwd(cwd: &str) -> String {
    if cwd.is_empty() || !std::path::Path::new(cwd).is_dir() {
        std::env::current_dir()
            .map(|d| d.to_string_lossy().to_string())
            .unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().into_owned())
    } else {
        cwd.to_string()
    }
}

pub fn default_shell() -> String {
    if cfg!(target_os = "windows") {
        std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".to_string())
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    }
}
