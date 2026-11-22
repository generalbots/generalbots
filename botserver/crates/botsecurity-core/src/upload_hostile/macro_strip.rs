//! #2522 — remove executable parts from a macro-enabled Office package.
//!
//! The default gate refuses macros; import boundaries that can safely
//! re-serialize a package (Docs/Sheets) strip them *before* the shared gate and
//! keep a bounded, machine-readable notice for the caller. Nothing is
//! executed, no relationship is fetched and no file is written.

use std::collections::HashSet;
use std::io::{Cursor, Read, Write};

use crate::upload_error::UploadError;

/// A part larger than this is refused: rewriting it would grow memory without
/// a bounded relationship to the compressed size the caller accepted.
const MAX_REWRITE_PART_BYTES: u64 = 8 * 1024 * 1024;
/// A notice lists at most this many part names; the rest are counted.
const MAX_NOTICE_PARTS: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StripResult {
    /// A macro-free copy of the package. Originals are never modified.
    pub bytes: Vec<u8>,
    /// Lower-cased part names that were removed (bounded, sorted).
    pub removed_parts: Vec<String>,
    /// Total number of removed parts, including any omitted from the notice.
    pub removed_count: usize,
}

/// True for VBA projects, XLM macro sheets, ActiveX controls and embedded OLE
/// binaries — every part an Office parser could treat as executable content.
pub fn is_executable_part(name: &str) -> bool {
    let path = name.trim_start_matches("./").to_ascii_lowercase();
    let base = path.rsplit('/').next().unwrap_or(path.as_str());
    base == "vbaproject.bin"
        || path.contains("/macrosheets/")
        || path.contains("/activex/")
        || (path.contains("/embeddings/") && base.ends_with(".bin"))
        || ((path.starts_with("word/") || path.starts_with("ppt/")) && base.ends_with(".bin"))
}

/// Returns `Ok(None)` when the package is not a zip or has no executable
/// parts, so callers only pay for a rewrite when a macro was actually present.
pub fn strip_executable_parts(data: &[u8]) -> Result<Option<StripResult>, UploadError> {
    if data.is_empty() {
        return Ok(None);
    }
    let mut archive = match zip::ZipArchive::new(Cursor::new(data)) {
        Ok(archive) => archive,
        Err(_) => return Ok(None),
    };

    let mut removed = Vec::new();
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|_| unsafe_error("unreadable entry"))?;
        let name = entry
            .name()
            .map_err(|_| unsafe_error("unsafe part name"))?
            .to_ascii_lowercase();
        if is_executable_part(&name) {
            removed.push(name);
        }
    }
    if removed.is_empty() {
        return Ok(None);
    }

    let removed_set: HashSet<String> = removed.iter().cloned().collect();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|_| unsafe_error("unreadable entry"))?;
        let name = entry
            .name()
            .map_err(|_| unsafe_error("unsafe part name"))?
            .to_string();
        let lower = name.to_ascii_lowercase();
        if removed_set.contains(&lower) {
            continue;
        }
        if entry.size() > MAX_REWRITE_PART_BYTES {
            return Err(unsafe_error("part too large to sanitize"));
        }
        let mut body = Vec::new();
        entry
            .read_to_end(&mut body)
            .map_err(|_| unsafe_error("part read failed"))?;
        if lower.ends_with(".rels") || lower.ends_with("[content_types].xml") {
            let rewritten = remove_references(&String::from_utf8_lossy(&body), &removed_set);
            body = rewritten.into_bytes();
        }
        let options = zip::write::FileOptions::<()>::default()
            .compression_method(zip::CompressionMethod::Deflated);
        writer
            .start_file(&name, options)
            .map_err(|_| unsafe_error("sanitized entry start failed"))?;
        writer
            .write_all(&body)
            .map_err(|_| unsafe_error("sanitized entry write failed"))?;
    }
    let finished = writer
        .finish()
        .map_err(|_| unsafe_error("sanitized package finish failed"))?;

    let removed_count = removed.len();
    let mut removed_parts: Vec<String> = removed.into_iter().take(MAX_NOTICE_PARTS).collect();
    removed_parts.sort();
    Ok(Some(StripResult {
        bytes: finished.into_inner(),
        removed_parts,
        removed_count,
    }))
}

