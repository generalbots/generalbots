//! Shared upload policy (#2132).
//!
//! One gate every upload handler calls before it persists anything. It answers
//! the questions the per-handler checks answered inconsistently (or not at
//! all): is the size within this endpoint's budget, is the filename safe, is
//! the extension one this endpoint accepts, and do the declared MIME type and
//! the leading bytes agree with that extension?
//!
//! Rules, in the order [`validate_upload`] applies them:
//!
//! 1. empty payloads are refused;
//! 2. the payload must fit the endpoint class cap ([`UploadClass`]);
//! 3. the filename is reduced to its last path component, stripped of control
//!    characters and of everything that is not alphanumeric, `.`, `-`, `_` or
//!    space, and refused when it starts with `.` or is too long;
//! 4. the final extension must be on the class allowlist — which is why
//!    `a.png.html` is refused (the *last* extension decides) and so is an
//!    unknown extension;
//! 5. the declared `Content-Type` must agree with the extension;
//! 6. the magic bytes must agree with the extension — except that a
//!    password-protected OOXML package (CFB magic under a `.docx`/`.xlsx`/
//!    `.pptx` name) is reported as [`UploadError::ArchiveEncrypted`] before
//!    this check, so the caller gets the actionable refusal instead of a
//!    generic content mismatch (#2522);
//! 7. zip containers are opened and refused when they look like a bomb
//!    (entry count, decompressed size, unsafe entry names);
//! 8. #2522 — zip containers additionally go through the hostile-file gate
//!    ([`crate::upload_hostile`]): compressed/uncompressed/ratio caps,
//!    encrypted, nested and macro-bearing parts, SSRF-checked relationship
//!    targets and an honest malware-scan decision;
//! 9. #2598 — every other payload (Drive binary/transfer/external, PDF, image,
//!    audio/video, text) goes through that same scanner through
//!    [`needs_standalone_malware_scan`], and the typed verdict is returned on
//!    [`ValidatedUpload::malware_scan`]. A missing or failing scanner is a
//!    typed refusal under the default policy, never a silent "clean".
//!
//! # Scanning boundary (#2598)
//!
//! This is a **persisted-upload gate**, not a whole-request-body middleware:
//! it runs in the handlers that buffer an upload within [`UploadLimits`] and
//! then write it to Drive or to an importer. Endpoints and bodies that never
//! reach an upload handler are not scanned here — do not claim protection at
//! a layer that does not scan.
//!
//! Handlers answer with [`rejection_json`] / [`rejection_message`], which
//! translate a stable catalogue code for the request locale and never echo
//! internal detail.

use crate::file_validation::{blocked_extensions, detect_mime_type};
pub use crate::upload_archive::{inspect_archive, MAX_ARCHIVE_ENTRIES, MAX_ARCHIVE_UNCOMPRESSED_BYTES};
pub use crate::upload_hostile::{
    inspect_hostile_upload, inspect_hostile_upload_named, inspect_hostile_upload_with,
    malware_decision, malware_scanner_installed, register_default_malware_scanner,
    scan_for_malware, set_malware_scanner, validate_relationship_target,
    HostileFilePolicy, HostileFileReport, MalwareScanDecision, MalwareScanner,
    DEFAULT_MAX_COMPRESSED_BYTES,
    DEFAULT_MAX_COMPRESSION_RATIO, DEFAULT_MAX_ENTRIES, DEFAULT_MAX_UNCOMPRESSED_BYTES,
};
pub use crate::upload_transport::{body_limit, check_declared_length};

pub use crate::upload_error::{
    locale_from_headers, rejection_json, rejection_message, UploadError,
};
pub use crate::upload_tables::{
    canonical_mime, extension_for_mime, is_text_mime, is_zip_container, mime_for_extension,
};

const DRIVE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff", "pdf", "zip", "gz", "rar", "7z",
    "xz", "bz2", "docx", "xlsx", "pptx", "odt", "ods", "odp", "doc", "xls", "ppt", "csv", "txt",
    "md", "markdown", "json", "xml", "rtf", "mp4", "m4v", "mov", "webm", "mkv", "avi", "mp3",
    "wav", "ogg", "oga", "m4a", "flac", "log", "yaml", "yml", "toml", "ini", "sql", "rs", "ts",
    "css", "py", "bas", "ast", "bin", "dat", "html", "htm", "svg",
];

