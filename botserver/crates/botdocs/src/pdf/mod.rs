//! Structure-aware PDF export for the Docs app.
//!
//! The pipeline is deliberately split so each stage is testable on its own:
//!
//! 1. [`blocks`] recovers document structure (headings, lists, page breaks)
//!    from the editor HTML.
//! 2. [`layout`] wraps and paginates the blocks into A4 pages.
//! 3. [`writer`] serialises the pages as a PDF 1.4 stream.
//!
//! The previous implementation stripped every tag and paginated the resulting
//! plain text with fixed `90 × 43` characters, which flattened numbering and
//! ignored page breaks (#2250, #2251).

pub mod blocks;
pub mod layout;
pub mod writer;

pub use blocks::{Block, BlockKind, InlineRun, parse_blocks};
pub use layout::{Line, Page, PageFrame, Segment, layout_pages, layout_pages_with_hf};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HeaderFooter {
    pub header: Option<String>,
    pub footer: Option<String>,
}

impl HeaderFooter {
    pub fn is_empty(&self) -> bool {
        self.header.as_deref().unwrap_or("").trim().is_empty()
            && self.footer.as_deref().unwrap_or("").trim().is_empty()
    }

    pub fn substitute(&self, page: usize, pages: usize) -> (String, String) {
        let replace = |text: &str| {
            text.replace("{{page}}", &page.to_string())
                .replace("{{pages}}", &pages.to_string())
                .replace("{page}", &page.to_string())
                .replace("{pages}", &pages.to_string())
        };
        (
            self.header.as_deref().map(replace).unwrap_or_default(),
            self.footer.as_deref().map(replace).unwrap_or_default(),
        )
    }
}

/// Renders editor HTML as a PDF document.
///
/// Explicit page breaks (`<div class="gb-doc-page-break"></div>` from an
/// imported .docx, `<hr class="gb-page-break">` from Ctrl+Enter) start a new
/// page; headings, bold/italic runs and ordered list numbering are preserved.
/// Header/footer zones (`.docs-header-zone`/`.docs-footer-zone`) repeat on
/// every page with `{{page}}`/`{{pages}}` tokens resolved.
pub fn html_to_pdf(html: &str) -> Vec<u8> {
    let (body, hf) = split_header_footer(html);
    let blocks = parse_blocks(&body);
    let pages = if hf.is_empty() {
        layout_pages(&blocks)
    } else {
        layout_pages_with_hf(&blocks, &hf)
    };
    writer::write_pdf(&pages)
}

fn split_header_footer(html: &str) -> (String, HeaderFooter) {
    let mut out = String::with_capacity(html.len());
    let mut hf = HeaderFooter::default();
    let mut rest = html;
    while let Some(start) = rest.find("<div") {
        out.push_str(&rest[..start]);
        let after = &rest[start..];
        let Some(end_tag) = after.find('>') else {
            out.push_str(after);
            return (out, hf);
        };
        let tag = &after[..=end_tag];
        let class = if tag.contains("docs-header-zone") {
            Some("header")
        } else if tag.contains("docs-footer-zone") {
            Some("footer")
        } else {
            None
        };
        if let Some(kind) = class {
            let Some(close_rel) = after.to_ascii_lowercase().find("</div>") else {
                out.push_str(after);
                return (out, hf);
            };
            let inner = &after[end_tag + 1..close_rel];
            let text = crate::storage_core::strip_html(inner).trim().to_string();
            if !text.is_empty() {
                match kind {
                    "header" => hf.header = Some(text),
                    "footer" => hf.footer = Some(text),
                    _ => {}
                }
            }
            rest = &after[close_rel + "</div>".len()..];
        } else {
            out.push_str(tag);
            rest = &after[end_tag + 1..];
        }
    }
    out.push_str(rest);
    (out, hf)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
