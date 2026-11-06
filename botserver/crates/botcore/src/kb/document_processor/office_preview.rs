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
//! Every string taken from the package is escaped with [`escape_html`] and no
//! attribute is copied, so sanitized output cannot carry scripts, frames or
//! external fetches. XLSX support is self-contained (`xl/*` member parsing)
//! so the preview does not depend on the opt-in `kb-extraction` feature.

use base64::Engine as _;
use std::io::{Cursor, Read};
use zip::ZipArchive;

/// Escapes any string from a package into safe text.
fn escape_html(value: &str) -> String {
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

fn document_shell(title: &str, body: &str) -> String {
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

/// Inline images embedded in the package (docx `word/media/`, pptx
/// `ppt/media/`). The preview cannot reproduce Office layout, so images are
/// appended as a bounded gallery instead of being dropped (#1714).
/// EMF/WMF/TIFF are skipped: browsers cannot render them.
fn media_gallery(bytes: &[u8], prefix: &str) -> String {
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
            r#"<figure><img src="data:{mime};base64,{encoded}" alt="embedded image" loading="lazy"/></figure>"#
        ));
        shown += 1;
    }
    if shown == 0 {
        String::new()
    } else {
        format!(r#"<div class="gb-media">{html}</div>"#)
    }
}

fn word_html(file_name: &str, bytes: &[u8]) -> Result<String, String> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("Failed to open DOCX as ZIP: {error}"))?;
    let xml = zip_entry_text(&mut archive, "word/document.xml")?;
    let mut body = docx_flow_html(&xml);
    body.push_str(&media_gallery(bytes, "word/media/"));
    Ok(document_shell(file_name, &body))
}

/// Flow paragraphs and tables of the document body. Mirrors
/// [`pptx_slide_html`]: table cell text is collected separately so a table is
/// not flattened into loose `<section>`s, which lost the shape of tables
/// generated by chat (Drive previews already kept them, #1714 parity).
fn docx_flow_html(xml: &str) -> String {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut html = String::new();
    let mut in_table = 0usize;
    let mut in_row = false;
    let mut in_cell = false;
    let mut in_paragraph = false;
    let mut in_text = false;
    let mut paragraph = String::new();
    let mut cell = String::new();
    let mut row: Vec<String> = Vec::new();
    let mut table_row = 0usize;
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => match event.name().as_ref() {
                b"w:tbl" => {
                    if in_table == 0 {
                        html.push_str("<table>");
                    }
                    in_table += 1;
                    table_row = 0;
                }
                b"w:tr" if in_table > 0 => {
                    in_row = true;
                    row.clear();
                }
                b"w:tc" if in_table > 0 => {
                    in_cell = true;
                    cell.clear();
                }
                b"w:p" if in_table == 0 => {
                    in_paragraph = true;
                    paragraph.clear();
                }
                b"w:t" => in_text = true,
                _ => {}
            },
            Ok(Event::Text(text)) if in_text => {
                if let Ok(decoded) = text.unescape() {
                    if in_cell {
                        cell.push_str(&decoded);
                    } else if in_paragraph {
                        paragraph.push_str(&decoded);
                    }
                }
            }
            Ok(Event::End(event)) => match event.name().as_ref() {
                b"w:t" => in_text = false,
                b"w:tc" if in_cell => {
                    row.push(std::mem::take(&mut cell));
                    in_cell = false;
                }
                b"w:tr" if in_row => {
                    if in_table == 1 {
                        html.push_str("<tr>");
                        for value in &row {
                            let tag = if table_row == 0 { "th" } else { "td" };
                            html.push_str(&format!(
                                "<{tag}>{}</{tag}>",
                                escape_html(value.trim())
                            ));
                        }
                        html.push_str("</tr>");
                    }
                    table_row += 1;
                    row.clear();
                    in_row = false;
                }
                b"w:tbl" => {
                    in_table = in_table.saturating_sub(1);
                    if in_table == 0 {
                        html.push_str("</table>");
                    }
                }
                b"w:p" if in_paragraph => {
                    let text = paragraph.trim();
                    if !text.is_empty() {
                        html.push_str(&format!(
                            "<section>{}</section>",
                            escape_html(text)
                        ));
                    }
                    in_paragraph = false;
                }
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    html
}

fn slides_html(file_name: &str, bytes: &[u8]) -> Result<String, String> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("Failed to open PPTX as ZIP: {error}"))?;
    let mut slides: Vec<(u32, String)> = Vec::new();
    for index in 0..archive.len() {
        let name = archive
            .by_index(index)
            .map_err(|error| format!("Failed to read PPTX entry: {error}"))?
            .name()
            .map_err(|error| format!("Failed to read PPTX entry name: {error}"))?
            .to_string();
        let Some(rest) = name.strip_prefix("ppt/slides/slide") else {
            continue;
        };
        let Some(number) = rest
            .strip_suffix(".xml")
            .and_then(|value| value.parse::<u32>().ok())
        else {
            continue;
        };
        let mut xml = String::new();
        archive
            .by_index(index)
            .map_err(|error| format!("Failed to read PPTX slide: {error}"))?
            .read_to_string(&mut xml)
            .map_err(|error| format!("Failed to read PPTX slide XML: {error}"))?;
        slides.push((number, xml));
    }
    if slides.is_empty() {
        return Err("No slide text found in PPTX archive".to_string());
    }
    slides.sort_by_key(|(number, _)| *number);
    let mut body = String::new();
    for (number, xml) in slides {
        body.push_str(&format!("<section class=\"gb-slide\"><h3>Slide {number}</h3>"));
        body.push_str(&pptx_slide_html(&xml));
        let notes = pptx_slide_notes(&mut archive, number);
        if !notes.is_empty() {
            body.push_str("<div class=\"gb-slide-notes\"><h4>Speaker notes</h4>");
            for note in notes {
                body.push_str(&format!("<p>{}</p>", escape_html(&note)));
            }
            body.push_str("</div>");
        }
        body.push_str("</section>");
    }
    body.push_str(&media_gallery(bytes, "ppt/media/"));
    Ok(document_shell(file_name, &body))
}

