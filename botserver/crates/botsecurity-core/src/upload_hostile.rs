//! Hostile-file policy for zip-based Office imports (#2522).
//!
//! [`crate::upload_policy::validate_upload`] checks the outside of an upload
//! (name, declared type, magic bytes). This module checks what is *inside* a
//! zip container, where a document importer is the victim:
//!
//! * compressed size, entry count, uncompressed size and compression ratio —
//!   the four independent zip-bomb signals;
//! * encrypted entries (a password-protected part cannot be inspected);
//! * nested zip containers (a second bomb stage);
//! * macro-bearing parts — **refused**, never stripped, because rewriting a
//!   signed OOXML package silently invalidates it;
//! * relationship targets — absolute URLs are re-checked with the same SSRF
//!   policy used for outbound fetches, so `file://` and loopback/link-local
//!   targets are rejected;
//! * a malware-scan hook whose default decision is `NotConfigured`, never a
//!   fabricated `Clean`.
//!
//! Nothing here writes to disk or extracts data: entry bodies are only read in
//! bounded slices (4 bytes for the nested-container magic, 1 MiB for `.rels`).

use std::io::{Cursor, Read};
use std::sync::OnceLock;

use crate::upload_error::UploadError;
use crate::validation::validate_url_ssrf_strict;

/// Compressed payload cap for the standalone gate (matches the largest
/// `UploadClass` budget; handlers may tighten it through their own limits).
pub const DEFAULT_MAX_COMPRESSED_BYTES: usize = 100 * 1024 * 1024;
pub const DEFAULT_MAX_UNCOMPRESSED_BYTES: u64 =
    crate::upload_archive::MAX_ARCHIVE_UNCOMPRESSED_BYTES;
pub const DEFAULT_MAX_ENTRIES: usize = crate::upload_archive::MAX_ARCHIVE_ENTRIES;
/// `uncompressed / compressed`. A normal Office package is well below 100;
/// highly compressible filler used in bombs is thousands of times larger.
pub const DEFAULT_MAX_COMPRESSION_RATIO: u64 = 100;

const MAX_RELATIONSHIPS_BYTES: u64 = 1024 * 1024;

/// Limits applied by [`inspect_hostile_upload_with`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostileFilePolicy {
    pub max_compressed_bytes: usize,
    pub max_uncompressed_bytes: u64,
    pub max_entries: usize,
    pub max_compression_ratio: u64,
    /// When true, a missing scanner is a typed error instead of a recorded
    /// `NotConfigured` decision. Off by default so deployments without a
    /// scanner keep working while the result stays honest.
    pub require_malware_scanner: bool,
}

impl Default for HostileFilePolicy {
    fn default() -> Self {
        Self {
            max_compressed_bytes: DEFAULT_MAX_COMPRESSED_BYTES,
            max_uncompressed_bytes: DEFAULT_MAX_UNCOMPRESSED_BYTES,
            max_entries: DEFAULT_MAX_ENTRIES,
            max_compression_ratio: DEFAULT_MAX_COMPRESSION_RATIO,
            require_malware_scanner: false,
        }
    }
}

/// Outcome of the malware hook. `Clean` is only ever produced by a registered
/// scanner; the default is `NotConfigured`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MalwareScanDecision {
    NotConfigured,
    Clean { scanner: String },
    Infected { scanner: String, signature: String },
    Failed { detail: String },
}

/// What the gate observed. Callers should persist/audit this alongside the
/// import so "scanned clean" is never inferred from "no error was returned".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostileFileReport {
    pub entry_count: usize,
    pub compressed_bytes: usize,
    pub uncompressed_bytes: u64,
    pub compression_ratio: u64,
    pub malware_scan: MalwareScanDecision,
}

/// Signature a caller implements to plug a real scanner (ClamAV, Defender, …)
/// into the gate. The scanner receives the in-memory payload and upload name.
pub type MalwareScanner = dyn Fn(&[u8], &str) -> MalwareScanDecision + Send + Sync;
static MALWARE_SCANNER: OnceLock<Box<MalwareScanner>> = OnceLock::new();

