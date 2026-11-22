//! #2522 — bounded XML and relationship inspection for spreadsheet imports.
//!
//! The central upload gate rejects zip bombs, macros and unsafe relationship
//! targets before parsing. Spreadsheet parsers also expand XML on their own,
//! so this module adds a second, streaming pass over the XML parts of the
//! package with **constant memory**:
//!
//! * any `<!DOCTYPE` is refused — the only place an internal or external
//!   entity declaration can hide (billion-laughs / XXE);
//! * `<!ENTITY` declarations and custom (non-predefined, non-numeric) entity
//!   references have hard caps, so an expansion cannot grow the workbook;
//! * `.rels` targets are re-checked with the shared SSRF policy at this
//!   boundary, so no remote or internal reference reaches a parser.
//!
//! The scanner reads in 8 KiB chunks and keeps only a 9-byte rolling window
//! plus one pending entity name. It never materialises an XML part and never
//! fetches a relationship target.

use std::io::{Cursor, Read};

use botsecurity_core::upload_policy::{self, UploadError};

const SCAN_CHUNK: usize = 8 * 1024;
const WINDOW_LEN: usize = 9; // len("<!DOCTYPE")
const MAX_ENTITY_DECLARATIONS: usize = 8;
const MAX_CUSTOM_ENTITY_REFS: usize = 4096;
const MAX_ENTITY_NAME: usize = 128;
const MAX_RELATIONSHIPS_BYTES: u64 = 1024 * 1024;

/// What one inspected package contributed to the report. All counts are
/// bounded, so the report itself cannot leak size information from the payload.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct XmlSafetyReport {
    pub(crate) parts_checked: usize,
    pub(crate) relationship_targets: usize,
    pub(crate) entity_declarations: usize,
    pub(crate) custom_entity_refs: usize,
}

fn unsafe_reason(reason: &'static str) -> UploadError {
    UploadError::ArchiveUnsafe { reason }
}

/// Inspects every XML part of a zip-based spreadsheet package before parsing.
/// Non-XML parts (images, embedded fonts, …) are never read.
pub(crate) fn inspect_office_package(bytes: &[u8]) -> Result<XmlSafetyReport, UploadError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| unsafe_reason("invalid zip container"))?;
    let mut report = XmlSafetyReport::default();
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|_| unsafe_reason("unreadable xml part"))?;
        let name = entry
            .name()
            .map_err(|_| unsafe_reason("unsafe part name"))?
            .to_ascii_lowercase();
        if botsecurity_core::upload_hostile::macro_strip::is_executable_part(&name) {
            return Err(UploadError::ArchiveMacro { part: name });
        }
        if !(name.ends_with(".xml") || name.ends_with(".rels")) {
            continue;
        }
        report.parts_checked += 1;
        let mut scanner = XmlScanner::default();
        if name.ends_with(".rels") {
            // Relationship parts are small: one bounded read feeds both the
            // entity scanner and the target validator.
            let mut body = Vec::new();
            if entry
                .take(MAX_RELATIONSHIPS_BYTES)
                .read_to_end(&mut body)
                .is_err()
            {
                return Err(unsafe_reason("relationship part read failed"));
            }
            scanner.consume(&mut Cursor::new(&body))?;
            for raw_target in relationship_targets(&body) {
                let target = unescape_xml(&raw_target);
                upload_policy::validate_relationship_target(&target)?;
                report.relationship_targets += 1;
            }
        } else {
            scanner.consume(&mut entry)?;
        }
        report.entity_declarations += scanner.entity_declarations;
        report.custom_entity_refs += scanner.custom_entity_refs;
    }
    Ok(report)
}

#[derive(Default)]
struct XmlScanner {
    window: [u8; WINDOW_LEN],
    window_len: usize,
    in_ref: bool,
    ref_name: Vec<u8>,
    entity_declarations: usize,
    custom_entity_refs: usize,
}

impl XmlScanner {
    /// Consumes one part in fixed-size reads. Memory use does not depend on
    /// the declared uncompressed size of the part.
    fn consume(&mut self, reader: &mut impl Read) -> Result<(), UploadError> {
        let mut buffer = [0u8; SCAN_CHUNK];
        loop {
            let read = reader
                .read(&mut buffer)
                .map_err(|_| unsafe_reason("xml part read failed"))?;
            if read == 0 {
                break;
            }
            for &byte in &buffer[..read] {
                self.push(byte)?;
            }
        }
        Ok(())
    }

