//! Read-only Office previews for the Drive modal.
//!
//! Chat/automation output often lands in Drive as a real `.docx`, `.xlsx` or
//! `.pptx`. Drive could download those bytes but had no renderer, so clicking
//! one fell through to a raw text dump. This module turns the three formats
//! into a safe HTML fragment:
//!
//! - DOCX/DOC/ODT/RTF: the existing `docx-rs` converter used by the editor;
//! - XLSX/XLS/XLSB/ODS: `calamine`, one HTML table per worksheet (bounded);
//! - PPTX/PPT: the slide XML text runs, one card per slide.
//!
//! Everything user-authored is escaped; the only HTML emitted is the wrapper.

mod docx;
mod sheet;
mod shell;
mod slides;

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use serde::Deserialize;

use crate::state::DocState;

pub use shell::preview_html;

#[derive(Debug, Deserialize)]
pub struct OfficePreviewQuery {
    pub bucket: String,
    pub path: String,
}

/// GET /api/files/office-preview?bucket=&path=
pub async fn handle_office_preview(
    State(state): State<Arc<DocState>>,
    Query(query): Query<OfficePreviewQuery>,
) -> Result<Html<String>, (StatusCode, String)> {
    if query.bucket.trim().is_empty() || query.path.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "bucket and path are required".to_string(),
        ));
    }
    let bytes = state
        .drive
        .get_object(&query.bucket, &query.path)
        .await
        .map_err(|error| {
            log::warn!(
                "office preview: drive read of {}/{} failed: {error}",
                query.bucket,
                query.path
            );
            (StatusCode::NOT_FOUND, "file not found".to_string())
        })?;
    let extension = query.path.rsplit('.').next().unwrap_or("").to_lowercase();
    let file_name = query.path.rsplit('/').next().unwrap_or(&query.path);
    preview_html(file_name, &extension, &bytes)
        .map(Html)
        .map_err(|error| {
            log::warn!(
                "office preview: conversion of {} failed: {error}",
                query.path
            );
            (StatusCode::UNPROCESSABLE_ENTITY, error)
        })
}
