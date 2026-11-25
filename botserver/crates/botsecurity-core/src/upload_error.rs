//! Upload rejections: stable codes, localized messages, HTTP shape (#2132).
//!
//! A handler never echoes the reason a driver or a parser produced. It answers
//! with a catalogue code (`error-file-*`) and a translated message, and logs
//! the detail server-side through [`UploadError::log_detail`].

use axum::http::{header, HeaderMap, StatusCode};
use axum::Json;
use botlib::i18n::{self, Locale};
use serde_json::{json, Value};
use std::fmt;

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
    /// A zip entry is encrypted, so its content cannot be inspected (#2522).
    ArchiveEncrypted,
    /// A second zip/Office container is embedded in the upload (#2522).
    ArchiveNested { part: String },
    /// A VBA/OLE macro part is present. We refuse the whole file rather than
    /// strip it: rewriting a signed OOXML package invalidates its signature.
    ArchiveMacro { part: String },
    /// The declared uncompressed-to-compressed ratio exceeds the cap (#2522).
    ArchiveCompressionRatio { ratio: u64, max: u64 },
    /// An OOXML/ODF relationship target failed the SSRF policy (#2522).
    RelationshipTargetUnsafe { kind: &'static str },
    /// A malware scan was mandatory but no scanner hook is installed (#2522).
    MalwareScannerMissing,
    /// The configured scanner could not complete, so the file is refused
    /// rather than treated as clean (#2522).
    MalwareScanFailed,
    /// The configured scanner reported a signature (#2522).
    MalwareDetected { signature: String },
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
            Self::ArchiveUnsafe { .. }
            | Self::ArchiveEncrypted
            | Self::ArchiveNested { .. }
            | Self::ArchiveMacro { .. }
            | Self::ArchiveCompressionRatio { .. }
            | Self::RelationshipTargetUnsafe { .. } => "error-file-archive-unsafe",
            Self::MalwareScannerMissing
            | Self::MalwareScanFailed
            | Self::MalwareDetected { .. } => "error-file-upload-failed",
        }
    }

    /// Localized, client-safe message.
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
            Self::ExtensionBlocked { extension } | Self::UnknownExtension { extension } => {
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
            Self::ArchiveEncrypted => {
                i18n::translate(locale, code, &[("reason", "encrypted entry")])
            }
            Self::ArchiveNested { .. } => {
                i18n::translate(locale, code, &[("reason", "nested archive")])
            }
            Self::ArchiveMacro { .. } => {
                i18n::translate(locale, code, &[("reason", "macro-bearing part")])
            }
            Self::ArchiveCompressionRatio { .. } => {
                i18n::translate(locale, code, &[("reason", "compression ratio too high")])
            }
            Self::RelationshipTargetUnsafe { kind } => {
                let reason = format!("unsafe external reference ({kind})");
                i18n::translate(locale, code, &[("reason", &reason)])
            }
            Self::MalwareScannerMissing => i18n::translate(
                locale,
                code,
                &[("message", "no malware scanner is configured")],
            ),
            Self::MalwareScanFailed => i18n::translate(
                locale,
                code,
                &[("message", "the malware scan could not be completed")],
            ),
            Self::MalwareDetected { .. } => {
                i18n::translate(locale, code, &[("message", "malware detected")])
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

/// A short, locale-independent description for logs and test failure output.
///
/// Every variant renders a fixed sentence: the structured fields (extension,
/// signature, detected type, archive part, …) are caller-controlled and may
/// carry user data, so they are deliberately never interpolated here. Detailed
/// diagnostics belong in [`UploadError::log_detail`], which stays server-side.
impl fmt::Display for UploadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Empty => "the upload is empty",
            Self::TooLarge { .. } => "the upload exceeds the maximum allowed size",
            Self::FilenameInvalid | Self::FilenameTooLong { .. } => "the file name is invalid",
            Self::ExtensionBlocked { .. } => "the file extension is blocked",
            Self::ExtensionNotAllowed { .. } => "the file type is not allowed",
            Self::UnknownExtension { .. } => "the file extension is not recognized",
            Self::TypeMismatch { .. } => "the declared type does not match the content",
            Self::ArchiveUnsafe { .. } => "the archive contains unsafe content",
            Self::ArchiveEncrypted => "the archive contains an encrypted entry",
            Self::ArchiveNested { .. } => "the archive contains a nested container",
            Self::ArchiveMacro { .. } => "the archive contains a macro-bearing part",
            Self::ArchiveCompressionRatio { .. } => {
                "the archive compression ratio exceeds the limit"
            }
            Self::RelationshipTargetUnsafe { .. } => {
                "the document contains an unsafe external reference"
            }
            Self::MalwareScannerMissing => "no malware scanner is configured",
            Self::MalwareScanFailed => "the malware scan could not be completed",
            Self::MalwareDetected { .. } => "malware detected",
        };
        f.write_str(message)
    }
}

impl std::error::Error for UploadError {}

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
mod tests {
    use super::*;

    fn assert_is_error<E: std::error::Error>() {}

    #[test]
    fn every_variant_has_a_display_message() {
        assert_is_error::<UploadError>();
        let variants = [
            UploadError::Empty,
            UploadError::TooLarge { max: 1 },
            UploadError::FilenameInvalid,
            UploadError::FilenameTooLong { max: 1 },
            UploadError::ExtensionBlocked {
                extension: "exe".into(),
            },
            UploadError::ExtensionNotAllowed {
                extension: "exe".into(),
                allowed: &["txt"],
            },
            UploadError::UnknownExtension {
                extension: "zzz".into(),
            },
            UploadError::TypeMismatch {
                expected: "pdf".into(),
                detected: "png".into(),
            },
            UploadError::ArchiveUnsafe { reason: "bad" },
            UploadError::ArchiveEncrypted,
            UploadError::ArchiveNested {
                part: "inner.zip".into(),
            },
            UploadError::ArchiveMacro {
                part: "vbaProject.bin".into(),
            },
            UploadError::ArchiveCompressionRatio {
                ratio: 1000,
                max: 10,
            },
            UploadError::RelationshipTargetUnsafe { kind: "external" },
            UploadError::MalwareScannerMissing,
            UploadError::MalwareScanFailed,
            UploadError::MalwareDetected {
                signature: "EICAR".into(),
            },
        ];
        for error in &variants {
            assert!(
                !error.to_string().is_empty(),
                "each variant must render a message"
            );
        }
    }

    #[test]
    fn display_never_embeds_caller_controlled_fields() {
        const SENTINEL: &str = "SENTINEL_DO_NOT_LEAK";
        let errors = [
            UploadError::ExtensionBlocked {
                extension: SENTINEL.into(),
            },
            UploadError::UnknownExtension {
                extension: SENTINEL.into(),
            },
            UploadError::ArchiveNested {
                part: SENTINEL.into(),
            },
            UploadError::ArchiveMacro {
                part: SENTINEL.into(),
            },
            UploadError::MalwareDetected {
                signature: SENTINEL.into(),
            },
            UploadError::TypeMismatch {
                expected: SENTINEL.into(),
                detected: SENTINEL.into(),
            },
        ];
        for error in &errors {
            assert!(
                !error.to_string().contains(SENTINEL),
                "Display leaked a structured field"
            );
        }
    }
}
