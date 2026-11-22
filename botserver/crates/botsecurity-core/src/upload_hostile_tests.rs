//! Hostile-file policy tests (#2522).

use super::*;
use std::io::{Cursor, Write};

fn zip_with(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, body) in entries {
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated);
        writer
            .start_file(*name, options)
            .expect("test zip entry must start");
        writer.write_all(body).expect("test zip entry must write");
    }
    writer.finish().expect("test zip must finish").into_inner()
}

fn docx() -> Vec<u8> {
    zip_with(&[
        ("[Content_Types].xml", b"<Types/>".to_vec()),
        ("word/document.xml", b"<w:document/>".to_vec()),
    ])
}

/// Tests that exercise a non-malware guard state the scanner requirement
/// explicitly, so they stay deterministic whatever the process environment
/// says about the fail-closed opt-out.
fn policy_allowing_unscanned() -> HostileFilePolicy {
    HostileFilePolicy {
        require_malware_scanner: false,
        ..HostileFilePolicy::default()
    }
}

/// Flips the ZipCrypto flag in both the local and the central header so the
/// reader recognises the entry as encrypted without needing a writer feature.
fn mark_encrypted(bytes: &mut [u8]) {
    if bytes.starts_with(&[0x50, 0x4b, 0x03, 0x04]) && bytes.len() > 7 {
        bytes[6] |= 0x01;
    }
    if let Some(position) = bytes
        .windows(4)
        .position(|window| window == &[0x50, 0x4b, 0x01, 0x02][..])
    {
        if bytes.len() > position + 9 {
            bytes[position + 8] |= 0x01;
        }
    }
}

#[test]
fn a_zip_bomb_over_the_uncompressed_cap_is_refused() {
    let policy = HostileFilePolicy {
        max_uncompressed_bytes: 1024,
        ..HostileFilePolicy::default()
    };
    let bytes = zip_with(&[("payload.txt", vec![b'a'; 8192])]);

    let error = inspect_hostile_upload_with(&bytes, "payload.zip", &policy)
        .expect_err("a payload over the uncompressed cap must be refused");

    assert!(matches!(error, UploadError::ArchiveUnsafe { .. }));
    assert_eq!(error.code(), "error-file-archive-unsafe");
}

#[test]
fn a_compression_ratio_bomb_is_refused() {
    // 512 KiB of zeroes compresses to roughly 500 bytes: well under the byte
    // cap but a ratio around 1000, so only the ratio guard catches it.
    let policy = HostileFilePolicy {
        max_compression_ratio: 10,
        ..HostileFilePolicy::default()
    };
    let bytes = zip_with(&[("zeros.dat", vec![0u8; 512 * 1024])]);

    let error = inspect_hostile_upload_with(&bytes, "payload.zip", &policy)
        .expect_err("an extreme compression ratio must be refused");

    assert!(matches!(error, UploadError::ArchiveCompressionRatio { .. }));
}

#[test]
fn the_entries_cap_is_enforced() {
    let policy = HostileFilePolicy {
        max_entries: 1,
        ..HostileFilePolicy::default()
    };
    let bytes = zip_with(&[("a.txt", b"a".to_vec()), ("b.txt", b"b".to_vec())]);

    let error = inspect_hostile_upload_with(&bytes, "payload.zip", &policy)
        .expect_err("a second entry must breach the cap");

    assert!(matches!(error, UploadError::ArchiveUnsafe { .. }));
}

#[test]
fn the_compressed_size_cap_is_enforced() {
    let policy = HostileFilePolicy {
        max_compressed_bytes: 16,
        ..HostileFilePolicy::default()
    };
    let bytes = docx();

    let error = inspect_hostile_upload_with(&bytes, "report.docx", &policy)
        .expect_err("a payload over the compressed cap must be refused");

    assert!(matches!(error, UploadError::TooLarge { .. }));
}

#[test]
fn a_macro_part_is_refused_with_a_typed_error() {
    let bytes = zip_with(&[
        ("word/document.xml", b"<w:document/>".to_vec()),
        ("word/vbaProject.bin", vec![0x01, 0x02, 0x03]),
    ]);

    let error = inspect_hostile_upload(&bytes).expect_err("macros must be refused, not stripped");

    assert!(matches!(error, UploadError::ArchiveMacro { .. }));
    assert_eq!(error.code(), "error-file-archive-unsafe");
    let detail = error.log_detail();
    assert!(detail.contains("vbaproject.bin"), "log detail: {detail}");
}

#[test]
fn a_bin_part_outside_word_and_ppt_is_not_a_macro() {
    let bytes = zip_with(&[("media/blob.bin", vec![0x01, 0x02])]);
    assert!(inspect_hostile_upload_with(&bytes, "media.zip", &policy_allowing_unscanned()).is_ok());
}

