//! Helpers shared by the three Office readers: escaping, the document shell,
//! the embedded-image gallery and ZIP/XML primitives.

use base64::Engine as _;
use std::collections::HashSet;
use std::io::{Cursor, Read};
use zip::ZipArchive;

/// Largest image embedded inline; bigger payloads would make the preview
/// fragment several megabytes and the editor janky.
const MAX_INLINE_IMAGE_BYTES: usize = 2 * 1024 * 1024;

/// Escapes any string from a package into safe text.
pub(super) fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            other => escaped.push(other),
        }
    }
    escaped
}

pub(super) fn document_shell(title: &str, body: &str) -> String {
    format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>{title}</title>\
<style>body{{font-family:system-ui,sans-serif;margin:0;padding:12px;background:#fff}}\
table{{border-collapse:collapse;margin:6px 0}}\
td,th{{border:1px solid #cbd5e1;padding:3px 8px;font-size:12px;color:#0f172a}}\
th{{background:#e2e8f0;text-align:left}}\
h3{{font-size:13px;margin:12px 0 4px;color:#334155}}\
details{{margin:0 0 10px}}summary{{font-weight:600;font-size:13px;color:#1e293b}}\
section{{font-size:13px;color:#1e293b;margin:0 0 6px}}\
.gb-media{{display:flex;flex-wrap:wrap;gap:8px;margin-top:10px}}\
.gb-media figure{{margin:0}}\
.gb-media img{{max-width:280px;max-height:200px;border:1px solid #cbd5e1;border-radius:6px}}\
figure.gb-doc-image{{margin:10px 0}}\
figure.gb-doc-image img{{display:block;max-width:100%;height:auto;border-radius:6px}}\
hr.gb-page-break{{border:0;border-top:2px dashed #94a3b8;margin:20px 0 18px;position:relative}}\
section.gb-slide{{border:1px solid #cbd5e1;border-radius:6px;padding:10px 12px;margin:8px 0}}\
section.gb-slide h3{{margin:0 0 6px;text-transform:uppercase;letter-spacing:.06em}}\
section.gb-slide p{{margin:3px 0}}\
.gb-slide-notes{{margin-top:8px;padding-top:6px;border-top:1px dashed #cbd5e1}}\
.gb-slide-notes h4{{margin:0 0 3px;font-size:11px;text-transform:uppercase;letter-spacing:.06em;color:#64748b}}\
.gb-slide-notes p{{font-size:12px;color:#475569;font-style:italic}}</style>\
</head><body>{body}</body></html>",
        title = escape_html(title),
        body = body
    )
}

/// MIME type browsers can render for an embedded image name, if any.
pub(super) fn image_mime(name: &str) -> Option<&'static str> {
    let extension = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match extension.as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "bmp" => Some("image/bmp"),
        "svg" => Some("image/svg+xml"),
        _ => None,
    }
}

/// Base64 data URI for one already-read image entry, or `None` when the
/// browser cannot render it or it exceeds the inline budget.
pub(super) fn image_data_uri(name: &str, data: &[u8]) -> Option<String> {
    if data.is_empty() || data.len() > MAX_INLINE_IMAGE_BYTES {
        return None;
    }
    let mime = image_mime(name)?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(data);
    Some(format!("data:{mime};base64,{encoded}"))
}

/// Inline images embedded in the package (docx `word/media/`, pptx
/// `ppt/media/`). The preview cannot reproduce Office layout, so images are
/// appended as a bounded gallery instead of being dropped (#1714).
/// EMF/WMF/TIFF are skipped: browsers cannot render them.
pub(super) fn media_gallery(bytes: &[u8], prefix: &str) -> String {
    media_gallery_filtered(bytes, prefix, &HashSet::new())
}

/// Gallery of the package images that are NOT referenced by the document
/// flow. A DOCX drawing is rendered where it appears (#2184); anything left
/// over (decorations, unreferenced parts) still shows at the end.
pub(super) fn media_gallery_filtered(bytes: &[u8], prefix: &str, skip: &HashSet<String>) -> String {
    let Ok(mut archive) = ZipArchive::new(Cursor::new(bytes)) else {
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
        if !name.starts_with(prefix) || skip.contains(&name) {
            continue;
        }
        let mut data = Vec::new();
        if file.read_to_end(&mut data).is_err() {
            continue;
        }
        let Some(uri) = image_data_uri(&name, &data) else {
            continue;
        };
        html.push_str(&format!(
            r#"<figure><img src="{uri}" alt="embedded image" loading="lazy"/></figure>"#
        ));
        shown += 1;
    }
    if shown == 0 {
        String::new()
    } else {
        format!(r#"<div class="gb-media">{html}</div>"#)
    }
}

pub(super) fn zip_entry_text(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    name: &str,
) -> Result<String, String> {
    let mut entry = archive
        .by_name(name)
        .map_err(|error| format!("{name}: {error}"))?;
    let mut content = String::new();
    entry
        .read_to_string(&mut content)
        .map_err(|error| format!("Failed to read {name}: {error}"))?;
    Ok(content)
}

/// Reads an XML attribute value; `name` is matched with a leading space so
/// `id` cannot match `r:id`.
pub(super) fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!(" {name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

pub(super) fn unescape_xml(value: &str) -> String {
    value
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}
