//! Path-traversal hardening for the suite static server (#2904).
//!
//! The suite fallback handler builds a filesystem path from the request URI
//! and reads it directly. Without validation, `..` segments, absolute paths,
//! percent-encoded traversal and symlinks escaping the UI root turn any GET
//! into an arbitrary file read. Keep all checks here so the handler has a
//! single, testable gate.

use std::path::{Path, PathBuf};

/// Decode `%XX` escapes once. Axum leaves percent-encoding intact in
/// `Uri::path()`, so traversal such as `%2e%2e` must be decoded before the
/// segment check. Malformed escapes are kept verbatim (they cannot traverse).
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_value(bytes[index + 1]), hex_value(bytes[index + 2])) {
                out.push(hi * 16 + lo);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// True when `relative` is safe to `Path::join` onto the UI root.
///
/// Rejects NUL bytes (literal and encoded), absolute paths (Unix and Windows),
/// and any `..` segment after percent-decoding. Both `/` and `\` are
/// treated as separators so Windows-style traversal is caught too.
pub fn is_safe_relative_path(relative: &str) -> bool {
    if relative.as_bytes().contains(&0) {
        return false;
    }
    // Decode up to three passes: a front proxy may have already decoded once,
    // so `%252e%252e` must be caught as well as `%2e%2e`.
    let mut decoded = percent_decode(relative);
    for _ in 0..2 {
        let next = percent_decode(&decoded);
        if next == decoded {
            break;
        }
        decoded = next;
    }
    let bytes = decoded.as_bytes();
    if bytes.contains(&0) {
        return false;
    }
    if decoded.starts_with('/') || decoded.starts_with('\\') {
        return false;
    }
    if bytes.get(1) == Some(&b':') && bytes.first().is_some_and(u8::is_ascii_alphabetic) {
        return false;
    }
    for segment in decoded.split(|c| c == '/' || c == '\\') {
        if segment == ".." {
            return false;
        }
    }
    true
}

/// Canonicalize `candidate` and return it only when it stays under `root`.
///
/// `canonicalize` resolves `..` and symlinks, so a symlink inside the root that
/// points outside is rejected by the `starts_with` check. A missing path also
/// yields `None` and is treated by the caller as 404.
pub fn resolve_static_path(root: &Path, candidate: &Path) -> Option<PathBuf> {
    let canonical_root = std::fs::canonicalize(root).ok()?;
    let canonical_candidate = std::fs::canonicalize(candidate).ok()?;
    if canonical_candidate.starts_with(&canonical_root) {
        Some(canonical_candidate)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{is_safe_relative_path, percent_decode, resolve_static_path};

    #[test]
    fn percent_decode_handles_escapes_and_malformed_input() {
        assert_eq!(percent_decode("js/%2e%2e/%2Fetc"), "js/..//etc");
        assert_eq!(percent_decode("a%41b"), "aAb");
        assert_eq!(percent_decode("100% done"), "100% done");
        assert_eq!(percent_decode("%zz"), "%zz");
    }

    #[test]
    fn rejects_traversal_corpus() {
        let corpus = [
            "..",
            "../etc/passwd",
            "js/../../etc/passwd",
            "suite/chat/../../../secret",
            "/etc/passwd",
            "//etc/passwd",
            "C:\\Windows\\win.ini",
            "js\\..\\..\\etc\\passwd",
            "%2e%2e/etc/passwd",
            "js/%2E%2E/%2e%2e/etc/passwd",
            "js%2f..%2f..%2fetc/passwd",
            "js/%252e%252e/passwd",
            "js/%00/../../etc/passwd",
            "js/\u{0}/etc/passwd",
        ];
        for case in corpus {
            assert!(!is_safe_relative_path(case), "must reject: {case}");
        }
    }

    #[test]
    fn accepts_normal_asset_paths() {
        let corpus = [
            "js/chat.js",
            "css/theme.css",
            "suite/chat/chat.js",
            "suite/docs/modules/99_init.js",
            "assets/logo.svg",
            "images/foo-bar_1.png",
            "partials/sidebar.html",
        ];
        for case in corpus {
            assert!(is_safe_relative_path(case), "must accept: {case}");
        }
    }

    #[test]
    fn canonicalize_requires_existing_path_under_root() {
        let base = std::env::temp_dir().join(format!(
            "gb-safe-path-missing-{}",
            std::process::id()
        ));
        let root = base.join("root");
        std::fs::create_dir_all(&root).unwrap();
        assert!(resolve_static_path(&root, &root.join("missing.txt")).is_none());
        std::fs::remove_dir_all(&base).ok();
    }

    #[cfg(unix)]
    #[test]
    fn canonicalize_rejects_symlink_escape() {
        use std::os::unix::fs::symlink;

        let base = std::env::temp_dir().join(format!(
            "gb-safe-path-link-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let root = base.join("root");
        let outside = base.join("outside");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("secret.txt"), b"secret").unwrap();
        symlink(&outside, root.join("escape")).unwrap();
        std::fs::write(root.join("ok.txt"), b"ok").unwrap();

        assert!(resolve_static_path(&root, &root.join("ok.txt")).is_some());
        assert!(resolve_static_path(&root, &root.join("escape/secret.txt")).is_none());

        std::fs::remove_dir_all(&base).ok();
    }
}