/// Installs the process-wide scanner hook. Returns `false` when one is already
/// installed (registration is intentionally one-shot, like the other globals
/// in this crate).
pub fn set_malware_scanner(scanner: Box<MalwareScanner>) -> bool {
    MALWARE_SCANNER.set(scanner).is_ok()
}

/// Runs the configured scanner, or reports `NotConfigured` when none exists.
pub fn scan_for_malware(data: &[u8], filename: &str) -> MalwareScanDecision {
    match MALWARE_SCANNER.get() {
        Some(scanner) => scanner(data, filename),
        None => MalwareScanDecision::NotConfigured,
    }
}

/// Turns a scanner decision into a typed rejection. Public so callers that
/// install their own scanner can apply the same fail-closed semantics.
pub fn malware_decision(
    decision: &MalwareScanDecision,
    require_scanner: bool,
) -> Result<(), UploadError> {
    match decision {
        MalwareScanDecision::NotConfigured if require_scanner => {
            Err(UploadError::MalwareScannerMissing)
        }
        MalwareScanDecision::Infected { signature, .. } => Err(UploadError::MalwareDetected {
            signature: signature.clone(),
        }),
        MalwareScanDecision::Failed { .. } => Err(UploadError::MalwareScanFailed),
        _ => Ok(()),
    }
}

/// Inspects a zip container with the default policy.
pub fn inspect_hostile_upload(data: &[u8]) -> Result<HostileFileReport, UploadError> {
    inspect_hostile_upload_named(data, "")
}

/// As [`inspect_hostile_upload`], with the upload name passed to the scanner.
pub fn inspect_hostile_upload_named(
    data: &[u8],
    filename: &str,
) -> Result<HostileFileReport, UploadError> {
    inspect_hostile_upload_with(data, filename, &HostileFilePolicy::default())
}

/// Full hostile-file gate. `data` must be a zip container (DOCX, XLSX, PPTX,
/// ODF or plain ZIP); non-zip formats have no OOXML parts to inspect.
pub fn inspect_hostile_upload_with(
    data: &[u8],
    filename: &str,
    policy: &HostileFilePolicy,
) -> Result<HostileFileReport, UploadError> {
    if data.is_empty() {
        return Err(UploadError::Empty);
    }
    if data.len() > policy.max_compressed_bytes {
        return Err(UploadError::TooLarge {
            max: policy.max_compressed_bytes,
        });
    }

    let mut archive = zip::ZipArchive::new(Cursor::new(data)).map_err(|_| {
        UploadError::ArchiveUnsafe {
            reason: "invalid zip container",
        }
    })?;
    if archive.len() > policy.max_entries {
        return Err(UploadError::ArchiveUnsafe {
            reason: "too many entries",
        });
    }

    // Pass 1: metadata only. No entry body is decompressed, so a bomb is
    // refused before it can cost anything to unpack.
    let mut uncompressed = 0u64;
    let mut compressed = 0u64;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|_| UploadError::ArchiveUnsafe {
                reason: "unreadable entry",
            })?;
        let name = entry
            .name()
            .map_err(|_| UploadError::ArchiveUnsafe {
                reason: "unsafe entry name",
            })?
            .to_ascii_lowercase();
        if entry.encrypted() {
            return Err(UploadError::ArchiveEncrypted);
        }
        if is_macro_part(&name) {
            return Err(UploadError::ArchiveMacro { part: name });
        }
        if entry.enclosed_name().is_none() {
            return Err(UploadError::ArchiveUnsafe {
                reason: "unsafe entry name",
            });
        }
        compressed = compressed.saturating_add(entry.compressed_size());
        uncompressed = uncompressed.saturating_add(entry.size());
        if uncompressed > policy.max_uncompressed_bytes {
            return Err(UploadError::ArchiveUnsafe {
                reason: "decompressed size",
            });
        }
    }

    let compression_ratio = if compressed == 0 {
        0
    } else {
        uncompressed / compressed
    };
    if compression_ratio > policy.max_compression_ratio {
        return Err(UploadError::ArchiveCompressionRatio {
            ratio: compression_ratio,
            max: policy.max_compression_ratio,
        });
    }

    // Pass 2: bounded body reads only after the size gates have passed.
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|_| UploadError::ArchiveUnsafe {
                reason: "unreadable entry",
            })?;
        let name = entry
            .name()
            .map_err(|_| UploadError::ArchiveUnsafe {
                reason: "unsafe entry name",
            })?
            .to_ascii_lowercase();
        if is_container_part(&name) && starts_with_zip_magic(&mut entry) {
            return Err(UploadError::ArchiveNested { part: name });
        }
        if name.ends_with(".rels") {
            let mut body = Vec::new();
            let _ = entry.take(MAX_RELATIONSHIPS_BYTES).read_to_end(&mut body);
            scan_relationship_xml(&body)?;
        }
    }

    let malware_scan = scan_for_malware(data, filename);
    malware_decision(&malware_scan, policy.require_malware_scanner)?;

    Ok(HostileFileReport {
        entry_count: archive.len(),
        compressed_bytes: data.len(),
        uncompressed_bytes: uncompressed,
        compression_ratio,
        malware_scan,
    })
}

