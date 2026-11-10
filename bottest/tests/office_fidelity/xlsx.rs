//! XLSX half of the fidelity contract (#2043/#2044): a retained OOXML package is
//! patched (every part the model cannot re-emit survives), and anything else is
//! an explicit `regenerated` plan — never a silent rebuild.

use std::io::{Cursor, Write};

use botsheet::storage::{
    create_new_spreadsheet, plan_for, produce_xlsx, retains_snapshot, RegenerateReason, XlsxPlan,
};
use botsheet::types::{CellData, Spreadsheet};
use botsheet_core::engine::value::CellValue;

use crate::{changed_parts, part, part_hashes};

fn zip_with(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = zip::ZipWriter::new(&mut buffer);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (name, data) in entries {
            writer.start_file(*name, options).expect("entry");
            writer.write_all(data).expect("write");
        }
        writer.finish().expect("finish");
    }
    buffer.into_inner()
}

/// A workbook with merged cells, conditional formatting, a comment, a chart, a
/// drawing, media, styles and a theme — everything the sheet model cannot
/// re-emit. The part layout matches the crate's own passthrough fixture, which
/// is the shape a real import produces.
fn rich_workbook() -> Vec<u8> {
    let workbook = concat!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
        r#"<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" "#,
        r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
        r#"<sheets><sheet name="Report" sheetId="1" r:id="rId1"/></sheets></workbook>"#
    );
    let rels = concat!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
        r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
        r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>"#,
        r#"<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>"#,
        r#"<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>"#,
        r#"<Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme/theme1.xml"/>"#,
        r#"</Relationships>"#
    );
    let sheet = concat!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
        r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" "#,
        r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
        r#"<sheetData><row r="1"><c r="A1" t="s"><v>0</v></c></row>"#,
        r#"<row r="2"><c r="B2"><v>1</v></c></row></sheetData>"#,
        r#"<mergeCells count="1"><mergeCell ref="A1:B1"/></mergeCells>"#,
        r#"<conditionalFormatting sqref="B2"><cfRule type="cellIs" dxfId="0" priority="1" "#,
        r#"operator="greaterThan"><formula>0</formula></cfRule></conditionalFormatting>"#,
        r#"<drawing r:id="rId1"/></worksheet>"#
    );
    let shared = concat!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
        r#"<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="2" uniqueCount="2">"#,
        r#"<si><t>Item</t></si><si><t>Total</t></si></sst>"#
    );
    let content_types = concat!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
        r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">"#,
        r#"<Default Extension="xml" ContentType="application/xml"/>"#,
        r#"<Default Extension="png" ContentType="image/png"/>"#,
        r#"<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>"#,
        r#"</Types>"#
    );
    zip_with(&[
        ("[Content_Types].xml", content_types.as_bytes()),
        ("_rels/.rels", br#"<Relationships/>"#),
        ("xl/workbook.xml", workbook.as_bytes()),
        ("xl/_rels/workbook.xml.rels", rels.as_bytes()),
        ("xl/worksheets/sheet1.xml", sheet.as_bytes()),
        ("xl/worksheets/_rels/sheet1.xml.rels", br#"<Relationships/>"#),
        ("xl/sharedStrings.xml", shared.as_bytes()),
        ("xl/styles.xml", br#"<styleSheet/>"#),
        ("xl/theme/theme1.xml", br#"<a:theme/>"#),
        (
            "xl/comments1.xml",
            br#"<comments><authors><author>Ana</author></authors><commentList/></comments>"#,
        ),
        ("xl/drawings/drawing1.xml", br#"<xdr:wsDr/>"#),
        ("xl/charts/chart1.xml", br#"<c:chartSpace/>"#),
        ("xl/media/image1.png", b"\x89PNG\r\n\x1a\nfixture"),
    ])
}

fn cell(typed: CellValue) -> CellData {
    CellData {
        value: None,
        typed: Some(typed),
        formula: None,
        style: None,
        format: None,
        note: None,
        locked: None,
        has_comment: None,
        array_formula_id: None,
    }
}

/// The model exactly as an import of [`rich_workbook`] leaves it.
fn workbook_model(original: &[u8]) -> Spreadsheet {
    let mut model = create_new_spreadsheet("test");
    model.source_bytes = Some(original.to_vec());
    model.source_path = Some("drive/report.xlsx".to_string());
    model.source_bucket = Some("office-fidelity.gbai".to_string());
    let worksheet = &mut model.worksheets[0];
    worksheet.name = "Report".to_string();
    worksheet
        .data
        .insert("0,0".to_string(), cell(CellValue::Text("Item".to_string())));
    worksheet
        .data
        .insert("1,1".to_string(), cell(CellValue::Number(1.0)));
    model
}

/// Rule 2+3: producing the export bytes for an untouched sheet patches the
/// retained package and leaves every part hash identical.
#[test]
fn xlsx_untouched_save_keeps_every_part_identical() {
    let original = rich_workbook();
    let sheet = workbook_model(&original);

    let produced = produce_xlsx(&sheet).expect("produce the xlsx");
    assert_eq!(produced.plan, XlsxPlan::Preserve);
    assert_eq!(produced.plan.as_str(), "preserved", "export header value");
    assert_eq!(
        part_hashes(&produced.bytes),
        part_hashes(&original),
        "every part must hash identically"
    );
}

/// Rule 2: one edited cell rewrites only its worksheet part.
#[test]
fn xlsx_single_cell_edit_changes_only_the_worksheet_part() {
    let original = rich_workbook();
    let mut sheet = workbook_model(&original);
    sheet
        .worksheets[0]
        .data
        .insert("1,1".to_string(), cell(CellValue::Number(42.0)));

    let produced = produce_xlsx(&sheet).expect("produce the xlsx");
    assert_eq!(produced.plan, XlsxPlan::Preserve);
    assert_eq!(
        changed_parts(&original, &produced.bytes),
        vec!["xl/worksheets/sheet1.xml".to_string()],
        "only the edited worksheet part may change"
    );
    assert!(
        part(&produced.bytes, "xl/worksheets/sheet1.xml").contains(r#"<c r="B2"><v>42</v></c>"#),
        "the edit must reach the worksheet XML"
    );
}

/// Rule 4: a sheet without a patchable package reports regeneration, and a
/// patch failure is an error that leaves the original untouched.
#[test]
fn xlsx_unpatchable_packages_are_never_rebuilt_silently() {
    let fresh = create_new_spreadsheet("test");
    assert_eq!(
        plan_for(&fresh),
        XlsxPlan::Regenerate(RegenerateReason::NoSnapshot)
    );
    let produced = produce_xlsx(&fresh).expect("regenerate a fresh sheet");
    assert_eq!(produced.plan.as_str(), "regenerated");
    assert!(!retains_snapshot("csv"), "csv has no package to keep");

    let mut legacy = create_new_spreadsheet("test");
    legacy.source_bytes = Some(zip_with(&[("content.xml", b"<x/>")]));
    assert_eq!(
        plan_for(&legacy),
        XlsxPlan::Regenerate(RegenerateReason::NonOoxmlSource)
    );

    // Marked as OOXML (`xl/workbook.xml` present) so the plan says preserve; a
    // model the patch cannot apply must surface as an error, not as a rebuilt
    // workbook.
    let mut broken = workbook_model(&rich_workbook());
    broken
        .worksheets[0]
        .data
        .insert("not-a-cell-key".to_string(), cell(CellValue::Number(7.0)));
    assert_eq!(plan_for(&broken), XlsxPlan::Preserve);
    assert!(
        produce_xlsx(&broken).is_err(),
        "a failed patch must be reported, never replaced by a rebuilt workbook"
    );
}
