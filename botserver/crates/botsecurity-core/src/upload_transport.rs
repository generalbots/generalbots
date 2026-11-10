//! Transport-level upload caps (#2132).
//!
//! `axum` applies the per-route [`DefaultBodyLimit`] before an extractor runs,
//! which is what stops a chunked upload that carries no `Content-Length` from
//! buffering unbounded. Handlers also call [`check_declared_length`] so a
//! browser upload is refused from the header alone, without reading the body.

use axum::extract::DefaultBodyLimit;
use axum::http::{header, HeaderMap};

use crate::upload_error::UploadError;
use crate::upload_policy::{max_bytes, UploadClass};

/// Transport-level cap for an upload route.
///
/// `axum` then refuses an oversized body with `413` before the handler runs —
/// this covers chunked uploads that carry no `Content-Length`, which the
/// in-handler [`check_declared_length`] cannot see.
#[must_use]
pub fn body_limit(class: UploadClass) -> DefaultBodyLimit {
    DefaultBodyLimit::max(max_bytes(class))
}

/// Refuses the request from `Content-Length` alone, before the body is
/// buffered. Handlers call this first; `validate_upload` re-checks the real
/// length for chunked requests that carry no header.
pub fn check_declared_length(class: UploadClass, headers: &HeaderMap) -> Result<(), UploadError> {
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

