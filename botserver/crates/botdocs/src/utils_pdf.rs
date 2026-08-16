//! Minimal, dependency-free PDF export.
//!
//! Produces a valid PDF 1.4 document using the built-in Helvetica font.
//! Text is stripped from HTML, wrapped, paginated, and rendered as a
//! simple content stream. No external PDF library is required, keeping
//! the `docs` build lightweight and independent of the OOXML SDK.

use crate::storage_core::strip_html;

const PAGE_WIDTH: usize = 612; // US Letter width in points
const PAGE_HEIGHT: usize = 792; // US Letter height in points
const MARGIN: usize = 72;
const FONT_SIZE: usize = 11;
const LEADING: usize = 15;
const CHARS_PER_LINE: usize = 90;
const LINES_PER_PAGE: usize = 43;

struct PdfObject {
    id: u32,
    content: Vec<u8>,
}

pub fn html_to_pdf(html: &str) -> Vec<u8> {
    let text = strip_html(html);
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
        let page_id = 4 + idx * 2;
        let content_id = page_id + 1;

        objects.push(PdfObject {
            id: page_id as u32,
            content: format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_WIDTH} {PAGE_HEIGHT}] \
                 /Resources << /Font << /F1 3 0 R >> >> /Contents {content_id} 0 R >>"
            )
            .into_bytes(),
        });

        let stream = build_content_stream(page_lines);
        objects.push(PdfObject {
            id: content_id as u32,
            content: build_stream_object(&stream),
        });
    }

    assemble_pdf(&objects)
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