/// Endpoint class: decides the size cap, filename length and extension
/// allowlist. One variant per upload entry point so the policy cannot drift
/// away from the handler that calls it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadClass {
    /// Suite Drive (`/api/files/write`, `/api/files/upload-binary`) — a general
    /// file store, so the allowlist is broad but still excludes executables.
    /// HTML/SVG are accepted here because the designers save them, and they are
    /// only ever served back as an opaque attachment (see the Drive handlers).
    Drive,
    /// Conversation attachment (composer upload).
    ChatAttachment,
    /// Knowledge-base document ingestion.
    KnowledgeBase,
    /// Rich-text document import (`/api/docs/import`).
    DocumentImport,
    /// Spreadsheet import (`/api/sheet/import`).
    SpreadsheetImport,
    /// Presentation import (`/api/slides/import`).
    PresentationImport,
    /// Project-plan import (`/api/project/import`).
    ProjectImport,
    /// Compliance evidence upload.
    Evidence,
    /// Video project media upload.
    VideoMedia,
    /// Avatar / branding image upload.
    Image,
    /// Calendar import (`/api/calendar/:id/import`).
    CalendarImport,
}

/// Size and extension budget for one [`UploadClass`].
#[derive(Debug, Clone, Copy)]
pub struct UploadLimits {
    pub max_bytes: usize,
    pub max_filename_len: usize,
    pub extensions: &'static [&'static str],
}

impl UploadLimits {
    #[must_use]
    pub fn allows_extension(&self, extension: &str) -> bool {
        self.extensions.contains(&extension)
    }
}

static DRIVE_LIMITS: UploadLimits = UploadLimits {
    max_bytes: 100 * 1024 * 1024,
    max_filename_len: 180,
    extensions: DRIVE_EXTENSIONS,
};

static CHAT_LIMITS: UploadLimits = UploadLimits {
    max_bytes: 25 * 1024 * 1024,
    max_filename_len: 180,
    extensions: &[
        "png", "jpg", "jpeg", "gif", "webp", "pdf", "docx", "xlsx", "pptx", "doc", "xls", "ppt",
        "csv", "txt", "md", "markdown", "json", "zip", "gz", "mp3", "wav", "ogg", "oga", "m4a",
        "mp4", "webm", "mov",
    ],
};

static KB_LIMITS: UploadLimits = UploadLimits {
    max_bytes: 50 * 1024 * 1024,
    max_filename_len: 180,
    extensions: &[
        "pdf", "docx", "doc", "txt", "md", "markdown", "html", "htm", "csv", "xlsx", "xls",
    ],
};

static DOC_IMPORT_LIMITS: UploadLimits = UploadLimits {
    max_bytes: 25 * 1024 * 1024,
    max_filename_len: 180,
    // The document importer explicitly converts HTML to editor content, so it
    // is the one endpoint that accepts `.html`. It accepts `.pdf` too: the PDF
    // is read back into editable paragraphs (#2254).
    extensions: &["docx", "odt", "rtf", "md", "markdown", "html", "htm", "txt", "pdf"],
};

static SHEET_IMPORT_LIMITS: UploadLimits = UploadLimits {
    max_bytes: 25 * 1024 * 1024,
    max_filename_len: 180,
    extensions: &["xlsx", "xls", "ods", "csv", "tsv", "txt"],
};

static SLIDES_IMPORT_LIMITS: UploadLimits = UploadLimits {
    max_bytes: 50 * 1024 * 1024,
    max_filename_len: 180,
    extensions: &["pptx", "md", "markdown", "json"],
};

static PROJECT_IMPORT_LIMITS: UploadLimits = UploadLimits {
    max_bytes: 25 * 1024 * 1024,
    max_filename_len: 180,
    extensions: &["xml", "mpp", "csv", "json"],
};

static EVIDENCE_LIMITS: UploadLimits = UploadLimits {
    max_bytes: 25 * 1024 * 1024,
    max_filename_len: 180,
    extensions: &[
        "pdf", "png", "jpg", "jpeg", "gif", "webp", "docx", "xlsx", "csv", "txt", "md",
    ],
};

static VIDEO_LIMITS: UploadLimits = UploadLimits {
    max_bytes: 512 * 1024 * 1024,
    max_filename_len: 180,
    extensions: &[
        "mp4", "m4v", "mov", "webm", "mkv", "avi", "mp3", "wav", "ogg", "m4a", "flac", "png",
        "jpg", "jpeg", "webp",
    ],
};

static CALENDAR_LIMITS: UploadLimits = UploadLimits {
    max_bytes: 10 * 1024 * 1024,
    max_filename_len: 180,
    extensions: &["ics", "ical"],
};

