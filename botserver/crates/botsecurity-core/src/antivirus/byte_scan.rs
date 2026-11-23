//! Byte-oriented ClamAV scan used by the upload malware hook (#2597).
//!
//! The upload gate owns an in-memory payload, so a filesystem-path scanner is
//! not directly usable. This module writes the bounded payload to a private,
//! owner-only staging file, runs the allowlisted `clamscan` binary through
//! [`SafeCommand`] and maps its exit status and output to a typed verdict.
//! A scanner error is always a failure — it is never reported as clean.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use super::{Context, Result, SafeCommand};

/// Same bound as the hostile-file gate's compressed payload cap.
pub const MAX_SCAN_BYTES: usize = 100 * 1024 * 1024;
const SCANNER_NAME: &str = "clamav";
const SCAN_DIR: &str = "gb-clamav-scan";

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

/// Typed result of a ClamAV byte scan. Callers must not turn `Failed` into a
/// clean verdict: an unavailable or broken scanner fails closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ByteScanVerdict {
    Clean { scanner: String },
    Infected { scanner: String, signature: String },
    Failed { detail: String },
}

/// Scan `data` with ClamAV. The staging file is created with `create_new` in a
/// 0700 directory (0600 on the file), so no other local user can influence the
/// bytes the privileged scanner reads.
pub fn scan_bytes(data: &[u8]) -> ByteScanVerdict {
    if data.is_empty() {
        return ByteScanVerdict::Failed {
            detail: "empty payload".to_string(),
        };
    }
    if data.len() > MAX_SCAN_BYTES {
        return ByteScanVerdict::Failed {
            detail: format!("payload exceeds {MAX_SCAN_BYTES} byte scan limit"),
        };
    }

    let path = match stage(data) {
        Ok(path) => path,
        Err(detail) => return ByteScanVerdict::Failed { detail },
    };

    let outcome = run_clamscan(&path);
    if let Err(error) = fs::remove_file(&path) {
        log::warn!("clamav staging cleanup failed: {error}");
    }
    outcome
}

fn stage(data: &[u8]) -> Result<PathBuf> {
    let dir = std::env::temp_dir().join(SCAN_DIR);
    fs::create_dir_all(&dir).context("clamav staging directory")?;
    #[cfg(unix)]
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700))
        .context("clamav staging directory permissions")?;

    let path = dir.join(format!("{}.bin", Uuid::new_v4()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(&path).context("clamav staging file create")?;
    file.write_all(data).context("clamav staging file write")?;
    file.flush().context("clamav staging file flush")?;
    drop(file);
    Ok(path)
}

fn run_clamscan(path: &Path) -> ByteScanVerdict {
    let command = SafeCommand::new("clamscan")
        .and_then(|cmd| cmd.arg("--no-summary"))
        .and_then(|cmd| cmd.arg("--infected"))
        .and_then(|cmd| cmd.path_arg(path))
        .and_then(|cmd| cmd.execute());

    let output = match command {
        Ok(output) => output,
        Err(error) => {
            return ByteScanVerdict::Failed {
                detail: format!("clamscan could not be executed: {error}"),
            }
        }
    };

    let signature = infected_signature(&String::from_utf8_lossy(&output.stdout));
    if output.status.success() {
        return match &signature {
            Some(signature) => ByteScanVerdict::Infected {
                scanner: SCANNER_NAME.to_string(),
                signature: signature.clone(),
            },
            None => ByteScanVerdict::Clean {
                scanner: SCANNER_NAME.to_string(),
            },
        };
    }

    // clamscan exits 1 when a virus is found; any other non-zero status is an
    // engine failure and must never be reported as clean.
    if output.status.code() == Some(1) {
        if let Some(signature) = signature {
            return ByteScanVerdict::Infected {
                scanner: SCANNER_NAME.to_string(),
                signature,
            };
        }
    }

    let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
    ByteScanVerdict::Failed {
        detail: if detail.is_empty() {
            format!("clamscan exited with {}", output.status)
        } else {
            detail
        },
    }
}

fn infected_signature(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Some(position) = line.find(" FOUND") {
            let head = &line[..position];
            if let Some((_, signature)) = head.rsplit_once(':') {
                let signature = signature.trim();
                if !signature.is_empty() {
                    return Some(signature.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clamscan_infected_line() {
        let output = "/tmp/gb-clamav-scan/x.bin: Eicar-Test-Signature FOUND\n";
        assert_eq!(
            infected_signature(output),
            Some("Eicar-Test-Signature".to_string())
        );
    }

    #[test]
    fn clean_and_empty_results_are_distinct() {
        assert_eq!(
            infected_signature("Scanning /tmp/x.bin\nInfected files: 0\n"),
            None
        );
        assert_eq!(
            scan_bytes(&[]),
            ByteScanVerdict::Failed {
                detail: "empty payload".to_string()
            }
        );
    }
}
