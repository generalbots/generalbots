//! #2522 — import-boundary evidence for the hostile-file gate on Docs.
//! Kept in its own file so the handler stays under the 450-line cap.

use super::*;
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

fn is_sanitized_rejection(
    result: Result<(), (StatusCode, Json<serde_json::Value>)>,
    code: &str,
) -> bool {
    match result {
        Err((status, Json(body))) => {
            status == StatusCode::BAD_REQUEST
                && body["code"] == code
                && body.get("document.xml").is_none()
        }
        Ok(()) => false,
    }
}

#[test]
fn a_compression_ratio_bomb_is_refused_before_conversion() {
    // ~512 KiB of spaces deflates to a few hundred bytes: under every byte
    // cap but far above the compression-ratio limit.
    let bytes = zip_with(&[("word/document.xml", vec![b' '; 512 * 1024])]);
    let result = audit_hostile_upload(
        &bytes,
        "bomb.docx",
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
    );
    assert!(is_sanitized_rejection(result, "error-file-archive-unsafe"));
}

#[test]
fn an_oversized_uncompressed_entry_is_refused_before_conversion() {
    let bytes = zip_with(&[("word/document.xml", vec![b'x'; 8192])]);
    let policy = upload_policy::HostileFilePolicy {
        max_uncompressed_bytes: 1024,
        ..upload_policy::HostileFilePolicy::default()
    };
    let result = audit_hostile_upload(
        &bytes,
        "bomb.docx",
        &axum::http::HeaderMap::new(),
        "test",
        &policy,
    );
    assert!(is_sanitized_rejection(result, "error-file-archive-unsafe"));
}

#[test]
fn a_macro_part_is_refused_before_conversion() {
    let bytes = zip_with(&[
        ("word/document.xml", b"<w:document/>".to_vec()),
        ("word/vbaProject.bin", vec![0x01, 0x02, 0x03]),
    ]);
    let result = audit_hostile_upload(
        &bytes,
        "macro.docx",
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
    );
    assert!(is_sanitized_rejection(result, "error-file-archive-unsafe"));
}

#[test]
fn a_plain_format_is_not_mistaken_for_a_zip_package() {
    let result = audit_hostile_upload(
        b"%PDF-1.7 not a zip",
        "report.pdf",
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
    );
    assert!(result.is_ok());
}

#[test]
fn an_xml_entity_bomb_is_refused_at_the_import_boundary() {
    // The archive gate sees a harmless ratio; only the XML pass can see the
    // billion-laughs payload before docx_rs expands it.
    let xml = b"<!DOCTYPE d [<!ENTITY a \"aaaaaaaaaa\"><!ENTITY b \"&a;&a;&a;&a;\">]><d>&b;</d>";
    let bytes = zip_with(&[("word/document.xml", xml.to_vec())]);
    let result = audit_hostile_upload(
        &bytes,
        "bomb.docx",
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
    );
    assert!(is_sanitized_rejection(result, "error-file-archive-unsafe"));
}

#[test]
fn a_clean_document_package_reaches_the_converter() {
    let bytes = zip_with(&[
        (
            "word/document.xml",
            b"<w:document><w:body/></w:document>".to_vec(),
        ),
        (
            "word/_rels/document.xml.rels",
            b"<Relationships><Relationship Target=\"../media/image1.png\"/></Relationships>"
                .to_vec(),
        ),
    ]);
    let result = audit_hostile_upload(
        &bytes,
        "clean.docx",
        &axum::http::HeaderMap::new(),
        "test",
        &upload_policy::HostileFilePolicy::default(),
    );
    assert!(result.is_ok());
}
