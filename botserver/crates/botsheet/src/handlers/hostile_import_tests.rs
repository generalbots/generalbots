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

fn rejection_body(
    bytes: &[u8],
    name: &str,
    policy: &upload_policy::HostileFilePolicy,
) -> Option<serde_json::Value> {
    match audit_hostile_upload(bytes, name, &axum::http::HeaderMap::new(), "test", policy) {
        Err((StatusCode::BAD_REQUEST, Json(body))) => Some(body),
        _ => None,
    }
}

fn rejects(bytes: &[u8], name: &str, policy: &upload_policy::HostileFilePolicy) -> bool {
    rejection_body(bytes, name, policy).is_some_and(|body| {
        body["code"] == "error-file-archive-unsafe" && body.get("sheet1.xml").is_none()
    })
}

fn unscanned_policy() -> upload_policy::HostileFilePolicy {
    upload_policy::HostileFilePolicy {
        require_malware_scanner: false,
        ..Default::default()
    }
}

/// Attacker-controlled markers that must never echo back in a 400 body: part
/// names, upload names, URLs, filesystem paths, entity text and the gate's
/// Rust debug types. The shared `rejection_json` only interpolates fixed
/// reason/kind strings, so none of these may appear in the serialized response.
fn leaks_internals(body: &serde_json::Value, secrets: &[&str]) -> bool {
    let rendered = body.to_string();
    rendered.contains("ArchiveUnsafe")
        || rendered.contains("RelationshipTargetUnsafe")
        || rendered.contains("UploadError")
        || secrets.iter().any(|secret| rendered.contains(secret))
}

/// The sanitized 400 asserted by `rejects`, plus an explicit check that the
/// payload's own internals were not leaked back to the client.
fn sanitized_rejection_without_leaking(
    bytes: &[u8],
    name: &str,
    policy: &upload_policy::HostileFilePolicy,
    secrets: &[&str],
) -> bool {
    match rejection_body(bytes, name, policy) {
        Some(body) => {
            !leaks_internals(&body, secrets)
                && body["code"] == "error-file-archive-unsafe"
                && body.get("sheet1.xml").is_none()
        }
        None => false,
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
            let message = match body["message"].as_str() {
                Some(message) => message,
                None => "",
            };
            assert!(
                message.contains("remove the password"),
                "actionable message: {message}"
            );
        }
        Ok(()) => assert!(false, "an encrypted package must be refused"),
    }
}

#[test]
fn an_extreme_compression_ratio_bomb_is_refused_without_leaking_internals() {
    // 1 MiB of one repeated byte deflates to a few KiB: the uncompressed cap
    // still passes, but the ratio is an order of magnitude over the ceiling.
    let bytes = zip_with(&[("xl/worksheets/sheet1.xml", vec![b'A'; 1024 * 1024])]);
    assert!(
        sanitized_rejection_without_leaking(
            &bytes,
            "ratio-bomb.xlsx",
            &unscanned_policy(),
            &["xl/worksheets/sheet1.xml", "ratio-bomb.xlsx", "AAAAAAAA"],
        ),
        "the ratio gate must return a localized 400 with no archive internals"
    );
}

#[test]
fn an_oversized_xml_entity_expansion_is_refused_without_leaking_internals() {
    // Three oversized entity payloads: a billion-laughs DOCTYPE and an XXE
    // file read (both caught by the archive pass), plus a body of custom
    // entity references past the streaming cap with no DOCTYPE at all (caught
    // only by the second pass at this boundary).
    let cases: &[(&str, Vec<u8>)] = &[
        (
            "billion-laughs",
            b"<!DOCTYPE lolz [<!ENTITY lol \"lololololol\"><!ENTITY lol2 \"&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;\"><!ENTITY lol3 \"&lol2;&lol2;&lol2;&lol2;&lol2;\">]><worksheet>&lol3;</worksheet>".to_vec(),
        ),
        (
            "xxe-file-read",
            b"<!DOCTYPE w [<!ENTITY xxe SYSTEM \"file:///etc/passwd\">]><worksheet>&xxe;</worksheet>".to_vec(),
        ),
        (
            "oversized-entity-refs",
            format!("<worksheet>{}</worksheet>", "&x;".repeat(8192)).into_bytes(),
        ),
    ];
    for (label, xml) in cases {
        let bytes = zip_with(&[("xl/worksheets/sheet1.xml", xml.clone())]);
        assert!(
            sanitized_rejection_without_leaking(
                &bytes,
                "entity-bomb.xlsx",
                &unscanned_policy(),
                &[
                    "xl/worksheets/sheet1.xml",
                    "/etc/passwd",
                    "lololololol",
                    "&x;"
                ],
            ),
            "{label}: oversized entity expansion must be a sanitized 400"
        );
    }
}

#[test]
fn an_ssrf_relationship_target_is_refused_without_leaking_internals() {
    // External relationships are the exfiltration surface: loopback admin
    // endpoints, cloud metadata, file reads, protocol-relative URLs and an
    // XML-entity-obfuscated slash must all fail closed without echoing the
    // target back to the client.
    let cases: &[(&str, &str)] = &[
        ("loopback", r"http://127.0.0.1:8080/admin"),
        (
            "cloud-metadata",
            r"http://169.254.169.254/latest/meta-data/",
        ),
        ("file-scheme", r"file:///etc/passwd"),
        ("protocol-relative", r"//169.254.169.254/latest/meta-data/"),
        ("entity-obfuscated", r"http:/&#47;127.0.0.1:8080/admin"),
    ];
    for (label, target) in cases {
        let rels = format!(
            "<Relationships><Relationship Target=\"{target}\" TargetMode=\"External\"/></Relationships>"
        );
        let bytes = zip_with(&[("xl/_rels/workbook.xml.rels", rels.into_bytes())]);
        assert!(
            sanitized_rejection_without_leaking(
                &bytes,
                "ssrf.xlsx",
                &unscanned_policy(),
                &[
                    "127.0.0.1",
                    "169.254.169.254",
                    "/etc/passwd",
                    "xl/_rels/workbook.xml.rels",
                    target,
                ],
            ),
            "{label}: relationship target {target} must be a sanitized 400"
        );
    }
}
