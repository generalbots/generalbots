//! ODT (OpenDocument Text) import/export.
//!
//! ODT is a ZIP archive whose text lives in `content.xml`. These helpers
//! read and write that archive directly using the `zip` crate — no OOXML
//! SDK and no external document library are required.

use std::io::{Cursor, Read, Write};

use crate::storage_core::{parse_html_to_paragraphs, ParagraphData};

const ODT_MIMETYPE: &str = "application/vnd.oasis.opendocument.text";

const ODT_CONTENT_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content
 xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
 xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
 xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
 xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"
 office:version="1.2">
<office:automatic-styles>
<style:style style:name="Bold" style:family="text"><style:text-properties fo:font-weight="bold"/></style:style>
<style:style style:name="Italic" style:family="text"><style:text-properties fo:font-style="italic"/></style:style>
<style:style style:name="Underline" style:family="text"><style:text-properties style:text-underline-style="solid" style:text-underline-width="auto" style:text-underline-color="font-color"/></style:style>
<style:style style:name="Code" style:family="text"><style:text-properties fo:font-family="Courier New"/></style:style>
</office:automatic-styles>
<office:body>
<office:text>
"#;

const ODT_CONTENT_FOOTER: &str = "</office:text>\n</office:body>\n</office:document-content>";

const ODT_STYLES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-styles
 xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
 xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
 xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
 xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"
 office:version="1.2">
<office:styles>
<style:style style:name="Heading 1" style:family="paragraph" style:parent-style-name="Standard">
<style:text-properties fo:font-size="24pt" fo:font-weight="bold"/>
</style:style>
<style:style style:name="Heading 2" style:family="paragraph" style:parent-style-name="Standard">
<style:text-properties fo:font-size="18pt" fo:font-weight="bold"/>
</style:style>
<style:style style:name="Heading 3" style:family="paragraph" style:parent-style-name="Standard">
<style:text-properties fo:font-size="14pt" fo:font-weight="bold"/>
</style:style>
</office:styles>
</office:document-styles>
"#;

const ODT_MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0" manifest:version="1.2">
<manifest:file-entry manifest:full-path="/" manifest:media-type="application/vnd.oasis.opendocument.text"/>
<manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/>
<manifest:file-entry manifest:full-path="styles.xml" manifest:media-type="text/xml"/>
<manifest:file-entry manifest:full-path="meta.xml" manifest:media-type="text/xml"/>
</manifest:manifest>
"#;

pub fn odt_zip_to_html(bytes: &[u8]) -> Result<String, String> {
    let reader = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| format!("Failed to open ODT: {e}"))?;

    let mut xml = String::new();
    {
        let mut file = archive
            .by_name("content.xml")
            .map_err(|e| format!("content.xml not found: {e}"))?;
        file.read_to_string(&mut xml)
            .map_err(|e| format!("Failed to read content.xml: {e}"))?;
    }

    Ok(odt_content_to_html_clean(&xml))
}

pub fn html_to_odt_zip(title: &str, html: &str) -> Result<Vec<u8>, String> {
    let content = html_to_odt_content_clean(html);
    let meta = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-meta xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
 xmlns:dc="http://purl.org/dc/elements/1.1/"
 xmlns:meta="urn:oasis:names:tc:opendocument:xmlns:meta:1.0"
 office:version="1.2">
<office:meta><dc:title>{}</dc:title><meta:generator>GeneralBots Docs</meta:generator></office:meta>
</office:document-meta>
"#,
        xml_escape(title)
    );

    let mut buf = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buf);
        let stored =
            zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        let deflated =
            zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // The mimetype entry must be first and stored (uncompressed).
        zip.start_file("mimetype", stored)
            .map_err(|e| e.to_string())?;
        zip.write_all(ODT_MIMETYPE.as_bytes())
            .map_err(|e| e.to_string())?;

        zip.start_file("content.xml", deflated)
            .map_err(|e| e.to_string())?;
        zip.write_all(content.as_bytes())
            .map_err(|e| e.to_string())?;

        zip.start_file("styles.xml", deflated)
            .map_err(|e| e.to_string())?;
        zip.write_all(ODT_STYLES.as_bytes())
            .map_err(|e| e.to_string())?;

        zip.start_file("meta.xml", deflated)
            .map_err(|e| e.to_string())?;
        zip.write_all(meta.as_bytes()).map_err(|e| e.to_string())?;

        zip.start_file("META-INF/manifest.xml", deflated)
            .map_err(|e| e.to_string())?;
        zip.write_all(ODT_MANIFEST.as_bytes())
            .map_err(|e| e.to_string())?;

        zip.finish().map_err(|e| e.to_string())?;
    }

    Ok(buf.into_inner())
}

fn html_to_odt_content_clean(html: &str) -> String {
    let mut out = String::from(ODT_CONTENT_HEADER);
    for p in parse_html_to_paragraphs(html) {
        out.push_str(&paragraph_to_odt(&p));
    }
    out.push_str(ODT_CONTENT_FOOTER);
    out
}

