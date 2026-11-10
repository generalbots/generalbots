//! Page layout and pagination expectations for the PDF writer.

use std::collections::HashMap;

use botsheet_core::types::CellData;

use super::*;

fn cell(value: &str) -> CellData {
    CellData {
        value: Some(value.to_string()),
        typed: None,
        formula: None,
        style: None,
        format: None,
        note: None,
        locked: None,
        has_comment: None,
        array_formula_id: None,
    }
}

fn rows(count: u32) -> HashMap<String, CellData> {
    (0..count)
        .map(|row| (format!("{row},0"), cell(&format!("r{row}"))))
        .collect()
}

fn options(landscape: bool, scale: f32) -> PdfOptions {
    PdfOptions { landscape, scale }
}

#[test]
fn portrait_and_landscape_swap_the_page_box() {
    let mut portrait = PdfWriter::new(options(false, 1.0));
    portrait.add_blank_page("S");
    let portrait_pdf = String::from_utf8_lossy(&portrait.finish()).to_string();
    assert!(portrait_pdf.contains("/MediaBox [0 0 595 842]"), "{portrait_pdf}");

    let mut landscape = PdfWriter::new(options(true, 1.0));
    landscape.add_blank_page("S");
    let landscape_pdf = String::from_utf8_lossy(&landscape.finish()).to_string();
    assert!(landscape_pdf.contains("/MediaBox [0 0 842 595]"), "{landscape_pdf}");
}

#[test]
fn scale_changes_grid_metrics_but_not_the_page() {
    let base = PdfWriter::new(options(false, 1.0));
    assert_eq!(base.row_height, ROW_HEIGHT);
    assert_eq!(base.col_width, COL_WIDTH);
    assert_eq!(base.font_size, FONT_SIZE);
    assert_eq!(base.rows_per_page, 48);

    let scaled = PdfWriter::new(options(false, 2.0));
    assert_eq!(scaled.row_height, ROW_HEIGHT * 2.0);
    assert_eq!(scaled.col_width, COL_WIDTH * 2.0);
    assert_eq!(scaled.font_size, FONT_SIZE * 2.0);
    assert_eq!(scaled.rows_per_page, 24);
    assert_eq!(scaled.page_w, PAGE_WIDTH);
    assert_eq!(scaled.page_h, PAGE_HEIGHT);
}

#[test]
fn out_of_range_and_non_finite_scales_fall_back() {
    assert_eq!(PdfWriter::new(options(false, 0.1)).row_height, ROW_HEIGHT * 0.5);
    assert_eq!(PdfWriter::new(options(false, 9.0)).row_height, ROW_HEIGHT * 2.0);
    assert_eq!(PdfWriter::new(options(false, f32::NAN)).row_height, ROW_HEIGHT);
    assert_eq!(
        PdfWriter::new(options(false, f32::INFINITY)).row_height,
        ROW_HEIGHT
    );
}

#[test]
fn rows_flow_onto_continuation_pages() {
    let mut writer = PdfWriter::new(options(false, 1.0));
    writer.add_worksheet("Report", &rows(100)).expect("layout");
    assert_eq!(writer.page_count(), 3);
    let pdf = String::from_utf8_lossy(&writer.finish()).to_string();
    assert_eq!(pdf.matches("/Type /Page ").count(), 3);
    assert!(pdf.contains("/Count 3"), "{pdf}");
    assert!(pdf.contains("rows 1-48"));
    assert!(pdf.contains("rows 97-100"));
    assert!(pdf.ends_with("%%EOF\n"));
}

#[test]
fn blank_rows_are_skipped_so_sparse_sheets_stay_compact() {
    let mut data = rows(3);
    data.insert("0,1".to_string(), cell("   "));
    let mut writer = PdfWriter::new(options(false, 1.0));
    writer.add_worksheet("Sparse", &data).expect("layout");
    assert_eq!(writer.page_count(), 1);
    let pdf = String::from_utf8_lossy(&writer.finish()).to_string();
    assert!(!pdf.contains("(   )"));
    assert!(pdf.contains("(r2)"));
}

#[test]
fn a_worksheet_without_data_emits_no_page() {
    let mut writer = PdfWriter::new(options(false, 1.0));
    writer.add_worksheet("Empty", &HashMap::new()).expect("layout");
    assert_eq!(writer.page_count(), 0);
    let pdf = String::from_utf8_lossy(&writer.finish()).to_string();
    assert!(pdf.contains("/Count 0"), "{pdf}");
    assert!(pdf.contains("%%EOF"));
}

#[test]
fn unparsable_cell_keys_are_ignored() {
    let mut data = HashMap::new();
    data.insert("bogus".to_string(), cell("x"));
    data.insert("1,1".to_string(), cell("y"));
    let mut writer = PdfWriter::new(options(false, 1.0));
    writer.add_worksheet("S", &data).expect("layout");
    let pdf = String::from_utf8_lossy(&writer.finish()).to_string();
    assert!(pdf.contains("(y)"));
    assert!(!pdf.contains("(x)"));
}

#[test]
fn xref_offsets_point_at_every_object() {
    let mut writer = PdfWriter::new(options(false, 1.0));
    writer.add_worksheet("S", &rows(1)).expect("layout");
    let pdf = writer.finish();
    // The header carries non-UTF-8 bytes, so offsets are checked on bytes.
    let marker = b"\nxref\n";
    let xref_at = pdf
        .windows(marker.len())
        .rposition(|window| window == marker)
        .map(|at| at + 1)
        .expect("xref table");
    let table = std::str::from_utf8(&pdf[xref_at..]).expect("xref table is ASCII");
    assert!(table.contains(" 65535 f \n"));

    let entries: Vec<&str> = table
        .lines()
        .filter(|line| line.ends_with(" 00000 n "))
        .collect();
    assert!(entries.len() >= HEADER_OBJECTS as usize);
    for entry in entries {
        let offset: usize = entry[..10].parse().expect("numeric offset");
        assert!(offset < xref_at, "offset {offset} must precede the xref");
        assert!(pdf[offset].is_ascii_digit(), "object header at {offset}");
    }
}

/// #2150 SHEET-10 step 3 — the page content must be a real stream object.
/// A bare `BT ... ET` body is not valid PDF: viewers open the file and render
/// a blank page, and `pdftotext` extracts nothing.
#[test]
fn page_content_is_a_declared_stream() {
    let mut writer = PdfWriter::new(options(false, 1.0));
    writer.add_blank_page("Sheet1");
    let pdf = String::from_utf8_lossy(&writer.finish()).to_string();

    assert!(pdf.contains("/Length "), "content stream needs a /Length: {pdf}");
    assert!(pdf.contains("stream\n"), "content stream needs the stream keyword");
    assert!(pdf.contains("endstream"), "content stream needs endstream");
    let content_start = pdf.find("stream\nBT").expect("content stream body");
    let content_end = pdf.find("\nendstream").expect("endstream marker");
    assert!(content_end > content_start);
}
