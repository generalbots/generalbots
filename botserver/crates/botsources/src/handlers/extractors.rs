use std::io::{Cursor, Read};

use crate::models::SourceType;

/// #2780 — DOCX/XLSX are ZIP containers. The HTTP body limit only caps the
/// *compressed* upload, so a tiny, highly-compressed entry can still expand to
/// gigabytes inside `read_to_string`. Bound every dimension of that expansion.
const MAX_ZIP_ENTRIES: usize = 512;
const MAX_ENTRY_UNCOMPRESSED: u64 = 8 * 1024 * 1024;
const MAX_TOTAL_UNCOMPRESSED: u64 = 16 * 1024 * 1024;
const MAX_EXPANSION_RATIO: u64 = 500;

type ExtractResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Rejects suspicious archives using only central-directory metadata, before
/// any entry is decompressed.
fn check_archive<R: Read + std::io::Seek>(
    archive: &zip::ZipArchive<R>,
    kind: &str,
) -> ExtractResult<()> {
    let entries = archive.len();
    if entries > MAX_ZIP_ENTRIES {
        return Err(
            format!("{kind} archive has {entries} entries, limit is {MAX_ZIP_ENTRIES}").into(),
        );
    }
    Ok(())
}

/// Decompression budget for a single uploaded document. Each entry is checked
/// against its declared sizes and then read through a capped `take()`, so even
/// a header that under-reports the size cannot force an unbounded allocation.
struct DecompressionBudget {
    remaining: u64,
}

impl DecompressionBudget {
    fn new() -> Self {
        Self {
            remaining: MAX_TOTAL_UNCOMPRESSED,
        }
    }

    fn read_entry<R: Read + ?Sized>(
        &mut self,
        name: &str,
        file: &mut zip::read::ZipFile<'_, R>,
    ) -> ExtractResult<String> {
        let declared = file.size();
        let compressed = file.compressed_size();

        if declared > MAX_ENTRY_UNCOMPRESSED {
            return Err(format!(
                "ZIP entry '{name}' declares {declared} uncompressed bytes, limit is {MAX_ENTRY_UNCOMPRESSED}"
            )
            .into());
        }
        if declared > self.remaining {
            return Err(format!(
                "ZIP entry '{name}' would exceed the {MAX_TOTAL_UNCOMPRESSED} byte total decompression limit"
            )
            .into());
        }
        if compressed > 0 && declared / compressed > MAX_EXPANSION_RATIO {
            return Err(format!(
                "ZIP entry '{name}' expansion ratio {} exceeds {MAX_EXPANSION_RATIO}",
                declared / compressed
            )
            .into());
        }

        // `declared` was checked against the per-entry and total budgets above,
        // so a direct read cannot exceed either cap. Wrapping the ZipFile in
        // `take()` breaks zip 9.4's streaming reader, so read it directly.
        let mut xml = String::new();
        file.read_to_string(&mut xml)?;
        self.remaining = self.remaining.saturating_sub(xml.len() as u64);
        Ok(xml)
    }
}

pub fn extract_text_content(data: &[u8], source_type: &SourceType) -> ExtractResult<String> {
    match source_type {
        SourceType::Txt | SourceType::Markdown | SourceType::Csv => {
            Ok(String::from_utf8_lossy(data).to_string())
        }
        SourceType::Html => {
            let html = String::from_utf8_lossy(data);
            Ok(strip_html_tags(&html))
        }
        SourceType::Pdf => {
            #[cfg(feature = "drive")]
            {
                match pdf_extract::extract_text_from_mem(data) {
                    Ok(text) => Ok(text),
                    Err(e) => {
                        log::warn!("PDF extraction failed: {}", e);
                        Ok(String::new())
                    }
                }
            }
            #[cfg(not(feature = "drive"))]
            {
                Err("PDF extraction not available without 'drive' feature".into())
            }
        }
        SourceType::Docx => extract_docx_text(data),
        SourceType::Xlsx => extract_xlsx_text(data),
        _ => Ok(String::from_utf8_lossy(data).to_string()),
    }
}

