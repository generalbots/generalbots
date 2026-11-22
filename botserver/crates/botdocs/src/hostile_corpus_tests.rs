//! #2522 — CI entry point for the malicious-file corpus.
//!
//! `botsecurity-core` owns the corpus and the gate; the Docs crate owns the
//! executable test target because its `lib` tests compile in CI. Every case
//! must fail closed, and the malware case must leave a quarantined payload
//! plus a persisted scan record.

use botsecurity_core::upload_hostile::corpus::{cases, ExpectedRefusal};
use botsecurity_core::upload_hostile::malware::{
    record_scan, scan_with, signature_scanner, QuarantineRecord,
};
use botsecurity_core::upload_hostile::{
    inspect_hostile_upload_with, inspect_hostile_upload_with_scanner, MalwareScanner,
};

#[test]
fn hostile_corpus_every_case_fails_closed() {
    let scanner: &MalwareScanner = &signature_scanner;
    for case in cases() {
        let result = if case.use_scanner {
            inspect_hostile_upload_with_scanner(&case.bytes, case.name, &case.policy, Some(scanner))
        } else {
            inspect_hostile_upload_with(&case.bytes, case.name, &case.policy)
        };
        assert!(result.is_err(), "corpus case {} must be refused", case.name);
        if let Err(error) = result {
            assert!(
                case.expected.matches(&error),
                "corpus case {}: expected {:?}, got {}",
                case.name,
                case.expected,
                error.log_detail()
            );
        }
    }
}

#[test]
fn hostile_corpus_malware_is_quarantined_and_recorded() -> Result<(), String> {
    let sample = b"prefix EICAR-STANDARD-ANTIVIRUS-TEST-FILE suffix";
    let decision = scan_with(&signature_scanner, sample, "eicar.docx");
    if !matches!(
        decision,
        botsecurity_core::upload_hostile::MalwareScanDecision::Infected { .. }
    ) {
        return Err("the EICAR sample must be refused as infected".to_string());
    }
    let record = record_scan(sample, "eicar.docx", &decision)
        .map_err(|error| format!("scan record: {error}"))?;
    if record.verdict != "infected" {
        return Err(format!("unexpected verdict: {}", record.verdict));
    }
    let payload = std::fs::read(&record.payload_path)
        .map_err(|error| format!("quarantined payload: {error}"))?;
    if payload != sample {
        return Err("quarantined payload differs from the sample".to_string());
    }
    let text = std::fs::read_to_string(&record.record_path)
        .map_err(|error| format!("scan record file: {error}"))?;
    let stored: QuarantineRecord =
        serde_json::from_str(&text).map_err(|error| format!("scan record JSON: {error}"))?;
    if stored.scan_id != record.scan_id || stored.sha256 != record.sha256 {
        return Err("stored scan record does not match the returned one".to_string());
    }
    Ok(())
}

#[test]
fn hostile_corpus_refusal_kinds_are_distinct() {
    assert!(ExpectedRefusal::ScannerMissing
        .matches(&botsecurity_core::upload_error::UploadError::MalwareScannerMissing));
    assert!(!ExpectedRefusal::Malware
        .matches(&botsecurity_core::upload_error::UploadError::MalwareScannerMissing));
}
