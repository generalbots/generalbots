//! Upload policy tests (#2132): magic-byte/extension agreement, filename
//! sanitization, size caps and archive inspection.

use super::*;
use axum::http::HeaderValue;

const PNG: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];
const PDF: &[u8] = b"%PDF-1.7\n1 0 obj\n<<>>\nendobj\n";
const OLE: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0, 0, 0, 0];

fn zip_with(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, body) in entries {
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated);
        writer
            .start_file(*name, options)
            .expect("test zip entry must start");
        std::io::Write::write_all(&mut writer, body).expect("test zip entry must write");
    }
    writer
        .finish()
        .expect("test zip must finish")
        .into_inner()
}

fn docx() -> Vec<u8> {
    zip_with(&[
        ("[Content_Types].xml", b"<Types/>"),
        ("word/document.xml", b"<w:document/>"),
    ])
}

#[test]
fn a_real_png_is_accepted() {
    let upload = validate_upload(UploadClass::ChatAttachment, "photo.png", "image/png", PNG)
        .expect("a real PNG must be accepted");

    assert_eq!(upload.extension, "png");
    assert_eq!(upload.mime, "image/png");
    assert_eq!(upload.size, PNG.len());
}

#[test]
fn a_png_whose_bytes_are_a_pdf_is_rejected() {
    let error = validate_upload(UploadClass::ChatAttachment, "photo.png", "image/png", PDF)
        .expect_err("a PNG extension over PDF bytes must be refused");

    assert!(
        matches!(error, UploadError::TypeMismatch { .. }),
        "expected TypeMismatch, got {error:?}"
    );
    assert_eq!(error.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn an_unknown_extension_is_rejected() {
    let error = validate_upload(UploadClass::Drive, "payload.qqq", "", PNG)
        .expect_err("an unknown extension must be refused");

    assert!(
        matches!(error, UploadError::UnknownExtension { .. }),
        "expected UnknownExtension, got {error:?}"
    );
}

#[test]
fn a_double_extension_is_rejected() {
    // The last extension decides: `.html` is not on any image allowlist.
    let error = validate_upload(UploadClass::ChatAttachment, "a.png.html", "image/png", PNG)
        .expect_err("a double extension must be refused");

    assert!(
        matches!(error, UploadError::ExtensionNotAllowed { .. }),
        "expected ExtensionNotAllowed, got {error:?}"
    );
}

#[test]
fn blocked_executable_extensions_are_rejected_before_any_other_check() {
    for name in ["shell.php", "page.jsp", "setup.exe", "lib.so", "run.sh", "macro.docm"] {
        let error = validate_upload(UploadClass::Drive, name, "", PDF)
            .expect_err("an executable extension must be refused");
        assert!(
            matches!(error, UploadError::ExtensionBlocked { .. }),
            "{name}: expected ExtensionBlocked, got {error:?}"
        );
    }
}

#[test]
fn an_extension_outside_the_endpoint_allowlist_is_rejected() {
    let error = validate_upload(UploadClass::SpreadsheetImport, "clip.mp4", "video/mp4", b"ftyp")
        .expect_err("the spreadsheet importer must refuse video");

    assert!(matches!(error, UploadError::ExtensionNotAllowed { .. }));
}

#[test]
fn a_declared_type_that_contradicts_the_extension_is_rejected() {
    let error = validate_upload(UploadClass::ChatAttachment, "photo.png", "application/pdf", PNG)
        .expect_err("a declared type must agree with the extension");

    assert!(matches!(error, UploadError::TypeMismatch { .. }));
}

#[test]
fn an_oversized_upload_is_refused() {
    let limits = limits_for(UploadClass::Image);
    let big = vec![0u8; limits.max_bytes + 1];

    let error = validate_upload(UploadClass::Image, "avatar.png", "image/png", &big)
        .expect_err("an oversized image must be refused");

    assert!(matches!(error, UploadError::TooLarge { .. }));
    assert_eq!(error.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[test]
fn a_content_length_over_the_cap_is_refused_before_buffering() {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&(max_bytes(UploadClass::ChatAttachment) + 1).to_string())
            .expect("test header"),
    );

    let error = check_declared_length(UploadClass::ChatAttachment, &headers)
        .expect_err("an oversized Content-Length must be refused");

    assert!(matches!(error, UploadError::TooLarge { .. }));
}

#[test]
fn a_content_length_within_the_cap_passes() {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_LENGTH, HeaderValue::from_static("1024"));

    assert!(check_declared_length(UploadClass::ChatAttachment, &headers).is_ok());
    assert!(check_declared_length(UploadClass::ChatAttachment, &HeaderMap::new()).is_ok());
}

#[test]
fn filenames_are_reduced_to_a_safe_basename() {
    let upload = validate_upload(
        UploadClass::ChatAttachment,
        "../../etc/pa\"sswd.png",
        "image/png",
        PNG,
    )
    .expect("a traversing name must be sanitized, not trusted");

    assert_eq!(upload.filename, "passwd.png");
}

#[test]
fn a_hidden_or_empty_filename_is_rejected() {
    for name in ["", ".env", "..", "   "] {
        let error = validate_upload(UploadClass::Drive, name, "", PNG)
            .expect_err("an unsafe filename must be refused");
        assert!(
            matches!(error, UploadError::FilenameInvalid | UploadError::UnknownExtension { .. }),
            "{name}: got {error:?}"
        );
    }
}

#[test]
fn a_docx_container_is_accepted_and_must_be_a_real_zip() {
    let bytes = docx();
    let declared =
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document";

    let upload = validate_upload(UploadClass::DocumentImport, "report.docx", declared, &bytes)
        .expect("a real DOCX must be accepted");
    assert_eq!(upload.extension, "docx");

    // Same extension and declared type, but the bytes are not a zip at all.
    let error = validate_upload(UploadClass::DocumentImport, "report.docx", declared, PDF)
        .expect_err("a DOCX that is not a zip must be refused");
    assert!(matches!(error, UploadError::TypeMismatch { .. }));
}

#[test]
fn a_zip_with_an_escaping_entry_name_is_refused() {
    let bytes = zip_with(&[("../../etc/passwd", b"root")]);

    let error = validate_upload(UploadClass::Drive, "archive.zip", "application/zip", &bytes)
        .expect_err("a zip-slip entry must be refused");

    assert!(
        matches!(error, UploadError::ArchiveUnsafe { .. }),
        "expected ArchiveUnsafe, got {error:?}"
    );
}

#[test]
fn legacy_office_ole_files_are_accepted() {
    let upload = validate_upload(UploadClass::Drive, "old.doc", "application/msword", OLE)
        .expect("an OLE2 document must be accepted");

    assert_eq!(upload.mime, "application/msword");
}

#[test]
fn html_is_only_accepted_where_the_endpoint_asks_for_it() {
    let page = b"<!DOCTYPE html><html><body>hi</body></html>";

    let error = validate_upload(UploadClass::ChatAttachment, "page.html", "text/html", page)
        .expect_err("a chat attachment must never be HTML");
    assert!(matches!(error, UploadError::ExtensionNotAllowed { .. }));

    let upload = validate_upload(UploadClass::DocumentImport, "page.html", "text/html", page)
        .expect("the document importer accepts HTML explicitly");
    assert_eq!(upload.extension, "html");
}

#[test]
fn svg_is_refused_everywhere() {
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>"#;

    let error = validate_upload(UploadClass::Drive, "logo.svg", "image/svg+xml", svg)
        .expect_err("SVG must never be stored through an upload endpoint");
    assert!(matches!(
        error,
        UploadError::UnknownExtension { .. } | UploadError::ExtensionNotAllowed { .. }
    ));
}

#[test]
fn every_class_refuses_the_empty_payload() {
    for class in [
        UploadClass::Drive,
        UploadClass::ChatAttachment,
        UploadClass::KnowledgeBase,
        UploadClass::DocumentImport,
        UploadClass::SpreadsheetImport,
        UploadClass::PresentationImport,
        UploadClass::ProjectImport,
        UploadClass::Evidence,
        UploadClass::VideoMedia,
        UploadClass::Image,
    ] {
        let error = validate_upload(class, "file.png", "image/png", &[])
            .expect_err("an empty payload must be refused");
        assert_eq!(error, UploadError::Empty);
    }
}

#[test]
fn the_rejection_body_carries_a_stable_code_and_a_message() {
    let error = UploadError::TypeMismatch {
        expected: "image/png".to_string(),
        detected: "application/pdf".to_string(),
    };
    let headers = HeaderMap::new();
    let (status, Json(body)) = rejection_json(&error, &headers);

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "error-file-content-mismatch");
    assert!(body["message"].as_str().is_some_and(|m| !m.is_empty()));
}
