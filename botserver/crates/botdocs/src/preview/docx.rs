//! DOCX-family rendering: paginated pages plus the embedded image gallery.

use super::shell::media_gallery;
use crate::storage_docx::PAGE_BREAK_MARKER;

/// Renders one DOCX as discrete pages.
///
/// The converter emits `PAGE_BREAK_MARKER` at every `<w:br w:type="page"/>`;
/// splitting on it is what makes a two-page document read as two pages instead
/// of one endless scroll (#2150). A document without explicit breaks is a
/// single page.
pub(super) fn render_docx(bytes: &[u8]) -> Result<String, String> {
    let html = crate::storage::convert_docx_to_html(bytes)?;
    let bodies: Vec<&str> = html.split(PAGE_BREAK_MARKER).collect();
    let total = bodies.len();
    let mut pages = String::new();
    for (index, body) in bodies.iter().enumerate() {
        pages.push_str(&format!(
            r#"<section class="gb-doc-page" data-page="{page}"><div class="gb-doc-page-body">{body}</div><div class="gb-doc-page-number">{page} / {total}</div></section>"#,
            page = index + 1,
            total = total,
            body = body
        ));
    }
    Ok(format!(
        r#"<div class="gb-office-doc"><div class="gb-doc-pages">{pages}</div>{gallery}</div>"#,
        gallery = media_gallery(bytes, "word/media/")
    ))
}
