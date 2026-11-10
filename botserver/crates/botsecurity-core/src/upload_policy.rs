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
//! 6. the magic bytes must agree with the extension;
//! 7. zip containers are opened and refused when they look like a bomb
//!    (entry count, decompressed size, unsafe entry names).
//!
//! Handlers answer with [`rejection_json`] / [`rejection_message`], which
//! translate a stable catalogue code for the request locale and never echo
//! internal detail.

use std::io::Cursor;

use axum::http::{header, HeaderMap, StatusCode};
use axum::Json;
use botlib::i18n::{self, Locale};
use serde_json::{json, Value};

use crate::file_validation::{blocked_extensions, detect_mime_type};

/// Maximum entries a zip container may hold (zip-bomb guard).
pub const MAX_ARCHIVE_ENTRIES: usize = 4096;
/// Maximum total *uncompressed* size a zip container may declare.
pub const MAX_ARCHIVE_UNCOMPRESSED_BYTES: u64 = 1024 * 1024 * 1024;

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff"];
const ARCHIVE_EXTENSIONS: &[&str] = &["zip", "gz", "rar", "7z", "xz", "bz2"];
const OFFICE_EXTENSIONS: &[&str] = &[
    "docx", "xlsx", "pptx", "odt", "ods", "odp", "doc", "xls", "ppt",
];
const MEDIA_EXTENSIONS: &[&str] = &[
    "mp4", "m4v", "mov", "webm", "mkv", "avi", "mp3", "wav", "ogg", "oga", "m4a", "flac",
];
const TEXT_EXTENSIONS: &[&str] = &[
    "csv", "txt", "md", "markdown", "json", "xml", "rtf", "log", "yaml", "yml", "toml", "ini",
    "sql", "rs", "ts", "css", "py", "bas", "ast",
];

/// Endpoint class: decides the size cap, filename length and extension
/// allowlist. One variant per upload entry point so the policy cannot drift
/// away from the handler that calls it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadClass {
    /// Suite Drive (`/api/files/write`, `/api/files/upload-binary`) — a general
    /// file store, so the allowlist is broad but still excludes executables.
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
    extensions: &[
        "png", "jpg", "jpeg", "gif", "webp", "bmp", "tif", "tiff", "pdf", "zip", "gz", "rar",
        "7z", "xz", "bz2", "docx", "xlsx", "pptx", "odt", "ods", "odp", "doc", "xls", "ppt",
        "csv", "txt", "md", "markdown", "json", "xml", "rtf", "mp4", "m4v", "mov", "webm",
        "mkv", "avi", "mp3", "wav", "ogg", "oga", "m4a", "flac", "log", "yaml", "yml", "toml",
        "ini", "sql", "rs", "ts", "css", "py", "bas", "ast", "bin", "dat", "html", "htm",
        "svg",
    ],
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
    // is the one endpoint that accepts `.html`.
    extensions: &["docx", "odt", "rtf", "md", "markdown", "html", "htm", "txt"],
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
    }
}

/// Size cap in bytes for `class`.
#[must_use]
pub fn max_bytes(class: UploadClass) -> usize {
    limits_for(class).max_bytes
}

/// An upload that passed every rule. `filename` is already sanitized, and
/// `extension` is the lowercased final extension used to build the stored key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedUpload {
    pub filename: String,
    pub extension: String,
    pub mime: String,
    pub size: usize,
}

/// Why an upload was refused. The `code` is a catalogue message id, stable
/// across locales, so clients can branch on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UploadError {
    Empty,
    TooLarge { max: usize },
    FilenameInvalid,
    FilenameTooLong { max: usize },
    ExtensionBlocked { extension: String },
    ExtensionNotAllowed { extension: String, allowed: &'static [&'static str] },
    UnknownExtension { extension: String },
    TypeMismatch { expected: String, detected: String },
    ArchiveUnsafe { reason: &'static str },
}

