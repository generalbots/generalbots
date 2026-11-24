mod csv;
mod odf;
mod xlsx;

pub use csv::parse_csv_to_worksheets;
pub use odf::{is_odf_spreadsheet, parse_ods_to_worksheets};
pub use xlsx::parse_xlsx_to_worksheets;

use crate::types::Spreadsheet;
use botcore::{ImportSize, WorkClass};
use botsheet_core::engine::value::CellValue;

use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Maps umya sheet state to the model visibility string (`None` = visible).
fn sheet_visibility(sheet: &umya_spreadsheet::Worksheet) -> Option<String> {
    use umya_spreadsheet::structs::SheetStateValues;
    match sheet.state() {
        SheetStateValues::Hidden => Some("hidden".to_string()),
        SheetStateValues::VeryHidden => Some("veryHidden".to_string()),
        SheetStateValues::Visible => None,
    }
}

pub fn create_new_spreadsheet(owner_id: &str) -> Spreadsheet {
    Spreadsheet {
        id: Uuid::new_v4().to_string(),
        name: "Untitled Spreadsheet".to_string(),
        owner_id: owner_id.to_string(),
        worksheets: vec![crate::types::Worksheet {
            name: "Sheet1".to_string(),
            data: HashMap::new(),
            column_widths: None,
            row_heights: None,
            frozen_rows: None,
            frozen_cols: None,
            merged_cells: None,
            filters: None,
            hidden_rows: None,
            validations: None,
            conditional_formats: None,
            charts: None,
            comments: None,
            protection: None,
            array_formulas: None,
            tables: None,
            hidden_columns: None,
            sheet_state: None,
            hyperlinks: None,
            print_setup: None,
            autofilter: None,
            row_page_breaks: None,
            column_page_breaks: None,
            images: None,
            print_areas: None,
            rich_text: None,
        }],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        named_ranges: None,
        external_links: None,
        source_bucket: None,
        source_path: None,
        source_bytes: None,
        acl: HashMap::new(),
    }
}

pub fn detect_spreadsheet_format(bytes: &[u8]) -> &'static str {
    if bytes.len() >= 4 {
        if &bytes[0..4] == b"PK\x03\x04" {
            let content_str = String::from_utf8_lossy(&bytes[0..4096.min(bytes.len())]);
            if content_str.contains("xl/workbook.bin") {
                return "xlsb";
            }
            if content_str.contains("xl/") || content_str.contains("[Content_Types].xml") {
                return "xlsx";
            }
            if content_str.contains("content.xml") || content_str.contains("mimetype") {
                return "ods";
            }
            return "zip";
        }
        if bytes[0] == 0xD0 && bytes[1] == 0xCF {
            return "xls";
        }
    }

    let text = String::from_utf8_lossy(&bytes[0..100.min(bytes.len())]);
    if text.contains('\t') && text.lines().count() > 1 {
        return "tsv";
    }
    if text.contains(',') && text.lines().count() > 1 {
        return "csv";
    }

    "unknown"
}

pub fn import_spreadsheet_bytes(
    bytes: &[u8],
    filename: &str,
    owner_id: &str,
) -> Result<Spreadsheet, String> {
    // #2530 — shared pure budget gate, applied before any parser builds its
    // in-memory workbook/grid. The row dimension is re-checked by the
    // streaming pre-scan in `xlsx.rs` and by the CSV parser once known.
    let budget = crate::handlers::admission::import_budget(WorkClass::SheetsImport);
    if let Err(error) = budget.check(ImportSize {
        bytes: bytes.len() as u64,
        ..ImportSize::default()
    }) {
        return Err(error.message());
    }

    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    let detected = detect_spreadsheet_format(bytes);

    let mut worksheets = match detected {
        "xlsx" | "xlsm" => parse_xlsx_to_worksheets(bytes, "xlsx")?,
        "xls" => super::import_binary::parse_binary_to_worksheets(bytes, "xls")?,
        "xlsb" => super::import_binary::parse_binary_to_worksheets(bytes, "xlsb")?,
        "ods" => odf::parse_ods_to_worksheets(bytes)?,
        "csv" => parse_csv_to_worksheets(bytes, b',', "Sheet1")?,
        "tsv" => parse_csv_to_worksheets(bytes, b'\t', "Sheet1")?,
        _ => {
            if ext == "csv" {
                parse_csv_to_worksheets(bytes, b',', "Sheet1")?
            } else if ext == "tsv" || ext == "txt" {
                parse_csv_to_worksheets(bytes, b'\t', "Sheet1")?
            } else if ext == "ods" {
                odf::parse_ods_to_worksheets(bytes)?
            } else {
                return Err(format!("Unsupported format: {detected}"));
            }
        }
    };

    let raw_filename = filename.rsplit('/').next().unwrap_or(filename);
    let suffix = format!(".{ext}");
    let name = raw_filename
        .strip_suffix(&suffix)
        .unwrap_or(raw_filename)
        .to_string();

    // A generated package carries formulas without cached results; without
    // this pass the grid shows labels and empty cells (#2152 follow-up).
    let recalculated = botsheet_core::recalc::recalculate_if_needed(&mut worksheets);
    if recalculated > 0 {
        log::info!("import recalculated {recalculated} formula cells");
    }
    let refreshed = botsheet_core::chart_data::refresh_chart_data(&mut worksheets);
    if refreshed > 0 {
        log::info!("import refreshed {refreshed} chart series");
    }

    Ok(Spreadsheet {
        id: Uuid::new_v4().to_string(),
        name,
        owner_id: owner_id.to_string(),
        worksheets,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        named_ranges: None,
        external_links: super::xlsx_external_links::extract_external_links(bytes),
        source_bucket: None,
        source_path: None,
        // #2043 — retain the uploaded package so a later save patches the
        // original (charts, comments, conditional formats, images survive)
        // instead of rebuilding the workbook from the model. `csv`/`tsv` are
        // plain text with no package to patch.
        source_bytes: if super::xlsx_source::retains_snapshot(detected) {
            Some(bytes.to_vec())
        } else {
            None
        },
        acl: HashMap::new(),
    })
}