/// Drops the relationship/content-type elements that still point at a part we
/// removed. The scanner is textual and bounded by the part size cap above; it
/// never interprets XML or resolves a target.
fn remove_references(text: &str, removed: &HashSet<String>) -> String {
    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('<') {
        output.push_str(&rest[..start]);
        let tail = &rest[start..];
        let end = tail
            .find('>')
            .map(|position| position + 1)
            .unwrap_or(tail.len());
        let element = &tail[..end];
        let lower = element.to_ascii_lowercase();
        let references_removed = removed.iter().any(|part| {
            let base = part.rsplit('/').next().unwrap_or(part.as_str());
            lower.contains(part.as_str()) || lower.contains(base)
        });
        if !references_removed {
            output.push_str(element);
        }
        rest = &tail[end..];
    }
    output.push_str(rest);
    output
}

fn unsafe_error(reason: &'static str) -> UploadError {
    UploadError::ArchiveUnsafe { reason }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn hostile_corpus_macro_vba_project_is_removed_and_rels_dropped() -> Result<(), String> {
        let bytes = zip_with(&[
            ("[Content_Types].xml", b"<Types><Override PartName=\"/word/vbaProject.bin\" ContentType=\"application/vnd.ms-office.vbaProject\"/></Types>".to_vec()),
            ("word/document.xml", b"<w:document/>".to_vec()),
            ("word/vbaProject.bin", vec![0x01, 0x02]),
            ("word/_rels/document.xml.rels", b"<Relationships><Relationship Id=\"rId1\" Target=\"vbaProject.bin\"/><Relationship Id=\"rId2\" Target=\"styles.xml\"/></Relationships>".to_vec()),
        ]);

        let result = strip_executable_parts(&bytes)
            .map_err(|error| format!("sanitize failed: {}", error.log_detail()))?
            .ok_or_else(|| "a macro part must be found".to_string())?;
        let mut archive = zip::ZipArchive::new(Cursor::new(&result.bytes))
            .map_err(|error| format!("sanitized bytes must be a zip: {error}"))?;
        if archive.len() != 3 {
            return Err(format!(
                "the macro part must be gone: {} entries",
                archive.len()
            ));
        }
        if !result
            .removed_parts
            .iter()
            .any(|part| part == "word/vbaproject.bin")
        {
            return Err("notice must name the removed part".to_string());
        }
        let mut rels = String::new();
        std::io::Read::read_to_string(
            &mut archive
                .by_name("word/_rels/document.xml.rels")
                .map_err(|error| format!("rels part must survive: {error}"))?,
            &mut rels,
        )
        .map_err(|error| format!("rels must be readable: {error}"))?;
        if rels.contains("vbaProject.bin") {
            return Err(format!("macro relationship was kept: {rels}"));
        }
        if !rels.contains("styles.xml") {
            return Err(format!("safe relationship was dropped: {rels}"));
        }
        Ok(())
    }

    #[test]
    fn hostile_corpus_macro_clean_package_is_not_rewritten() -> Result<(), String> {
        let bytes = zip_with(&[("word/document.xml", b"<w:document/>".to_vec())]);
        let result = strip_executable_parts(&bytes)
            .map_err(|error| format!("clean package must pass: {}", error.log_detail()))?;
        if result.is_some() {
            return Err("nothing to strip means no rewrite".to_string());
        }
        Ok(())
    }

    #[test]
    fn hostile_corpus_macro_xlm_activex_and_ole_parts_are_executable() {
        for part in [
            "xl/macrosheets/sheet1.xml",
            "word/activeX/control1.xml",
            "word/embeddings/oleObject1.bin",
            "ppt/vbaProject.bin",
        ] {
            assert!(is_executable_part(part), "{part} must be executable");
        }
        assert!(!is_executable_part("word/media/image1.png"));
    }
}