impl UploadError {
    #[must_use]
    pub fn status(&self) -> StatusCode {
        match self {
            Self::TooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            _ => StatusCode::BAD_REQUEST,
        }
    }

    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Empty => "error-file-empty",
            Self::TooLarge { .. } => "error-file-too-large",
            Self::FilenameInvalid | Self::FilenameTooLong { .. } => "error-file-name-invalid",
            Self::ExtensionBlocked { .. } => "error-file-extension-blocked",
            Self::ExtensionNotAllowed { .. } => "error-file-type-not-allowed",
            Self::UnknownExtension { .. } => "error-file-extension-unknown",
            Self::TypeMismatch { .. } => "error-file-content-mismatch",
            Self::ArchiveUnsafe { .. } => "error-file-archive-unsafe",
        }
    }

    /// Localized, client-safe message. Internal detail (paths, driver errors)
    /// never reaches this string.
    #[must_use]
    pub fn message(&self, locale: &Locale) -> String {
        let code = self.code();
        match self {
            Self::TooLarge { max } => {
                i18n::translate(locale, code, &[("maxSize", &human_size(*max))])
            }
            Self::FilenameTooLong { max } => {
                i18n::translate(locale, code, &[("max", &max.to_string())])
            }
            Self::ExtensionBlocked { extension }
            | Self::UnknownExtension { extension } => {
                i18n::translate(locale, code, &[("extension", extension)])
            }
            Self::ExtensionNotAllowed { extension, allowed } => {
                let allowed_types = allowed
                    .iter()
                    .map(|ext| format!(".{ext}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                i18n::translate(
                    locale,
                    code,
                    &[("extension", extension), ("allowedTypes", &allowed_types)],
                )
            }
            Self::TypeMismatch { expected, detected } => i18n::translate(
                locale,
                code,
                &[("expected", expected), ("detected", detected)],
            ),
            Self::ArchiveUnsafe { reason } => {
                i18n::translate(locale, code, &[("reason", reason)])
            }
            Self::Empty | Self::FilenameInvalid => i18n::get(locale, code),
        }
    }

    /// Server-side detail for the log line; never sent to the client.
    #[must_use]
    pub fn log_detail(&self) -> String {
        format!("{self:?}")
    }
}

/// MIME type implied by an extension, or `None` when the extension is unknown.
#[must_use]
pub fn mime_for_extension(extension: &str) -> Option<&'static str> {
    Some(match extension {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "tif" | "tiff" => "image/tiff",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "gz" => "application/gzip",
        "rar" => "application/vnd.rar",
        "7z" => "application/x-7z-compressed",
        "xz" => "application/x-xz",
        "bz2" => "application/x-bzip2",
        "docx" => {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        }
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "odt" => "application/vnd.oasis.opendocument.text",
        "ods" => "application/vnd.oasis.opendocument.spreadsheet",
        "odp" => "application/vnd.oasis.opendocument.presentation",
        "doc" => "application/msword",
        "xls" => "application/vnd.ms-excel",
        "ppt" => "application/vnd.ms-powerpoint",
        "csv" => "text/csv",
        "txt" | "log" | "bas" | "ast" => "text/plain",
        "md" | "markdown" => "text/markdown",
        "json" => "application/json",
        "xml" | "mpp" => "application/xml",
        "rtf" => "application/rtf",
        "html" | "htm" => "text/html",
        "svg" => "image/svg+xml",
        "yaml" | "yml" | "toml" | "ini" | "sql" | "rs" | "ts" | "css" | "py" => "text/plain",
        "mp4" | "m4v" => "video/mp4",
        "mov" => "video/quicktime",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "avi" => "video/x-msvideo",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" | "oga" => "audio/ogg",
        "m4a" => "audio/mp4",
        "flac" => "audio/flac",
        "tsv" => "text/tab-separated-values",
        _ => return None,
    })
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

    if trimmed.is_empty() || trimmed.starts_with('.') || trimmed == "." || trimmed == ".." {
        return Err(UploadError::FilenameInvalid);
    }
    if trimmed.len() > max_len {
        return Err(UploadError::FilenameTooLong { max: max_len });
    }

    Ok(trimmed.to_string())
}

