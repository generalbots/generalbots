//! Office preview rendering (#1672 follow-up).
//!
//! Web parity target: ChatGPT/Claude render docx/xlsx/pptx INLINE in the
//! conversation instead of handing the file to an external viewer. This
//! module converts an OOXML package to a fully sanitized, script-free HTML
//! document:
//!
//! - xlsx: one `<details>` section per sheet, tables built from cell values.
//! - docx: paragraphs as `<section>`s, tables as real `<table>`s.
//! - pptx: one `<section>` per slide.
//!
//! Every string taken from the package is escaped with [`common::escape_html`]
//! and no attribute is copied, so sanitized output cannot carry scripts,
//! frames or external fetches. XLSX support is self-contained (`xl/*` member
//! parsing) so the preview does not depend on the opt-in `kb-extraction`
//! feature. The three readers live in submodules; this file only dispatches.

mod common;
mod slides;
mod spreadsheet;
mod word;

pub use self::common::escape_html;
use self::slides::slides_html;
use self::spreadsheet::spreadsheet_html;
use self::word::word_html;

/// Builds the inline preview for an OOXML package when the kind is
/// supported. Returns `None` for anything else so the caller falls back to
/// the existing window preview.
#[must_use]
pub fn office_html(file_name: &str, bytes: &[u8]) -> Option<std::result::Result<String, String>> {
    let kind = file_name.rsplit('.').next()?.to_lowercase();
    match kind.as_str() {
        "xlsx" => Some(spreadsheet_html(file_name, bytes)),
        "docx" => Some(word_html(file_name, bytes)),
        "pptx" => Some(slides_html(file_name, bytes)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{escape_html, office_html};

    #[test]
    fn unsupported_kinds_neither_panic_nor_render() {
        assert!(office_html("photo.png", b"data").is_none());
        assert!(office_html("model.pdf", b"%PDF").is_none());
    }

    #[test]
    fn escaping_blocks_payload_html() {
        let escaped = escape_html("<script>alert(1)</script>");
        assert!(!escaped.contains('<'));
        assert!(escaped.contains("&lt;script&gt;"));
    }
}
