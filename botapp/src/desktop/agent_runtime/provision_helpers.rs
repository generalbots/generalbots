//! Helper functions for the provisioning step machine.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::desktop::agent_runtime::host::{DownloadError, RuntimeHost};
use crate::desktop::agent_runtime::manifest::{self, Manifest};
use super::ProvisionEvents;
use crate::desktop::agent_runtime::state::{
    ErrorEvent, LastError, PersistedState, ProgressEvent,
};

pub(super) fn wsl_exe() -> PathBuf {
    #[cfg(windows)]
    {
        let root = std::env::var("SystemRoot")
            .unwrap_or_else(|_| r"C:\Windows".to_string());
        PathBuf::from(root).join("System32").join("wsl.exe")
    }
    #[cfg(not(windows))]
    {
        PathBuf::from("wsl")
    }
}

pub(super) fn cancelled(flag: &AtomicBool) -> bool {
    flag.load(Ordering::Relaxed)
}

pub(super) fn emit_progress(
    events: &dyn ProvisionEvents,
    step: &'static str,
    done: u64,
    total: u64,
    pct: u8,
) {
    events.progress(ProgressEvent {
        step,
        done_bytes: done,
        total_bytes: total,
        percent: pct,
        state: "installing",
    });
}

pub(super) fn save_state(host: &dyn RuntimeHost, path: &Path, state: &PersistedState) {
    if let Ok(json) = state.to_json() {
        if let Err(e) = host.write_file(path, &json) {
            log::warn!("agent_runtime: cannot save state: {e}");
        }
    }
}

pub(super) fn record_error(state: &mut PersistedState, step: &str, code: &str, detail: &str) {
    state.last_error = Some(LastError {
        step: step.into(),
        code: code.into(),
        detail: detail.into(),
    });
}

pub(super) fn fetch_manifest(
    host: &dyn RuntimeHost,
    events: &dyn ProvisionEvents,
) -> Result<Manifest, &'static str> {
    let url = crate::desktop::agent_runtime::state::manifest_url();
    let data = host.fetch_bytes(&url).map_err(|e| {
        log::warn!("agent_runtime: manifest fetch failed: {e}");
        let detail = match &e {
            DownloadError::CertificateNotTrusted(msg) => msg.clone(),
            other => other.to_string(),
        };
        let code = match &e {
            DownloadError::CertificateNotTrusted(_) => "offline",
            _ => "offline",
        };
        events.error(ErrorEvent {
            step: "rootfs_download",
            code,
            detail,
        });
        "offline"
    })?;
    manifest::parse(&data).map_err(|e| {
        log::warn!("agent_runtime: manifest invalid: {e}");
        events.error(ErrorEvent {
            step: "rootfs_download",
            code: "unknown",
            detail: e,
        });
        "unknown"
    })
}

/// Verify SHA-256 of a file by hashing in 64 KB chunks through the host trait.
pub(super) fn verify_sha256(host: &dyn RuntimeHost, path: &Path, expected: &str) -> bool {
    match host.sha256_file(path) {
        Ok(digest) => digest == expected,
        Err(_) => false,
    }
}

/// Convert a Windows path to a WSL path.
///
/// Returns an error when the path is a UNC path or has no drive letter,
/// since those cannot be mapped to `/mnt/<drive>/...`.
///
/// For the production code path after first boot, the caller should use
/// `wslpath -u` inside the distro instead of this heuristic.
pub(super) fn windows_to_wsl_path(win_path: &str) -> Result<String, String> {
    let path = win_path.replace('\\', "/");
    // Reject UNC paths: they start with // after backslash normalisation.
    if path.starts_with("//") {
        return Err(format!("UNC paths cannot be converted to WSL paths: {win_path}"));
    }
    if path.len() >= 2 && path.as_bytes()[1] == b':' {
        let drive = (path.as_bytes()[0] as char).to_ascii_lowercase();
        Ok(format!("/mnt/{drive}{}", &path[2..]))
    } else {
        Err(format!("path has no drive letter, cannot convert to WSL path: {win_path}"))
    }
}

/// Download an asset with resume, verify size and SHA-256.
pub(super) fn download_and_verify(
    host: &dyn RuntimeHost,
    url: &str,
    dest: &Path,
    expected_size: u64,
    expected_sha: &str,
    step_name: &'static str,
    events: &dyn ProvisionEvents,
    cancel: &AtomicBool,
) -> Result<(), &'static str> {
    let download_result = host.download(
        url,
        dest,
        expected_size,
        &|done, total| {
            let pct = if total > 0 {
                ((done as f64 / total as f64) * 100.0) as u8
            } else {
                0
            };
            emit_progress(events, step_name, done, total, pct);
        },
        &|| cancel.load(Ordering::Relaxed),
    );
    match download_result {
        Ok(size) => {
            if size != expected_size {
                let _ = host.remove_file(dest);
                events.error(ErrorEvent {
                    step: step_name,
                    code: "checksum",
                    detail: format!("size mismatch: got {size}, expected {expected_size}"),
                });
                return Err("checksum");
            }
        }
        Err(DownloadError::Cancelled) => return Err("cancelled"),
        Err(DownloadError::CertificateNotTrusted(msg)) => {
            events.error(ErrorEvent {
                step: step_name,
                code: "offline",
                detail: msg,
            });
            return Err("offline");
        }
        Err(e) => {
            events.error(ErrorEvent {
                step: step_name,
                code: "offline",
                detail: e.to_string(),
            });
            return Err("offline");
        }
    }

    if !verify_sha256(host, dest, expected_sha) {
        let _ = host.remove_file(dest);
        events.error(ErrorEvent {
            step: step_name,
            code: "checksum",
            detail: "SHA-256 mismatch".into(),
        });
        return Err("checksum");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::windows_to_wsl_path;

    #[test]
    fn drive_letter_paths_convert() {
        assert_eq!(
            windows_to_wsl_path(r"C:\Users\me\file.tar.gz").unwrap(),
            "/mnt/c/Users/me/file.tar.gz"
        );
        assert_eq!(
            windows_to_wsl_path(r"D:\data\image.tar.xz").unwrap(),
            "/mnt/d/data/image.tar.xz"
        );
    }

    #[test]
    fn forward_slashes_work() {
        assert_eq!(
            windows_to_wsl_path("C:/Users/me/file").unwrap(),
            "/mnt/c/Users/me/file"
        );
    }

    #[test]
    fn spaces_and_non_ascii_work() {
        assert_eq!(
            windows_to_wsl_path(r"C:\Users\José da Silva\AppData\Local\file.tar.gz").unwrap(),
            "/mnt/c/Users/Jos\u{e9} da Silva/AppData/Local/file.tar.gz"
        );
    }

    #[test]
    fn unc_path_rejected() {
        let result = windows_to_wsl_path(r"\\server\share\file");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("UNC"));
    }

    #[test]
    fn no_drive_letter_rejected() {
        let result = windows_to_wsl_path("relative/path");
        assert!(result.is_err());
    }
}