/// Applies the full policy for `class` to an upload.
pub fn validate_upload(
    class: UploadClass,
    filename: &str,
    content_type: &str,
    data: &[u8],
) -> Result<ValidatedUpload, UploadError> {
    let limits = limits_for(class);

    if data.is_empty() {
        return Err(UploadError::Empty);
    }
    if data.len() > limits.max_bytes {
        return Err(UploadError::TooLarge { max: limits.max_bytes });
    }

    let filename = sanitize_upload_filename(filename, limits.max_filename_len)?;
    let extension = final_extension(&filename).to_lowercase();

    if extension.is_empty() {
        return Err(UploadError::UnknownExtension { extension });
    }
    if blocked_extensions().contains(&extension.as_str()) {
        return Err(UploadError::ExtensionBlocked { extension });
    }

    let expected = mime_for_extension(&extension)
        .ok_or_else(|| UploadError::UnknownExtension { extension: extension.clone() })?;

    if !limits.allows_extension(&extension) {
        return Err(UploadError::ExtensionNotAllowed {
            extension,
            allowed: limits.extensions,
        });
    }

    let declared = normalize_mime(content_type);
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

    if is_zip_container(expected) {
        inspect_archive(data)?;
    }

    Ok(ValidatedUpload {
        filename,
        extension,
        mime: expected.to_string(),
        size: data.len(),
    })
}

/// Formats without a reliable leading signature; their extension is the only
/// type signal available.
const UNVERIFIABLE_EXTENSIONS: &[&str] = &["bin", "dat", "mpp"];

fn final_extension(filename: &str) -> &str {
    filename.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("")
}

fn normalize_mime(raw: &str) -> String {
    raw.split(';').next().unwrap_or_default().trim().to_ascii_lowercase()
}

fn is_text_mime(mime: &str) -> bool {
    mime.starts_with("text/")
        || matches!(
            mime,
            "application/json" | "application/xml" | "application/rtf"
        )
}

fn is_zip_container(mime: &str) -> bool {
    mime == "application/zip"
        || mime.starts_with("application/vnd.openxmlformats-officedocument.")
        || mime.starts_with("application/vnd.oasis.opendocument.")
}

fn declared_matches(declared: &str, expected: &str) -> bool {
    // No declaration (or the generic one browsers send for unknown files) is
    // not a claim to contradict: the extension and the magic bytes decide.
    if declared.is_empty()
        || declared == "application/octet-stream"
        || declared == "binary/octet-stream"
    {
        return true;
    }
    if declared == expected {
        return true;
    }
    // Browsers label `.md`, `.csv`, `.json` and friends inconsistently.
    if is_text_mime(declared) && is_text_mime(expected) {
        return true;
    }
    // Office/ODF packages are zip files, and some clients say so.
    is_zip_container(expected)
        && matches!(declared, "application/zip" | "application/x-zip-compressed")
}

fn magic_matches(extension: &str, expected: &str, detected: &str, data: &[u8]) -> bool {
    if detected == expected {
        return true;
    }
    // OOXML/ODF are zip containers; legacy Office is an OLE2 compound file.
    if is_zip_container(expected) && detected == "application/zip" {
        return true;
    }
    if matches!(expected, "application/msword" | "application/vnd.ms-excel" | "application/vnd.ms-powerpoint")
        && detected == "application/x-ole-storage"
    {
        return true;
    }
    // Text formats: ASCII payloads all read as `text/plain`, and an XML
    // declaration reads as `text/xml`.
    if is_text_mime(expected) && matches!(detected, "text/plain" | "text/xml") {
        return true;
    }
    // SVG is XML text: `<svg` is the discriminator, not the magic table.
    if expected == "image/svg+xml" && matches!(detected, "text/plain" | "text/xml") {
        return contains_svg(data);
    }
    // ISO base media: MP4/MOV/M4A share the `ftyp` box.
    if matches!(expected, "video/mp4" | "video/quicktime" | "audio/mp4")
        && detected == "video/mp4"
    {
        return true;
    }
    // RIFF carries WAV and AVI; the form type at offset 8 disambiguates.
    if expected == "video/x-msvideo" && detected == "audio/wav" && is_riff_avi(data) {
        return true;
    }
    // `.m4a` is detected as audio/mp4 but some encoders emit a plain ftyp box.
    if extension == "m4a" && detected == "video/mp4" {
        return true;
    }
    false
}

fn contains_svg(data: &[u8]) -> bool {
    let head = String::from_utf8_lossy(&data[0..data.len().min(4096)]).to_ascii_lowercase();
    head.contains("<svg")
}

