//! PDF export — stub for future implementation.
//!
//! Currently only DOCX and PPTX exporters are implemented. The PDF
//! module is reserved for a future HTML-to-PDF rendering pipeline
//! (likely backed by `printpdf` or `wkhtmltopdf`).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfDocument {
    pub title: String,
    pub pages: Vec<PdfPage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfPage {
    pub width_pt: f32,
    pub height_pt: f32,
    pub content: String,
}

impl PdfDocument {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            pages: Vec::new(),
        }
    }

    pub fn add_page(&mut self, page: PdfPage) {
        self.pages.push(page);
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
}

impl Default for PdfDocument {
    fn default() -> Self {
        Self::new("untitled")
    }
}

pub fn from_html(_html: &str) -> Result<PdfDocument, String> {
    Err("PDF export is not yet implemented".to_string())
}