static IMAGE_LIMITS: UploadLimits = UploadLimits {
    max_bytes: 10 * 1024 * 1024,
    max_filename_len: 120,
    extensions: &["png", "jpg", "jpeg", "gif", "webp"],
};

/// Budget and allowlist for `class`.
#[must_use]
pub fn limits_for(class: UploadClass) -> &'static UploadLimits {
    match class {
        UploadClass::Drive => &DRIVE_LIMITS,
        UploadClass::ChatAttachment => &CHAT_LIMITS,
        UploadClass::KnowledgeBase => &KB_LIMITS,
        UploadClass::DocumentImport => &DOC_IMPORT_LIMITS,
        UploadClass::SpreadsheetImport => &SHEET_IMPORT_LIMITS,
        UploadClass::PresentationImport => &SLIDES_IMPORT_LIMITS,
        UploadClass::ProjectImport => &PROJECT_IMPORT_LIMITS,
        UploadClass::Evidence => &EVIDENCE_LIMITS,
        UploadClass::VideoMedia => &VIDEO_LIMITS,
        UploadClass::Image => &IMAGE_LIMITS,
        UploadClass::CalendarImport => &CALENDAR_LIMITS,
    }
}

/// Size cap in bytes for `class`.
#[must_use]
pub fn max_bytes(class: UploadClass) -> usize {
    limits_for(class).max_bytes
}

/// #2598 — pure coverage predicate for the shared malware gate. `true` means
/// the payload has no structural zip gate and must be scanned directly before
/// it is persisted (Drive binary/transfer/external, PDF, images, media, text).
/// `false` means the structural gate at
/// [`crate::upload_hostile::inspect_hostile_upload_with`] owns the scan.
/// Either route scans; the predicate only decides *where*. It reads no scanner
/// state and performs no I/O, so the "no bypass" contract is testable.
#[must_use]
pub fn needs_standalone_malware_scan(expected_mime: &str) -> bool {
    !is_zip_container(expected_mime)
}

/// An upload that passed every rule. `filename` is already sanitized, and
/// `extension` is the lowercased final extension used to build the stored key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedUpload {
    pub filename: String,
    pub extension: String,
    pub mime: String,
    pub size: usize,
    /// #2598 — typed verdict from the shared malware gate. Returned so callers
    /// can audit the scanner name instead of inferring "clean" from a write.
    pub malware_scan: MalwareScanDecision,
}

/// Reduces `raw` to a safe stored filename: last path component, no control
/// characters, no quotes or path separators, no leading dot, length-capped.
pub fn sanitize_upload_filename(raw: &str, max_len: usize) -> Result<String, UploadError> {
    let base = raw.rsplit(['/', '\\']).next().unwrap_or_default();
    let cleaned: String = base
        .chars()
        .filter(|c| !c.is_control() && !matches!(c, '\0' | '"' | '\'' | '<' | '>' | '|' | '?' | '*'))
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '.' | '-' | '_' | ' ') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim();

    // A leading dot is a hidden file, not an escape: Drive stores
    // `.gbdesktop.json` and friends. A name made only of dots is refused.
    if trimmed.is_empty() || trimmed.chars().all(|c| c == '.') {
        return Err(UploadError::FilenameInvalid);
    }
    if trimmed.len() > max_len {
        return Err(UploadError::FilenameTooLong { max: max_len });
    }

    Ok(trimmed.to_string())
}

/// Applies the full policy for `class` to an upload.
/// Backwards-compatible wrapper using the default hostile-file policy.
pub fn validate_upload(
    class: UploadClass,
    filename: &str,
    content_type: &str,
    data: &[u8],
) -> Result<ValidatedUpload, UploadError> {
    validate_upload_with_policy(
        class,
        filename,
        content_type,
        data,
        &crate::upload_hostile::HostileFilePolicy::default(),
    )
}