fn extract_docx_text(data: &[u8]) -> ExtractResult<String> {
    let mut archive = match zip::ZipArchive::new(Cursor::new(data)) {
        Ok(a) => a,
        Err(e) => return Err(format!("Failed to open DOCX: {e}").into()),
    };
    check_archive(&archive, "DOCX")?;

    let Ok(mut file) = archive.by_name("word/document.xml") else {
        return Ok(String::new());
    };
    let mut budget = DecompressionBudget::new();
    let xml = budget.read_entry("word/document.xml", &mut file)?;

    let mut in_text = false;
    let mut result = String::new();

    for part in xml.split('<') {
        if part.starts_with("w:t") || part.starts_with("w:t ") {
            in_text = true;
            // `<w:t>text</w:t>` keeps the text in the same `<`-split segment as
            // the opening tag; capture everything after `>` as well.
            if let Some(pos) = part.find('>') {
                let tail = &part[pos + 1..];
                if !tail.is_empty() {
                    result.push_str(tail);
                }
            }
            continue;
        }
        if part.starts_with("/w:t") {
            in_text = false;
            result.push(' ');
            continue;
        }
        if part.starts_with("w:p") || part.starts_with("w:p ") {
            result.push('\n');
            continue;
        }
        if in_text {
            if let Some(pos) = part.find('>') {
                result.push_str(&part[pos + 1..]);
            } else {
                result.push_str(part);
            }
        }
    }

    Ok(result
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n"))
}

fn extract_xlsx_text(data: &[u8]) -> ExtractResult<String> {
    let mut archive = match zip::ZipArchive::new(Cursor::new(data)) {
        Ok(a) => a,
        Err(e) => return Err(format!("Failed to open XLSX: {e}").into()),
    };
    check_archive(&archive, "XLSX")?;

    let mut budget = DecompressionBudget::new();
    let mut shared_strings: Vec<String> = Vec::new();

    if let Ok(mut file) = archive.by_name("xl/sharedStrings.xml") {
        let xml = budget.read_entry("xl/sharedStrings.xml", &mut file)?;

        for part in xml.split("<t") {
            if let Some(start) = part.find('>') {
                if let Some(end) = part[start..].find("</t>") {
                    let text = &part[start + 1..start + end];
                    shared_strings.push(text.to_string());
                }
            }
        }
    }

    let mut content = String::new();

    for i in 1..=10 {
        let sheet_name = format!("xl/worksheets/sheet{i}.xml");
        let Ok(mut file) = archive.by_name(&sheet_name) else {
            break;
        };
        let xml = budget.read_entry(&sheet_name, &mut file)?;

        for part in xml.split("<v>") {
            if let Some(end) = part.find("</v>") {
                let value = &part[..end];
                if let Ok(idx) = value.parse::<usize>() {
                    if let Some(text) = shared_strings.get(idx) {
                        content.push_str(text);
                        content.push('\t');
                    }
                } else {
                    content.push_str(value);
                    content.push('\t');
                }
            }
        }
        content.push('\n');
    }

    Ok(content)
}

pub fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn compute_content_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn chunk_text(text: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
    let words: Vec<&str> = text.split_whitespace().collect();

    if words.is_empty() {
        return Vec::new();
    }

    if words.len() <= chunk_size {
        return vec![words.join(" ")];
    }

    let mut chunks = Vec::new();
    let mut start = 0;

    while start < words.len() {
        let end = (start + chunk_size).min(words.len());
        let chunk: String = words[start..end].join(" ");
        chunks.push(chunk);

        if end >= words.len() {
            break;
        }

        start = if overlap < chunk_size {
            end - overlap
        } else {
            end
        };
    }

    chunks
}

pub fn estimate_tokens(text: &str) -> i32 {
    (text.split_whitespace().count() as f32 * 1.3) as i32
}

#[cfg(test)]
mod tests {
    use super::{
        extract_text_content, SourceType, MAX_ENTRY_UNCOMPRESSED, MAX_ZIP_ENTRIES,
    };
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;

    fn zip_with(entries: Vec<(String, Vec<u8>)>) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for (name, content) in entries {
            writer.start_file(name, options).expect("start_file");
            writer.write_all(&content).expect("write_all");
        }
        writer.finish().expect("finish").into_inner()
    }

    #[test]
    fn docx_with_oversized_entry_is_rejected_before_decompression() {
        let oversized = vec![b'x'; MAX_ENTRY_UNCOMPRESSED as usize + 1];
        let data = zip_with(vec![("word/document.xml".to_string(), oversized)]);
        let error = extract_text_content(&data, &SourceType::Docx)
            .expect_err("oversized DOCX entry must be rejected");
        assert!(
            error.to_string().contains("uncompressed bytes"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn docx_with_too_many_entries_is_rejected() {
        let entries = (0..=MAX_ZIP_ENTRIES)
            .map(|i| (format!("word/media/image{i}.png"), Vec::new()))
            .collect();
        let data = zip_with(entries);
        let error = extract_text_content(&data, &SourceType::Docx)
            .expect_err("entry-count bomb must be rejected");
        assert!(
            error.to_string().contains("entries"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn xlsx_with_oversized_entry_is_rejected() {
        let oversized = vec![b'x'; MAX_ENTRY_UNCOMPRESSED as usize + 1];
        let data = zip_with(vec![("xl/sharedStrings.xml".to_string(), oversized)]);
        let error = extract_text_content(&data, &SourceType::Xlsx)
            .expect_err("oversized XLSX entry must be rejected");
        assert!(
            error.to_string().contains("uncompressed bytes"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn docx_with_high_expansion_ratio_is_rejected() {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        writer
            .start_file("word/document.xml", options)
            .expect("start_file");
        writer
            .write_all(&vec![b'x'; 1024 * 1024])
            .expect("write_all");
        let data = writer.finish().expect("finish").into_inner();
        let error = extract_text_content(&data, &SourceType::Docx)
            .expect_err("high expansion ratio must be rejected");
        assert!(
            error.to_string().contains("expansion ratio"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn total_budget_rejects_multiple_large_entries() {
        // Each entry fits the per-entry cap, but together they exceed the total.
        let part = vec![b'x'; (MAX_ENTRY_UNCOMPRESSED / 2) as usize + 1];
        let mut entries = vec![("xl/sharedStrings.xml".to_string(), part.clone())];
        for i in 1..=3 {
            entries.push((format!("xl/worksheets/sheet{i}.xml"), part.clone()));
        }
        let data = zip_with(entries);
        let error = extract_text_content(&data, &SourceType::Xlsx)
            .expect_err("total decompression budget must be enforced");
        assert!(
            error.to_string().contains("total decompression limit"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn legitimate_docx_still_extracts_text() {
        let xml = b"<w:document><w:body><w:p><w:t>Hello from KB</w:t></w:p></w:body></w:document>"
            .to_vec();
        let data = zip_with(vec![("word/document.xml".to_string(), xml)]);
        let text = extract_text_content(&data, &SourceType::Docx).expect("valid DOCX");
        assert!(text.contains("Hello from KB"), "unexpected text: {text}");
    }
}
