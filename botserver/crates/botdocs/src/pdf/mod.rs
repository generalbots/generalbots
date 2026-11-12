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

pub use blocks::{parse_blocks, Block, BlockKind, InlineRun};
pub use layout::{layout_pages, Line, Page, Segment};

/// Renders editor HTML as a PDF document.
///
/// Explicit page breaks (`<div class="gb-doc-page-break"></div>` from an
/// imported .docx, `<hr class="gb-page-break">` from Ctrl+Enter) start a new
/// page; headings, bold/italic runs and ordered list numbering are preserved.
pub fn html_to_pdf(html: &str) -> Vec<u8> {
    let blocks = parse_blocks(html);
    let pages = layout_pages(&blocks);
    writer::write_pdf(&pages)
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
