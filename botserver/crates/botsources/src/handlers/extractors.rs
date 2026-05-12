use crate::models::SourceType;

pub fn extract_text_content(
    data: &[u8],
    source_type: &SourceType,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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

fn extract_docx_text(
    data: &[u8],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use std::io::{Cursor, Read};

    let cursor = Cursor::new(data);
    let mut archive = match zip::ZipArchive::new(cursor) {
        Ok(a) => a,
        Err(e) => return Err(format!("Failed to open DOCX: {e}").into()),
    };

    let Ok(mut file) = archive.by_name("word/document.xml") else {
        return Ok(String::new());
    };

    let mut xml = String::new();
    file.read_to_string(&mut xml)?;

    let mut in_text = false;
    let mut result = String::new();

    for part in xml.split('<') {
        if part.starts_with("w:t") || part.starts_with("w:t ") {
            in_text = true;
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

fn extract_xlsx_text(
    data: &[u8],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use std::io::{Cursor, Read};

    let cursor = Cursor::new(data);
    let mut archive = match zip::ZipArchive::new(cursor) {
        Ok(a) => a,
        Err(e) => return Err(format!("Failed to open XLSX: {e}").into()),
    };

    let mut shared_strings: Vec<String> = Vec::new();

    if let Ok(mut file) = archive.by_name("xl/sharedStrings.xml") {
        let mut xml = String::new();
        file.read_to_string(&mut xml)?;

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

        let mut xml = String::new();
        file.read_to_string(&mut xml)?;

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
