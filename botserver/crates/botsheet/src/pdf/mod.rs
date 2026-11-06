//! Minimal PDF 1.4 exporter (#788).
//!
//! Produces a dependency-free PDF: one A4 page set per worksheet, each grid
//! rendered as a table of Helvetica text with a repeated header row. TrueType
//! embedding, colors and vector graphics are intentionally out of scope; the
//! goal is a faithful, printable snapshot of the data — not a designer layout.
//!
//! Split by responsibility: `options` (print layout), `writer` (objects and
//! pages), `text` (content-stream text and cell metrics) and `export` (entry
//! points).

mod export;
mod options;
mod text;
mod writer;

pub use export::{export_pdf, export_pdf_with};
pub use options::PdfOptions;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CellData, Spreadsheet};
    use std::collections::HashMap;

    fn ws() -> Spreadsheet {
        let mut data = HashMap::new();
        data.insert(
            "0,0".to_string(),
            CellData {
                value: Some("Name".to_string()),
                typed: None,
                formula: None,
                style: None,
                format: None,
                note: None,
                locked: None,
                has_comment: None,
                array_formula_id: None,
            },
        );
        data.insert(
            "1,0".to_string(),
            CellData {
                value: Some("Alice".to_string()),
                typed: None,
                formula: None,
                style: None,
                format: None,
                note: None,
                locked: None,
                has_comment: None,
                array_formula_id: None,
            },
        );
        data.insert(
            "1,1".to_string(),
            CellData {
                value: Some("42".to_string()),
                typed: None,
                formula: None,
                style: None,
                format: None,
                note: None,
                locked: None,
                has_comment: None,
                array_formula_id: None,
            },
        );
        data.insert(
            "5,2".to_string(),
            CellData {
                value: Some("Deep (parens) value\\with\\slashes".to_string()),
                typed: None,
                formula: None,
                style: None,
                format: None,
                note: None,
                locked: None,
                has_comment: None,
                array_formula_id: None,
            },
        );
        sheet_with(data, "Report", "Sheet1")
    }

    fn sheet_with(data: HashMap<String, CellData>, name: &str, ws_name: &str) -> Spreadsheet {
        Spreadsheet {
            id: "s".to_string(),
            name: name.to_string(),
            owner_id: "o".to_string(),
            worksheets: vec![crate::types::Worksheet {
                name: ws_name.to_string(),
                data,
                ..crate::types::Worksheet::default()
            }],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            named_ranges: None,
            external_links: None,
            source_bucket: None,
            source_path: None,
            source_bytes: None,
            acl: HashMap::new(),
        }
    }

    #[test]
    fn produces_well_formed_pdf() {
        let pdf = export_pdf(&ws()).expect("pdf export should succeed");
        let header = &pdf[..8];
        assert_eq!(header, b"%PDF-1.4");
        let text = String::from_utf8_lossy(&pdf);
        assert!(text.contains("endobj"));
        assert!(text.contains("startxref"));
        assert!(text.contains("%%EOF"));
        assert!(text.contains("/Helvetica"));
        assert!(text.contains("(Alice)"));
        // The parens and backslashes must be escaped in the content stream.
        assert!(text.contains("\\("));
        assert!(text.contains("\\"));
    }

    #[test]
    fn empty_spreadsheet_still_renders() {
        let empty = sheet_with(HashMap::new(), "Blank", "Sheet1");
        let pdf = export_pdf(&empty).expect("blank export should succeed");
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(String::from_utf8_lossy(&pdf).contains("%%EOF"));
    }
}
