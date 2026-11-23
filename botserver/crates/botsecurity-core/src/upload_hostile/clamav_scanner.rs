//! Real ClamAV adapter for the upload malware hook (#2597).
//!
//! The adapter maps the byte-scan verdict onto [`MalwareScanDecision`] without
//! ever fabricating a clean result: engine errors and oversized payloads stay
//! failures so the gate refuses the upload.

use super::{MalwareScanDecision, MalwareScanner};
use crate::antivirus::byte_scan::{self, ByteScanVerdict};

/// Scanner name reported in the durable scan record.
pub const SCANNER_NAME: &str = "clamav";

/// Adapter matching [`MalwareScanner`]'s callback signature.
pub fn scan_with_clamav(data: &[u8], _filename: &str) -> MalwareScanDecision {
    match byte_scan::scan_bytes(data) {
        ByteScanVerdict::Clean { scanner } => MalwareScanDecision::Clean { scanner },
        ByteScanVerdict::Infected {
            scanner,
            signature,
        } => MalwareScanDecision::Infected { scanner, signature },
        ByteScanVerdict::Failed { detail } => MalwareScanDecision::Failed { detail },
    }
}

/// Installs the ClamAV adapter as the one-shot process-wide scanner.
/// Returns `false` when another scanner was registered first.
pub fn register_default_malware_scanner() -> bool {
    let installed = super::set_malware_scanner(Box::new(scan_with_clamav) as Box<MalwareScanner>);
    if installed {
        log::info!("upload malware scanner registered: {SCANNER_NAME}");
    } else {
        log::warn!("upload malware scanner already registered; {SCANNER_NAME} not installed");
    }
    installed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_payload_is_failed_not_clean() {
        // No scanner process is needed for this case, so the test is independent
        // of a ClamAV installation and of global registration order.
        assert!(matches!(
            scan_with_clamav(&[], "empty.docx"),
            MalwareScanDecision::Failed { .. }
        ));
    }

    /// #2602 — the live ClamAV gate, driven through the *real* upload handler
    /// (`inspect_hostile_upload_with_scanner`), not the in-memory signature
    /// matcher. CI installs ClamAV+signatures and runs this with `--ignored`.
    /// The EICAR corpus case is a stored ZIP, so the raw bytes handed to
    /// `clamscan` really are an upload container. Asserts the typed
    /// `MalwareDetected` refusal, the durable quarantine/audit record, a
    /// `Clean{scanner}` acceptance for a clean upload, and that a broken
    /// scanner is never reported clean.
    #[test]
    #[ignore = "requires an installed ClamAV with signatures; CI job malware-clamav runs it"]
    fn clamav_upload_gate_detects_eicar_and_accepts_clean_file() -> Result<(), String> {
        use std::io::Write;

        use crate::upload_error::UploadError;
        use crate::upload_hostile::corpus::cases;
        use crate::upload_hostile::{
            inspect_hostile_upload_with_scanner, malware_decision, MalwareScanDecision,
        };

        let eicar = cases()
            .into_iter()
            .find(|case| case.name == "malware-eicar")
            .ok_or_else(|| "the corpus lost its malware-eicar case".to_string())?;

        let result = inspect_hostile_upload_with_scanner(
            &eicar.bytes,
            eicar.name,
            &eicar.policy,
            Some(&scan_with_clamav),
        );
        let signature = match result {
            Err(UploadError::MalwareDetected { signature }) => signature,
            other => {
                return Err(format!(
                    "expected a typed MalwareDetected refusal, got {other:?}"
                ))
            }
        };
        if !signature.to_ascii_lowercase().contains("eicar") {
            return Err(format!("unexpected ClamAV signature: {signature}"));
        }
        println!("clamscan upload verdict: MalwareDetected {{ signature: {signature} }}");

        let entry = crate::upload_hostile::malware::list_quarantine_records()
            .map_err(|error| format!("quarantine listing: {error}"))?
            .into_iter()
            .find(|entry| entry.scanner == SCANNER_NAME && entry.verdict == "infected")
            .ok_or_else(|| "no infected ClamAV quarantine/audit record".to_string())?;
        if !entry.quarantine_path.ends_with(".bin") {
            return Err("infected record has no quarantined payload".to_string());
        }
        println!(
            "quarantine audit: id={} signature={:?} payload={}",
            entry.id, entry.threat_name, entry.quarantine_path
        );

        let clean = {
            let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
            let options = zip::write::FileOptions::<()>::default()
                .compression_method(zip::CompressionMethod::Stored);
            writer
                .start_file("word/document.xml", options)
                .map_err(|error| format!("clean zip entry: {error}"))?;
            writer
                .write_all(b"<w:document/>")
                .map_err(|error| format!("clean zip body: {error}"))?;
            writer
                .finish()
                .map_err(|error| format!("clean zip finish: {error}"))?
                .into_inner()
        };
        let report = inspect_hostile_upload_with_scanner(
            &clean,
            "clean.docx",
            &eicar.policy,
            Some(&scan_with_clamav),
        )
        .map_err(|error| format!("a clean upload was refused: {error}"))?;
        let expected = MalwareScanDecision::Clean {
            scanner: SCANNER_NAME.to_string(),
        };
        if report.malware_scan != expected {
            return Err(format!("clean upload verdict: {:?}", report.malware_scan));
        }
        malware_decision(&report.malware_scan, true)
            .map_err(|error| format!("a clean upload must be accepted: {error}"))?;
        println!("clamscan clean verdict: {expected:?}");

        // A failed engine must never collapse into a clean verdict.
        if !matches!(
            scan_with_clamav(&[], "empty.docx"),
            MalwareScanDecision::Failed { .. }
        ) {
            return Err("a failed scanner must never be reported clean".to_string());
        }
        Ok(())
    }
}
