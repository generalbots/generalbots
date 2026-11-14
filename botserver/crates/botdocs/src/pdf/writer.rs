//! Minimal PDF 1.4 writer for laid-out pages.
//!
//! Only the built-in Helvetica family is used, so no font embedding is
//! required. Text is emitted with explicit text matrices: every line is
//! positioned independently, which keeps indentation and per-line font sizes
//! faithful to the block layout.

use super::layout::{Line, MARGIN, PAGE_HEIGHT, PAGE_WIDTH, Page};

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
    format!("<< /Type /Font /Subtype /Type1 /BaseFont /{base_font} /Encoding /WinAnsiEncoding >>")
        .into_bytes()
}

struct PdfObject {
    id: u32,
    content: Vec<u8>,
}

/// First page object id; page/content pairs follow the four font objects.
const FIRST_PAGE_OBJECT: u32 = 7;

fn page_object_id(page_index: usize) -> u32 {
    FIRST_PAGE_OBJECT + page_index as u32 * 2
}

/// Internal destinations (`/Names /Dests`) collected from the heading anchors
/// laid out on each page, sorted by name as the PDF name tree requires.
///
/// Duplicated anchors keep their first (top-most) occurrence so a link always
/// lands on the first heading carrying that id.
fn collect_destinations(pages: &[Page]) -> Vec<(String, usize, f32)> {
    let mut out: Vec<(String, usize, f32)> = Vec::new();
    for (page_index, page) in pages.iter().enumerate() {
        for (name, y) in &page.anchors {
            if out.iter().any(|(existing, _, _)| existing == name) {
                continue;
            }
            out.push((name.clone(), page_index, *y));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Link annotations for the lines that carry a same-document link (#2252).
///
/// A link whose destination is not in the document is dropped: a `/Dest` that
/// no name tree resolves would be a dead annotation.
fn link_annotations(page: &Page, destinations: &[(String, usize, f32)]) -> Vec<String> {
    let mut annotations = Vec::new();
    for line in &page.lines {
        let Some(target) = line.link.as_deref() else {
            continue;
        };
        if !destinations.iter().any(|(name, _, _)| name == target) {
            continue;
        }
        let left = line.x;
        let right = (line.x + line.width.max(4.0)).min(PAGE_WIDTH - MARGIN * 0.5);
        let bottom = line.y - 2.0;
        let top = line.y + line.size;
        annotations.push(format!(
            "<< /Type /Annot /Subtype /Link /Rect [{left:.1} {bottom:.1} {right:.1} {top:.1}] \
             /Border [0 0 0] /Dest ({}) >>",
            pdf_escape(target)
        ));
    }
    annotations
}

/// Builds the PDF bytes for the paginated document.
pub fn write_pdf(pages: &[Page]) -> Vec<u8> {
    let mut objects: Vec<PdfObject> = Vec::new();
    let destinations = collect_destinations(pages);

    // 1: Catalog (with the destination name tree), 2: Pages, 3-6: fonts,
    // then page/content pairs.
    let names: Vec<String> = destinations
        .iter()
        .map(|(name, page_index, y)| {
            format!(
                "({}) [{} 0 R /XYZ {MARGIN:.1} {y:.1} 0]",
                pdf_escape(name),
                page_object_id(*page_index)
            )
        })
        .collect();
    let catalog = if names.is_empty() {
        "<< /Type /Catalog /Pages 2 0 R >>".to_string()
    } else {
        format!(
            "<< /Type /Catalog /Pages 2 0 R /Names << /Dests << /Names [{}] >> >> >>",
            names.join(" ")
        )
    };
    objects.push(PdfObject {
        id: 1,
        content: catalog.into_bytes(),
    });

    let kids: Vec<String> = (0..pages.len())
        .map(|index| format!("{} 0 R", page_object_id(index)))
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
        let page_id = page_object_id(index);
        let content_id = page_id + 1;
        let annots = link_annotations(page, &destinations);
        let annots_entry = if annots.is_empty() {
            String::new()
        } else {
            format!(" /Annots [{}]", annots.join(" "))
        };

        objects.push(PdfObject {
            id: page_id,
            content: format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_WIDTH:.1} {PAGE_HEIGHT:.1}] \
                 /Resources << /Font << /F1 3 0 R /F2 4 0 R /F3 5 0 R /F4 6 0 R >> >> \
                 /Contents {content_id} 0 R{annots_entry} >>"
            )
            .into_bytes(),
        });

        let stream = build_content_stream(page);
        objects.push(PdfObject {
            id: content_id,
            content: build_stream_object(&stream),
        });
    }

    assemble_pdf(&objects)
}

fn build_content_stream(page: &Page) -> Vec<u8> {
    let mut ops = String::from("BT\n");
    let mut current_font: Option<(&'static str, u32)> = None;

    for line in &page.lines {
        let size = line.size.max(1.0);
        ops.push_str(&format!("1 0 0 1 {:.1} {:.1} Tm\n", line.x, line.y));
        for segment in &line.segments {
            let resource = font_resource(segment.mono, segment.bold, segment.italic);
            let size_key = (size * 10.0).round() as u32;
            if current_font != Some((resource, size_key)) {
                ops.push_str(&format!("/{resource} {:.1} Tf\n", size_key as f32 / 10.0));
                current_font = Some((resource, size_key));
            }
            ops.push_str(&format!("({}) Tj\n", pdf_escape(&segment.text)));
        }
    }

    // Repeating header/footer zones (#2155 DOC-08): rendered on every page.
    if !page.header.trim().is_empty() {
        draw_simple_text(
            &mut ops,
            &page.header,
            MARGIN,
            PAGE_HEIGHT - MARGIN + 2.0,
            9.0,
        );
    }
    if !page.footer.trim().is_empty() {
        draw_simple_text(&mut ops, &page.footer, MARGIN, MARGIN - 14.0, 9.0);
    }

    ops.push_str("ET");
    ops.into_bytes()
}

fn draw_simple_text(ops: &mut String, text: &str, x: f32, y: f32, size: f32) {
    let clean: String = text.chars().filter(|c| *c != '\n' && *c != '\r').collect();
    if clean.trim().is_empty() {
        return;
    }
    ops.push_str(&format!("/F1 {:.1} Tf\n", size));
    ops.push_str(&format!("1 0 0 1 {:.1} {:.1} Tm\n", x, y));
    ops.push_str(&format!("({}) Tj\n", pdf_escape(&clean)));
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
