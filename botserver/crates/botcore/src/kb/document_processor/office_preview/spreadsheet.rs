//! XLSX → sanitized HTML: one `<details>` section per worksheet. The reader
//! maps workbook relationships, decodes shared/inline strings, booleans and
//! literals, and honours cell references so sparse rows keep their columns.

use super::common::{attribute, document_shell, escape_html, unescape_xml, zip_entry_text};
use std::io::Cursor;
use zip::ZipArchive;

struct SheetRow {
    cells: Vec<String>,
}

struct Sheet {
    name: String,
    rows: Vec<SheetRow>,
}

pub(super) fn spreadsheet_html(file_name: &str, bytes: &[u8]) -> Result<String, String> {
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

/// Minimal XLSX reader: workbook sheet names mapped through the workbook
/// relationships and each sheet's rows decoded from shared strings, inline
/// strings, booleans and literal values. Cell references (`r="B2"`) are
/// honoured so sparse rows keep their columns aligned.
fn read_spreadsheet(file_name: &str, bytes: &[u8]) -> Result<Vec<Sheet>, String> {
    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader)
        .map_err(|error| format!("Failed to open XLSX as ZIP: {error}"))?;

    let shared: Vec<String> = match zip_entry_text(&mut archive, "xl/sharedStrings.xml") {
        Ok(xml) => collect_shared_strings(&xml),
        Err(_) => Vec::new(),
    };

    let targets = workbook_targets(&mut archive);
    let workbook = zip_entry_text(&mut archive, "xl/workbook.xml")
        .map_err(|error| format!("XLSX has no workbook.xml: {error}"))?;
    let mut sheets = Vec::new();
    let mut remaining = workbook.as_str();
    while let Some(open) = remaining.find("<sheet ") {
        let entry_end = match remaining[open..].find('>') {
            Some(end) => open + end,
            None => break,
        };
        let entry = &remaining[open..entry_end];
        let name = attribute(entry, "name")
            .map(str::to_string)
            .unwrap_or_else(|| format!("Sheet {}", sheets.len() + 1));
        let part = attribute(entry, "r:id")
            .and_then(|id| targets.get(id))
            .map(|target| {
                let cleaned = target.trim_start_matches('/');
                if cleaned.starts_with("xl/") {
                    cleaned.to_string()
                } else {
                    format!("xl/{cleaned}")
                }
            })
            .unwrap_or_else(|| format!("xl/worksheets/sheet{}.xml", sheets.len() + 1));
        let rows = match zip_entry_text(&mut archive, &part) {
            Ok(xml) => decode_sheet_rows(&xml, &shared),
            Err(_) => Vec::new(),
        };
        sheets.push(Sheet { name, rows });
        remaining = &remaining[entry_end + 1..];
    }
    if sheets.is_empty() {
        return Err(format!("No sheets found in {file_name}"));
    }
    Ok(sheets)
}

/// `r:id` → worksheet part from `xl/_rels/workbook.xml.rels`.
fn workbook_targets(archive: &mut ZipArchive<Cursor<&[u8]>>) -> std::collections::HashMap<String, String> {
    let mut targets = std::collections::HashMap::new();
    let Ok(rels) = zip_entry_text(archive, "xl/_rels/workbook.xml.rels") else {
        return targets;
    };
    let mut remaining = rels.as_str();
    while let Some(open) = remaining.find("<Relationship") {
        let Some(end) = remaining[open..].find('>') else { break };
        let entry = &remaining[open..open + end];
        if let (Some(id), Some(target)) = (attribute(entry, "Id"), attribute(entry, "Target")) {
            targets.insert(id.to_string(), target.to_string());
        }
        remaining = &remaining[open + end + 1..];
    }
    targets
}

/// One shared string per `<si>` (rich text runs are concatenated), preserving
/// empty entries so later indices stay aligned.
fn collect_shared_strings(xml: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut remaining = xml;
    while let Some(open) = remaining.find("<si") {
        let after = &remaining[open + 3..];
        let next = after.chars().next();
        if !matches!(next, Some('>') | Some('/')) && !next.is_some_and(char::is_whitespace) {
            remaining = after;
            continue;
        }
        let Some(gt) = after.find('>') else { break };
        if after[..gt].trim_end().ends_with('/') {
            values.push(String::new());
            remaining = &after[gt + 1..];
            continue;
        }
        let Some(close) = after.find("</si>") else { break };
        values.push(collect_text_runs(&after[gt + 1..close]));
        remaining = &after[close + 5..];
    }
    values
}

fn collect_text_runs(xml: &str) -> String {
    let mut output = String::new();
    let mut remaining = xml;
    while let Some(open) = remaining.find("<t") {
        let Some(gt) = remaining[open..].find('>') else { break };
        let text_start = open + gt + 1;
        let Some(end) = remaining[text_start..].find("</t>") else { break };
        output.push_str(&unescape_xml(&remaining[text_start..text_start + end]));
        remaining = &remaining[text_start + end + 4..];
    }
    output
}

fn column_index(reference: &str) -> Option<usize> {
    let mut index = 0usize;
    let mut any = false;
    for character in reference.chars().take_while(|value| value.is_ascii_alphabetic()) {
        any = true;
        index = index * 26 + (character.to_ascii_uppercase() as usize - 'A' as usize + 1);
    }
    if any { Some(index - 1) } else { None }
}

fn decode_sheet_rows(xml: &str, shared: &[String]) -> Vec<SheetRow> {
    let mut rows = Vec::new();
    let mut remaining = xml;
    while let Some(row_open) = remaining.find("<row ") {
        let after = &remaining[row_open + 4..];
        let Some(gt) = after.find('>') else { break };
        let self_closing = after[..gt].trim_end().ends_with('/');
        let (row_xml, next_row) = if self_closing {
            ("", &after[gt + 1..])
        } else {
            let Some(close) = after.find("</row>") else { break };
            (&after[gt + 1..close], &after[close + 6..])
        };
        let mut cells: Vec<String> = Vec::new();
        let mut cell_remaining = row_xml;
        while let Some(cell_open) = cell_remaining.find("<c") {
            let Some(cell_gt) = cell_remaining[cell_open..].find('>') else { break };
            let tag = &cell_remaining[cell_open + 2..cell_open + cell_gt];
            let cell_self_closing = tag.trim_end().ends_with('/');
            let content_start = cell_open + cell_gt + 1;
            let (cell_xml, next_cell) = if cell_self_closing {
                ("", &cell_remaining[content_start..])
            } else {
                let Some(end) = cell_remaining[content_start..].find("</c>") else { break };
                (
                    &cell_remaining[content_start..content_start + end],
                    &cell_remaining[content_start + end + 4..],
                )
            };
            let column = attribute(tag, "r")
                .and_then(column_index)
                .unwrap_or(cells.len());
            let kind = attribute(tag, "t").unwrap_or("");
            let raw = inner_value(cell_xml);
            let text = match kind {
                "s" => raw
                    .trim()
                    .parse::<usize>()
                    .ok()
                    .and_then(|index| shared.get(index))
                    .cloned()
                    .unwrap_or_default(),
                "b" => match raw.trim() {
                    "1" => "TRUE".to_string(),
                    "0" => "FALSE".to_string(),
                    other => other.to_string(),
                },
                _ => unescape_xml(&raw),
            };
            if column >= cells.len() {
                cells.resize(column + 1, String::new());
            }
            cells[column] = text;
            cell_remaining = next_cell;
        }
        rows.push(SheetRow { cells });
        remaining = next_row;
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
    use super::decode_sheet_rows;

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