fn paragraph_to_odt(p: &ParagraphData) -> String {
    let text = p.text.trim();
    let inline = odt_inline(text, p.bold, p.italic, p.underline, p.style == "code");

    match heading_level(&p.style) {
        Some(level) => format!("<text:h text:outline-level=\"{level}\">{inline}</text:h>\n"),
        None if p.style == "li" => {
            format!("<text:p><text:span text:style-name=\"Bold\">•</text:span> {inline}</text:p>\n")
        }
        None if p.style == "blockquote" => {
            format!("<text:p text:style-name=\"Quotations\">{inline}</text:p>\n")
        }
        None => format!("<text:p>{inline}</text:p>\n"),
    }
}

fn odt_inline(text: &str, bold: bool, italic: bool, underline: bool, code: bool) -> String {
    let mut s = xml_escape(text).replace('\n', "<text:line-break/>");

    if code {
        s = format!("<text:span text:style-name=\"Code\">{s}</text:span>");
    }
    if underline {
        s = format!("<text:span text:style-name=\"Underline\">{s}</text:span>");
    }
    if italic {
        s = format!("<text:span text:style-name=\"Italic\">{s}</text:span>");
    }
    if bold {
        s = format!("<text:span text:style-name=\"Bold\">{s}</text:span>");
    }

    s
}

fn heading_level(style: &str) -> Option<u8> {
    let level = style.strip_prefix('h')?.parse::<u8>().ok()?;
    (1..=6).contains(&level).then_some(level)
}

fn odt_content_to_html_clean(xml: &str) -> String {
    let mut out = String::new();
    let mut pos = 0usize;

    while let Some((tag, start)) = next_odt_block(xml, pos) {
        let close_tag = format!("</{tag}>");
        let Some(close_rel) = xml[start..].find(&close_tag) else {
            break;
        };
        let close_abs = start + close_rel;
        let open_end = xml[start..]
            .find('>')
            .map(|r| start + r + 1)
            .unwrap_or(start);

        let content = &xml[open_end..close_abs];
        let inner = inline_odt_to_html(content);

        if tag.starts_with("text:h") {
            let level = extract_outline_level(&xml[start..open_end]).unwrap_or(1);
            out.push_str(&format!("<h{level}>{inner}</h{level}>"));
        } else {
            out.push_str(&format!("<p>{inner}</p>"));
        }

        pos = close_abs + close_tag.len();
    }

    out
}

fn next_odt_block(xml: &str, from: usize) -> Option<(&'static str, usize)> {
    let h = xml[from..].find("<text:h").map(|r| from + r);
    let p = xml[from..].find("<text:p").map(|r| from + r);

    match (h, p) {
        (Some(hh), Some(pp)) if hh <= pp => Some(("text:h", hh)),
        (Some(hh), _) => Some(("text:h", hh)),
        (None, Some(pp)) => Some(("text:p", pp)),
        (None, None) => None,
    }
}

fn inline_odt_to_html(content: &str) -> String {
    let mut out = String::new();
    let mut stack: Vec<&str> = Vec::new();
    let mut i = 0usize;
    let b = content.as_bytes();

    while i < b.len() {
        if b[i] == b'<' {
            let Some(end_rel) = content[i..].find('>') else {
                break;
            };
            let end = i + end_rel + 1;
            let raw = &content[i..end];

            if raw.starts_with("</text:span") {
                if let Some(close) = stack.pop() {
                    out.push_str(close);
                }
            } else if raw.starts_with("<text:span") {
                if let Some((open, close)) = span_style_to_html(&extract_style_name(raw)) {
                    out.push_str(open);
                    stack.push(close);
                }
            } else if raw.starts_with("<text:line-break") {
                out.push_str("<br>");
            } else if raw.starts_with("<text:tab") {
                out.push_str("&nbsp;&nbsp;&nbsp;&nbsp;");
            }
            // All other inline tags are ignored; their text content remains.

            i = end;
        } else {
            let ch = content[i..].chars().next().unwrap_or(' ');
            out.push(ch);
            i += ch.len_utf8();
        }
    }

    while let Some(close) = stack.pop() {
        out.push_str(close);
    }

    xml_unescape(&out)
}

fn span_style_to_html(style: &str) -> Option<(&'static str, &'static str)> {
    match style {
        "Bold" => Some(("<strong>", "</strong>")),
        "Italic" => Some(("<em>", "</em>")),
        "Underline" => Some(("<u>", "</u>")),
        "Code" => Some(("<code>", "</code>")),
        _ => None,
    }
}

fn extract_style_name(tag: &str) -> String {
    if let Some(pos) = tag.find("style-name=\"") {
        let rest = &tag[pos + 12..];
        if let Some(end) = rest.find('"') {
            return rest[..end].to_string();
        }
    }
    String::new()
}

fn extract_outline_level(tag: &str) -> Option<u8> {
    let pos = tag.find("text:outline-level=\"")?;
    let rest = &tag[pos + 20..];
    let end = rest.find('"')?;
    rest[..end].parse::<u8>().ok()
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn xml_unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}
