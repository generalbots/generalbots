//! Parser for the line-oriented structured text format.
//!
//! Supported lines (all optional, all whitespace-tolerant):
//! - `# Title`, `## Title`, ... — a heading at that level.
//! - `[[PAGE:n]]` — page marker, inherited by following elements.
//! - `[[CLAUSE:x]]` — clause marker, inherited by following elements.
//! - `| a | b |` — one table row. Consecutive rows form one table element.
//! - any other non-empty line — a paragraph line; consecutive lines are joined.
//!
//! Blank lines close the current paragraph and table. Malformed markers are
//! ignored rather than aborting the parse; the parser never panics.

use super::types::{Element, Locator};

/// Parses structured text into an ordered list of elements.
pub fn parse_structured_text(input: &str) -> Vec<Element> {
    let mut elements = Vec::new();
    let mut paragraph: Vec<String> = Vec::new();
    let mut table: Vec<Vec<String>> = Vec::new();
    let mut locator = Locator::default();

    for raw_line in input.lines() {
        let line = raw_line.trim();

        if line.is_empty() {
            flush_paragraph(&mut paragraph, &locator, &mut elements);
            flush_table(&mut table, &locator, &mut elements);
            continue;
        }

        if let Some((level, title)) = parse_heading(line) {
            flush_paragraph(&mut paragraph, &locator, &mut elements);
            flush_table(&mut table, &locator, &mut elements);
            elements.push(Element::Heading {
                level,
                text: title.to_string(),
            });
            continue;
        }

        if let Some(number) = tagged_value(line, "PAGE").and_then(|value| value.parse::<u32>().ok()) {
            flush_paragraph(&mut paragraph, &locator, &mut elements);
            flush_table(&mut table, &locator, &mut elements);
            locator.page = Some(number);
            continue;
        }

        if let Some(clause) = tagged_value(line, "CLAUSE") {
            flush_paragraph(&mut paragraph, &locator, &mut elements);
            flush_table(&mut table, &locator, &mut elements);
            locator.clause = Some(clause.to_string());
            continue;
        }

        if let Some(row) = parse_table_row(line) {
            flush_paragraph(&mut paragraph, &locator, &mut elements);
            table.push(row);
            continue;
        }

        flush_table(&mut table, &locator, &mut elements);
        paragraph.push(line.to_string());
    }

    flush_paragraph(&mut paragraph, &locator, &mut elements);
    flush_table(&mut table, &locator, &mut elements);
    elements
}

fn flush_paragraph(paragraph: &mut Vec<String>, locator: &Locator, elements: &mut Vec<Element>) {
    if paragraph.is_empty() {
        return;
    }
    elements.push(Element::Paragraph {
        text: paragraph.join(" "),
        locator: locator.clone(),
    });
    paragraph.clear();
}

fn flush_table(table: &mut Vec<Vec<String>>, locator: &Locator, elements: &mut Vec<Element>) {
    if table.is_empty() {
        return;
    }
    elements.push(Element::Table {
        rows: std::mem::take(table),
        locator: locator.clone(),
    });
}

fn parse_heading(line: &str) -> Option<(usize, &str)> {
    let level = line.bytes().take_while(|byte| *byte == b'#').count();
    if level == 0 {
        return None;
    }
    let title = line[level..].trim();
    if title.is_empty() {
        return None;
    }
    Some((level, title))
}

fn tagged_value<'a>(line: &'a str, tag: &str) -> Option<&'a str> {
    let inner = line.strip_prefix("[[")?.strip_suffix("]]")?;
    let (key, value) = inner.split_once(':')?;
    if key.trim() == tag {
        let value = value.trim();
        if !value.is_empty() {
            return Some(value);
        }
    }
    None
}

fn parse_table_row(line: &str) -> Option<Vec<String>> {
    if !line.starts_with('|') || !line.ends_with('|') {
        return None;
    }
    let cells: Vec<String> = line[1..line.len().saturating_sub(1)]
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect();
    if cells.is_empty() || cells.iter().all(|cell| cell.is_empty()) {
        return None;
    }
    Some(cells)
}
