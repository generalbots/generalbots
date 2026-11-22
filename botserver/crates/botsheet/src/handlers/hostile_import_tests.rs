//! #2522 — import-boundary evidence for the hostile-file gate on Sheets.
//! Kept in its own file so the handler stays under the 450-line cap.

use super::*;
use botsecurity_core::upload_hostile::macro_strip;
use std::io::{Cursor, Write};

fn zip_with(entries: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, body) in entries {
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated);
        assert!(writer.start_file(*name, options).is_ok(), "zip entry");
        assert!(writer.write_all(body).is_ok(), "zip body");
    }
    match writer.finish() {
        Ok(writer) => writer.into_inner(),
        Err(error) => {
            log::warn!("test zip creation failed: {error}");
            Vec::new()
        }
    }
}

fn rejects(bytes: &[u8], name: &str, policy: &upload_policy::HostileFilePolicy) -> bool {
    let result = audit_hostile_upload(bytes, name, &axum::http::HeaderMap::new(), "test", policy);
    match result {
        Err((status, Json(body))) => {
            status == StatusCode::BAD_REQUEST
                && body["code"] == "error-file-archive-unsafe"
                && body.get("sheet1.xml").is_none()
        }
        Ok(()) => false,
    }
}

fn unscanned_policy() -> upload_policy::HostileFilePolicy {
    upload_policy::HostileFilePolicy {
        require_malware_scanner: false,
        ..Default::default()
    }
}


#[test]
fn a_compression_ratio_bomb_is_refused_before_conversion() {
    let bytes = zip_with(&[("xl/worksheets/sheet1.xml", vec![b' '; 512 * 1024])]);
    assert!(rejects(
        &bytes,
        "bomb.xlsx",
        &upload_policy::HostileFilePolicy::default()
    ));
}

#[test]
fn an_oversized_uncompressed_entry_is_refused_before_conversion() {
    let bytes = zip_with(&[("xl/worksheets/sheet1.xml", vec![b'x'; 8192])]);
    let policy = upload_policy::HostileFilePolicy {
        max_uncompressed_bytes: 1024,
        ..upload_policy::HostileFilePolicy::default()
    };
    assert!(rejects(&bytes, "bomb.xlsx", &policy));
}

#[test]
fn a_macro_part_is_refused_before_conversion() {
    let bytes = zip_with(&[
        ("xl/workbook.xml", b"<workbook/>".to_vec()),
        ("xl/vbaProject.bin", vec![0x01, 0x02, 0x03]),
    ]);
    assert!(rejects(
        &bytes,
        "macro.xlsx",
        &upload_policy::HostileFilePolicy::default()
    ));
}

#[test]
fn an_xml_entity_bomb_is_refused_at_the_import_boundary() {
    // The archive gate sees a harmless ratio; only the XML pass can see the
    // bomb before a spreadsheet parser expands it.
    let xml = b"<!DOCTYPE w [<!ENTITY a \"aaaaaaaaaa\"><!ENTITY b \"&a;&a;&a;&a;\">]><worksheet>&b;</worksheet>";
    let bytes = zip_with(&[("xl/worksheets/sheet1.xml", xml.to_vec())]);
    assert!(rejects(
        &bytes,
        "bomb.xlsx",
        &upload_policy::HostileFilePolicy::default()
    ));
}

#[test]
fn a_clean_workbook_package_reaches_the_parser() {
    let bytes = zip_with(&[
        (
            "xl/worksheets/sheet1.xml",
            b"<worksheet><sheetData/></worksheet>".to_vec(),
        ),
        (
            "xl/_rels/workbook.xml.rels",
            b"<Relationships><Relationship Target=\"../worksheets/sheet1.xml\"/></Relationships>"
                .to_vec(),
        ),
    ]);
    let result = audit_hostile_upload(
        &bytes,
        "clean.xlsx",
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
    );
    assert!(result.is_ok());
}

#[test]
fn a_macro_workbook_is_stripped_and_noticed_before_conversion() {
    let bytes = zip_with(&[
        (
            "xl/worksheets/sheet1.xml",
            b"<worksheet><sheetData/></worksheet>".to_vec(),
        ),
        ("xl/macrosheets/sheet1.xml", b"<macrosheet/>".to_vec()),
    ]);
    let stripped = macro_strip::strip_executable_parts(&bytes)
        .expect("macro package must be sanitizable")
        .expect("a macro part must be found");
    let upload = upload_policy::validate_upload_with_policy(
        UploadClass::SpreadsheetImport,
        "macro.xlsx",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        &stripped.bytes,
        &unscanned_policy(),
    )
    .expect("sanitized workbook must pass the shared gate");
    let result = audit_hostile_upload(
        &stripped.bytes,
        &upload.filename,
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
    );
    assert!(result.is_ok(), "no executable parts may remain");
    let notice =
        crate::import_notice::macro_notice(stripped.removed_parts.clone(), stripped.removed_count);
    assert_eq!(notice.code, "file-macros-removed");
    assert_eq!(notice.count, 1);
    assert!(notice
        .parts
        .iter()
        .any(|part| part == "xl/macrosheets/sheet1.xml"));
}

#[test]
fn a_password_protected_workbook_gets_an_actionable_refusal() {
    let mut bytes = zip_with(&[(
        "xl/worksheets/sheet1.xml",
        b"<worksheet><sheetData/></worksheet>".to_vec(),
    )]);
    bytes[6] |= 0x01;
    if let Some(position) = bytes
        .windows(4)
        .position(|window| window == &[0x50, 0x4b, 0x01, 0x02][..])
    {
        bytes[position + 8] |= 0x01;
    }
    let result = audit_hostile_upload(
        &bytes,
        "protected.xlsx",
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
    );
    match result {
        Err((status, Json(body))) => {
            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert_eq!(body["action"], "remove-password");
            let message = body["message"].as_str().unwrap_or_default();
            assert!(
                message.contains("remove the password"),
                "actionable message: {message}"
            );
        }
        Ok(()) => assert!(false, "an encrypted package must be refused"),
    }
}
