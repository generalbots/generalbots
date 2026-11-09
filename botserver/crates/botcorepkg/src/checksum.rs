//! Checksum policy for downloaded runtime components (#2001).
//!
//! `3rdparty.toml` pins a component by `url + sha256`, and both download paths
//! must enforce that pin. Two gaps made the guarantee weaker than documented:
//!
//! * `verify_checksum()` returned `Ok(true)` for an empty checksum, so the ten
//!   entries without one (dns, email, meet, proxy, observability, timeseries_db,
//!   webmail, libpq_win, table_editor, botmodels, plus every Windows/arm64
//!   variant missing its own hash) installed with **no verification at all**;
//! * `facade_download::download_and_install` — the path the first-run wizard
//!   uses — never verified anything, even when a checksum existed.
//!
//! This module is the single policy:
//!
//! * a configured checksum is always verified and a mismatch is fatal;
//! * a missing checksum is a loud warning by default, and fatal when
//!   `GB_REQUIRE_CHECKSUMS=1` (CI, production fleet, hardened installs).

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

/// Whether a missing checksum must abort the install.
///
/// Accepts `1`/`true`/`TRUE`/`yes` so both shell and TOML-style configs work.
#[must_use]
pub fn require_checksums() -> bool {
    matches!(
        std::env::var("GB_REQUIRE_CHECKSUMS").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("yes")
    )
}

/// Lowercase hex SHA-256 of a file, streamed so a multi-GB archive does not
/// have to fit in memory.
pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path)
        .map_err(|error| anyhow!("cannot open {} for checksum: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| anyhow!("cannot read {} for checksum: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Verifies `path` against an expected checksum with an explicit strict flag.
///
/// Split from [`verify_download`] so the policy is testable without mutating
/// the process environment.
pub fn verify_with_policy(path: &Path, expected: &str, component: &str, strict: bool) -> Result<()> {
    let expected = expected.trim();
    if expected.is_empty() {
        if strict {
            return Err(anyhow!(
                "no SHA-256 configured for '{component}' and GB_REQUIRE_CHECKSUMS is on; \
                 refusing to install an unverified download"
            ));
        }
        log::warn!(
            "No SHA-256 checksum configured for '{component}'; the downloaded file is NOT verified"
        );
        return Ok(());
    }
    let actual = sha256_file(path)?;
    if actual != expected.to_lowercase() {
        return Err(anyhow!(
            "checksum mismatch for '{component}': expected {expected}, got {actual} \
             ({}). The file is corrupt or the upstream asset changed.",
            path.display()
        ));
    }
    log::info!("Checksum verified for '{component}'");
    Ok(())
}

/// Verifies `path` against the checksum pinned for `component`, applying
/// `GB_REQUIRE_CHECKSUMS`.
pub fn verify_download(path: &Path, expected: &str, component: &str) -> Result<()> {
    verify_with_policy(path, expected, component, require_checksums())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_file(name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("gb-checksum-{name}"));
        let mut file = std::fs::File::create(&path).expect("create temp file");
        file.write_all(bytes).expect("write temp file");
        path
    }

    #[test]
    fn sha256_matches_known_digest() {
        let path = temp_file("known", b"abc");
        // SHA-256("abc")
        assert_eq!(
            sha256_file(&path).expect("hash"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn mismatch_is_fatal() {
        let path = temp_file("mismatch", b"payload");
        let error = verify_with_policy(&path, &"0".repeat(64), "demo", false)
            .expect_err("mismatch must fail");
        assert!(error.to_string().contains("checksum mismatch"));
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn match_passes_case_insensitively() {
        let path = temp_file("match", b"abc");
        let expected = "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD";
        verify_with_policy(&path, expected, "demo", true).expect("match");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn missing_checksum_warns_without_strict_mode() {
        let path = temp_file("missing", b"abc");
        verify_with_policy(&path, "", "demo", false).expect("lenient mode allows it");
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn missing_checksum_is_fatal_in_strict_mode() {
        let path = temp_file("strict", b"abc");
        let error = verify_with_policy(&path, "  ", "demo", true)
            .expect_err("strict mode must refuse");
        assert!(error.to_string().contains("no SHA-256"));
        std::fs::remove_file(path).ok();
    }
}