pub fn validate_upload_with_policy(
    class: UploadClass,
    filename: &str,
    content_type: &str,
    data: &[u8],
    policy: &crate::upload_hostile::HostileFilePolicy,
) -> Result<ValidatedUpload, UploadError> {
    let limits = limits_for(class);

    // #2606 — the emergency rollback is allowed to stay, but every skipped
    // scan produces a durable, queryable audit record.
    if !policy.require_malware_scanner {
        if let Err(error) = crate::upload_hostile::malware::record_opt_out(filename) {
            log::error!("could not record malware opt-out: {error}");
        }
    }

    if data.is_empty() {
        return Err(UploadError::Empty);
    }
    if data.len() > limits.max_bytes {
        return Err(UploadError::TooLarge { max: limits.max_bytes });
    }

    let filename = sanitize_upload_filename(filename, limits.max_filename_len)?;
    let extension = final_extension(&filename).to_lowercase();

    if blocked_extensions().contains(&extension.as_str()) {
        return Err(UploadError::ExtensionBlocked { extension });
    }

    // The Drive is a file manager: an extensionless text artifact (`LICENSE`,
    // `Makefile`, `Dockerfile`) is normal there. Every other endpoint needs a
    // known extension, and an extensionless binary still fails the magic check
    // below because `text/plain` is expected.
    let expected = if extension.is_empty() {
        if class != UploadClass::Drive {
            return Err(UploadError::UnknownExtension { extension });
        }
        "text/plain"
    } else {
        mime_for_extension(&extension)
            .ok_or_else(|| UploadError::UnknownExtension { extension: extension.clone() })?
    };

    // An extensionless Drive file is allowed (see above) and has nothing to
    // check against the allowlist.
    if !extension.is_empty() && !limits.allows_extension(&extension) {
        return Err(UploadError::ExtensionNotAllowed {
            extension,
            allowed: limits.extensions,
        });
    }

    // #2522 — a password-protected DOCX/XLSX/PPTX is an OLE/CFB compound file,
    // not a zip, so the magic-byte check below would otherwise reject it with
    // the generic `error-file-content-mismatch`. Detect it before that check
    // and surface the typed `ArchiveEncrypted` error, which import handlers map
    // to the actionable `action: "remove-password"` refusal. The helper only
    // matches CFB magic *and* a zip-based Office extension, so a genuinely
    // mislabeled binary is never mistaken for an encrypted container here.
    if crate::upload_hostile::password::is_password_protected_container(data, &filename) {
        return Err(UploadError::ArchiveEncrypted);
    }

    let declared_raw = normalize_mime(content_type);
    let declared = canonical_mime(&declared_raw).to_string();
    if !declared_matches(&declared, expected) {
        return Err(UploadError::TypeMismatch {
            expected: expected.to_string(),
            detected: declared,
        });
    }

    match detect_mime_type(data) {
        Some(detected) => {
            if !magic_matches(&extension, expected, &detected, data) {
                return Err(UploadError::TypeMismatch {
                    expected: expected.to_string(),
                    detected,
                });
            }
        }
        None => {
            // A binary extension whose signature we cannot recognise is only
            // acceptable when the format genuinely has no magic bytes.
            if !UNVERIFIABLE_EXTENSIONS.contains(&extension.as_str()) {
                return Err(UploadError::TypeMismatch {
                    expected: expected.to_string(),
                    detected: "unknown".to_string(),
                });
            }
        }
    }

    // #2598 — the malware gate covers every payload, container or not.
    // Containers take the structural pass (which already scans); everything
    // else takes the standalone pass. No `malware_scanner_installed()`
    // shortcut: a missing scanner yields NotConfigured and a scanner error
    // yields Failed, and `malware_decision` turns both into typed refusals
    // under the default policy. Both passes are bounded by `limits.max_bytes`.
    let malware_scan = match needs_standalone_malware_scan(expected) {
        false => {
            let report =
                crate::upload_hostile::inspect_hostile_upload_with(data, &filename, policy)?;
            report.malware_scan
        }
        true => {
            let decision = scan_for_malware(data, &filename);
            malware_decision(&decision, policy.require_malware_scanner)?;
            decision
        }
    };

    Ok(ValidatedUpload {
        filename,
        extension,
        mime: expected.to_string(),
        size: data.len(),
        malware_scan,
    })
}

/// Formats without a reliable leading signature; their extension is the only
/// type signal available.
const UNVERIFIABLE_EXTENSIONS: &[&str] = &["bin", "dat", "mpp"];

#[path = "upload_policy_helpers.rs"]
mod helpers;
use helpers::*;

#[cfg(test)]
#[path = "upload_policy_malware_tests.rs"]
mod upload_policy_malware_tests;

#[cfg(test)]
mod malware_coverage_tests {
    use super::*;

    #[test]
    fn non_container_payloads_need_the_standalone_scan() {
        assert!(!needs_standalone_malware_scan("application/zip"));
        assert!(needs_standalone_malware_scan("application/pdf"));
        assert!(needs_standalone_malware_scan("application/msword"));
    }
}

