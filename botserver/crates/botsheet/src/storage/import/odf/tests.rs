use super::parser::{ensure_capacity, parse_repeat};
use super::*;
use botsheet_core::engine::value::CellValue;
use std::io::{Cursor, Write};

fn ods_zip(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, body) in entries {
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated);
        assert!(writer.start_file(*name, options).is_ok(), "zip entry");
        assert!(writer.write_all(body).is_ok(), "zip body");
    }
    writer
        .finish()
        .map(|writer| writer.into_inner())
        .unwrap_or_default()
}

fn document(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:draw="urn:oasis:names:tc:opendocument:xmlns:drawing:1.0" office:version="1.3">
<office:body><office:spreadsheet>{body}</office:spreadsheet></office:body>
</office:document-content>"#
    )
}

fn ods_package(content: &str) -> Vec<u8> {
    ods_zip(&[
        ("mimetype", ODF_SPREADSHEET_MIME.as_bytes().to_vec()),
        ("content.xml", content.as_bytes().to_vec()),
    ])
}

fn value(sheet: &Worksheet, key: &str) -> String {
    sheet
        .data
        .get(key)
        .and_then(|cell| cell.value.clone())
        .unwrap_or_default()
}

#[test]
fn detects_a_genuine_ods_package() {
    let bytes = ods_package(&document(""));
    assert!(is_odf_spreadsheet(&bytes));
    assert!(!is_odf_spreadsheet(b"not a zip"));
}

#[test]
fn refuses_a_non_spreadsheet_odf_package() {
    let bytes = ods_zip(&[
        (
            "mimetype",
            b"application/vnd.oasis.opendocument.text".to_vec(),
        ),
        ("content.xml", document("").into_bytes()),
    ]);
    assert!(!is_odf_spreadsheet(&bytes));
    let error = parse_ods_to_worksheets(&bytes).expect_err("ODT must be refused");
    assert!(error.contains("not an ODF spreadsheet"), "{error}");
}

#[test]
fn requires_a_content_xml_part() {
    let bytes = ods_zip(&[("mimetype", ODF_SPREADSHEET_MIME.as_bytes().to_vec())]);
    let error = parse_ods_to_worksheets(&bytes).expect_err("missing part");
    assert!(error.contains("content.xml not found"), "{error}");
}

#[test]
fn parses_cell_values_and_typed_numbers() {
    let body = r#"
<table:table table:name="Data">
 <table:table-row>
  <table:table-cell office:value-type="string"><text:p>Name</text:p></table:table-cell>
  <table:table-cell office:value-type="string"><text:p>Score</text:p></table:table-cell>
 </table:table-row>
 <table:table-row>
  <table:table-cell office:value-type="string"><text:p>Alice</text:p></table:table-cell>
  <table:table-cell office:value-type="float" office:value="42.5"><text:p>42.5</text:p></table:table-cell>
 </table:table-row>
</table:table>"#;
    let sheets = parse_ods_to_worksheets(&ods_package(&document(body))).expect("parse");
    assert_eq!(sheets.len(), 1);
    assert_eq!(sheets[0].name, "Data");
    assert_eq!(value(&sheets[0], "0,0"), "Name");
    assert_eq!(value(&sheets[0], "0,1"), "Score");
    assert_eq!(value(&sheets[0], "1,0"), "Alice");
    assert_eq!(value(&sheets[0], "1,1"), "42.5");
    assert_eq!(
        sheets[0]
            .data
            .get("1,1")
            .and_then(|cell| cell.typed.clone()),
        Some(CellValue::Number(42.5))
    );
}

#[test]
fn expands_repeated_columns_and_rows() {
    let body = r#"
<table:table table:name="Reps">
 <table:table-row>
  <table:table-cell office:value-type="string"><text:p>A</text:p></table:table-cell>
  <table:table-cell table:number-columns-repeated="3" office:value-type="string"><text:p>B</text:p></table:table-cell>
 </table:table-row>
 <table:table-row table:number-rows-repeated="2">
  <table:table-cell office:value-type="string"><text:p>C</text:p></table:table-cell>
 </table:table-row>
</table:table>"#;
    let sheets = parse_ods_to_worksheets(&ods_package(&document(body))).expect("parse");
    let sheet = &sheets[0];
    assert_eq!(value(sheet, "0,0"), "A");
    assert_eq!(value(sheet, "0,1"), "B");
    assert_eq!(value(sheet, "0,2"), "B");
    assert_eq!(value(sheet, "0,3"), "B");
    assert_eq!(value(sheet, "1,0"), "C");
    assert_eq!(value(sheet, "2,0"), "C");
    assert!(
        value(sheet, "3,0").is_empty(),
        "the repeat must stop after two rows"
    );
}

