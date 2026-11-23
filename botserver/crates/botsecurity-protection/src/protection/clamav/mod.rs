//! Real ClamAV scan behind the Protection API (#2599).
//!
//! The old dispatch sent `ProtectionTool::ClamAV` to `lynis::run_scan()`, which
//! produced a Lynis audit under a ClamAV label. This module runs `clamscan`
//! through [`SafeCommand`] on the hostile-file quarantine root (or an explicit
//! local target) and reports only what the engine actually found.

use anyhow::{Context, Result};
use tracing::info;

use botsecurity_core::command_guard::SafeCommand;

use super::manager::{Finding, FindingSeverity, ScanResultStatus};

/// Optional local scan target. When unset, the hostile-file quarantine root is
/// used so the endpoint never silently claims to have scanned the whole host.
const TARGET_ENV: &str = "GB_CLAMAV_SCAN_TARGET";

/// Run a real ClamAV scan and return findings in the Protection API format.
pub async fn run_scan() -> Result<(ScanResultStatus, Vec<Finding>, String)> {
    let (target, allowed) = match std::env::var(TARGET_ENV) {
        Ok(value) if !value.trim().is_empty() => {
            let path = std::path::PathBuf::from(value.trim());
            let allowed = path
                .parent()
                .map(std::path::Path::to_path_buf)
                .unwrap_or_else(|| path.clone());
            (path, allowed)
        }
        _ => (botsecurity_core::upload_hostile::malware::quarantine_root(), None),
    };

    if !target.exists() {
        std::fs::create_dir_all(&target).context("ClamAV scan target directory")?;
    }

    info!("Running ClamAV scan on {}", target.display());
    let mut command = SafeCommand::new("clamscan")?
        .arg("--infected")?
        .arg("--no-summary")?;
    if let Some(allowed) = allowed {
        command = command.allow_path(allowed);
    }
    let command = command.path_arg(&target)?;
    let output = command.execute().context("clamscan execution failed")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let raw_output = format!("{stdout}\n{stderr}");
    let findings = parse_clamav_output(&stdout);
    let status = if findings.is_empty() {
        ScanResultStatus::Clean
    } else {
        ScanResultStatus::Infected
    };

    Ok((status, findings, raw_output))
}

/// Parse `path: signature FOUND` lines, ignoring ClamAV's summary chatter.
pub fn parse_clamav_output(output: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for line in output.lines() {
        let Some(position) = line.find(" FOUND") else {
            continue;
        };
        let (path, signature) = match line[..position].rsplit_once(':') {
            Some(parts) => parts,
            None => continue,
        };
        let signature = signature.trim();
        if signature.is_empty() {
            continue;
        }
        findings.push(Finding {
            id: format!("clamav-{}", findings.len()),
            severity: FindingSeverity::High,
            category: "malware".to_string(),
            title: signature.to_string(),
            description: format!("ClamAV detected {signature}"),
            file_path: Some(path.trim().to_string()),
            remediation: Some(
                "Quarantine or remove the file, then update ClamAV signatures".to_string(),
            ),
        });
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_found_lines_and_ignores_summary() {
        let output = "/tmp/quarantine/a.bin: Eicar-Test-Signature FOUND\nInfected files: 1\n";
        let findings = parse_clamav_output(output);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].title, "Eicar-Test-Signature");
        assert_eq!(findings[0].file_path.as_deref(), Some("/tmp/quarantine/a.bin"));
    }

    #[test]
    fn clean_output_has_no_findings() {
        assert!(parse_clamav_output("Infected files: 0\n").is_empty());
    }
}
