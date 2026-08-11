//! Clipboard HTML table parsing for paste (#787).
//!
//! Excel and Google Sheets put an HTML table flavour on the clipboard. This
//! module parses `<table><tr><td>/<th>` structure into a value grid, keeps
//! inline styles for the `formats` Paste Special mode, and honours colspan /
//! rowspan spanning.

use std::collections::HashMap;

use crate::types::CellStyle;

/// A single parsed clipboard cell (HTML table flavor).
#[derive(Clone)]
pub struct HtmlCell {
    pub value: String,
    pub style: Option<String>,
    pub colspan: usize,
    pub rowspan: usize,
}

/// Parses an HTML table fragment from the clipboard (Excel/Sheets flavor) into
/// a value grid. Handles `<table><tr><td>` structure, inline styles for the
/// `formats` Paste Special mode, and cells spanning via colspan/rowspan.
pub fn parse_html_table(html: &str) -> Vec<Vec<HtmlCell>> {
    let mut grid: Vec<Vec<HtmlCell>> = Vec::new();
    let mut pos = 0usize;
    let lower = html.to_ascii_lowercase();

    // `occupied` tracks columns still claimed by a rowspan from a previous row.
    let mut occupied: Vec<(usize, usize)> = Vec::new();

    while let Some(tr_start) = find_tag(&lower, "tr", pos) {
        let row_open_end = find_tag_close(&lower, tr_start);
        let row_close = find_tag(&lower, "/tr", row_open_end)
            .or_else(|| find_tag(&lower, "/table", row_open_end))
            .unwrap_or(lower.len());
        if row_close <= tr_start {
            break;
        }
        let mut out_row: Vec<HtmlCell> = Vec::new();
        let mut col = 0usize;
        let mut cell_pos = row_open_end;

        while cell_pos < row_close {
            let next_td = find_tag(&lower, "td", cell_pos);
            let next_th = find_tag(&lower, "th", cell_pos);
            let cell_start = match (next_td, next_th) {
                (Some(td), Some(th)) => td.min(th),
                (Some(td), None) => td,
                (None, Some(th)) => th,
                (None, None) => break,
            };
            if cell_start >= row_close {
                break;
            }
            let cell_open_end = find_tag_close(&lower, cell_start);
            if cell_open_end <= cell_start {
                break;
            }
            let cell_close = [
                find_tag(&lower, "/td", cell_open_end),
                find_tag(&lower, "/th", cell_open_end),
            ]
            .into_iter()
            .flatten()
            .filter(|&c| c < row_close)
            .min()
            .unwrap_or(row_close);
            let content = extract_text(html, cell_open_end, cell_close);
            let attrs = extract_tag_attrs(&lower, cell_start);
            let cell = HtmlCell {
                value: content,
                style: attrs.get("style").cloned(),
                colspan: attrs.get("colspan").and_then(|v| v.parse().ok()).unwrap_or(1),
                rowspan: attrs.get("rowspan").and_then(|v| v.parse().ok()).unwrap_or(1),
            };

            while occupied.iter().any(|(c, _)| *c == col) {
                col += 1;
            }
            while out_row.len() < col + cell.colspan {
                out_row.push(HtmlCell {
                    value: String::new(),
                    style: None,
                    colspan: 1,
                    rowspan: 1,
                });
            }
            if out_row.get(col).is_some() {
                out_row[col] = cell.clone();
            }
            if cell.rowspan > 1 {
                for c in col..col + cell.colspan {
                    occupied.push((c, cell.rowspan));
                }
            }
            col += cell.colspan;
            cell_pos = cell_open_end;
        }

        // Account for rowspans continuing into this row.
        for (_, span) in occupied.iter_mut() {
            *span = span.saturating_sub(1);
        }
        occupied.retain(|(_, span)| *span > 0);

        if !out_row.is_empty() {
            grid.push(out_row);
        }
        pos = find_tag_close(&lower, row_close);
    }

    grid
}

fn find_tag(haystack: &str, tag: &str, from: usize) -> Option<usize> {
    let needle = format!("<{tag}");
    let mut idx = from;
    while idx < haystack.len() {
        let rel = haystack[idx..].find(&needle)?;
        let abs = idx + rel;
        let boundary_ok = haystack[abs + needle.len()..]
            .chars()
            .next()
            .map(|c| matches!(c, '>' | ' ' | '/' | '\n' | '\t' | '\r'))
            .unwrap_or(true);
        if boundary_ok {
            return Some(abs);
        }
        idx = abs + needle.len();
    }
    None
}

fn find_tag_close(haystack: &str, tag_start: usize) -> usize {
    let rel = haystack[tag_start..].find('>').unwrap_or(0);
    tag_start + rel + 1
}

fn extract_tag_attrs(lower: &str, tag_start: usize) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let rest = &lower[tag_start..];
    let end = rest.find('>').unwrap_or(rest.len());
    let tag = &rest[..end];
    let mut iter = tag.split_whitespace().skip(1);
    for part in iter.by_ref() {
        if let Some(eq) = part.find('=') {
            let key = &part[..eq];
            let value = part[eq + 1..]
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            out.insert(key.to_string(), value);
        }
    }
    out
}

fn extract_text(html: &str, start: usize, end: usize) -> String {
    let slice = &html[start..end.min(html.len())];
    let mut out = String::new();
    let mut in_tag = false;
    for c in slice.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Translates an HTML inline style snippet into a cell style for the
/// `formats` Paste Special mode. Values mode drops styling entirely.
pub fn style_for(style: Option<&str>, mode: &str) -> Option<CellStyle> {
    if mode == "values" {
        return None;
    }
    style.map(|s| CellStyle {
        font_family: None,
        font_size: None,
        font_weight: None,
        font_style: None,
        text_decoration: None,
        color: None,
        background: None,
        text_align: None,
        vertical_align: None,
        border: Some(s.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_html_table() {
        let html = "<table><tr><td>A1</td><td>B1</td></tr><tr><td>A2</td><td>42</td></tr></table>";
        let grid = parse_html_table(html);
        assert_eq!(grid.len(), 2);
        assert_eq!(grid[0].len(), 2);
        assert_eq!(grid[0][0].value, "A1");
        assert_eq!(grid[0][1].value, "B1");
        assert_eq!(grid[1][1].value, "42");
    }

    #[test]
    fn parses_th_headers_and_styles() {
        let html = "<table><tr><th style=\"font-weight:bold\">Name</th><th>Qty</th></tr><tr><td>x</td><td>1</td></tr></table>";
        let grid = parse_html_table(html);
        assert_eq!(grid[0][0].value, "Name");
        assert!(grid[0][0].style.is_some());
        assert_eq!(grid[1][0].value, "x");
    }

    #[test]
    fn handles_colspan() {
        let html = "<table><tr><td colspan=\"2\">wide</td></tr><tr><td>a</td><td>b</td></tr></table>";
        let grid = parse_html_table(html);
        assert_eq!(grid[0].len(), 2);
        assert_eq!(grid[0][0].value, "wide");
    }
}