#[test]
fn decodes_builtin_entities_in_cell_text() {
    let body = r#"
<table:table table:name="Entities"><table:table-row>
 <table:table-cell office:value-type="string"><text:p>A &amp; B</text:p></table:table-cell>
</table:table-row></table:table>"#;
    let sheets = parse_ods_to_worksheets(&ods_package(&document(body))).expect("parse");
    assert_eq!(value(&sheets[0], "0,0"), "A & B");
}

#[test]
fn refuses_an_embedded_object_part() {
    let bytes = ods_zip(&[
        ("mimetype", ODF_SPREADSHEET_MIME.as_bytes().to_vec()),
        ("content.xml", document("").into_bytes()),
        ("Object 1/content.xml", b"<object/>".to_vec()),
    ]);
    let error = parse_ods_to_worksheets(&bytes).expect_err("embedded object must be refused");
    assert!(error.contains("embedded objects"), "{error}");
}

#[test]
fn refuses_an_embedded_object_element() {
    let body = r#"
<table:table table:name="Data">
 <draw:frame><draw:object xlink:href="./Object 1"/></draw:frame>
 <table:table-row><table:table-cell office:value-type="string"><text:p>A</text:p></table:table-cell></table:table-row>
</table:table>"#;
    let error = parse_ods_to_worksheets(&ods_package(&document(body)))
        .expect_err("embedded object element must be refused");
    assert!(error.contains("embedded objects"), "{error}");
}

#[test]
fn refuses_a_doctype_declaration() {
    let content =
        "<!DOCTYPE x [<!ENTITY xxe SYSTEM \"file:///etc/passwd\">]><office:document-content/>";
    let error =
        parse_ods_to_worksheets(&ods_package(content)).expect_err("DOCTYPE must be refused");
    assert!(error.contains("DOCTYPE"), "{error}");
}

#[test]
fn refuses_custom_xml_entities() {
    let body = r#"
<table:table table:name="Data"><table:table-row>
 <table:table-cell office:value-type="string"><text:p>&xxe;</text:p></table:table-cell>
</table:table-row></table:table>"#;
    let error = parse_ods_to_worksheets(&ods_package(&document(body)))
        .expect_err("custom entity must be refused");
    assert!(error.contains("custom XML entities"), "{error}");
}

#[test]
fn an_empty_document_yields_one_empty_sheet() {
    let sheets = parse_ods_to_worksheets(&ods_package(&document(""))).expect("parse");
    assert_eq!(sheets.len(), 1);
    assert_eq!(sheets[0].name, "Sheet1");
    assert!(sheets[0].data.is_empty());
}

#[test]
fn repeats_and_capacity_are_bounded() {
    assert_eq!(parse_repeat(None), 1);
    assert_eq!(parse_repeat(Some("0")), 1);
    assert_eq!(parse_repeat(Some("4")), 4);
    assert_eq!(parse_repeat(Some("9999999999")), MAX_REPEAT);
    assert!(ensure_capacity(MAX_CELLS_PER_SHEET, 0, 1).is_err());
    assert!(ensure_capacity(MAX_CELLS_PER_SHEET - 1, 0, 1).is_ok());
}

#[test]
fn refuses_a_package_with_too_many_entries() {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Stored);
    for index in 0..=MAX_ODS_ENTRIES {
        assert!(
            writer.start_file(format!("f{index}"), options).is_ok(),
            "entry {index}"
        );
    }
    let bytes = writer.finish().map(|w| w.into_inner()).expect("zip");
    assert!(!is_odf_spreadsheet(&bytes));
    let error = parse_ods_to_worksheets(&bytes).expect_err("entry cap must be refused");
    assert!(error.contains("unsupported number of entries"), "{error}");
}

