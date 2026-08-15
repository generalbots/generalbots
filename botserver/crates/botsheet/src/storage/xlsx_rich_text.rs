//! Rich-text run extraction (#788, E6).
//!
//! umya-spreadsheet flattens rich text — `get_value()` returns the concatenated
//! string and drops per-run bold/italic/colour. This walks the raw package
//! (`xl/sharedStrings.xml` for the runs, `xl/worksheets/sheetN.xml` for the
//! `t="s"` cells that reference them) and rebuilds a `"row,col"` -> runs map.
//! Conservative: a malformed part yields no record, never an error.

use crate::types::RichTextRun;
use quick_xml::events::Event;
use std::collections::HashMap;
use std::io::Cursor;

/// Returns, per 0-based worksheet index, a `"row,col"` map of rich-text runs.
/// Cells whose shared string is a single unformatted `<t>` are omitted — their
/// flattened value is already in the model via umya.
pub fn extract_rich_text_all(bytes: &[u8]) -> HashMap<usize, HashMap<String, Vec<RichTextRun>>> {
    let mut result = HashMap::new();
    let Ok(files) = read_zip_entries(bytes) else {
        return result;
    };
    let Some(sst_xml) = files.get("xl/sharedStrings.xml") else {
        return result;
    };
    let Some(workbook_xml) = files.get("xl/workbook.xml") else {
        return result;
    };
    let rels = files
        .get("xl/_rels/workbook.xml.rels")
        .map(|r| parse_rels(r))
        .unwrap_or_default();

    let rich_strings = parse_shared_strings(sst_xml);
    if rich_strings.is_empty() {
        return result;
    }

    for (sheet_index, part_path) in sheet_parts(workbook_xml, &rels) {
        let Some(sheet_xml) = files.get(&part_path) else {
            continue;
        };
        if let Some(map) = map_sheet_rich_text(sheet_xml, &rich_strings) {
            result.insert(sheet_index, map);
        }
    }
    result
}