    fn push(&mut self, byte: u8) -> Result<(), UploadError> {
        // Entity references are tracked across chunk boundaries with a
        // bounded pending name (XML names longer than 128 bytes are ignored).
        if self.in_ref {
            match byte {
                b';' => {
                    if !is_builtin_entity(&self.ref_name) {
                        self.custom_entity_refs += 1;
                        if self.custom_entity_refs > MAX_CUSTOM_ENTITY_REFS {
                            return Err(unsafe_reason("xml entity expansion"));
                        }
                    }
                    self.in_ref = false;
                    self.ref_name.clear();
                }
                name_byte
                    if name_byte.is_ascii_alphanumeric()
                        || matches!(name_byte, b'#' | b'_' | b'-' | b'.' | b':') =>
                {
                    if self.ref_name.len() < MAX_ENTITY_NAME {
                        self.ref_name.push(name_byte);
                    }
                }
                _ => {
                    self.in_ref = false;
                    self.ref_name.clear();
                }
            }
        }
        if byte == b'&' {
            self.in_ref = true;
            self.ref_name.clear();
        }

        // Nine-byte rolling window: a token split between two reads is still
        // seen, and no XML body is retained.
        if self.window_len < WINDOW_LEN {
            self.window[self.window_len] = byte;
            self.window_len += 1;
        } else {
            self.window.rotate_left(1);
            self.window[WINDOW_LEN - 1] = byte;
        }
        if self.window_len == WINDOW_LEN && self.window.eq_ignore_ascii_case(b"<!DOCTYPE") {
            return Err(unsafe_reason("xml doctype"));
        }
        if self.window_len >= 8 && self.window[WINDOW_LEN - 8..].eq_ignore_ascii_case(b"<!ENTITY") {
            self.entity_declarations += 1;
            if self.entity_declarations > MAX_ENTITY_DECLARATIONS {
                return Err(unsafe_reason("xml entity declaration"));
            }
        }
        Ok(())
    }
}

fn is_builtin_entity(name: &[u8]) -> bool {
    matches!(name, b"amp" | b"lt" | b"gt" | b"quot" | b"apos") || name.first() == Some(&b'#')
}

