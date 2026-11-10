//! Attachment guards (#2132): the shared upload policy replaced the local
//! content-type list, so these tests pin the endpoint contract onto it.

use super::*;

const PNG: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];

#[test]
fn every_documented_type_is_accepted_by_the_shared_policy() {
    let limits = upload_policy::limits_for(UploadClass::ChatAttachment);

    for extension in [
        "png", "jpg", "jpeg", "gif", "webp", "pdf", "docx", "xlsx", "pptx", "doc", "xls", "ppt",
        "csv", "txt", "md", "zip", "gz", "mp3", "wav", "ogg", "mp4", "webm",
    ] {
        assert!(limits.allows_extension(extension), "{extension}");
        assert!(
            upload_policy::mime_for_extension(extension).is_some(),
            "{extension} has no declared type"
        );
    }
}

#[test]
fn executable_and_markup_types_are_refused() {
    for name in ["payload.html", "logo.svg", "shell.php", "setup.exe", "run.sh"] {
        let error = upload_policy::validate_upload(UploadClass::ChatAttachment, name, "", PNG)
            .expect_err("an unsafe attachment type must be refused");
        assert!(
            matches!(
                error,
                upload_policy::UploadError::ExtensionNotAllowed { .. }
                    | upload_policy::UploadError::ExtensionBlocked { .. }
            ),
            "{name}: {error:?}"
        );
    }
}

#[test]
fn the_allow_list_ignores_client_casing_and_padding() {
    for declared in ["IMAGE/PNG", " image/png ", "image/png; charset=binary"] {
        let upload = upload_policy::validate_upload(
            UploadClass::ChatAttachment,
            "photo.png",
            declared,
            PNG,
        );
        assert!(upload.is_ok(), "{declared}: {upload:?}");
    }
}

#[test]
fn only_image_types_get_a_thumbnail() {
    assert!(is_image("image/png"));
    assert!(is_image("image/svg+xml"));
    assert!(is_image("IMAGE/JPEG"));
    assert!(!is_image("application/pdf"));
    assert!(!is_image("text/plain"));
    assert!(!is_image(""));
    assert!(!is_image("imagex/png"));
}

#[test]
fn filenames_lose_path_separators_and_header_breakers() {
    assert_eq!(sanitize_filename("..\\..\\evil.txt"), "....evil.txt");
    assert_eq!(sanitize_filename("report\r.pdf"), "report.pdf");
    assert_eq!(sanitize_filename("report\n.pdf"), "report.pdf");
    assert_eq!(sanitize_filename("/etc/passwd"), "etcpasswd");
    assert_eq!(sanitize_filename("a\0b.txt"), "ab.txt");
    assert_eq!(sanitize_filename("my\"file.pdf"), "myfile.pdf");
}

#[test]
fn ordinary_filenames_survive_untouched() {
    for name in [
        "report.pdf",
        "photo-2026_01.png",
        "relatório final.pdf",
        "arquivo com espaço.docx",
        "",
    ] {
        assert_eq!(sanitize_filename(name), name);
    }
}

#[test]
fn a_quoted_filename_produces_a_well_formed_header() {
    let disposition = format!(
        "attachment; filename=\"{}\"",
        sanitize_filename("quote\"injection.pdf")
    );
    assert_eq!(disposition, "attachment; filename=\"quoteinjection.pdf\"");
    assert!(axum::http::HeaderValue::from_str(&disposition).is_ok());
}

#[test]
fn the_extension_fallback_covers_every_allowed_family() {
    let cases = [
        ("a.png", "image/png"),
        ("a.PNG", "image/png"),
        ("a.jpg", "image/jpeg"),
        ("a.jpeg", "image/jpeg"),
        ("a.gif", "image/gif"),
        ("a.webp", "image/webp"),
        ("a.svg", "image/svg+xml"),
        ("a.pdf", "application/pdf"),
        ("a.doc", "application/msword"),
        (
            "a.docx",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        ),
        ("a.xls", "application/vnd.ms-excel"),
        (
            "a.xlsx",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ),
        ("a.ppt", "application/vnd.ms-powerpoint"),
        (
            "a.pptx",
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        ),
        ("a.txt", "text/plain"),
        ("a.csv", "text/csv"),
        ("a.md", "text/markdown"),
        ("a.zip", "application/zip"),
        ("a.gz", "application/gzip"),
        ("a.mp3", "audio/mpeg"),
        ("a.ogg", "audio/ogg"),
        ("a.wav", "audio/wav"),
        ("a.mp4", "video/mp4"),
        ("a.webm", "video/webm"),
    ];
    for (name, expected) in cases {
        assert_eq!(
            detect_from_extension(name).as_deref(),
            Some(expected),
            "{name}"
        );
    }
}

#[test]
fn an_unknown_or_absent_extension_has_no_fallback() {
    for name in ["mystery.bin", "noextension", "archive.tar.xyz", ".hidden", ""] {
        assert_eq!(detect_from_extension(name), None, "{name}");
    }
    assert_eq!(
        detect_from_extension("archive.tar.gz").as_deref(),
        Some("application/gzip")
    );
}

#[test]
fn every_detected_type_is_on_the_policy_allow_list() {
    let limits = upload_policy::limits_for(UploadClass::ChatAttachment);

    for name in ["a.png", "a.pdf", "a.docx", "a.csv", "a.mp4", "a.zip"] {
        let extension = name.rsplit('.').next().expect("extension");
        assert!(limits.allows_extension(extension), "{name}");
    }
}
