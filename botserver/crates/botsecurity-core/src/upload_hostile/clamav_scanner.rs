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
}
