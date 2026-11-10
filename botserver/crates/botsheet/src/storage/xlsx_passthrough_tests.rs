//! Round-trip regression tests for the zip-level passthrough (#2043, #2044).
//!
//! Split from `xlsx_passthrough.rs` to keep that module under the file limit.

use super::xlsx_passthrough::merge_into_original;
use std::io::{Cursor, Read, Write};

fn zip_with(names: &[&str]) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buf);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for name in names {
            zip.start_file(*name, options).expect("entry");
            zip.write_all(b"<x/>").expect("write");
        }
        zip.finish().expect("finish");
    }
    buf.into_inner()
}

fn parts(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes.to_vec())).expect("zip");
    let mut out = Vec::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).expect("entry");
        let name = file.name().map(|name| name.to_string()).unwrap_or_default();
        let mut content = Vec::new();
        file.read_to_end(&mut content).expect("read");
        out.push((name, content));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn entry<'a>(parts: &'a [(String, Vec<u8>)], name: &str) -> &'a [u8] {
    parts
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, d)| d.as_slice())
        .unwrap_or_else(|| panic!("missing part {name}"))
}

/// A workbook with merged cells, conditional formatting, a comment, a drawing,
/// a media file and a chart — everything the model cannot re-emit.
fn rich_package() -> Vec<u8> {
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

    zip_with_entries(&[
        ("[Content_Types].xml", content_types.as_bytes()),
        ("_rels/.rels", br#"<Relationships/>"#),
        ("xl/workbook.xml", workbook.as_bytes()),
        ("xl/_rels/workbook.xml.rels", rels.as_bytes()),
        ("xl/worksheets/sheet1.xml", sheet.as_bytes()),
        (
            "xl/worksheets/_rels/sheet1.xml.rels",
            br#"<Relationships/>"#,
        ),
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

fn zip_with_entries(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buf);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (name, data) in entries {
            zip.start_file(*name, options).expect("entry");
            zip.write_all(data).expect("write");
        }
        zip.finish().expect("finish");
    }
    buf.into_inner()
}

/// The model of `rich_package()` exactly as an import leaves it: same sheet
/// name, same cells, nothing else modelled.
fn rich_model(original: &[u8]) -> crate::types::Spreadsheet {
    let mut model = crate::storage::create_new_spreadsheet("test");
    model.source_bytes = Some(original.to_vec());
    model.source_path = Some("drive/report.xlsx".to_string());
    model.source_bucket = Some("gbo".to_string());
    let ws = &mut model.worksheets[0];
    ws.name = "Report".to_string();
    ws.data.insert(
        "0,0".to_string(),
        cell(botsheet_core::engine::value::CellValue::Text(
            "Item".to_string(),
        )),
    );
    ws.data.insert(
        "1,1".to_string(),
        cell(botsheet_core::engine::value::CellValue::Number(1.0)),
    );
    model
}

fn cell(typed: botsheet_core::engine::value::CellValue) -> crate::types::CellData {
    crate::types::CellData {
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

/// #2043 — an ODS/legacy package is kept as a snapshot but must never be
/// patched with xlsx parts; the caller regenerates instead.
#[test]
fn non_xlsx_packages_are_refused() {
    let ods = zip_with(&["content.xml", "META-INF/manifest.xml"]);
    let sheet = crate::storage::create_new_spreadsheet("test");
    let result = merge_into_original(&ods, &sheet);
    assert!(result.is_err(), "an .ods package must not be patched");
}

/// #2043 acceptance — saving an untouched workbook returns a package whose
/// parts are byte-identical to the source, including charts, drawings, media,
/// comments, styles, theme, shared strings and the worksheet itself.
#[test]
fn untouched_save_keeps_every_part_identical() {
    let original = rich_package();
    let model = rich_model(&original);
    let merged = merge_into_original(&original, &model).expect("merge");

    let before = parts(&original);
    let after = parts(&merged);
    assert_eq!(before.len(), after.len(), "part list must not change");
    for ((name, a), (merged_name, b)) in before.iter().zip(after.iter()) {
        assert_eq!(name, merged_name);
        assert_eq!(a, b, "untouched part {name} must stay byte-identical");
    }
}

/// #2043 acceptance — editing one cell changes only that cell's XML; charts,
/// comments, images, conditional formatting and merged cells stay verbatim.
#[test]
fn cell_edit_keeps_every_rich_part() {
    let original = rich_package();
    let mut model = rich_model(&original);
    model.worksheets[0].data.insert(
        "1,1".to_string(),
        cell(botsheet_core::engine::value::CellValue::Number(42.0)),
    );
    let merged = merge_into_original(&original, &model).expect("merge");

    let before = parts(&original);
    let after = parts(&merged);
    let worksheet_before = entry(&before, "xl/worksheets/sheet1.xml");
    let worksheet_after = entry(&after, "xl/worksheets/sheet1.xml");
    let expected = String::from_utf8_lossy(worksheet_before)
        .replace(r#"<c r="B2"><v>1</v></c>"#, r#"<c r="B2"><v>42</v></c>"#);
    assert_eq!(
        String::from_utf8_lossy(worksheet_after).to_string(),
        expected,
        "only the edited cell may change"
    );

    for (name, a) in before.iter() {
        if name == "xl/worksheets/sheet1.xml" {
            continue;
        }
        assert_eq!(
            a.as_slice(),
            entry(&after, name),
            "part {name} must survive"
        );
    }

    let text = String::from_utf8_lossy(worksheet_after);
    assert!(
        text.contains("<conditionalFormatting"),
        "conditional formats kept"
    );
    assert!(
        text.contains(r#"<mergeCell ref="A1:B1"/>"#),
        "merged cells kept"
    );
    assert!(
        text.contains(r#"<drawing r:id="rId1"/>"#),
        "drawing anchor kept"
    );
}
