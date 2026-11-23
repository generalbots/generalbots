//! #2831 — confinement of Designer local file writes to the server-owned
//! bot/tenant work dir.

use super::types::{is_safe_file_name, is_safe_slug};
use std::path::{Path, PathBuf};

/// Resolve a Designer write target under the server-owned `.gbapp` root.
///
/// `site_path`, `org_id` and `bot_slug` are server-derived; `app_name` and
/// `file_name` come from the HTTP body / LLM JSON and are untrusted. Every
/// component is validated as a single safe segment (no separators, `..`, NUL
/// or absolute paths), and the canonicalized target must remain under the
/// canonicalized root so a pre-planted symlink cannot redirect the write.
pub(super) fn confined_write_path(
    site_path: &str,
    org_id: &str,
    bot_slug: &str,
    app_name: &str,
    file_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    if !is_safe_slug(org_id) {
        return Err("invalid org id".into());
    }
    if !is_safe_slug(bot_slug) {
        return Err("invalid bot slug".into());
    }
    if !is_safe_slug(app_name) {
        return Err("invalid app name".into());
    }
    if !is_safe_file_name(file_name) {
        return Err("invalid file name".into());
    }

    let root = Path::new(site_path)
        .join(format!("{org_id}.gborg"))
        .join(format!("{bot_slug}.gbai"))
        .join(format!("{bot_slug}.gbapp"));
    std::fs::create_dir_all(&root)?;
    let canonical_root = std::fs::canonicalize(&root)?;

    let app_dir = root.join(app_name);
    std::fs::create_dir_all(&app_dir)?;
    let canonical_app_dir = std::fs::canonicalize(&app_dir)?;
    if !canonical_app_dir.starts_with(&canonical_root) {
        return Err("app directory escapes designer root".into());
    }

    let target = app_dir.join(file_name);
    if let Ok(metadata) = std::fs::symlink_metadata(&target) {
        if metadata.file_type().is_symlink() {
            return Err("refusing to write through a symlink".into());
        }
        let canonical_target = std::fs::canonicalize(&target)?;
        if !canonical_target.starts_with(&canonical_root) {
            return Err("target escapes designer root".into());
        }
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORG: &str = "00000000-0000-0000-0000-000000000000";

    fn sandbox(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gb-designer-2831-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn rejects_traversal_absolute_and_nul_payloads() {
        let dir = sandbox("payloads");
        let site = dir.to_string_lossy().to_string();
        for (app, file) in [
            ("../../etc", "passwd"),
            ("/etc", "passwd"),
            ("app", "../outside.txt"),
            ("app", "/tmp/outside.txt"),
            ("app", "a\\b"),
            ("app", "nul\0byte"),
            ("app", ".."),
        ] {
            assert!(
                confined_write_path(&site, ORG, "default", app, file).is_err(),
                "payload must be rejected: {app:?}/{file:?}"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn accepts_valid_payload_and_stays_under_root() {
        let dir = sandbox("valid");
        let site = dir.join("sites");
        let target = confined_write_path(
            &site.to_string_lossy(),
            ORG,
            "default",
            "my-app",
            "index.html",
        )
        .expect("valid payload must resolve");
        assert!(target.ends_with("default.gbapp/my-app/index.html"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_escape_in_app_dir() {
        use std::os::unix::fs::symlink;
        let dir = sandbox("symlink");
        let outside = dir.join("outside");
        let _ = std::fs::create_dir_all(&outside);
        let site = dir.join("sites");
        let root = site
            .join(format!("{ORG}.gborg"))
            .join("default.gbai")
            .join("default.gbapp");
        let _ = std::fs::create_dir_all(&root);
        let _ = symlink(&outside, root.join("escape"));
        let result = confined_write_path(
            &site.to_string_lossy(),
            ORG,
            "default",
            "escape",
            "index.html",
        );
        assert!(result.is_err(), "symlinked app dir must be rejected");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
