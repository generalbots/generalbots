//! Page setup, print margins and header/footer (E11).
//!
//! Extracted from the worksheet's `<pageSetup>`, `<pageMargins>` and
//! `<headerFooter>` elements. These round-trip byte-for-byte through the
//! preserve-and-passthrough save path; modelling them makes the values
//! available to the PDF export and any future print UI.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrintSetup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paper_size: Option<u32>,
    /// "landscape" or "portrait".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fit_to_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fit_to_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_left: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_right: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_top: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_bottom: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_header: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_footer: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub odd_header: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub odd_footer: Option<String>,
    /// Rows/columns repeated on each printed page (`_xlnm.Print_Titles`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_titles: Option<PrintTitles>,
}

/// Repeated print rows/columns (E11).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrintTitles {
    /// Rows repeated on each printed page, e.g. "1:3".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<String>,
    /// Columns repeated on each printed page, e.g. "A:B".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<String>,
}
