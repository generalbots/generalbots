//! Minimal dependency-free PDF writer for the `GENERATE PDF` keyword.
//!
//! The keyword receives an HTML template rendered with the caller's data. This
//! module turns that HTML into a valid PDF 1.4 document using the built-in
//! Helvetica font: text is extracted from the markup, wrapped, paginated and
//! written as a content stream, so callers get a real `%PDF` file instead of
//! markup saved under a `.pdf` name.

const PAGE_WIDTH: usize = 612; // US Letter width in points
const PAGE_HEIGHT: usize = 792; // US Letter height in points
const MARGIN: usize = 72;
const FONT_SIZE: usize = 11;
const LEADING: usize = 15;
const CHARS_PER_LINE: usize = 92;
const LINES_PER_PAGE: usize = 43;

struct PdfObject {
    id: u32,
    content: Vec<u8>,
}

/// Convert an HTML document (already rendered with its data) into PDF bytes.
pub fn html_to_pdf(html: &str) -> Vec<u8> {
    let text = html_to_text(html);
    let lines = wrap_lines(&text, CHARS_PER_LINE);
    let pages = paginate(&lines);

    let mut objects: Vec<PdfObject> = Vec::new();

    // 1: Catalog, 2: Pages, 3: Font. Page + content stream pairs follow.
    objects.push(PdfObject {
        id: 1,
        content: b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
    });

    let kids: Vec<String> = (0..pages.len())
        .map(|i| format!("{} 0 R", 4 + i * 2))
        .collect();
    objects.push(PdfObject {
        id: 2,
        content: format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            kids.join(" "),
            pages.len()
        )
        .into_bytes(),
    });

    objects.push(PdfObject {
        id: 3,
        content: b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec(),
    });

    for (idx, page_lines) in pages.iter().enumerate() {
        let page_id = (4 + idx * 2) as u32;
        let content_id = page_id + 1;

        objects.push(PdfObject {
            id: page_id,
            content: format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_WIDTH} {PAGE_HEIGHT}] \
                 /Resources << /Font << /F1 3 0 R >> >> /Contents {content_id} 0 R >>"
            )
            .into_bytes(),
        });

        let stream = build_content_stream(page_lines);
        objects.push(PdfObject {
            id: content_id,
            content: build_stream_object(&stream),
        });
    }

    assemble_pdf(&objects)
}

/// Extract readable text from HTML: block tags become line breaks, list items
/// become `- ` bullets, entities are decoded and whitespace is normalized.
fn html_to_text(html: &str) -> String {
    let stripped = strip_tags(html);
    let decoded = decode_entities(&stripped);

    let mut lines: Vec<String> = Vec::new();
    for raw in decoded.lines() {
        let line = raw.split_whitespace().collect::<Vec<_>>().join(" ");
        if line.is_empty() {
            if lines.last().map(|l| !l.is_empty()).unwrap_or(false) {
                lines.push(String::new());
            }
        } else {
            lines.push(line);
        }
    }
    while lines.last().map(String::is_empty).unwrap_or(false) {
        lines.pop();
    }
    if lines.is_empty() {
        lines.push("(empty document)".to_string());
    }
    lines.join("\n")
}

/// Walk the markup once: emit text outside tags and a line break (plus a
/// bullet for list items) at every block boundary.
fn strip_tags(html: &str) -> String {
    let mut result = String::new();
    let mut chars = html.char_indices().peekable();

    while let Some((_, ch)) = chars.next() {
        if ch != '<' {
            result.push(ch);
            continue;
        }

        // Collect the tag name up to `>`, then emit its separator.
        let mut tag = String::new();
        for (_, c) in chars.by_ref() {
            if c == '>' {
                break;
            }
            tag.push(c);
        }
        let name = tag
            .trim_start_matches('/')
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches('/')
            .to_ascii_lowercase();

        match name.as_str() {
            "li" => {
                if !result.ends_with('\n') && !result.is_empty() {
                    result.push('\n');
                }
                result.push_str("- ");
            }
            "br" | "p" | "div" | "tr" | "table" | "section" | "article" | "ul" | "ol" | "h1"
            | "h2" | "h3" | "h4" | "h5" | "h6" | "hr" | "blockquote" | "header" | "footer" => {
                if !result.ends_with('\n') {
                    result.push('\n');
                }
            }
            _ => {}
        }
    }
    result
}