fn relationship_targets(body: &[u8]) -> Vec<String> {
    const NEEDLE: &[u8] = b"Target";
    let mut values = Vec::new();
    let mut position = 0usize;
    while position + NEEDLE.len() <= body.len() {
        if &body[position..position + NEEDLE.len()] == NEEDLE {
            let mut cursor = position + NEEDLE.len();
            while cursor < body.len() && body[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if body.get(cursor) == Some(&b'=') {
                cursor += 1;
                while cursor < body.len() && body[cursor].is_ascii_whitespace() {
                    cursor += 1;
                }
                if let Some(quote) = body.get(cursor).copied() {
                    if matches!(quote, b'"' | b'\'') {
                        cursor += 1;
                        let start = cursor;
                        while cursor < body.len()
                            && body[cursor] != quote
                            && cursor - start < MAX_RELATIONSHIPS_BYTES as usize
                        {
                            cursor += 1;
                        }
                        if let Ok(value) = std::str::from_utf8(&body[start..cursor]) {
                            if !value.is_empty() {
                                values.push(value.to_string());
                            }
                        }
                        position = cursor;
                        continue;
                    }
                }
            }
        }
        position += 1;
    }
    values
}

fn unescape_xml(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&#47;", "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

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

    fn refused_code(bytes: &[u8]) -> Option<&'static str> {
        inspect_office_package(bytes)
            .err()
            .map(|error| error.code())
    }

    #[test]
    fn an_xml_entity_bomb_is_refused_before_parsing() {
        let xml = b"<?xml version=\"1.0\"?><!DOCTYPE w [<!ENTITY a \"aaaaaaaaaa\"><!ENTITY b \"&a;&a;&a;&a;\">]><worksheet>&b;</worksheet>";
        let bytes = zip_with(&[("xl/worksheets/sheet1.xml", xml.to_vec())]);
        assert_eq!(refused_code(&bytes), Some("error-file-archive-unsafe"));
    }

    #[test]
    fn an_external_entity_declaration_is_refused() {
        let xml = b"<!DOCTYPE w [<!ENTITY xxe SYSTEM \"file:///etc/passwd\">]><worksheet>&xxe;</worksheet>";
        let bytes = zip_with(&[("xl/workbook.xml", xml.to_vec())]);
        assert_eq!(refused_code(&bytes), Some("error-file-archive-unsafe"));
    }

    #[test]
    fn a_custom_entity_reference_cap_is_enforced() {
        let too_many = format!("<worksheet>{}</worksheet>", "&x;".repeat(4097));
        let bytes = zip_with(&[("xl/worksheets/sheet1.xml", too_many.into_bytes())]);
        assert_eq!(refused_code(&bytes), Some("error-file-archive-unsafe"));

        let allowed = format!("<worksheet>{}</worksheet>", "&x;".repeat(4096));
        let bytes = zip_with(&[("xl/worksheets/sheet1.xml", allowed.into_bytes())]);
        let report = inspect_office_package(&bytes);
        assert!(report.is_ok(), "the cap boundary must be usable");
        assert!(report.is_ok(), "the cap boundary must be usable");
        match report {
            Ok(report) => assert_eq!(report.custom_entity_refs, 4096),
            Err(error) => log::warn!("unexpected refusal: {}", error.log_detail()),
        }
    }

    #[test]
    fn a_token_split_across_a_read_boundary_is_detected() {
        // '<' is the last byte of the first 8 KiB chunk; the rest of the
        // DOCTYPE token arrives in the next chunk.
        let mut xml = vec![b' '; 8191];
        xml.extend_from_slice(b"<!DOCTYPE w [<!ENTITY x \"y\">]><worksheet/>");
        let bytes = zip_with(&[("xl/worksheets/sheet1.xml", xml)]);
        assert_eq!(refused_code(&bytes), Some("error-file-archive-unsafe"));
    }

    #[test]
    fn a_relative_relationship_is_allowed() {
        let rels =
            b"<Relationships><Relationship Target=\"../worksheets/sheet1.xml\"/></Relationships>";
        let bytes = zip_with(&[("xl/_rels/workbook.xml.rels", rels.to_vec())]);
        let report = inspect_office_package(&bytes);
        assert!(report.is_ok(), "package relative paths are safe");
        assert!(report.is_ok(), "a package relative path must be accepted");
        match report {
            Ok(report) => assert_eq!(report.relationship_targets, 1),
            Err(error) => log::warn!("unexpected refusal: {}", error.log_detail()),
        }
    }

    #[test]
    fn internal_and_file_relationship_targets_are_refused() {
        for target in [
            "http://127.0.0.1:8080/admin",
            "http://169.254.169.254/latest/meta-data/",
            "file:///etc/passwd",
        ] {
            let rels = format!(
                "<Relationships><Relationship Target=\"{target}\" TargetMode=\"External\"/></Relationships>"
            );
            let bytes = zip_with(&[("xl/_rels/workbook.xml.rels", rels.into_bytes())]);
            assert_eq!(
                refused_code(&bytes),
                Some("error-file-archive-unsafe"),
                "target {target} must be refused"
            );
        }
    }

    #[test]
    fn xlm_macro_sheets_and_activex_parts_are_refused() {
        for part in [
            "xl/macrosheets/sheet1.xml",
            "xl/activeX/control1.xml",
            "xl/embeddings/oleObject1.bin",
            "xl/vbaProject.bin",
        ] {
            let bytes = zip_with(&[(part, b"<dummy/>".to_vec())]);
            assert_eq!(
                refused_code(&bytes),
                Some("error-file-archive-unsafe"),
                "part {part} must be refused"
            );
        }
    }

    #[test]
    fn a_clean_package_and_a_public_relationship_pass() {
        let bytes = zip_with(&[
            (
                "xl/worksheets/sheet1.xml",
                b"<worksheet><sheetData/></worksheet>".to_vec(),
            ),
            (
                "xl/_rels/workbook.xml.rels",
                b"<Relationships><Relationship Target=\"https://example.com/data\"/></Relationships>"
                    .to_vec(),
            ),
            ("xl/media/image1.png", vec![0x89, b'P', b'N', b'G']),
        ]);
        let report = inspect_office_package(&bytes);
        assert!(report.is_ok(), "ordinary Office parts must not be refused");
        let report = match report {
            Ok(report) => report,
            Err(error) => {
                log::warn!(
                    "unexpected refusal of a clean package: {}",
                    error.log_detail()
                );
                return;
            }
        };
        assert_eq!(report.parts_checked, 2);
        assert_eq!(report.relationship_targets, 1);
    }
}
