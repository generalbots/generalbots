//! #2522 — remove executable parts from a macro-enabled Office package.
//!
//! The default gate refuses macros; boundaries that can safely re-serialize a
//! package (Docs/Sheets/Slides imports and Drive writes, #2927) strip them
//! *before* the shared gate and keep a bounded, machine-readable notice for
//! the caller. Nothing is executed, no relationship is fetched and no file is
//! written.

use std::borrow::Cow;
use std::collections::HashSet;
use std::io::{Cursor, Read, Write};

use serde::Serialize;

use crate::upload_error::UploadError;

/// A part larger than this is refused: rewriting it would grow memory without
/// a bounded relationship to the compressed size the caller accepted.
const MAX_REWRITE_PART_BYTES: u64 = 8 * 1024 * 1024;
/// A rewrite runs *before* the upload gate (Drive writes, Office imports), so
/// it needs its own total cap: many individually bounded parts could otherwise
/// accumulate unbounded memory before the gate's ratio check sees the archive.
const MAX_REWRITE_TOTAL_BYTES: u64 = 100 * 1024 * 1024;
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
///
/// #2927 — a generic `*.bin` under `word/` or `ppt/` is *not* executable by
/// itself: those folders legitimately hold passive binaries (chart caches,
/// theme blobs, thumbnails). Only the well-known macro containers and macro
/// names count, so a macro-free PPTX is no longer refused by the Drive gate.
pub fn is_executable_part(name: &str) -> bool {
    let path = name.trim_start_matches("./").to_ascii_lowercase();
    let base = path.rsplit('/').next().unwrap_or(path.as_str());
    let binary = base.ends_with(".bin");
    base == "vbaproject.bin"
        || (binary
            && (base.starts_with("vba") || base.contains("oleobject") || base.contains("activex")))
        || path.contains("/macrosheets/")
        || path.contains("/activex/")
        || (path.contains("/embeddings/") && binary)
}

/// Machine-readable notice for a stored object whose executable parts were
/// removed before the upload gate. `message_key` is an existing catalogue key
/// in every locale (`ui-common.ftl`); no user-facing literal is emitted here.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MacroNotice {
    pub code: &'static str,
    pub message_key: &'static str,
    pub count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parts: Vec<String>,
}

/// The shared Office prepare step: bytes to validate/store and an optional
/// notice. `bytes` borrows the caller's payload when nothing was removed, so a
/// PDF, image or clean package pays no copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedUpload<'a> {
    pub bytes: Cow<'a, [u8]>,
    pub notice: Option<MacroNotice>,
}

/// Sanitize executable parts *before* the upload gate and return a bounded,
/// machine-readable notice for the caller (#2927). The importers call
/// [`strip_executable_parts`] directly and build the same notice shape. The
/// gate still runs on the returned bytes, so bombs, XXE, unsafe relationships
/// and any part we could not safely rewrite remain fail-closed.
pub fn prepare_office_upload(data: &[u8]) -> Result<PreparedUpload<'_>, UploadError> {
    match strip_executable_parts(data)? {
        Some(result) => Ok(PreparedUpload {
            notice: Some(MacroNotice {
                code: "file-macros-removed",
                message_key: "file-macros-removed",
                count: result.removed_count,
                parts: result.removed_parts,
            }),
            bytes: Cow::Owned(result.bytes),
        }),
        None => Ok(PreparedUpload {
            bytes: Cow::Borrowed(data),
            notice: None,
        }),
    }
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
    let mut rewritten_total = 0u64;
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
        accumulate_rewrite_size(entry.size(), &mut rewritten_total)?;
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

