//! Attachment guards: the type allow-list, the filename sanitizer and the
//! extension fallback.

use super::{content_type_allowed, detect_from_extension, is_image, sanitize_filename};
use std::str::FromStr;

#[test]
fn every_documented_type_is_accepted() {
    for content_type in [
        "image/png",
        "image/jpeg",
        "image/gif",
        "image/webp",
        "image/svg+xml",
        "application/pdf",
        "application/msword",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "application/vnd.ms-excel",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "application/vnd.ms-powerpoint",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "text/plain",
        "text/csv",
        "text/markdown",
        "application/zip",
        "application/gzip",
        "audio/mpeg",
        "audio/ogg",
        "audio/wav",
        "video/mp4",
        "video/webm",
    ] {
        assert!(content_type_allowed(content_type), "{content_type}");
    }
}

#[test]
fn executable_and_markup_types_are_refused() {
    for content_type in [
        "application/x-msdownload",
        "application/x-sh",
        "application/octet-stream",
        "text/html",
        "text/javascript",
        "image/svg",
        "",
        " ",
    ] {
        assert!(!content_type_allowed(content_type), "{content_type}");
    }
}

#[test]
fn the_allow_list_ignores_client_casing_and_padding() {
    assert!(content_type_allowed("IMAGE/PNG"));
    assert!(content_type_allowed("Application/PDF"));
    assert!(content_type_allowed(" text/csv "));
    assert!(!content_type_allowed("TEXT/HTML"));
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
fn every_detected_type_passes_the_allow_list() {
    for name in ["a.png", "a.pdf", "a.docx", "a.csv", "a.mp4", "a.zip"] {
        let content_type = detect_from_extension(name).expect("detected");
        assert!(content_type_allowed(&content_type), "{name}");
    }
}
