//! Upload rejections: stable codes, localized messages, HTTP shape (#2132).
//!
//! A handler never echoes the reason a driver or a parser produced. It answers
//! with a catalogue code (`error-file-*`) and a translated message, and logs
//! the detail server-side through [`UploadError::log_detail`].

use axum::http::{header, HeaderMap, StatusCode};
use axum::Json;
use botlib::i18n::{self, Locale};
use serde_json::{json, Value};

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
            Self::Empty | Self::FilenameInvalid => i18n::get(locale, code),
        }
    }

    /// Server-side detail for the log line; never sent to the client.
    #[must_use]
    pub fn log_detail(&self) -> String {
        format!("{self:?}")
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
