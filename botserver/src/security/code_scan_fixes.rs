//! Code scanning alert fixes (Issue #500).
//! Security helpers to address GitHub code scanning findings.

/// Sanitizes a filename for path operations to prevent path traversal.
pub fn sanitize_filename(name: &str) -> String {
    name.replace(['/', '\\'], "_").replace("..", "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
        .collect()
}

/// Validates that a path doesn't escape the base directory.
/// If the target exists, uses canonicalization for defense in depth.
/// If it doesn't exist, checks manually for ".." traversal patterns.
pub fn is_safe_path(base: &std::path::Path, path: &std::path::Path) -> bool {
    // Check for ".." traversal in the sub-path first (works regardless of canonicalize)
    for component in path.components() {
        if component == std::path::Component::ParentDir {
            return false;
        }
    }

    let target = base.join(path);

    // Try canonicalizing base — if base doesn't exist, fall back to string comparison
    let base_for_check = match base.canonicalize() {
        Ok(canonical) => canonical.to_path_buf(),
        Err(_) => {
            let base_str = base.to_string_lossy().to_string();
            let target_str = target.to_string_lossy().to_string();
            return target_str.starts_with(&base_str);
        }
    };

    match target.canonicalize() {
        Ok(canonical_target) => {
            let base_canonical = base_for_check.canonicalize().unwrap_or(base_for_check);
            canonical_target.starts_with(&base_canonical)
        }
        Err(_) => {
            let target_str = target.to_string_lossy();
            let base_str = base_for_check.to_string_lossy();
            target_str.starts_with(base_str.as_ref())
        }
    }
}
