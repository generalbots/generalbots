//! HTML table export expectations.

use super::support_tests::{cell, sheet, ws};
use super::*;
use botsheet_core::types::CellStyle;

#[test]
fn document_scaffold_wraps_the_tables() {
    let html = export_to_html(&sheet("Budget", vec![ws("Q1", &[(0, 0, "x")])]));
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<title>Budget</title>"));
    assert!(html.contains("<h2>Q1</h2>"));
    assert!(html.ends_with("</body></html>"));
}

#[test]
fn markup_in_values_and_names_is_escaped() {
    let book = sheet("<b>&Co</b>", vec![ws("<i>S</i>", &[(0, 0, "<script>alert('x')</script>")])]);
    let html = export_to_html(&book);
    assert!(html.contains("&lt;b&gt;&amp;Co&lt;/b&gt;"));
    assert!(html.contains("&lt;i&gt;S&lt;/i&gt;"));
    assert!(html.contains("&lt;script&gt;alert(&#39;x&#39;)&lt;/script&gt;"));
    assert!(!html.contains("<script>"));
}

#[test]
fn header_row_lists_column_letters_and_body_lists_row_numbers() {
    let html = export_to_html(&sheet("Book", vec![ws("S", &[(0, 0, "a"), (1, 1, "b")])]));
    assert!(html.contains("<thead><tr><th></th><th>A</th><th>B</th></tr></thead>"));
    assert!(html.contains("<tr><td><strong>1</strong></td>"));
    assert!(html.contains("<tr><td><strong>2</strong></td>"));
}

#[test]
fn cell_styles_become_inline_css() {
    let mut worksheet = ws("S", &[(0, 0, "styled")]);
    worksheet.data.insert(
        "0,0".to_string(),
        botsheet_core::types::CellData {
            style: Some(CellStyle {
                background: Some("#ff0".to_string()),
                color: Some("red".to_string()),
                font_weight: Some("700".to_string()),
                text_align: Some("right".to_string()),
                ..Default::default()
            }),
            ..cell("styled")
        },
    );
    let html = export_to_html(&sheet("Book", vec![worksheet]));
    assert!(html.contains(
        "<td style=\"background-color:#ff0;color:red;font-weight:700;text-align:right;\">styled</td>"
    ));
}

#[test]
fn empty_workbook_still_produces_a_closed_document() {
    let html = export_to_html(&sheet("Book", vec![]));
    assert!(html.ends_with("</body></html>"));
    assert!(!html.contains("<table>"));
}
