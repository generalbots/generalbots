//! Rich-text runs for a cell (E6).
//!
//! A shared string can hold multiple `<r>` runs, each with its own
//! bold/italic/underline/colour/font/size. umya-spreadsheet flattens these to
//! a single string, so the runs are recovered from the raw `sharedStrings.xml`
//! part (see `botsheet::storage::xlsx_rich_text`) and stored here.

use serde::{Deserialize, Serialize};

/// A single formatted run within a rich-text cell. Runs are ordered; their
/// concatenated `text` equals the cell's displayed value. All formatting
/// fields are optional so an unformatted run stays a bare `text`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichTextRun {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strike: Option<bool>,
    /// Font colour as `#RRGGBB` (the OOXML `rgb` alpha byte is dropped).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<f64>,
}