fn decode_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn build_content_stream(lines: &[String]) -> Vec<u8> {
    let mut ops = String::new();
    ops.push_str(&format!(
        "BT /F1 {FONT_SIZE} Tf {MARGIN} {} Td {LEADING} TL\n",
        PAGE_HEIGHT - MARGIN
    ));
    for line in lines {
        ops.push_str(&format!("({}) Tj T*\n", pdf_escape(line)));
    }
    ops.push_str("ET");
    ops.into_bytes()
}

fn build_stream_object(stream: &[u8]) -> Vec<u8> {
    let mut content = format!("<< /Length {} >>\nstream\n", stream.len()).into_bytes();
    content.extend_from_slice(stream);
    content.extend_from_slice(b"\nendstream");
    content
}

fn assemble_pdf(objects: &[PdfObject]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

    let mut offsets = Vec::with_capacity(objects.len());
    for obj in objects {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", obj.id).as_bytes());
        out.extend_from_slice(&obj.content);
        out.extend_from_slice(b"\nendobj\n");
    }

    let xref_offset = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for off in &offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }

    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

fn pdf_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            c if (c as u32) < 128 => out.push(c),
            // Helvetica WinAnsi cannot represent arbitrary unicode; degrade safely.
            _ => out.push('?'),
        }
    }
    out
}

fn wrap_lines(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for raw in text.lines() {
        let line = raw.trim_end();
        if line.is_empty() {
            lines.push(String::new());
            continue;
        }
        if line.chars().count() <= width {
            lines.push(line.to_string());
            continue;
        }
        let mut current = String::new();
        for word in line.split_whitespace() {
            if current.is_empty() {
                current = word.to_string();
            } else if current.chars().count() + 1 + word.chars().count() <= width {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current);
                current = word.to_string();
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn paginate(lines: &[String]) -> Vec<Vec<String>> {
    let mut pages = Vec::new();
    let mut current = Vec::new();
    for line in lines {
        current.push(line.clone());
        if current.len() >= LINES_PER_PAGE {
            pages.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() || pages.is_empty() {
        pages.push(current);
    }
    pages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_a_valid_pdf_header_and_trailer() {
        let pdf = html_to_pdf("<html><body><h1>Hello</h1><p>World</p></body></html>");
        assert!(pdf.starts_with(b"%PDF-1.4"));
        let tail = String::from_utf8_lossy(&pdf[pdf.len() - 32..]);
        assert!(tail.contains("%%EOF"));
        assert!(pdf.windows(4).any(|w| w == b"xref"));
    }

    #[test]
    fn renders_visible_text_from_html() {
        let pdf = html_to_pdf("<h1>Quarterly Report</h1><p>Revenue grew 12%.</p>");
        let body = String::from_utf8_lossy(&pdf);
        assert!(body.contains("Quarterly Report"));
        assert!(body.contains("Revenue grew 12%."));
        assert!(!body.contains("<h1>"));
    }

    #[test]
    fn decodes_entities_and_escapes_pdf_delimiters() {
        let pdf = html_to_pdf("<p>Tom &amp; Jerry (2026)</p>");
        let body = String::from_utf8_lossy(&pdf);
        assert!(body.contains("Tom & Jerry \\(2026\\)"));
    }

    #[test]
    fn long_documents_paginate() {
        let html = (0..200)
            .map(|i| format!("<p>Line number {i} of the report.</p>"))
            .collect::<String>();
        let pdf = html_to_pdf(&html);
        let body = String::from_utf8_lossy(&pdf);
        // Two or more pages means more than one /Type /Page object plus kids.
        let page_count = body.matches("/Type /Page ").count();
        assert!(page_count >= 2, "expected pagination, got {page_count} pages");
        assert!(body.contains("/Count 2") || body.contains("/Count 3") || body.contains("/Count 4"));
    }
}
