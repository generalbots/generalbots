use std::sync::LazyLock;

const MAX_FILE_SIZE: usize = 100 * 1024 * 1024;

static MAGIC_BYTES: LazyLock<Vec<(&'static [u8], &'static str)>> = LazyLock::new(|| {
    vec![
        (&[0xFF, 0xD8, 0xFF], "image/jpeg"),
        (&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A], "image/png"),
        (b"GIF87a", "image/gif"),
        (b"GIF89a", "image/gif"),
        (b"BM", "image/bmp"),
        (b"II*\x00", "image/tiff"),
        (b"MM\x00*", "image/tiff"),
        (b"%PDF-", "application/pdf"),
        // Legacy Microsoft Office (doc/xls/ppt) and other OLE2 compound files.
        (
            &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1],
            "application/x-ole-storage",
        ),
        (b"PK\x03\x04", "application/zip"),
        (b"PK\x05\x06", "application/zip"),
        (b"PK\x07\x08", "application/zip"),
        (b"Rar!\x1A\x07", "application/vnd.rar"),
        (&[0x1F, 0x8B, 0x08], "application/gzip"),
        (b"BZh", "application/x-bzip2"),
        (&[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00], "application/x-xz"),
        (&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C], "application/7z"),
        (b"ftyp", "video/mp4"),
        (&[0x1A, 0x45, 0xDF, 0xA3], "video/webm"),
        (&[0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62, 0xCE, 0x6C], "video/asf"),
        (&[0x00, 0x00, 0x00, 0x1C, 0x66, 0x74, 0x79, 0x70], "video/mp4"),
        (&[0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70], "video/mp4"),
        (b"ID3", "audio/mpeg"),
        (&[0xFF, 0xFB], "audio/mpeg"),
        (&[0xFF, 0xFA], "audio/mpeg"),
        (&[0xFF, 0xF3], "audio/mpeg"),
        (&[0xFF, 0xF2], "audio/mpeg"),
        (b"OggS", "audio/ogg"),
        (b"fLaC", "audio/flac"),
        (&[0x00, 0x00, 0x00, 0x14, 0x66, 0x74, 0x79, 0x70, 0x69, 0x73, 0x6F, 0x6D], "audio/mp4"),
        (&[0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70, 0x6D, 0x70, 0x34, 0x32], "audio/mp4"),
        (&[0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70, 0x6D, 0x70, 0x34, 0x32], "audio/mp4"),
        (&[0x00, 0x00, 0x00, 0x1C, 0x66, 0x74, 0x79, 0x70, 0x69, 0x73, 0x6F, 0x6D], "audio/mp4"),
        (b"RIFF", "audio/wav"),
        (&[0xE0, 0x00, 0x00, 0x00], "audio/aiff"),
    ]
});

#[derive(Debug, Clone)]
pub struct FileValidationConfig {
    pub max_size: usize,
    pub allowed_types: Vec<String>,
    pub block_executables: bool,
    pub check_magic_bytes: bool,
    defang_pdf: bool,
}

