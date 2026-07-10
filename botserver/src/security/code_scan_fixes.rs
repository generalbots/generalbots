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

    // If base directory exists, use canonical base for consistent path resolution.
    // This avoids string comparison mismatches when base is a relative path
    // (e.g. "./botserver-stack/data/system/work") but canonicalize resolves it to
    // an absolute path (e.g. "/opt/gbo/bin/botserver-stack/...").
    if let Ok(canonical_base) = base.canonicalize() {
        let target = canonical_base.join(path);
        match target.canonicalize() {
            Ok(canonical_target) => {
                canonical_target.starts_with(&canonical_base)
            }
            Err(_) => {
                let target_str = target.to_string_lossy();
                let base_str = canonical_base.to_string_lossy();
                target_str.starts_with(base_str.as_ref())
            }
        }
    } else {
        // Base doesn't exist — use string comparison as-is
        let target = base.join(path);
        let base_str = base.to_string_lossy().to_string();
        let target_str = target.to_string_lossy().to_string();
        target_str.starts_with(&base_str)
    }
}
