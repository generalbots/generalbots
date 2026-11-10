//! Preview shell: format dispatch, escaped HTML wrapper and embedded media.
//!
//! This is the only place that emits markup around user content, so the CSS
//! and the escaping wrapper stay together here.

use base64::Engine as _;
use std::io::{Cursor, Read};

/// Renders one Office document as an HTML fragment (escaped user content).
pub fn preview_html(file_name: &str, extension: &str, bytes: &[u8]) -> Result<String, String> {
    let body = match extension {
        "docx" | "doc" | "odt" | "rtf" => super::docx::render_docx(bytes)?,
        "xlsx" | "xls" | "xlsb" | "ods" => super::sheet::spreadsheet_html(extension, bytes)?,
        "pptx" | "ppt" => super::slides::presentation_html(bytes)?,
        _ => return Err(format!("preview not supported for .{extension}")),
    };
    Ok(format!(
        r#"<div class="gb-office-preview">
<style>
.gb-office-preview{{background:#fff;color:#111827;border-radius:8px;padding:24px;max-height:70vh;overflow:auto;width:100%;font-family:Georgia,serif;line-height:1.6;font-size:14px;}}
.gb-office-preview .gb-preview-name{{font:12px/1.4 sans-serif;color:#64748b;margin-bottom:14px;letter-spacing:.02em;}}
.gb-office-preview h1,.gb-office-preview h2,.gb-office-preview h3{{color:#0f172a;margin:16px 0 8px;}}
.gb-office-preview table{{border-collapse:collapse;margin:8px 0 20px;font-size:12px;font-family:sans-serif;}}
.gb-office-preview td,.gb-office-preview th{{border:1px solid #cbd5e1;padding:4px 8px;text-align:left;vertical-align:top;}}
.gb-office-preview tr:nth-child(even) td{{background:#f1f5f9;}}
.gb-office-preview .gb-sheet-name{{font:600 13px sans-serif;color:#334155;margin:18px 0 4px;}}
.gb-office-preview .gb-slide{{border:1px solid #cbd5e1;border-radius:8px;padding:16px 20px;margin:12px 0;background:#f8fafc;}}
.gb-office-preview .gb-slide h3{{margin:0 0 8px;font:600 12px sans-serif;text-transform:uppercase;letter-spacing:.08em;color:#64748b;}}
.gb-office-preview .gb-slide p{{margin:4px 0;}}
.gb-office-preview .gb-slide-notes{{margin-top:10px;padding-top:8px;border-top:1px dashed #cbd5e1;}}
.gb-office-preview .gb-slide-notes h4{{margin:0 0 4px;font:600 11px sans-serif;text-transform:uppercase;letter-spacing:.06em;color:#64748b;}}
.gb-office-preview .gb-slide-notes p{{font-size:12px;color:#475569;font-style:italic;}}
.gb-office-preview .gb-office-media{{display:flex;flex-wrap:wrap;gap:12px;margin-top:16px;}}
.gb-office-preview .gb-office-media img{{max-width:320px;max-height:240px;border:1px solid #cbd5e1;border-radius:6px;}}
.gb-office-preview .gb-office-media figcaption{{font:11px sans-serif;color:#64748b;}}
.gb-office-preview .gb-doc-pages{{display:flex;flex-direction:column;align-items:center;gap:18px;background:#e2e8f0;padding:18px;border-radius:8px;}}
.gb-office-preview .gb-doc-page{{position:relative;width:100%;max-width:794px;min-height:1050px;background:#fff;box-shadow:0 2px 12px rgba(15,23,42,.18);padding:56px 60px;box-sizing:border-box;}}
.gb-office-preview .gb-doc-page-number{{position:absolute;right:16px;bottom:12px;font:11px sans-serif;color:#94a3b8;letter-spacing:.06em;}}
.gb-office-preview .gb-doc-page-break{{display:none;}}
</style>
<div class="gb-preview-name">{name}</div>
{body}
</div>"#,
        name = crate::ui_fragments::html_escape(file_name),
        body = body
    ))
}

/// Inline images embedded in the package (docx `word/media/`, pptx
/// `ppt/media/`), appended as a bounded gallery: the converter does not
/// reproduce Office layout, but dropping the images entirely made image-heavy
/// documents preview as a blank page (#1714). EMF/WMF/TIFF are skipped (not
/// browser-renderable); each image is capped at 2 MiB, 12 per document.
pub(super) fn media_gallery(bytes: &[u8], prefix: &str) -> String {
    let Ok(mut archive) = zip::ZipArchive::new(Cursor::new(bytes)) else {
        return String::new();
    };
    let mut html = String::new();
    let mut shown = 0usize;
    for index in 0..archive.len() {
        if shown >= 12 {
            break;
        }
        let Ok(mut file) = archive.by_index(index) else {
            continue;
        };
        let name = match file.name() {
            Ok(value) => value.to_string(),
            Err(_) => continue,
        };
        if !name.starts_with(prefix) {
            continue;
        }
        let extension = name.rsplit('.').next().unwrap_or("").to_lowercase();
        let mime = match extension.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "bmp" => "image/bmp",
            "svg" => "image/svg+xml",
            _ => continue,
        };
        let mut data = Vec::new();
        if file.read_to_end(&mut data).is_err() || data.len() > 2 * 1024 * 1024 {
            continue;
        }
        let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
        html.push_str(&format!(
            r#"<figure><img src="data:{mime};base64,{encoded}" alt="embedded image" loading="lazy"/><figcaption>{}</figcaption></figure>"#,
            crate::ui_fragments::html_escape(&name)
        ));
        shown += 1;
    }
    if shown == 0 {
        String::new()
    } else {
        format!(r#"<div class="gb-office-media">{html}</div>"#)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_rejects_unknown_extensions() {
        assert!(preview_html("a.bin", "bin", b"x").is_err());
    }
}