impl Default for FileValidationConfig {
    fn default() -> Self {
        Self {
            max_size: MAX_FILE_SIZE,
            allowed_types: vec![
                "image/jpeg".into(),
                "image/png".into(),
                "image/gif".into(),
                "application/pdf".into(),
                "text/plain".into(),
                "application/zip".into(),
            ],
            block_executables: true,
            check_magic_bytes: true,
            defang_pdf: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileValidationResult {
    pub is_valid: bool,
    pub detected_type: Option<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn validate_file_upload(
    filename: &str,
    content_type: &str,
    data: &[u8],
    config: &FileValidationConfig,
) -> FileValidationResult {
    let mut result = FileValidationResult {
        is_valid: true,
        detected_type: None,
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    if data.len() > config.max_size {
        result.is_valid = false;
        result.errors.push(format!(
            "File size {} bytes exceeds maximum allowed size of {} bytes",
            data.len(),
            config.max_size
        ));
    }

    if let Some(extensions) = get_blocked_extensions() {
        if let Some(ext) = filename.split('.').next_back() {
            if extensions.contains(&ext.to_lowercase().as_str()) {
                result.is_valid = false;
                result.errors.push(format!(
                    "File extension .{} is blocked for security reasons",
                    ext
                ));
            }
        }
    }

    if config.check_magic_bytes {
        if let Some(detected) = detect_file_type(data) {
            result.detected_type = Some(detected.clone());

            if !config.allowed_types.is_empty() && !config.allowed_types.contains(&detected) {
                result.is_valid = false;
                result.errors.push(format!(
                    "Detected file type '{}' is not in the allowed types list",
                    detected
                ));
            }

            if content_type != detected && !content_type.starts_with("text/plain") && !content_type.starts_with("application/octet-stream") {
                result.warnings.push(format!(
                    "Content-Type header '{}' does not match detected file type '{}'",
                    content_type, detected
                ));
            }
        }
    }

    if config.block_executables && is_potentially_executable(data) {
        result.is_valid = false;
        result.errors.push(
            "File appears to be executable or contains executable code, which is blocked".into(),
        );
    }

    if config.defang_pdf && content_type == "application/pdf"
        && has_potential_malicious_pdf_content(data) {
            result.warnings.push(
                "PDF file may contain potentially malicious content (JavaScript, forms, or embedded files)".into(),
            );
        }

    result
}

fn detect_file_type(data: &[u8]) -> Option<String> {
    for (magic, mime_type) in MAGIC_BYTES.iter() {
        if data.starts_with(magic) {
            return Some(mime_type.to_string());
        }
    }

    // ISO base media (MP4/MOV/M4A/M4V): the `ftyp` box sits at offset 4,
    // behind a 4-byte size, so a `starts_with(b"ftyp")` test never matches.
    if data.len() >= 8 && &data[4..8] == b"ftyp" {
        return Some("video/mp4".into());
    }

    if data.starts_with(b"<") || data.starts_with(b"<!DOCTYPE") {
        if data.to_ascii_lowercase().windows(5).any(|w| w == b"<html") {
            return Some("text/html".into());
        }
        if data.windows(5).any(|w| w == b"<?xml") {
            return Some("text/xml".into());
        }
        return Some("text/plain".into());
    }

    if looks_like_text(data) {
        return Some("text/plain".into());
    }

    None
}

/// Whether the payload reads as text (ASCII or UTF-8) rather than binary.
///
/// Whitespace is allowed — a `.csv`, `.ics` or `.md` upload is full of newlines
/// and a non-ASCII name/description is still text. Any other control character
/// (or invalid UTF-8) means binary.
fn looks_like_text(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }

    // Cap the scan: a 100 MiB upload does not need a full UTF-8 validation to
    // prove it is not text.
    let head = &data[0..data.len().min(8192)];

    match std::str::from_utf8(head) {
        Ok(text) => text.chars().all(|c| !c.is_control() || c.is_whitespace()),
        Err(error) => {
            // A multi-byte character split at the 8 KiB boundary is still
            // text: judge the valid prefix.
            let valid = &head[..error.valid_up_to()];
            std::str::from_utf8(valid)
                .is_ok_and(|text| text.chars().all(|c| !c.is_control() || c.is_whitespace()))
        }
    }
}

/// MIME type implied by the leading bytes of `data`, or `None` when the
/// payload carries no recognised signature (#2132). Upload handlers use this
/// to prove the bytes agree with the extension they were stored under.
#[must_use]
pub fn detect_mime_type(data: &[u8]) -> Option<String> {
    detect_file_type(data)
}

/// Extensions refused on every upload surface: executables, scripts and
/// installer/macro containers. Markup types (`.html`, `.svg`) are not blocked
/// here — they are simply absent from the endpoint allowlists, except where an
/// endpoint explicitly accepts them (the document importer).
#[must_use]
pub fn blocked_extensions() -> &'static [&'static str] {
    &[
        "exe", "dll", "so", "dylib", "app", "deb", "rpm", "dmg", "pkg", "msi", "scr", "bat",
        "cmd", "com", "pif", "vbs", "vbe", "js", "jse", "ws", "wsf", "wsc", "wsh", "ps1",
        "ps1xml", "ps2", "ps2xml", "psc1", "psc2", "msh", "msh1", "msh2", "mshxml", "msh1xml",
        "msh2xml", "scf", "lnk", "inf", "reg", "docm", "dotm", "xlsm", "xltm", "xlam",
        "pptm", "potm", "ppam", "ppsm", "sldm", "jar", "appx", "appxbundle", "msix",
        "msixbundle", "sh", "csh", "bash", "zsh", "fish", "php", "php3", "php4", "php5",
        "phtml", "jsp", "jspx", "asp", "aspx", "cgi",
    ]
}

fn get_blocked_extensions() -> Option<Vec<&'static str>> {
    Some(blocked_extensions().to_vec())
}

fn is_potentially_executable(data: &[u8]) -> bool {
    if data.len() < 2 {
        return false;
    }

    let magic = &data[0..2];

    if matches!(magic, [0x4D, 0x5A]) {
        return true;
    }

    if data.len() >= 4 {
        let header = &data[0..4];
        if matches!(header, [0x7F, 0x45, 0x4C, 0x46]) {
            return true;
        }
    }

    if data.len() >= 8 {
        let header = &data[0..8];
        if matches!(header, [0xFE, 0xED, 0xFA, 0xCF, 0x00, 0x00, 0x00, 0x01])
            || matches!(header, [0xCF, 0xFA, 0xED, 0xFE, 0x01, 0x00, 0x00, 0x00])
        {
            return true;
        }
    }

    if data.len() >= 4 {
        let text_content = String::from_utf8_lossy(&data[0..data.len().min(4096)]);
        let lower = text_content.to_lowercase();
        if lower.contains("#!/bin/") || lower.contains("#!/usr/bin/") {
            return true;
        }
    }

    false
}

fn has_potential_malicious_pdf_content(data: &[u8]) -> bool {
    let text_content = String::from_utf8_lossy(data);
    let lower = text_content.to_lowercase();

    lower.contains("/javascript")
        || lower.contains("/action")
        || lower.contains("/launch")
        || lower.contains("/embeddedfile")
        || lower.contains("/efilename")
}


#[cfg(test)]
mod file_validation_tests {
    use super::*;

    fn cfg() -> FileValidationConfig {
        FileValidationConfig::default()
    }

    const PNG: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 0];
    const PDF: &[u8] = b"%PDF-1.7\n";

    #[test]
    fn a_png_is_detected_and_accepted() {
        let result = validate_file_upload("photo.png", "image/png", PNG, &cfg());
        assert!(result.is_valid, "{:?}", result.errors);
        assert_eq!(result.detected_type.as_deref(), Some("image/png"));
    }

    #[test]
    fn a_pdf_is_detected_and_accepted() {
        let result = validate_file_upload("doc.pdf", "application/pdf", PDF, &cfg());
        assert!(result.is_valid, "{:?}", result.errors);
        assert_eq!(result.detected_type.as_deref(), Some("application/pdf"));
    }

    #[test]
    fn a_mime_mismatch_is_a_warning_not_a_rejection() {
        let result = validate_file_upload("photo.png", "image/jpeg", PNG, &cfg());
        assert!(result.is_valid);
        assert!(result.warnings.iter().any(|w| w.contains("does not match")));
    }

    #[test]
    fn a_disallowed_detected_type_is_refused() {
        let result = validate_file_upload("clip.mp4", "video/mp4", b"ftypisom", &cfg());
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("not in the allowed types")));
    }

    #[test]
    fn an_oversized_file_is_refused() {
        let config = FileValidationConfig {
            max_size: 4,
            ..cfg()
        };
        let result = validate_file_upload("big.png", "image/png", PNG, &config);
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("exceeds maximum")));
    }

    #[test]
    fn blocked_extensions_are_refused_even_with_a_benign_mime() {
        for name in ["setup.exe", "lib.so", "run.sh", "macro.docm", "app.jar"] {
            let result = validate_file_upload(name, "application/pdf", PDF, &cfg());
            assert!(!result.is_valid, "{name} must be refused");
            assert!(result.errors.iter().any(|e| e.contains("blocked")));
        }
    }

    #[test]
    fn executables_are_detected_by_magic_bytes() {
        // Windows PE ("MZ").
        let result = validate_file_upload("x.png", "image/png", &[0x4D, 0x5A, 0x90, 0x00], &cfg());
        assert!(!result.is_valid);
        assert!(result.errors.iter().any(|e| e.contains("executable")));

        // ELF.
        let elf = [0x7F, 0x45, 0x4C, 0x46, 2, 1, 1, 0];
        let result = validate_file_upload("x.png", "image/png", &elf, &cfg());
        assert!(!result.is_valid);

        // A shebang script.
        let script = b"#!/bin/sh\nrm -rf /\n";
        let result = validate_file_upload("notes.txt", "text/plain", script, &cfg());
        assert!(!result.is_valid);
    }

    #[test]
    fn plain_text_is_detected_as_text() {
        let result = validate_file_upload("notes.txt", "text/plain", b"hello world", &cfg());
        assert!(result.is_valid);
        assert_eq!(result.detected_type.as_deref(), Some("text/plain"));
    }

    #[test]
    fn html_is_detected_even_though_it_starts_with_a_bracket() {
        let result = validate_file_upload("page.html", "text/html", b"<!DOCTYPE html><html></html>", &cfg());
        assert_eq!(result.detected_type.as_deref(), Some("text/html"));
    }

    #[test]
    fn a_pdf_with_javascript_only_warns() {
        let data = b"%PDF-1.7\n/JavaScript alert(1)\n";
        let result = validate_file_upload("doc.pdf", "application/pdf", data, &cfg());
        assert!(result.is_valid, "defanging must not reject the upload");
        assert!(result.warnings.iter().any(|w| w.contains("malicious")));
    }

    #[test]
    fn magic_byte_checking_can_be_disabled() {
        let config = FileValidationConfig {
            check_magic_bytes: false,
            ..cfg()
        };
        let result = validate_file_upload("clip.mp4", "video/mp4", b"ftypisom", &config);
        assert!(result.is_valid);
        assert!(result.detected_type.is_none());
    }
}
