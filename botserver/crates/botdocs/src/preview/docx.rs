//! DOCX-family rendering: the editor converter plus the embedded image gallery.

use super::shell::media_gallery;

pub(super) fn render_docx(bytes: &[u8]) -> Result<String, String> {
    let html = crate::storage::convert_docx_to_html(bytes)?;
    Ok(format!(
        r#"<div class="gb-office-doc">{html}</div>{}"#,
        media_gallery(bytes, "word/media/")
    ))
}
