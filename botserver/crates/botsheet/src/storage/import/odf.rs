//! #2463 — focused ODF 1.3 spreadsheet (ODS) import slice.
//!
//! Reads the `content.xml` part of an ODF spreadsheet package into the
//! existing worksheet cell model. Unlike the legacy `import_ods.rs` this
//! module: validates the package `mimetype`, honours repeated rows/columns,
//! types cells from `office:value-*`, applies hard size/entry caps and refuses
//! embedded objects instead of silently dropping them.
//!
//! The parser is XXE-safe by construction: `quick-xml` never resolves external
//! DTDs, and this module rejects any `DOCTYPE` or non-builtin entity reference
//! before it reaches the cell model. No panic path is used.

use std::io::{Cursor, Read};

use crate::types::Worksheet;

mod parser;
#[cfg(test)]
mod tests;

const ODF_SPREADSHEET_MIME: &str = "application/vnd.oasis.opendocument.spreadsheet";

// Bounded resource envelope. The upload handler already gates the compressed
// size; these caps keep a malformed package from expanding into unbounded
// strings or a dense grid inside this parser (#2530 posture).
const MAX_ODS_ENTRIES: usize = 10_000;
const MAX_TOTAL_UNCOMPRESSED_BYTES: u64 = 256 * 1024 * 1024;
const MAX_CONTENT_XML_BYTES: u64 = 64 * 1024 * 1024;
const MAX_MIMETYPE_BYTES: u64 = 256;
const MAX_SHEETS: usize = 256;
const MAX_CELLS_PER_SHEET: usize = 250_000;
const MAX_COLUMNS: u32 = 16_384; // ODF 1.3 / XFD
const MAX_ROWS: u32 = 1_048_576;
const MAX_REPEAT: u32 = 1_048_576;
const MAX_TEXT_CHARS: usize = 32_768;

/// Logs the detailed reason but returns a short, non-sensitive message.
fn reject(detail: impl std::fmt::Display, message: &str) -> String {
    log::warn!("ods import refused: {detail}");
    message.to_string()
}

/// True when the byte slice is an ODF *spreadsheet* package: a valid zip with
/// both a `mimetype` entry naming the spreadsheet media type and a
/// `content.xml` part.
pub fn is_odf_spreadsheet(bytes: &[u8]) -> bool {
    let Ok(mut archive) = zip::ZipArchive::new(Cursor::new(bytes)) else {
        return false;
    };
    if archive.len() == 0 || archive.len() > MAX_ODS_ENTRIES {
        return false;
    }
    let mut mime_ok = false;
    let mut has_content = false;
    for index in 0..archive.len() {
        let Ok(mut entry) = archive.by_index(index) else {
            continue;
        };
        let Ok(name) = entry.name() else {
            continue;
        };
        if name.as_ref() == "mimetype" {
            let mut body = Vec::new();
            if entry
                .by_ref()
                .take(MAX_MIMETYPE_BYTES)
                .read_to_end(&mut body)
                .is_err()
            {
                continue;
            }
            mime_ok = String::from_utf8_lossy(&body).trim() == ODF_SPREADSHEET_MIME;
        } else if name.as_ref() == "content.xml" {
            has_content = true;
        }
    }
    mime_ok && has_content
}

/// OLE/embedded payloads live under `Object N/` and `ObjectReplacements/`.
/// They cannot be represented in the worksheet model, so their presence is
/// reported to the caller rather than dropped.
fn is_embedded_object_path(name: &str) -> bool {
    let path = name.to_ascii_lowercase();
    let path = path.trim_start_matches("./");
    path.starts_with("object ")
        || path.starts_with("object/")
        || path.starts_with("objectreplacements/")
        || path.contains("/object ")
        || path.contains("/object/")
        || path.contains("/objectreplacements/")
}

fn is_embedded_element_name(name: &str) -> bool {
    matches!(name, "object" | "object-ole" | "plugin" | "applet")
}

/// Opens the package, enforces the envelope and returns the parsed worksheets.
pub fn parse_ods_to_worksheets(bytes: &[u8]) -> Result<Vec<Worksheet>, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|error| {
        reject(
            format!("invalid zip container: {error}"),
            "ODS import failed: invalid package",
        )
    })?;

    let entries = archive.len();
    if entries == 0 || entries > MAX_ODS_ENTRIES {
        return Err(reject(
            format!("entry count {entries}"),
            "ODS import failed: package has an unsupported number of entries",
        ));
    }

    let mut mimetype = String::new();
    let mut has_content = false;
    let mut embedded_part = None;
    let mut uncompressed = 0u64;
    for index in 0..entries {
        let mut entry = archive.by_index(index).map_err(|error| {
            reject(
                format!("unreadable entry {index}: {error}"),
                "ODS import failed: unreadable package entry",
            )
        })?;
        let name = entry
            .name()
            .map_err(|error| {
                reject(
                    format!("unsafe entry name: {error}"),
                    "ODS import failed: unsafe package entry name",
                )
            })?
            .into_owned();
        uncompressed = uncompressed.saturating_add(entry.size());
        if uncompressed > MAX_TOTAL_UNCOMPRESSED_BYTES {
            return Err(reject(
                "uncompressed envelope exceeded",
                "ODS import failed: package exceeds the uncompressed size limit",
            ));
        }
        if name == "mimetype" {
            let mut body = Vec::new();
            entry
                .by_ref()
                .take(MAX_MIMETYPE_BYTES)
                .read_to_end(&mut body)
                .map_err(|error| {
                    reject(
                        format!("mimetype read failed: {error}"),
                        "ODS import failed: unreadable mimetype entry",
                    )
                })?;
            mimetype = String::from_utf8_lossy(&body).trim().to_ascii_lowercase();
        } else if name == "content.xml" {
            has_content = true;
        }
        if embedded_part.is_none() && is_embedded_object_path(&name) {
            embedded_part = Some(name);
        }
    }

    if mimetype != ODF_SPREADSHEET_MIME {
        return Err(reject(
            format!("mimetype was '{mimetype}'"),
            "ODS import failed: not an ODF spreadsheet package",
        ));
    }
    if !has_content {
        return Err(reject(
            "content.xml missing",
            "ODS import failed: content.xml not found",
        ));
    }
    if let Some(part) = embedded_part {
        return Err(reject(
            format!("embedded object part: {part}"),
            "ODS import failed: embedded objects are not supported; extract or save the object separately before importing",
        ));
    }

    let mut content = archive.by_name("content.xml").map_err(|error| {
        reject(
            format!("content.xml open failed: {error}"),
            "ODS import failed: unreadable content.xml",
        )
    })?;
    if content.size() > MAX_CONTENT_XML_BYTES {
        return Err(reject(
            "content.xml declared size exceeded",
            "ODS import failed: content.xml exceeds the size limit",
        ));
    }
    let mut xml = String::new();
    content
        .by_ref()
        .take(MAX_CONTENT_XML_BYTES.saturating_add(1))
        .read_to_string(&mut xml)
        .map_err(|error| {
            reject(
                format!("content.xml read failed: {error}"),
                "ODS import failed: unreadable content.xml",
            )
        })?;
    if xml.len() as u64 > MAX_CONTENT_XML_BYTES {
        return Err(reject(
            "content.xml actual size exceeded",
            "ODS import failed: content.xml exceeds the size limit",
        ));
    }

    parser::parse_ods_content(&xml)
}