/// Parses `sharedStrings.xml`, returning `Some(runs)` for rich entries and
/// `None` for plain single-text entries. The vector index is the shared-string
/// index (`t="s"` cell `<v>` value).
fn parse_shared_strings(xml: &[u8]) -> Vec<Option<Vec<RichTextRun>>> {
    let mut out = Vec::new();
    let mut reader = quick_xml::Reader::from_reader(xml);
    let mut buf = Vec::new();

    let mut runs: Vec<RichTextRun> = Vec::new();
    let mut has_runs = false;
    let mut in_r = false;
    let mut in_t = false;
    let mut text_buf = String::new();
    let mut current = default_run();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match local_name(e.name().as_ref()) {
                b"si" => {
                    runs.clear();
                    has_runs = false;
                }
                b"r" => {
                    has_runs = true;
                    in_r = true;
                    current = default_run();
                }
                b"t" => {
                    in_t = true;
                    text_buf.clear();
                }
                name => run_prop(name, e, in_r, &mut current),
            },
            // `<b/>`, `<i/>`, `<color/>` etc. are self-closing (Empty) events.
            Ok(Event::Empty(ref e)) => run_prop(local_name(e.name().as_ref()), e, in_r, &mut current),
            Ok(Event::Text(ref t)) => {
                if in_t {
                    if let Ok(text) = t.unescape() {
                        text_buf.push_str(&text);
                    }
                }
            }
            Ok(Event::CData(ref t)) => {
                if in_t {
                    text_buf.push_str(&String::from_utf8_lossy(&t[..]));
                }
            }
            Ok(Event::End(ref e)) => match local_name(e.name().as_ref()) {
                b"t" => {
                    in_t = false;
                    if in_r {
                        current.text = text_buf.clone();
                    }
                }
                b"r" => {
                    in_r = false;
                    runs.push(current.clone());
                }
                b"si" => {
                    out.push(if has_runs { Some(runs.clone()) } else { None });
                }
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    out
}

/// Maps a worksheet part's `t="s"` cells onto rich-text runs.
fn map_sheet_rich_text(
    xml: &[u8],
    rich_strings: &[Option<Vec<RichTextRun>>],
) -> Option<HashMap<String, Vec<RichTextRun>>> {
    let mut map = HashMap::new();
    let mut reader = quick_xml::Reader::from_reader(xml);
    let mut buf = Vec::new();

    let mut current_ref: Option<String> = None;
    let mut current_is_shared = false;
    let mut in_v = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                match local_name(e.name().as_ref()) {
                    b"c" => {
                        current_ref = None;
                        current_is_shared = false;
                        in_v = false;
                        for attr in e.attributes().flatten() {
                            match local_name(attr.key.as_ref()) {
                                b"r" => current_ref = Some(attr_value(&attr)),
                                b"t" => current_is_shared = attr_value(&attr) == "s",
                                _ => {}
                            }
                        }
                    }
                    b"v" => in_v = true,
                    _ => {}
                }
            }
            Ok(Event::Text(ref t)) => {
                if !(in_v && current_is_shared) {
                    continue;
                }
                let Some(cell_ref) = &current_ref else {
                    continue;
                };
                let Ok(text) = t.unescape() else {
                    continue;
                };
                let Ok(idx) = text.trim().parse::<usize>() else {
                    continue;
                };
                let Some(Some(runs)) = rich_strings.get(idx) else {
                    continue;
                };
                if let Some((row, col)) = super::xlsx_layout::parse_cell_ref(cell_ref) {
                    map.insert(format!("{row},{col}"), runs.clone());
                }
            }
            Ok(Event::End(ref e)) => {
                if local_name(e.name().as_ref()) == b"v" {
                    in_v = false;
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

fn default_run() -> RichTextRun {
    RichTextRun {
        text: String::new(),
        bold: None,
        italic: None,
        underline: None,
        strike: None,
        color: None,
        font: None,
        size: None,
    }
}

/// Applies a run-property element (`<b/>`, `<color/>`, `<sz/>`, ...) to the
/// current run. No-ops outside a run or for unknown elements.
fn run_prop(name: &[u8], e: &quick_xml::events::BytesStart, in_r: bool, current: &mut RichTextRun) {
    if !in_r {
        return;
    }
    match name {
        b"b" => current.bold = Some(true),
        b"i" => current.italic = Some(true),
        b"u" => current.underline = Some(true),
        b"strike" => current.strike = Some(true),
        b"color" => current.color = attr(e, b"rgb").and_then(|a| argb_to_hex(&a)),
        b"rFont" => current.font = attr(e, b"val"),
        b"sz" => current.size = attr(e, b"val").and_then(|s| s.parse().ok()),
        _ => {}
    }
}

/// `#AARRGGBB` -> `#RRGGBB` (the alpha byte is dropped for display).
fn argb_to_hex(argb: &str) -> Option<String> {
    if argb.len() >= 8 {
        Some(format!("#{}", &argb[2..8]))
    } else if argb.is_empty() {
        None
    } else {
        Some(format!("#{argb}"))
    }
}

/// Returns the ordered `(sheet_index, part_path)` pairs from workbook.xml.
/// The sheet `r:id` is resolved through `xl/_rels/workbook.xml.rels`; when that
/// mapping is missing the conventional `xl/worksheets/sheetN.xml` name is used.
fn sheet_parts(xml: &[u8], rels: &HashMap<String, String>) -> Vec<(usize, String)> {
    let mut ids = Vec::new();
    let mut reader = quick_xml::Reader::from_reader(xml);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if local_name(e.name().as_ref()) == b"sheet" =>
            {
                for attr in e.attributes().flatten() {
                    if local_name(attr.key.as_ref()) == b"id" {
                        ids.push(attr_value(&attr));
                    }
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    ids.into_iter()
        .enumerate()
        .map(|(index, rid)| {
            let part_path = rels
                .get(&rid)
                .and_then(|target| normalize_path("xl", target))
                .unwrap_or_else(|| format!("xl/worksheets/sheet{}.xml", index + 1));
            (index, part_path)
        })
        .collect()
}

fn read_zip_entries(bytes: &[u8]) -> Result<HashMap<String, Vec<u8>>, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| format!("Failed to open xlsx zip: {e}"))?;
    let mut files = HashMap::new();
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read zip entry {i}: {e}"))?;
        let name = entry.name().to_string();
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut data)
            .map_err(|e| format!("Failed to read zip entry {name}: {e}"))?;
        files.insert(name, data);
    }
    Ok(files)
}

fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|&c| c == b':').next().unwrap_or(name)
}

fn attr_value(attr: &quick_xml::events::attributes::Attribute) -> String {
    attr.unescape_value()
        .map(|v| v.into_owned())
        .unwrap_or_else(|_| String::from_utf8_lossy(&attr.value).into_owned())
}

/// Returns the value of the first attribute whose local name matches `name`.
fn attr(e: &quick_xml::events::BytesStart, name: &[u8]) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| local_name(a.key.as_ref()) == name)
        .map(|a| attr_value(&a))
}

fn parse_rels(xml: &[u8]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut reader = quick_xml::Reader::from_reader(xml);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if local_name(e.name().as_ref()) == b"Relationship" =>
            {
                let mut id = String::new();
                let mut target = String::new();
                for attr in e.attributes().flatten() {
                    match local_name(attr.key.as_ref()) {
                        b"Id" => id = attr_value(&attr),
                        b"Target" => target = attr_value(&attr),
                        _ => {}
                    }
                }
                if !id.is_empty() && !target.is_empty() {
                    map.insert(id, target);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    map
}

/// Normalizes a package-relative target against a base directory into a zip
/// entry path (see `xlsx_external_links` for the same helper).
fn normalize_path(base_dir: &str, target: &str) -> Option<String> {
    let mut parts: Vec<&str> = if let Some(rest) = target.strip_prefix('/') {
        rest.split('/').filter(|s| !s.is_empty()).collect()
    } else {
        base_dir.split('/').filter(|s| !s.is_empty()).collect()
    };
    if target.starts_with('/') {
        if parts.is_empty() {
            return None;
        }
        return Some(parts.join("/"));
    }
    for comp in target.split('/') {
        match comp {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            c => parts.push(c),
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("/"))
    }
}
