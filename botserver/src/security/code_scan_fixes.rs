//! Code scanning alert fixes (Issue #500).
//! Security helpers to address GitHub code scanning findings.

/// Sanitizes a filename for path operations to prevent path traversal.
pub fn sanitize_filename(name: &str) -> String {
    name.replace('/', "_")
        .replace('\\', "_")
        .replace("..", "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
        .collect()
}

/// Validates that a path doesn't escape the base directory.
pub fn is_safe_path(base: &std::path::Path, path: &std::path::Path) -> bool {
    let canonical_base = base.canonicalize().unwrap_or_else(|_| base.to_path_buf());
    let target = base.join(path);
    let canonical_target = target.canonicalize().unwrap_or_else(|_| target);
    canonical_target.starts_with(&canonical_base)
}
