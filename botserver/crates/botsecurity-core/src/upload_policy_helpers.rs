use super::*;

pub(super) fn final_extension(filename: &str) -> &str {
    filename.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("")
}

pub(super) fn normalize_mime(raw: &str) -> String {
    raw.split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

pub(super) fn declared_matches(declared: &str, expected: &str) -> bool {
    // No declaration (or the generic one browsers send for unknown files) is
    // not a claim to contradict: the extension and the magic bytes decide.
    if declared.is_empty()
        || declared == "application/octet-stream"
        || declared == "binary/octet-stream"
    {
        return true;
    }
    if declared == expected {
        return true;
    }
    // Browsers label `.md`, `.csv`, `.json` and friends inconsistently.
    if is_text_mime(declared) && is_text_mime(expected) {
        return true;
    }
    // Windows labels a `.csv` as `application/vnd.ms-excel`.
    if expected == "text/csv" && declared == "application/vnd.ms-excel" {
        return true;
    }
    // Office/ODF packages are zip files, and some clients say so.
    is_zip_container(expected)
        && matches!(declared, "application/zip" | "application/x-zip-compressed")
}

pub(super) fn magic_matches(extension: &str, expected: &str, detected: &str, data: &[u8]) -> bool {
    if detected == expected {
        return true;
    }
    // OOXML/ODF are zip containers; legacy Office is an OLE2 compound file.
    if is_zip_container(expected) && detected == "application/zip" {
        return true;
    }
    if matches!(
        expected,
        "application/msword"
            | "application/vnd.ms-excel"
            | "application/vnd.ms-powerpoint"
            | "application/vnd.ms-project"
    ) && detected == "application/x-ole-storage"
    {
        return true;
    }
    // Text formats: ASCII payloads all read as `text/plain`, and an XML
    // declaration reads as `text/xml`.
    if is_text_mime(expected) && matches!(detected, "text/plain" | "text/xml") {
        return true;
    }
    // SVG is XML text: `<svg` is the discriminator, not the magic table.
    if expected == "image/svg+xml" && matches!(detected, "text/plain" | "text/xml") {
        return contains_svg(data);
    }
    // ISO base media: MP4/MOV/M4A share the `ftyp` box.
    if matches!(expected, "video/mp4" | "video/quicktime" | "audio/mp4")
        && detected == "video/mp4"
    {
        return true;
    }
    // RIFF carries WAV and AVI; the form type at offset 8 disambiguates.
    if expected == "video/x-msvideo" && detected == "audio/wav" && is_riff_avi(data) {
        return true;
    }
    // `.m4a` is detected as audio/mp4 but some encoders emit a plain ftyp box.
    if extension == "m4a" && detected == "video/mp4" {
        return true;
    }
    false
}

pub(super) fn contains_svg(data: &[u8]) -> bool {
    let head = String::from_utf8_lossy(&data[0..data.len().min(4096)]).to_ascii_lowercase();
    head.contains("<svg")
}

pub(super) fn is_riff_avi(data: &[u8]) -> bool {
    data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"AVI "
}

#[cfg(test)]
#[path = "upload_policy_tests.rs"]
mod upload_policy_tests;