#[test]
fn an_encrypted_entry_is_refused_before_any_body_read() {
    let mut bytes = docx();
    mark_encrypted(&mut bytes);

    let error = inspect_hostile_upload(&bytes).expect_err("encrypted parts cannot be inspected");

    assert_eq!(error, UploadError::ArchiveEncrypted);
}

#[test]
fn a_nested_zip_container_is_refused() {
    let inner = zip_with(&[("inner.txt", b"inner".to_vec())]);
    let bytes = zip_with(&[("embedded.zip", inner)]);

    let error = inspect_hostile_upload(&bytes).expect_err("a nested bomb stage must be refused");

    assert!(matches!(error, UploadError::ArchiveNested { .. }));
}

#[test]
fn internal_relationship_targets_are_rejected() {
    for target in [
        "http://127.0.0.1/admin",
        "http://localhost:8080/x",
        "http://[::1]/x",
        "http://169.254.169.254/latest/meta-data/",
        "file:///etc/passwd",
        "//internal.example/x",
        "javascript:alert(1)",
    ] {
        assert!(
            validate_relationship_target(target).is_err(),
            "{target} must be rejected"
        );
    }

    validate_relationship_target("https://example.com/document").expect("public https is allowed");
    validate_relationship_target("../media/image1.png")
        .expect("relative package paths are allowed");
}

#[test]
fn a_relationship_part_with_a_loopback_target_is_refused() {
    let rels = br#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Target="http://127.0.0.1/secret" TargetMode="External"/>
</Relationships>"#;
    let bytes = zip_with(&[("word/_rels/document.xml.rels", rels.to_vec())]);

    let error = inspect_hostile_upload(&bytes).expect_err("a loopback target must be refused");

    assert!(matches!(
        error,
        UploadError::RelationshipTargetUnsafe { .. }
    ));
}

#[test]
fn a_safe_relationship_part_is_accepted() {
    let rels = br#"<Relationships><Relationship Id="rId1" Target="https://example.com/a&amp;b" TargetMode="External"/><Relationship Id="rId2" Target="media/image1.png"/></Relationships>"#;
    let bytes = zip_with(&[("_rels/.rels", rels.to_vec())]);

    assert!(inspect_hostile_upload_with(&bytes, "rels.zip", &policy_allowing_unscanned()).is_ok());
}

#[test]
fn the_default_malware_hook_reports_not_configured() {
    assert_eq!(
        scan_for_malware(b"bytes", "report.docx"),
        MalwareScanDecision::NotConfigured
    );
    // With no scanner, a clean package passes only a policy that does not
    // require one; the decision stays NotConfigured, never Clean.
    let report = inspect_hostile_upload_with(&docx(), "report.docx", &policy_allowing_unscanned())
        .expect("a clean docx must pass a policy that does not require a scanner");
    assert_eq!(report.malware_scan, MalwareScanDecision::NotConfigured);
    assert_eq!(report.entry_count, 2);
}

#[test]
fn the_shared_default_is_tied_to_the_opt_out_resolver() {
    // The policy may be permissive only through the documented opt-out; the
    // field must always reflect the resolver rather than a hardcoded value.
    assert_eq!(
        HostileFilePolicy::default().require_malware_scanner,
        malware_scanner_required()
    );
}

#[test]
fn a_fail_closed_policy_refuses_a_clean_package_without_a_scanner() {
    let policy = HostileFilePolicy {
        require_malware_scanner: true,
        ..HostileFilePolicy::default()
    };
    let error = inspect_hostile_upload_with(&docx(), "report.docx", &policy)
        .expect_err("a missing scanner must be refused when the policy requires one");
    assert_eq!(error, UploadError::MalwareScannerMissing);
}

#[test]
fn malware_decisions_drive_typed_errors() {
    let clean = MalwareScanDecision::Clean {
        scanner: "test".to_string(),
    };
    assert!(malware_decision(&clean, false).is_ok());

    let infected = MalwareScanDecision::Infected {
        scanner: "test".to_string(),
        signature: "EICAR".to_string(),
    };
    assert!(matches!(
        malware_decision(&infected, false),
        Err(UploadError::MalwareDetected { .. })
    ));

    let failed = MalwareScanDecision::Failed {
        detail: "timeout".to_string(),
    };
    assert_eq!(
        malware_decision(&failed, false),
        Err(UploadError::MalwareScanFailed)
    );
    assert_eq!(
        malware_decision(&MalwareScanDecision::NotConfigured, true),
        Err(UploadError::MalwareScannerMissing)
    );
}

#[test]
fn validate_upload_enforces_the_hostile_policy_for_zip_containers() {
    let bytes = zip_with(&[("word/vbaProject.bin", vec![0x01])]);

    let error = crate::upload_policy::validate_upload(
        crate::upload_policy::UploadClass::DocumentImport,
        "report.docx",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        &bytes,
    )
    .expect_err("the shared upload gate must enforce the hostile-file policy");

    assert!(matches!(error, UploadError::ArchiveMacro { .. }));
}