/// Bounds both one part and the whole rewrite before any body is read. Kept as
/// a pure function so the many-entry bomb case is testable without allocating
/// hundreds of megabytes.
fn accumulate_rewrite_size(size: u64, total: &mut u64) -> Result<(), UploadError> {
    if size > MAX_REWRITE_PART_BYTES {
        return Err(unsafe_error("part too large to sanitize"));
    }
    *total = total.saturating_add(size);
    if *total > MAX_REWRITE_TOTAL_BYTES {
        return Err(unsafe_error("sanitized package too large"));
    }
    Ok(())
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

    #[test]
    fn hostile_corpus_rewrite_budget_is_bounded() {
        let mut total = 0u64;
        for _ in 0..12 {
            assert!(accumulate_rewrite_size(MAX_REWRITE_PART_BYTES, &mut total).is_ok());
        }
        assert!(accumulate_rewrite_size(MAX_REWRITE_PART_BYTES, &mut total).is_err());
        assert!(accumulate_rewrite_size(MAX_REWRITE_PART_BYTES + 1, &mut total).is_err());
    }

    #[test]
    fn hostile_corpus_benign_binaries_are_not_macro_parts() {
        // #2927 — these are the false positives that blocked a clean PPTX.
        for part in [
            "ppt/media/chart.bin",
            "ppt/cache/fallback.bin",
            "word/theme/themeData.bin",
            "docProps/thumbnail.bin",
        ] {
            assert!(!is_executable_part(part), "{part} is a passive binary");
        }
    }

    #[test]
    fn hostile_corpus_genuine_macro_names_are_still_executable() {
        for part in [
            "word/vbaProject.bin",
            "ppt/vbaProject.bin",
            "ppt/vbaData.bin",
            "word/oleObject3.bin",
            "ppt/activeX/control1.bin",
            "ppt/embeddings/oleObject2.bin",
            "xl/macrosheets/sheet1.xml",
            "word/activeX/control1.xml",
        ] {
            assert!(is_executable_part(part), "{part} must be executable");
        }
    }

    #[test]
    fn hostile_corpus_prepare_strips_real_macro_and_emits_a_catalogue_notice() -> Result<(), String>
    {
        let bytes = zip_with(&[
            ("[Content_Types].xml", b"<Types><Override PartName=\"/ppt/vbaProject.bin\" ContentType=\"application/vnd.ms-office.vbaProject\"/></Types>".to_vec()),
            ("ppt/presentation.xml", b"<p:presentation/>".to_vec()),
            ("ppt/vbaProject.bin", vec![0xd0, 0xcf, 0x11, 0xe0]),
            ("ppt/media/chart.bin", vec![0x01, 0x02]),
        ]);
        let prepared = prepare_office_upload(&bytes)
            .map_err(|error| format!("sanitize failed: {}", error.log_detail()))?;
        let notice = prepared
            .notice
            .ok_or_else(|| "a macro part must produce a notice".to_string())?;
        assert_eq!(notice.code, "file-macros-removed");
        assert_eq!(notice.message_key, "file-macros-removed");
        assert_eq!(notice.count, 1);
        assert!(notice.parts.iter().any(|part| part == "ppt/vbaproject.bin"));

        let stored = prepared.bytes.into_owned();
        let mut archive = zip::ZipArchive::new(Cursor::new(&stored))
            .map_err(|error| format!("sanitized bytes must be a zip: {error}"))?;
        if archive.by_name("ppt/vbaproject.bin").is_ok() {
            return Err("the VBA project must not be stored".to_string());
        }
        if archive.by_name("ppt/media/chart.bin").is_err() {
            return Err("a benign binary must be preserved".to_string());
        }
        Ok(())
    }

    #[test]
    fn hostile_corpus_prepare_keeps_a_clean_package_borrowed_and_unnoticed() -> Result<(), String> {
        let bytes = zip_with(&[
            ("ppt/presentation.xml", b"<p:presentation/>".to_vec()),
            ("ppt/media/chart.bin", vec![0x01, 0x02]),
        ]);
        let prepared = prepare_office_upload(&bytes)
            .map_err(|error| format!("clean package must pass: {}", error.log_detail()))?;
        if prepared.notice.is_some() {
            return Err("no macro means no notice".to_string());
        }
        match prepared.bytes {
            Cow::Borrowed(slice) if slice.as_ptr() == bytes.as_ptr() => Ok(()),
            Cow::Owned(_) => Err("a clean package must not be copied".to_string()),
            Cow::Borrowed(_) => Err("the original payload must be borrowed".to_string()),
        }
    }

    #[test]
    fn hostile_corpus_gate_refuses_real_vba_but_accepts_a_benign_binary() -> Result<(), String> {
        let clean_scanner: Box<crate::upload_hostile::MalwareScanner> =
            Box::new(|_, _| crate::upload_hostile::MalwareScanDecision::Clean {
                scanner: "corpus-test".to_string(),
            });
        let policy = crate::upload_hostile::HostileFilePolicy::default();
        let macro_bytes = zip_with(&[
            ("ppt/presentation.xml", b"<p:presentation/>".to_vec()),
            ("ppt/vbaProject.bin", vec![0xd0, 0xcf, 0x11, 0xe0]),
        ]);
        let macro_error = crate::upload_hostile::inspect_hostile_upload_with_scanner(
            &macro_bytes,
            "general_bots_presentation.pptx",
            &policy,
            Some(clean_scanner.as_ref()),
        )
        .err()
        .ok_or_else(|| "the shared gate must still refuse real VBA".to_string())?;
        if !matches!(macro_error, UploadError::ArchiveMacro { .. }) {
            return Err(format!("expected ArchiveMacro, got {}", macro_error.code()));
        }

        let benign_bytes = zip_with(&[("ppt/media/chart.bin", vec![0x01, 0x02])]);
        crate::upload_hostile::inspect_hostile_upload_with_scanner(
            &benign_bytes,
            "general_bots_presentation.pptx",
            &policy,
            Some(clean_scanner.as_ref()),
        )
        .map_err(|error| format!("benign binary must pass: {}", error.log_detail()))?;
        Ok(())
    }
}
