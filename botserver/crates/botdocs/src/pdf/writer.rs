//! Minimal PDF 1.4 writer for laid-out pages.
//!
//! Only the built-in Helvetica family is used, so no font embedding is
//! required. Text is emitted with explicit text matrices: every line is
//! positioned independently, which keeps indentation and per-line font sizes
//! faithful to the block layout.

use super::layout::{Line, Page, PAGE_HEIGHT, PAGE_WIDTH};

/// Font resource name per emphasis combination.
fn font_resource(mono: bool, bold: bool, italic: bool) -> &'static str {
    match (mono, bold, italic) {
        (true, _, _) => "F4",
        (false, true, _) => "F2",
        (false, false, true) => "F3",
        (false, false, false) => "F1",
    }
}

fn font_object(base_font: &str) -> Vec<u8> {
    format!(
        "<< /Type /Font /Subtype /Type1 /BaseFont /{base_font} /Encoding /WinAnsiEncoding >>"
    )
    .into_bytes()
}

struct PdfObject {
    id: u32,
    content: Vec<u8>,
}

/// Builds the PDF bytes for the paginated document.
pub fn write_pdf(pages: &[Page]) -> Vec<u8> {
    let mut objects: Vec<PdfObject> = Vec::new();

    // 1: Catalog, 2: Pages, 3-6: fonts, then page/content pairs.
    objects.push(PdfObject {
        id: 1,
        content: b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
    });

    const FIRST_PAGE_OBJECT: u32 = 7;

    let kids: Vec<String> = (0..pages.len())
        .map(|index| format!("{} 0 R", FIRST_PAGE_OBJECT + index as u32 * 2))
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
        content: font_object("Helvetica"),
    });
    objects.push(PdfObject {
        id: 4,
        content: font_object("Helvetica-Bold"),
    });
    objects.push(PdfObject {
        id: 5,
        content: font_object("Helvetica-Oblique"),
    });
    objects.push(PdfObject {
        id: 6,
        content: font_object("Courier"),
    });

    for (index, page) in pages.iter().enumerate() {
        let page_id = FIRST_PAGE_OBJECT + index as u32 * 2;
        let content_id = page_id + 1;

        objects.push(PdfObject {
            id: page_id,
            content: format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_WIDTH:.1} {PAGE_HEIGHT:.1}] \
                 /Resources << /Font << /F1 3 0 R /F2 4 0 R /F3 5 0 R /F4 6 0 R >> >> \
                 /Contents {content_id} 0 R >>"
            )
            .into_bytes(),
        });

        let stream = build_content_stream(&page.lines);
        objects.push(PdfObject {
            id: content_id,
            content: build_stream_object(&stream),
        });
    }

    assemble_pdf(&objects)
}

fn build_content_stream(lines: &[Line]) -> Vec<u8> {
    let mut ops = String::from("BT\n");
    let mut current_font: Option<(&'static str, u32)> = None;

    for line in lines {
        let size = line.size.max(1.0);
        ops.push_str(&format!("1 0 0 1 {:.1} {:.1} Tm\n", line.x, line.y));
        for segment in &line.segments {
            let resource = font_resource(segment.mono, segment.bold, segment.italic);
            // Font size is kept as an integer tenths value to avoid printing
            // long floats in the stream.
            let size_key = (size * 10.0).round() as u32;
            if current_font != Some((resource, size_key)) {
                ops.push_str(&format!("/{resource} {:.1} Tf\n", size_key as f32 / 10.0));
                current_font = Some((resource, size_key));
            }
            ops.push_str(&format!("({}) Tj\n", pdf_escape(&segment.text)));
        }
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
    for object in objects {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", object.id).as_bytes());
        out.extend_from_slice(&object.content);
        out.extend_from_slice(b"\nendobj\n");
    }

    let xref_offset = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for offset in &offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
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

/// WinAnsi code for a character Helvetica can render without a font embed.
///
/// Latin-1 text (Portuguese accents in a resume or contract) is preserved
/// through the WinAnsiEncoding declared on every font; typographic punctuation
/// used by word processors is mapped to its WinAnsi slot as well.
fn win_ansi_code(ch: char) -> Option<u8> {
    let code = ch as u32;
    match ch {
        '\u{2013}' => Some(0x96), // en dash
        '\u{2014}' => Some(0x97), // em dash
        '\u{2018}' => Some(0x91),
        '\u{2019}' => Some(0x92),
        '\u{201C}' => Some(0x93),
        '\u{201D}' => Some(0x94),
        '\u{2022}' => Some(0x95), // bullet
        '\u{2026}' => Some(0x85), // ellipsis
        _ if (0x20..=0x7E).contains(&code) => Some(code as u8),
        _ if (0xA0..=0xFF).contains(&code) => Some(code as u8),
        _ => None,
    }
}

fn pdf_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            _ => match win_ansi_code(ch) {
                Some(byte) if (0x20..=0x7E).contains(&(byte as u32)) => out.push(byte as char),
                Some(byte) => out.push_str(&format!("\\{byte:03o}")),
                // Outside WinAnsi there is no glyph in the base-14 fonts; a
                // question mark degrades legibly instead of corrupting bytes.
                None => out.push('?'),
            },
        }
    }
    out
}