/// Overwrite the central-directory "uncompressed size" field (PK\x01\x02 +
/// 24) of one entry. `ZipFile::size()` reads that field without inflating the
/// payload, so a zip-bomb declaration can be modeled without allocating it.
fn patch_cd_uncompressed_size(archive: &mut [u8], entry_name: &str, declared_size: u32) -> bool {
    let mut offset = 0usize;
    while offset + 46 <= archive.len() {
        if archive[offset..offset + 4] == [0x50, 0x4b, 0x01, 0x02] {
            let name_len = u16::from_le_bytes([archive[offset + 28], archive[offset + 29]]) as usize;
            let extra_len = u16::from_le_bytes([archive[offset + 30], archive[offset + 31]]) as usize;
            let comment_len =
                u16::from_le_bytes([archive[offset + 32], archive[offset + 33]]) as usize;
            let name_start = offset + 46;
            let name_end = name_start.saturating_add(name_len);
            if name_end > archive.len() {
                return false;
            }
            if &archive[name_start..name_end] == entry_name.as_bytes() {
                archive[offset + 24..offset + 28].copy_from_slice(&declared_size.to_le_bytes());
                return true;
            }
            offset = name_end
                .saturating_add(extra_len)
                .saturating_add(comment_len);
        } else {
            offset += 1;
        }
    }
    false
}

#[test]
fn refuses_a_package_exceeding_the_uncompressed_size_limit() {
    let mut bytes = ods_package(&document(""));
    assert!(patch_cd_uncompressed_size(
        &mut bytes,
        "mimetype",
        (MAX_TOTAL_UNCOMPRESSED_BYTES + 1) as u32
    ));
    let error = parse_ods_to_worksheets(&bytes).expect_err("uncompressed cap must be refused");
    assert!(error.contains("uncompressed size limit"), "{error}");
}

#[test]
fn refuses_a_content_xml_exceeding_the_size_limit() {
    let mut bytes = ods_package(&document(""));
    assert!(patch_cd_uncompressed_size(
        &mut bytes,
        "content.xml",
        (MAX_CONTENT_XML_BYTES + 1) as u32
    ));
    let error = parse_ods_to_worksheets(&bytes).expect_err("content.xml cap must be refused");
    assert!(error.contains("content.xml exceeds the size limit"), "{error}");
}

#[test]
fn refuses_an_overlong_mimetype() {
    let overlong = format!(
        "{}{}",
        ODF_SPREADSHEET_MIME,
        "x".repeat(MAX_MIMETYPE_BYTES as usize + 1)
    );
    let bytes = ods_zip(&[
        ("mimetype", overlong.into_bytes()),
        ("content.xml", document("").into_bytes()),
    ]);
    assert!(!is_odf_spreadsheet(&bytes));
    let error = parse_ods_to_worksheets(&bytes).expect_err("overlong mimetype must be refused");
    assert!(error.contains("not an ODF spreadsheet"), "{error}");
}

#[test]
fn refuses_too_many_worksheets() {
    let body: String = (0..=MAX_SHEETS)
        .map(|index| format!("<table:table table:name=\"S{index}\"/>"))
        .collect();
    let error = parse_ods_to_worksheets(&ods_package(&document(&body)))
        .expect_err("sheet cap must be refused");
    assert!(error.contains("too many worksheets"), "{error}");
}

#[test]
fn clamps_cell_text_to_the_text_limit() {
    let body = format!(
        "<table:table table:name=\"T\"><table:table-row><table:table-cell office:value-type=\"string\"><text:p>{}</text:p></table:table-cell></table:table-row></table:table>",
        "a".repeat(MAX_TEXT_CHARS + 512)
    );
    let sheets = parse_ods_to_worksheets(&ods_package(&document(&body))).expect("parse");
    let text = value(&sheets[0], "0,0");
    assert_eq!(text.len(), MAX_TEXT_CHARS, "cell text must be clamped");
    assert!(text.chars().all(|character| character == 'a'));
}

#[test]
fn refuses_a_self_closing_embedded_object_element() {
    let body = r#"
<table:table table:name="Data">
 <draw:frame><draw:object-ole/></draw:frame>
 <table:table-row><table:table-cell office:value-type="string"><text:p>A</text:p></table:table-cell></table:table-row>
</table:table>"#;
    let error = parse_ods_to_worksheets(&ods_package(&document(body)))
        .expect_err("self-closing embedded object must be refused");
    assert!(error.contains("embedded objects"), "{error}");
}
