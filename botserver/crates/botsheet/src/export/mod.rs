//! Spreadsheet exporters, split by output format.
//!
//! `csv`, `html`, `json`, `markdown`, `ods`, `pdf_data` and `xlsx` each own one
//! format; `util` holds the cell-number and column-letter helpers they share.

mod csv;
mod html;
mod json;
mod markdown;
mod ods;
mod pdf_data;
mod util;
mod xlsx;

#[cfg(test)]
mod csv_tests;
#[cfg(test)]
mod html_tests;
#[cfg(test)]
mod json_tests;
#[cfg(test)]
mod markdown_tests;
#[cfg(test)]
mod support_tests;
#[cfg(test)]
mod util_tests;

pub use csv::export_to_csv;
pub use html::export_to_html;
pub use json::export_to_json;
pub use markdown::export_to_markdown;
pub use ods::export_to_ods;
pub use pdf_data::{export_to_pdf_data, export_to_pdf_data_with};
pub use xlsx::export_to_xlsx;

#[cfg(test)]
mod tests {
    use super::*;
    use botsheet_core::types::{CellData, Spreadsheet, Worksheet};
    use chrono::Utc;

    fn sample_sheet() -> Spreadsheet {
        let mut worksheet = Worksheet::default();
        worksheet.name = "Sheet1".to_string();
        worksheet.data.insert(
            "0,0".to_string(),
            CellData {
                value: Some("hello".to_string()),
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
        Spreadsheet {
            id: "test".to_string(),
            name: "Test".to_string(),
            owner_id: "owner".to_string(),
            worksheets: vec![worksheet],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            named_ranges: None,
            external_links: None,
            source_bucket: None,
            source_path: None,
            source_bytes: None,
            acl: Default::default(),
        }
    }

    #[test]
    fn xlsx_export_is_a_zip_package() {
        let bytes = export_to_xlsx(&sample_sheet()).expect("xlsx export");
        assert_eq!(&bytes[..2], b"PK");
    }

    #[test]
    fn ods_export_is_an_odf_package() {
        let bytes = export_to_ods(&sample_sheet()).expect("ods export");
        assert_eq!(&bytes[..2], b"PK");
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip");
        {
            let mut mime = zip.by_name("mimetype").expect("mimetype entry");
            let mut text = String::new();
            std::io::Read::read_to_string(&mut mime, &mut text).expect("read mimetype");
            assert_eq!(text, "application/vnd.oasis.opendocument.spreadsheet");
        }
        assert!(zip.by_name("content.xml").is_ok());
        assert!(zip.by_name("META-INF/manifest.xml").is_ok());
    }
}
