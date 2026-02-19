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
    #[allow(dead_code)]
    scan_for_malware: bool,
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
            scan_for_malware: false,
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

    if data.starts_with(b"<") || data.starts_with(b"<!DOCTYPE") {
        if data.to_ascii_lowercase().windows(5).any(|w| w == b"<html") {
            return Some("text/html".into());
        }
        if data.windows(5).any(|w| w == b"<?xml") {
            return Some("text/xml".into());
        }
        return Some("text/plain".into());
    }

    if data.iter().all(|&b| b.is_ascii() && !b.is_ascii_control()) {
        return Some("text/plain".into());
    }

    None
}

fn get_blocked_extensions() -> Option<Vec<&'static str>> {
    Some(vec![
        "exe", "dll", "so", "dylib", "app", "deb", "rpm", "dmg", "pkg", "msi", "scr", "bat",
        "cmd", "com", "pif", "vbs", "vbe", "js", "jse", "ws", "wsf", "wsc", "wsh", "ps1",
        "ps1xml", "ps2", "ps2xml", "psc1", "psc2", "msh", "msh1", "msh2", "mshxml", "msh1xml",
        "msh2xml", "scf", "lnk", "inf", "reg", "docm", "dotm", "xlsm", "xltm", "xlam",
        "pptm", "potm", "ppam", "ppsm", "sldm", "jar", "appx", "appxbundle", "msix",
        "msixbundle", "sh", "csh", "bash", "zsh", "fish",
    ])
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