/// Extension a detected MIME type is stored under. Used by handlers that
/// receive raw bytes with no filename (the legacy `/api/files/upload-binary`).
#[must_use]
pub fn extension_for_mime(mime: &str) -> Option<&'static str> {
    Some(match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/bmp" => "bmp",
        "image/tiff" => "tiff",
        "image/svg+xml" => "svg",
        "application/pdf" => "pdf",
        "application/zip" => "zip",
        "application/gzip" => "gz",
        "application/vnd.rar" => "rar",
        "application/x-7z-compressed" => "7z",
        "application/x-xz" => "xz",
        "application/x-bzip2" => "bz2",
        "application/x-ole-storage" => "doc",
        "text/plain" => "txt",
        "text/csv" => "csv",
        "text/markdown" => "md",
        "text/html" => "html",
        "text/xml" | "application/xml" => "xml",
        "application/json" => "json",
        "application/rtf" => "rtf",
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        "video/x-matroska" => "mkv",
        "video/x-msvideo" => "avi",
        "video/quicktime" => "mov",
        "audio/mpeg" => "mp3",
        "audio/wav" => "wav",
        "audio/ogg" => "ogg",
        "audio/mp4" => "m4a",
        "audio/flac" => "flac",
        _ => return None,
    })
}

fn is_riff_avi(data: &[u8]) -> bool {
    data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"AVI "
}

/// Opens a zip container and refuses bombs: too many entries, a declared
/// uncompressed size over the cap, or an entry name that escapes the archive.
fn inspect_archive(data: &[u8]) -> Result<(), UploadError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(data)).map_err(|_| {
        UploadError::TypeMismatch {
            expected: "application/zip".to_string(),
            detected: "not a zip container".to_string(),
        }
    })?;

    if archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err(UploadError::ArchiveUnsafe {
            reason: "too many entries",
        });
    }

    let mut total: u64 = 0;
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|_| UploadError::ArchiveUnsafe {
                reason: "unreadable entry",
            })?;
        total = total.saturating_add(entry.size());
        if total > MAX_ARCHIVE_UNCOMPRESSED_BYTES {
            return Err(UploadError::ArchiveUnsafe {
                reason: "decompressed size",
            });
        }
        if entry.enclosed_name().is_none() {
            return Err(UploadError::ArchiveUnsafe {
                reason: "unsafe entry name",
            });
        }
    }

    Ok(())
}

/// Refuses the request from `Content-Length` alone, before the body is
/// buffered. Handlers call this first; [`validate_upload`] re-checks the real
/// length for chunked requests that carry no header.
pub fn check_declared_length(
    class: UploadClass,
    headers: &HeaderMap,
) -> Result<(), UploadError> {
    let max = max_bytes(class);
    let declared = headers
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<u64>().ok());

    match declared {
        Some(length) if length > max as u64 => Err(UploadError::TooLarge { max }),
        _ => Ok(()),
    }
}

/// Request locale from `X-Locale` / `Accept-Language` (#2094).
#[must_use]
pub fn locale_from_headers(headers: &HeaderMap) -> Locale {
    i18n::locale_from_request(
        headers.get(i18n::LOCALE_HEADER).and_then(|v| v.to_str().ok()),
        headers
            .get(header::ACCEPT_LANGUAGE)
            .and_then(|v| v.to_str().ok()),
    )
}

/// Localized `400`/`413` body: `{ code, message, error }`.
#[must_use]
pub fn rejection_json(error: &UploadError, headers: &HeaderMap) -> (StatusCode, Json<Value>) {
    let message = error.message(&locale_from_headers(headers));
    (
        error.status(),
        Json(json!({
            "code": error.code(),
            "message": &message,
            "error": message,
        })),
    )
}

/// Localized `(status, message)` pair for handlers whose error type is a plain
/// string tuple.
#[must_use]
pub fn rejection_message(error: &UploadError, headers: &HeaderMap) -> (StatusCode, String) {
    (
        error.status(),
        error.message(&locale_from_headers(headers)),
    )
}

fn human_size(bytes: usize) -> String {
    const MIB: usize = 1024 * 1024;
    if bytes >= MIB {
        format!("{} MiB", bytes / MIB)
    } else {
        format!("{} KiB", bytes / 1024)
    }
}

#[cfg(test)]
#[path = "upload_policy_tests.rs"]
mod upload_policy_tests;
