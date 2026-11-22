//! #2522 — deterministic malicious-file corpus.
//!
//! Every case is built in memory from bounded slices; no fixture file, network
//! call or external scanner is needed. The corpus is shared with the Docs and
//! Sheets import-boundary tests and is run on every PR by `.github/workflows/ci.yml`.

use std::io::Cursor;

use super::{HostileFilePolicy, UploadError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedRefusal {
    ArchiveUnsafe,
    CompressionRatio,
    Encrypted,
    Macro,
    Relationship,
    Nested,
    Malware,
    ScannerMissing,
}

impl ExpectedRefusal {
    pub fn matches(self, error: &UploadError) -> bool {
        match self {
            Self::ArchiveUnsafe => matches!(error, UploadError::ArchiveUnsafe { .. }),
            Self::CompressionRatio => matches!(error, UploadError::ArchiveCompressionRatio { .. }),
            Self::Encrypted => matches!(error, UploadError::ArchiveEncrypted),
            Self::Macro => matches!(error, UploadError::ArchiveMacro { .. }),
            Self::Relationship => matches!(error, UploadError::RelationshipTargetUnsafe { .. }),
            Self::Nested => matches!(error, UploadError::ArchiveNested { .. }),
            Self::Malware => matches!(error, UploadError::MalwareDetected { .. }),
            Self::ScannerMissing => matches!(error, UploadError::MalwareScannerMissing),
        }
    }
}

#[derive(Debug)]
pub struct MaliciousCase {
    pub name: &'static str,
    pub bytes: Vec<u8>,
    pub policy: HostileFilePolicy,
    pub expected: ExpectedRefusal,
    /// Run through the built-in signature hook instead of the global scanner.
    pub use_scanner: bool,
}

fn zip_with(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, body) in entries {
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated);
        if writer.start_file(*name, options).is_err() {
            log::warn!("corpus zip entry could not be created: {name}");
            return Vec::new();
        }
        if std::io::Write::write_all(&mut writer, body).is_err() {
            log::warn!("corpus zip body could not be written: {name}");
            return Vec::new();
        }
    }
    match writer.finish() {
        Ok(writer) => writer.into_inner(),
        Err(error) => {
            log::warn!("corpus zip could not be finished: {error}");
            Vec::new()
        }
    }
}

/// The malware hook scans the upload bytes, so the EICAR marker must appear in
/// plaintext: a deflated entry would hide it from a byte-level scanner.
fn zip_with_stored(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, body) in entries {
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Stored);
        if writer.start_file(*name, options).is_err() {
            log::warn!("corpus stored zip entry could not be created: {name}");
            return Vec::new();
        }
        if std::io::Write::write_all(&mut writer, body).is_err() {
            log::warn!("corpus stored zip body could not be written: {name}");
            return Vec::new();
        }
    }
    match writer.finish() {
        Ok(writer) => writer.into_inner(),
        Err(error) => {
            log::warn!("corpus stored zip could not be finished: {error}");
            Vec::new()
        }
    }
}

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

/// The corpus itself: each entry has an independent, tightened policy so the
/// specific guard — not an unrelated size cap — must reject it.
pub fn cases() -> Vec<MaliciousCase> {
    let bom = zip_with(&[("word/document.xml", vec![b'a'; 8192])]);
    let ratio = zip_with(&[("payload.bin", vec![0u8; 512 * 1024])]);
    let mut encrypted = zip_with(&[("word/document.xml", b"<w:document/>".to_vec())]);
    mark_encrypted(&mut encrypted);
    let macro_doc = zip_with(&[
        ("word/document.xml", b"<w:document/>".to_vec()),
        ("word/vbaProject.bin", vec![0x01, 0x02, 0x03]),
    ]);
    let xlm = zip_with(&[
        ("xl/workbook.xml", b"<workbook/>".to_vec()),
        ("xl/macrosheets/sheet1.xml", b"<macrosheet/>".to_vec()),
    ]);
    let ssrf = zip_with(&[(
        "word/_rels/document.xml.rels",
        b"<Relationships><Relationship Target=\"http://127.0.0.1:8080/admin\"/></Relationships>"
            .to_vec(),
    )]);
    let xxe = zip_with(&[(
        "word/document.xml",
        b"<!DOCTYPE d [<!ENTITY xxe SYSTEM \"file:///etc/passwd\">]><d>&xxe;</d>".to_vec(),
    )]);
    let nested_inner = zip_with(&[("inner.txt", b"inner".to_vec())]);
    let nested = zip_with(&[("embedded.zip", nested_inner)]);
    let malware = zip_with_stored(&[(
        "word/document.xml",
        b"X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*".to_vec(),
    )]);

    vec![
        MaliciousCase {
            name: "zip-bomb-uncompressed",
            bytes: bom,
            policy: HostileFilePolicy {
                max_uncompressed_bytes: 1024,
                ..HostileFilePolicy::default()
            },
            expected: ExpectedRefusal::ArchiveUnsafe,
            use_scanner: false,
        },
        MaliciousCase {
            name: "zip-bomb-ratio",
            bytes: ratio,
            policy: HostileFilePolicy {
                max_compression_ratio: 10,
                ..HostileFilePolicy::default()
            },
            expected: ExpectedRefusal::CompressionRatio,
            use_scanner: false,
        },
        MaliciousCase {
            name: "encrypted-entry",
            bytes: encrypted,
            policy: HostileFilePolicy::default(),
            expected: ExpectedRefusal::Encrypted,
            use_scanner: false,
        },
        MaliciousCase {
            name: "vba-macro",
            bytes: macro_doc,
            policy: HostileFilePolicy::default(),
            expected: ExpectedRefusal::Macro,
            use_scanner: false,
        },
        MaliciousCase {
            name: "xlm-macro-sheet",
            bytes: xlm,
            policy: HostileFilePolicy::default(),
            expected: ExpectedRefusal::Macro,
            use_scanner: false,
        },
        MaliciousCase {
            name: "ssrf-relationship",
            bytes: ssrf,
            policy: HostileFilePolicy::default(),
            expected: ExpectedRefusal::Relationship,
            use_scanner: false,
        },
        MaliciousCase {
            name: "xml-entity-bomb",
            bytes: xxe,
            policy: HostileFilePolicy::default(),
            expected: ExpectedRefusal::ArchiveUnsafe,
            use_scanner: false,
        },
        MaliciousCase {
            name: "nested-container",
            bytes: nested,
            policy: HostileFilePolicy::default(),
            expected: ExpectedRefusal::Nested,
            use_scanner: false,
        },
        MaliciousCase {
            name: "malware-eicar",
            bytes: malware.clone(),
            policy: HostileFilePolicy {
                require_malware_scanner: true,
                ..HostileFilePolicy::default()
            },
            expected: ExpectedRefusal::Malware,
            use_scanner: true,
        },
        MaliciousCase {
            name: "malware-scanner-missing",
            bytes: malware,
            policy: HostileFilePolicy {
                require_malware_scanner: true,
                ..HostileFilePolicy::default()
            },
            expected: ExpectedRefusal::ScannerMissing,
            use_scanner: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upload_hostile::{
        inspect_hostile_upload_with, inspect_hostile_upload_with_scanner, MalwareScanner,
    };

    #[test]
    fn hostile_corpus_every_case_fails_closed() {
        let scanner: &MalwareScanner = &super::super::malware::signature_scanner;
        for case in cases() {
            let result = if case.use_scanner {
                inspect_hostile_upload_with_scanner(
                    &case.bytes,
                    case.name,
                    &case.policy,
                    Some(scanner),
                )
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
}
