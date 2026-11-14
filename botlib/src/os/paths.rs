use std::path::{Path, PathBuf};

/// Canonicalize a path without producing a Windows verbatim prefix (`\\?\`).
///
/// On Windows, `std::fs::canonicalize()` returns extended-length paths
/// (`\\?\C:\...`). These paths reject `/` as a separator and break callers
/// that build sub-paths with `format!("{work}/{sub}")`. This helper strips the
/// `\\?\` prefix when the remainder is a normal drive-letter path (e.g.
/// `C:\...`), leaving `\\?\UNC\...` and non-Windows paths untouched.
///
/// When canonicalization itself fails (e.g. the path does not exist yet), the
/// error is returned as-is so callers can decide their own fallback.
pub fn safe_canonicalize(path: &Path) -> std::io::Result<PathBuf> {
    let canonical = std::fs::canonicalize(path)?;
    Ok(strip_verbatim_prefix(canonical))
}

/// Strip a Windows `\\?\` verbatim prefix when the remainder is a plain
/// drive-letter path (`X:\...`). Other verbatim forms (`\\?\UNC\...`) and
/// non-Windows paths pass through unchanged.
fn strip_verbatim_prefix(path: PathBuf) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let s = path.to_string_lossy();
        if let Some(rest) = s.strip_prefix(r"\\?\") {
            let bytes = rest.as_bytes();
            if bytes.len() >= 3
                && bytes[0].is_ascii_alphabetic()
                && bytes[1] == b':'
                && bytes[2] == b'\\'
            {
                return PathBuf::from(rest.to_string());
            }
        }
        path
    }
    #[cfg(not(target_os = "windows"))]
    {
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_verbatim_drive_letter() {
        let p = PathBuf::from(r"\\?\C:\Users\test\work");
        let result = strip_verbatim_prefix(p);
        #[cfg(target_os = "windows")]
        assert_eq!(result, PathBuf::from(r"C:\Users\test\work"));
        #[cfg(not(target_os = "windows"))]
        assert_eq!(result, PathBuf::from(r"\\?\C:\Users\test\work"));
    }

    #[test]
    fn strip_verbatim_preserves_unc() {
        let p = PathBuf::from(r"\\?\UNC\server\share");
        let result = strip_verbatim_prefix(p.clone());
        assert_eq!(result, p);
    }

    #[test]
    fn strip_verbatim_preserves_unix() {
        let p = PathBuf::from("/opt/gbo/work");
        let result = strip_verbatim_prefix(p.clone());
        assert_eq!(result, p);
    }

    #[test]
    fn safe_canonicalize_existing_dir() {
        let tmp = std::env::temp_dir();
        let result = safe_canonicalize(&tmp);
        assert!(result.is_ok());
        let canon = result.unwrap();
        let s = canon.to_string_lossy();
        assert!(
            !s.starts_with(r"\\?\") || s.starts_with(r"\\?\UNC\"),
            "drive-letter verbatim prefix should be stripped: {s}"
        );
    }
}
