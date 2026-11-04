//! Office preview rendering (#1672 follow-up).
//!
//! Web parity target: ChatGPT/Claude render docx/xlsx/pptx INLINE in the
//! conversation instead of handing the file to an external viewer. This
//! module converts an OOXML package to a fully sanitized, script-free HTML
//! document:
//!
//! - xlsx: one `<details>` section per sheet, tables built from cell values.
//! - docx: paragraphs as `<section>`s.
//! - pptx: one `<section>` per slide.
//!
//! Every string taken from the package is escaped with [`escape_html`] and no
//! attribute is copied, so sanitized output cannot carry scripts, frames or
//! external fetches. XLSX support is self-contained (`xl/*` member parsing)
//! so the preview does not depend on the opt-in `kb-extraction` feature.

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
section{{font-size:13px;color:#1e293b;margin:0 0 6px}}</style>\
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

fn paragraphs_html(text: &str) -> String {
    text.lines()
        .filter_map(|line| {
            let paragraph = escape_html(line);
            if paragraph.trim().is_empty() {
                None
            } else {
                Some(format!("<section>{paragraph}</section>"))
            }
        })
        .collect::<String>()
}

fn word_html(file_name: &str, bytes: &[u8]) -> Result<String, String> {
    let text =
        super::ooxml_extract::extract_docx_text_from_zip(bytes)?;
    Ok(document_shell(file_name, &paragraphs_html(&text)))
}

fn slides_html(file_name: &str, bytes: &[u8]) -> Result<String, String> {
    let text =
        super::ooxml_extract::extract_pptx_text_from_zip(bytes)?;
    Ok(document_shell(file_name, &paragraphs_html(&text)))
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
}