/// Flow paragraphs and tables of one slide. Table cells are skipped by the
/// paragraph walker so their text is not duplicated above the table (#1714).
fn pptx_slide_html(xml: &str) -> String {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut html = String::new();
    let mut in_table = 0usize;
    let mut in_row = false;
    let mut in_cell = false;
    let mut in_paragraph = false;
    let mut in_text = false;
    let mut paragraph = String::new();
    let mut cell = String::new();
    let mut row: Vec<String> = Vec::new();
    let mut table_row = 0usize;
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => match event.name().as_ref() {
                b"a:tbl" => {
                    in_table += 1;
                    table_row = 0;
                    html.push_str("<table>");
                }
                b"a:tr" if in_table > 0 => {
                    in_row = true;
                    row.clear();
                }
                b"a:tc" if in_table > 0 => {
                    in_cell = true;
                    cell.clear();
                }
                b"a:p" if in_table == 0 => {
                    in_paragraph = true;
                    paragraph.clear();
                }
                b"a:t" => in_text = true,
                _ => {}
            },
            Ok(Event::Text(text)) if in_text => {
                if let Ok(decoded) = text.unescape() {
                    if in_cell {
                        cell.push_str(&decoded);
                    } else if in_paragraph {
                        paragraph.push_str(&decoded);
                    }
                }
            }
            Ok(Event::End(event)) => match event.name().as_ref() {
                b"a:t" => in_text = false,
                b"a:tc" if in_cell => {
                    row.push(std::mem::take(&mut cell));
                    in_cell = false;
                }
                b"a:tr" if in_row => {
                    html.push_str("<tr>");
                    for value in &row {
                        let tag = if table_row == 0 { "th" } else { "td" };
                        html.push_str(&format!("<{tag}>{}</{tag}>", escape_html(value.trim())));
                    }
                    html.push_str("</tr>");
                    table_row += 1;
                    row.clear();
                    in_row = false;
                }
                b"a:tbl" => {
                    in_table = in_table.saturating_sub(1);
                    html.push_str("</table>");
                }
                b"a:p" if in_paragraph => {
                    let text = paragraph.trim();
                    if !text.is_empty() {
                        html.push_str(&format!("<p>{}</p>", escape_html(text)));
                    }
                    in_paragraph = false;
                }
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    html
}

/// Speaker notes for a slide, resolved through its `.rels` part because the
/// notes part number is not guaranteed to match the slide number (#1714).
fn pptx_slide_notes(archive: &mut ZipArchive<Cursor<&[u8]>>, slide_number: u32) -> Vec<String> {
    let rels = format!("ppt/slides/_rels/slide{slide_number}.xml.rels");
    let Ok(rels_xml) = zip_entry_text(archive, &rels) else {
        return Vec::new();
    };
    let Some(target) = notes_target(&rels_xml) else {
        return Vec::new();
    };
    let path = rels_target_path("ppt/slides", &target);
    let Ok(xml) = zip_entry_text(archive, &path) else {
        return Vec::new();
    };
    notes_paragraphs(&xml)
}

fn notes_target(rels_xml: &str) -> Option<String> {
    let mut remaining = rels_xml;
    while let Some(position) = remaining.find("Target=\"") {
        let after = &remaining[position + 8..];
        let end = after.find('"')?;
        let value = &after[..end];
        if value.contains("notesSlide") && value.ends_with(".xml") {
            return Some(value.to_string());
        }
        remaining = &after[end..];
    }
    None
}

fn rels_target_path(base_dir: &str, target: &str) -> String {
    let mut parts: Vec<&str> = base_dir.split('/').filter(|part| !part.is_empty()).collect();
    for segment in target.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                parts.truncate(parts.len().saturating_sub(1));
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

fn notes_paragraphs(xml: &str) -> Vec<String> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut paragraphs = Vec::new();
    let mut current = String::new();
    let mut in_paragraph = false;
    let mut in_text = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => match event.name().as_ref() {
                b"a:p" => {
                    in_paragraph = true;
                    current.clear();
                }
                b"a:t" => in_text = true,
                _ => {}
            },
            Ok(Event::Text(text)) if in_text => {
                if let Ok(decoded) = text.unescape() {
                    current.push_str(&decoded);
                }
            }
            Ok(Event::End(event)) => match event.name().as_ref() {
                b"a:t" => in_text = false,
                b"a:p" => {
                    in_text = false;
                    if in_paragraph {
                        let text = std::mem::take(&mut current);
                        let trimmed = text.trim();
                        if !trimmed.is_empty()
                            && !trimmed.chars().all(|character| character.is_ascii_digit())
                        {
                            paragraphs.push(text);
                        }
                        in_paragraph = false;
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    paragraphs
}

struct SheetRow {
    cells: Vec<String>,
}

struct Sheet {
    name: String,
    rows: Vec<SheetRow>,
}

fn spreadsheet_html(file_name: &str, bytes: &[u8]) -> Result<String, String> {
    let sheets = read_spreadsheet(file_name, bytes)?;
    let body = sheets
        .iter()
        .enumerate()
        .map(|(index, sheet)| {
            let open = if index == 0 { " open" } else { "" };
            let table = sheet
                .rows
                .iter()
                .map(|row| {
                    let cells = row
                        .cells
                        .iter()
                        .map(|cell| format!("<td>{}</td>", escape_html(cell)))
                        .collect::<String>();
                    format!("<tr>{cells}</tr>")
                })
                .collect::<String>();
            format!(
                "<details{open}><summary>{} · {} row{}</summary><table>{}</table></details>",
                escape_html(&sheet.name),
                sheet.rows.len(),
                if sheet.rows.len() == 1 { "" } else { "s" },
                table
            )
        })
        .collect::<String>();
    Ok(document_shell(file_name, &body))
}

/// Minimal XLSX reader: workbook sheet names in workbook order and each
/// sheet's rows decoded from shared strings and literal values. Errors
/// surface as `Result` so the caller degrades to the old window preview
/// instead of guessing about a broken package.
fn read_spreadsheet(file_name: &str, bytes: &[u8]) -> Result<Vec<Sheet>, String> {
    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader)
        .map_err(|error| format!("Failed to open XLSX as ZIP: {error}"))?;

    let shared: Vec<String> = match zip_entry_text(&mut archive, "xl/sharedStrings.xml") {
        Ok(xml) => collect_tag_values(&xml, "<t"),
        Err(_) => Vec::new(),
    };

    let workbook = zip_entry_text(&mut archive, "xl/workbook.xml")
        .map_err(|error| format!("XLSX has no workbook.xml: {error}"))?;
    let mut sheets = Vec::new();
    let mut remaining = workbook.as_str();
    while let Some(open) = remaining.find("<sheet ") {
        let entry_end = match remaining[open..].find("/>") {
            Some(end) => open + end,
            None => break,
        };
        let entry = &remaining[open..entry_end];
        let name = match entry.find("name=\"") {
            Some(start) => {
                let after = &entry[start + 6..];
                match after.find('"') {
                    Some(end) => after[..end].to_string(),
                    None => format!("Sheet {}", sheets.len() + 1),
                }
            }
            None => format!("Sheet {}", sheets.len() + 1),
        };
        // The part requires the alphabetical member mapping the agent's own
        // writer (`umya-spreadsheet`) and every common producer use:
        // `xl/worksheets/sheet1.xml`, `sheet2.xml`, … in workbook order.
        let sheet_number = sheets.len() + 1;
        let part = format!("xl/worksheets/sheet{sheet_number}.xml");
        let rows = match zip_entry_text(&mut archive, &part) {
            Ok(xml) => decode_sheet_rows(&xml, &shared),
            Err(_) => Vec::new(),
        };
        sheets.push(Sheet { name, rows });
        remaining = &remaining[entry_end..];
    }
    if sheets.is_empty() {
        return Err(format!("No sheets found in {file_name}"));
    }
    Ok(sheets)
}

fn zip_entry_text(
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

/// Collects the text of every `<t…>…</t>` value in an XML document — the
/// shape `xl/sharedStrings.xml` uses.
fn collect_tag_values(xml: &str, tag_start: &str) -> Vec<String> {
    let closing = format!("</{}", &tag_start[1..]);
    let mut values = Vec::new();
    let mut remaining = xml;
    loop {
        let open = match remaining.find(tag_start) {
            Some(open) => open,
            None => break,
        };
        let after_open = open + tag_start.len();
        let inner_start = match remaining[after_open..].find('>') {
            Some(end) => after_open + end + 1,
            None => break,
        };
        let rest = &remaining[inner_start..];
        let end = match rest.find(closing.as_str()) {
            Some(end) => end,
            None => break,
        };
        values.push(rest[..end].to_string());
        remaining = &rest[end + closing.len()..];
    }
    values.retain(|value| !value.is_empty());
    values
}

fn decode_sheet_rows(xml: &str, shared: &[String]) -> Vec<SheetRow> {
    let mut rows = Vec::new();
    let mut remaining = xml;
    loop {
        let row_open = match remaining.find("<row ") {
            Some(open) => open,
            None => break,
        };
        let row_end = match remaining[row_open..].find("</row>") {
            Some(end) => row_open + end + "</row>".len(),
            None => break,
        };
        let row_xml = &remaining[row_open..row_end];
        let mut cells = Vec::new();
        let mut cell_remaining = row_xml;
        loop {
            let cell_open = match cell_remaining.find("<c ") {
                Some(open) => open,
                None => break,
            };
            let cell_end = match cell_remaining[cell_open..].find("</c>") {
                Some(end) => cell_open + end + "</c>".len(),
                None => break,
            };
            let cell_xml = &cell_remaining[cell_open..cell_end];
            let value = inner_value(cell_xml);
            // `t="s"` marks a shared-string index; everything else is stored
            // inline (number, boolean or inline string). Numbers keep their
            // textual form — no localization surprises.
            let text = if cell_xml.contains("t=\"s\"") {
                value
                    .trim()
                    .parse::<usize>()
                    .ok()
                    .and_then(|index| shared.get(index))
                    .cloned()
                    .unwrap_or_default()
            } else {
                value
            };
            cells.push(text);
            cell_remaining = &cell_remaining[cell_end..];
        }
        rows.push(SheetRow { cells });
        remaining = &remaining[row_end..];
    }
    rows
}

/// The `<v>…</v>` (or inline-string `<t>…</t>`) payload of one cell.
fn inner_value(cell_xml: &str) -> String {
    for (tag, closing) in [("<v>", "</v>"), ("<t>", "</t>")] {
        if let Some(start) = cell_xml.find(tag) {
            let after = start + tag.len();
            if let Some(end) = cell_xml[after..].find(closing) {
                return cell_xml[after..after + end].to_string();
            }
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn shared_string_index_resolves_to_text() {
        let shared = vec!["Revenue".to_string(), "Cost".to_string()];
        let rows = decode_sheet_rows(
            "<row r=\"1\"><c r=\"A1\" t=\"s\"><v>0</v></c><c r=\"B1\"><v>42</v></c></row>",
            &shared,
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cells[0], "Revenue");
        assert_eq!(rows[0].cells[1], "42");
    }

    #[test]
    fn docx_renders_tables_instead_of_loose_paragraphs() {
        let xml = r#"<w:document xmlns:w="w"><w:body><w:p><w:r><w:t>Quarterly Report</w:t></w:r></w:p><w:tbl><w:tr><w:tc><w:p><w:r><w:t>Region</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Revenue</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>LATAM</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>120000</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>"#;
        let html = docx_flow_html(xml);
        assert!(html.contains("<section>Quarterly Report</section>"), "{html}");
        assert!(
            html.contains(
                "<table><tr><th>Region</th><th>Revenue</th></tr><tr><td>LATAM</td><td>120000</td></tr></table>"
            ),
            "{html}"
        );
        assert!(
            !html.contains("<section>Region</section>"),
            "table cells leaked as sections: {html}"
        );
    }

    #[test]
    fn pptx_slide_renders_tables_without_duplicating_cells() {
        let xml = r#"<p:sld xmlns:a="a"><a:p><a:t>Title</a:t></a:p><a:tbl><a:tr><a:tc><a:p><a:t>A</a:t></a:p></a:tc><a:tc><a:p><a:t>B</a:t></a:p></a:tc></a:tr></a:tbl></p:sld>"#;
        let html = pptx_slide_html(xml);
        assert!(html.contains("<p>Title</p>"), "{html}");
        assert!(html.contains("<table><tr><th>A</th><th>B</th></tr></table>"), "{html}");
        assert_eq!(html.matches("Title").count(), 1);
    }

    #[test]
    fn pptx_notes_relationship_resolves_and_skips_slide_number() {
        let rels = r#"<Relationships><Relationship Id="rId2" Target="../notesSlides/notesSlide4.xml"/></Relationships>"#;
        let target = notes_target(rels).expect("notes relationship");
        assert_eq!(
            rels_target_path("ppt/slides", &target),
            "ppt/notesSlides/notesSlide4.xml"
        );
        let notes = r#"<p:notes xmlns:a="a"><a:p><a:t>Explain the table</a:t></a:p><a:p><a:t>4</a:t></a:p></p:notes>"#;
        assert_eq!(notes_paragraphs(notes), vec!["Explain the table"]);
    }
}