/// Validates one OOXML/ODF relationship target. Relative package references
/// (e.g. `../media/image1.png`) stay valid; anything that looks like an
/// absolute URL goes through the outbound-request SSRF policy.
pub fn validate_relationship_target(target: &str) -> Result<(), UploadError> {
    let value = target.trim();
    if value.is_empty() {
        return Ok(());
    }
    if value.starts_with("//") || has_uri_scheme(value) {
        validate_url_ssrf_strict(value).map_err(|error| UploadError::RelationshipTargetUnsafe {
            kind: error.kind().as_str(),
        })?;
    }
    Ok(())
}

fn scan_relationship_xml(body: &[u8]) -> Result<(), UploadError> {
    let text = String::from_utf8_lossy(body);
    for target in attribute_values(&text, "Target") {
        validate_relationship_target(&unescape_xml(&target))?;
    }
    Ok(())
}

fn is_macro_part(name: &str) -> bool {
    let path = name.trim_start_matches("./").to_ascii_lowercase();
    let base = path.rsplit('/').next().unwrap_or(path.as_str());
    base == "vbaproject.bin"
        || ((path.starts_with("word/") || path.starts_with("ppt/")) && base.ends_with(".bin"))
}

fn is_container_part(name: &str) -> bool {
    [
        ".zip", ".docx", ".xlsx", ".pptx", ".odt", ".ods", ".odp", ".jar", ".7z", ".rar", ".gz",
    ]
    .iter()
    .any(|extension| name.ends_with(extension))
}

fn starts_with_zip_magic(entry: &mut zip::read::ZipFile<'_, impl std::io::Read>) -> bool {
    let mut head = Vec::new();
    match entry.take(4).read_to_end(&mut head) {
        Ok(4) => head == [0x50, 0x4b, 0x03, 0x04],
        _ => false,
    }
}

fn has_uri_scheme(value: &str) -> bool {
    let cut = value.find(['/', '?', '#']).unwrap_or(value.len());
    let head = &value[..cut];
    match head.find(':') {
        Some(position) => {
            position > 0
                && head.as_bytes()[0].is_ascii_alphabetic()
                && head[..position]
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
        }
        None => false,
    }
}

fn attribute_values(text: &str, attribute: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let needle = attribute.as_bytes();
    let mut values = Vec::new();
    let mut position = 0usize;
    while position + needle.len() <= bytes.len() {
        if &bytes[position..position + needle.len()] == needle {
            let mut cursor = position + needle.len();
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if cursor < bytes.len() && bytes[cursor] == b'=' {
                cursor += 1;
                while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                    cursor += 1;
                }
                if cursor < bytes.len() && matches!(bytes[cursor], b'"' | b'\'') {
                    let quote = bytes[cursor];
                    cursor += 1;
                    let value_start = cursor;
                    while cursor < bytes.len() && bytes[cursor] != quote {
                        cursor += 1;
                    }
                    if let Ok(value) = std::str::from_utf8(&bytes[value_start..cursor.min(bytes.len())]) {
                        if !value.is_empty() {
                            values.push(value.to_string());
                        }
                    }
                    position = cursor.saturating_add(1);
                    continue;
                }
            }
        }
        position += 1;
    }
    values
}

fn unescape_xml(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#47;", "/")
}

#[cfg(test)]
#[path = "upload_hostile_tests.rs"]
mod upload_hostile_tests;
