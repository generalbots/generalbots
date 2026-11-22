//! #2522 — encrypted/password-protected upload handling.
//!
//! The shared gate already flags ZipCrypto entries. This module adds the
//! compound-file signature of a password-protected OOXML package and builds a
//! refusal that tells the caller what to do. The message reuses the existing
//! localized `error-file-archive-unsafe` catalogue entry with an actionable
//! reason argument, so no new catalog key is required for the refusal text.

use axum::http::HeaderMap;
use axum::Json;
use botlib::i18n;
use serde_json::{json, Value};

use crate::upload_error::{locale_from_headers, UploadError};
use crate::upload_policy::{is_zip_container, mime_for_extension};

/// OLE/CFB magic. A password-protected DOCX/XLSX/PPTX is a CFB wrapper rather
/// than a zip; a plain legacy DOC/XLS has the same magic, so callers only use
/// this signal for names that claim to be a zip-based Office package.
const CFB_MAGIC: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
const ACTIONABLE_REASON: &str =
    "password-protected file; remove the password and upload an unprotected copy";

pub fn is_encrypted_rejection(error: &UploadError) -> bool {
    matches!(error, UploadError::ArchiveEncrypted)
}

pub fn is_password_protected_container(data: &[u8], filename: &str) -> bool {
    if !data.starts_with(CFB_MAGIC) {
        return false;
    }
    filename
        .rsplit('.')
        .next()
        .and_then(mime_for_extension)
        .is_some_and(is_zip_container)
}

/// Localized, sanitized refusal with a concrete next step. The detail that
/// produced the detection is never echoed; only the catalogue code and the
/// fixed reason are returned.
pub fn actionable_rejection(
    error: &UploadError,
    headers: &HeaderMap,
) -> (axum::http::StatusCode, Json<Value>) {
    let locale = locale_from_headers(headers);
    let message = i18n::translate(
        &locale,
        "error-file-archive-unsafe",
        &[("reason", ACTIONABLE_REASON)],
    );
    (
        error.status(),
        Json(json!({
            "code": error.code(),
            "message": message,
            "error": message,
            "action": "remove-password",
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn hostile_corpus_password_cfb_wrapper_is_detected() {
        let mut data = vec![0u8; 16];
        data[..CFB_MAGIC.len()].copy_from_slice(CFB_MAGIC);
        assert!(is_password_protected_container(&data, "report.docx"));
        assert!(!is_password_protected_container(&data, "report.doc"));
        assert!(!is_password_protected_container(
            b"PK\x03\x04",
            "report.docx"
        ));
    }

    #[test]
    fn hostile_corpus_password_refusal_is_actionable() {
        let error = UploadError::ArchiveEncrypted;
        assert!(is_encrypted_rejection(&error));
        let (status, Json(body)) = actionable_rejection(&error, &HeaderMap::new());
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["action"], "remove-password");
        let message = body["message"].as_str().unwrap_or_default();
        assert!(
            message.contains("remove the password"),
            "actionable message: {message}"
        );
        assert!(
            !message.contains("CFB"),
            "no internal detail leaked: {message}"
        );
    }
}
