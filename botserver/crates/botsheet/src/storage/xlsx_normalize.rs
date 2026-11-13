//! OOXML repairs applied before the reader sees a package.
//!
//! `umya-spreadsheet` 3.1 parses `<mergeCell>` with a strict start/end-tag pair
//! and panics on the self-closing form (`<mergeCell ref="B6:C6"/>`) that
//! `rust_xlsxwriter` — and most generators, including our own export — emit.
//! A workbook exported here could therefore not be re-imported (#2152).

use std::io::{Cursor, Read, Write};

/// Repairs `<mergeCell ref="B6:C6"/<mergeCell …` — a tag that lost its `>`
/// because an older layout patcher wrote `"` where it meant `"">` (#2152).
/// Without this a workbook patched by the previous build cannot be opened at
/// all, and `"/>` never appears in valid XML text, so the rewrite is safe.
pub fn repair_missing_tag_end(xml: &str) -> String {
    xml.replace("\"/<", "\"/><")
}

/// Expands `<mergeCell …/>` into `<mergeCell …></mergeCell>` inside a single
/// worksheet XML document. Any other markup is copied verbatim.
pub fn expand_merge_cells(xml: &str) -> String {
    let mut out = String::with_capacity(xml.len() + 32);
    let mut rest = xml;
    while let Some(position) = rest.find("<mergeCell ref=") {
        out.push_str(&rest[..position]);
        let tail = &rest[position..];
        let Some(end) = tail.find('>') else {
            out.push_str(tail);
            return out;
        };
        let tag = &tail[..=end];
        if tag.ends_with("/>") {
            out.push_str(&tag[..tag.len() - 2]);
            out.push_str("></mergeCell>");
        } else {
            out.push_str(tag);
        }
        rest = &tail[end + 1..];
    }
    out.push_str(rest);
    out
}

/// Rebuilds the package with the repaired worksheet parts. Returns the input
/// untouched when nothing needed fixing.
pub fn normalise_merge_cells(bytes: &[u8]) -> Result<Vec<u8>, String> {
    if !bytes.starts_with(b"PK\x03\x04") {
        return Ok(bytes.to_vec());
    }
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("open package: {e}"))?;
    let mut repaired = false;
    let mut parts: Vec<(String, Vec<u8>)> = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|e| format!("read entry {index}: {e}"))?;
        let name = entry.name().map(|n| n.to_string()).unwrap_or_default();
        let mut buffer = Vec::new();
        entry
            .read_to_end(&mut buffer)
            .map_err(|e| format!("decompress {name}: {e}"))?;
        if name.starts_with("xl/worksheets/") && name.ends_with(".xml") {
            if let Ok(text) = std::str::from_utf8(&buffer) {
                let patched = expand_merge_cells(&repair_missing_tag_end(text));
                if patched.len() != text.len() {
                    repaired = true;
                    buffer = patched.into_bytes();
                }
            }
        }
        parts.push((name, buffer));
    }
    if !repaired {
        return Ok(bytes.to_vec());
    }
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default();
    for (name, buffer) in parts {
        writer
            .start_file(name, options)
            .map_err(|e| format!("rebuild package: {e}"))?;
        writer
            .write_all(&buffer)
            .map_err(|e| format!("rebuild package: {e}"))?;
    }
    let cursor = writer
        .finish()
        .map_err(|e| format!("finish package: {e}"))?;
    Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_self_closing_merge_cells() {
        let xml = r#"<worksheet><mergeCells count="2"><mergeCell ref="B6:C6"/><mergeCell ref="D6:E6"></mergeCell></mergeCells></worksheet>"#;
        let patched = expand_merge_cells(xml);
        assert!(patched.contains(r#"<mergeCell ref="B6:C6"></mergeCell>"#));
        assert!(patched.contains(r#"<mergeCell ref="D6:E6"></mergeCell>"#));
        assert_eq!(patched.matches("</mergeCell>").count(), 2);
    }

    #[test]
    fn repairs_tags_that_lost_their_closing_bracket() {
        let xml = r#"<mergeCells count="2"><mergeCell ref="D6:E6"/<mergeCell ref="B6:C6"/</mergeCells>"#;
        let patched = expand_merge_cells(&repair_missing_tag_end(xml));
        assert_eq!(
            patched,
            r#"<mergeCells count="2"><mergeCell ref="D6:E6"></mergeCell><mergeCell ref="B6:C6"></mergeCell></mergeCells>"#
        );
    }

    #[test]
    fn leaves_documents_without_merges_untouched() {
        let xml = r#"<worksheet><sheetData/></worksheet>"#;
        assert_eq!(expand_merge_cells(xml), xml);
    }

    #[test]
    fn non_zip_input_passes_through() {
        let bytes = b"not a package";
        assert_eq!(normalise_merge_cells(bytes).expect("ok"), bytes.to_vec());
    }
}
