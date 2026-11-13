//! PDF → editable Docs HTML (#2254).
//!
//! Docs could export a document as PDF but never import one: attaching a PDF
//! fell through `detect_import_format` to `Unsupported format: pdf` and the
//! Drive "open in Docs" path answered `Unsupported file type for docs editor`.
//! Reading a PDF is the inverse of [`crate::pdf`], so the extraction lives here
//! and is shared by both entry points.
//!
//! Pages are read in order. Because a PDF stores text as absolutely placed
//! runs, every visual line arrives as its own line; consecutive lines of the
//! same block are re-joined into one paragraph, and a blank line or a form feed
//! (page boundary) starts the next one.

use crate::utils_format::html_escape;

/// Leading bytes every PDF file starts with.
pub const PDF_MAGIC: &[u8] = b"%PDF-";

/// True when `bytes` looks like a PDF by its signature.
#[must_use]
pub fn looks_like_pdf(bytes: &[u8]) -> bool {
    bytes.starts_with(PDF_MAGIC)
}

/// Converts PDF bytes into editor HTML: one `<p>` per recovered paragraph,
/// pages kept in reading order.
///
/// Every failure mode returns a message safe to show to the user — a corrupt,
/// password-protected, empty or image-only PDF never panics.
pub fn pdf_to_editor_html(bytes: &[u8]) -> Result<String, String> {
    if bytes.is_empty() {
        return Err("The PDF file is empty".to_string());
    }
    if !looks_like_pdf(bytes) {
        return Err("The uploaded file is not a PDF document".to_string());
    }

    let text = pdf_extract::extract_text_from_mem(bytes).map_err(|error| {
        log::warn!("docs import: PDF text extraction failed: {error}");
        "The PDF could not be read: it may be corrupt, truncated or password-protected".to_string()
    })?;

    let paragraphs = paragraphs_from_text(&text);
    if paragraphs.is_empty() {
        return Err(
            "The PDF has no extractable text: it may contain only scanned images".to_string(),
        );
    }

    Ok(paragraphs
        .iter()
        .map(|paragraph| format!("<p>{}</p>", html_escape(paragraph)))
        .collect::<Vec<_>>()
        .join("\n"))
}

/// Recovers paragraphs from extracted PDF text.
///
/// A wrapped line continues the paragraph it belongs to, so it is joined with a
/// single space; an empty line, or a form feed emitted at a page boundary, ends
/// the paragraph. This keeps page order while restoring editable blocks.
#[must_use]
pub fn paragraphs_from_text(text: &str) -> Vec<String> {
    let normalized = text
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        // pdf-extract separates pages with a form feed; treat it as a break so
        // the last paragraph of one page never merges into the next page.
        .replace('\u{000C}', "\n\n");

    let mut paragraphs: Vec<String> = Vec::new();
    let mut current = String::new();

    for raw_line in normalized.split('\n') {
        let line = raw_line.trim();
        if line.is_empty() {
            push_paragraph(&mut paragraphs, &mut current);
            continue;
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(line);
    }
    push_paragraph(&mut paragraphs, &mut current);

    paragraphs
}

fn push_paragraph(paragraphs: &mut Vec<String>, current: &mut String) {
    let paragraph = current.trim();
    if !paragraph.is_empty() {
        paragraphs.push(paragraph.to_string());
    }
    current.clear();
}

#[cfg(test)]
#[path = "import_pdf_tests.rs"]
mod tests;
