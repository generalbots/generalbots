//! Real ClamAV scan behind the Protection API (#2599, #2601).
//!
//! Every scan goes through the single shared byte primitive in
//! `botsecurity-core` (`antivirus::byte_scan::scan_bytes`). The old dispatch
//! sent `ProtectionTool::ClamAV` to `lynis::run_scan()`, which produced a Lynis
//! audit under a ClamAV label; this module reports only real engine verdicts.

use anyhow::Result;
use tracing::info;

use botsecurity_core::antivirus::byte_scan::{self, ByteScanVerdict};
use botsecurity_core::upload_hostile::malware;

use super::manager::{Finding, FindingSeverity, ScanResultStatus};

/// Optional single-file scan target. When unset, every payload in the canonical
/// hostile-file quarantine is re-scanned, so the endpoint never claims to have
/// scanned the whole host.
const TARGET_ENV: &str = "GB_CLAMAV_SCAN_TARGET";

/// Run a real ClamAV scan through the shared primitive and return findings in
/// the Protection API format.
pub async fn run_scan() -> Result<(ScanResultStatus, Vec<Finding>, String)> {
    let single_target = std::env::var(TARGET_ENV).ok().filter(|value| !value.trim().is_empty());
    let mut findings = Vec::new();
    let mut scanned = 0usize;
    let mut raw_output = String::new();

    let targets: Vec<(String, std::path::PathBuf)> = match single_target {
        Some(value) => {
            let path = std::path::PathBuf::from(value.trim());
            vec![(path.to_string_lossy().into_owned(), path)]
        }
        None => malware::list_quarantine_records()
            .map_err(|e| anyhow::anyhow!("listing hostile-file quarantine: {e}"))?
            .into_iter()
            .filter(|record| !record.quarantine_path.is_empty())
            .map(|record| (record.original_path, std::path::PathBuf::from(record.quarantine_path)))
            .collect(),
    };

    info!("Running ClamAV scan over {} quarantined payload(s)", targets.len());
    for (name, path) in targets {
        let data = match std::fs::read(&path) {
            Ok(data) => data,
            Err(error) => {
                log::warn!("ClamAV target unreadable {}: {error}", path.display());
                continue;
            }
        };
        scanned += 1;
        let signature = match verdict_outcome(byte_scan::scan_bytes(&data)) {
            Ok(Some(signature)) => signature,
            Ok(None) => {
                raw_output.push_str(&format!("{name}: clean\n"));
                continue;
            }
            Err(detail) => {
                // Fail closed: an unavailable or broken engine must never be
                // reported as a clean scan (#2599).
                return Err(anyhow::anyhow!("ClamAV scan failed for {name}: {detail}"));
            }
        };
        raw_output.push_str(&format!("{name}: {signature} FOUND\n"));
        findings.push(Finding {
            id: format!("clamav-{}-{}", scanned, findings.len()),
            severity: FindingSeverity::High,
            category: "malware".to_string(),
            title: signature.clone(),
            description: format!("ClamAV detected {signature}"),
            file_path: Some(path.to_string_lossy().into_owned()),
            remediation: Some("Keep the payload quarantined and update ClamAV signatures".to_string()),
        });
    }

    let status = if findings.is_empty() {
        ScanResultStatus::Clean
    } else {
        ScanResultStatus::Infected
    };
    Ok((status, findings, raw_output))
}

/// Map a typed engine verdict to an optional signature. `Failed` becomes an
/// `Err` so no scanner error is ever collapsed into a clean result (#2599).
fn verdict_outcome(verdict: ByteScanVerdict) -> Result<Option<String>> {
    match verdict {
        ByteScanVerdict::Clean { .. } => Ok(None),
        ByteScanVerdict::Infected { signature, .. } => Ok(Some(signature)),
        ByteScanVerdict::Failed { detail } => Err(anyhow::anyhow!("{detail}")),
    }
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
            remediation: Some("Keep the payload quarantined and update ClamAV signatures".to_string()),
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

    #[test]
    fn scanner_failure_never_becomes_a_clean_verdict() {
        let failed = ByteScanVerdict::Failed {
            detail: "clamscan could not be executed".to_string(),
        };
        assert!(verdict_outcome(failed).is_err());
        assert!(matches!(
            verdict_outcome(ByteScanVerdict::Clean {
                scanner: "clamav".to_string(),
            }),
            Ok(None)
        ));
        assert!(matches!(
            verdict_outcome(ByteScanVerdict::Infected {
                scanner: "clamav".to_string(),
                signature: "Eicar-Test-Signature".to_string(),
            }),
            Ok(Some(signature)) if signature == "Eicar-Test-Signature"
        ));
    }
}
