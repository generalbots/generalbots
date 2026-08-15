//! Cell comment / note extraction (#788, gap 27).
//!
//! umya-spreadsheet drops cell comments on read, so this walks the raw
//! package: the sheet's relationships point at `xl/commentsN.xml`, whose
//! `<commentList>` carries the cell ref, author id and rich-text runs.
//! Conservative: a malformed part yields no record, never an error.

use crate::types::CellComment;
use chrono::Utc;
use quick_xml::events::Event;
use std::collections::HashMap;
use std::io::Cursor;
use uuid::Uuid;

/// Returns, per 0-based worksheet index, a `"row,col"` map of comments.
pub fn extract_comments(bytes: &[u8]) -> HashMap<usize, HashMap<String, CellComment>> {
    let mut result = HashMap::new();
    let Ok(files) = read_zip_entries(bytes) else {
        return result;
    };
    let Some(workbook_xml) = files.get("xl/workbook.xml") else {
        return result;
    };
    let rels = files
        .get("xl/_rels/workbook.xml.rels")
        .map(|r| parse_rels(r))
        .unwrap_or_default();

    for (sheet_index, sheet_path) in sheet_parts(workbook_xml, &rels) {
        let Some(rels_path) = rels_path_for(&sheet_path) else {
            continue;
        };
        let Some(rels_xml) = files.get(&rels_path) else {
            continue;
        };
        let Some(target) = comments_target(rels_xml) else {
            continue;
        };
        let Some(comments_path) = normalize_path("xl/worksheets", &target) else {
            continue;
        };
        let Some(comments_xml) = files.get(&comments_path) else {
            continue;
        };
        if let Some(map) = parse_comments(comments_xml) {
            result.insert(sheet_index, map);
        }
    }
    result
}

/// Parses `xl/commentsN.xml` into a `"row,col"` map of comments. Authors are
/// resolved by their `authorId` index; runs concatenate into the comment text.
fn parse_comments(xml: &[u8]) -> Option<HashMap<String, CellComment>> {
    let mut map = HashMap::new();
    let mut reader = quick_xml::Reader::from_reader(xml);
    let mut buf = Vec::new();

    let mut authors: Vec<String> = Vec::new();
    let mut current_ref: Option<String> = None;
    let mut current_author: Option<usize> = None;
    let mut author_buf = String::new();
    let mut comment_buf = String::new();
    let mut in_authors = false;
    let mut in_author = false;
    let mut in_comment = false;
    let mut in_t = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match local_name(e.name().as_ref()) {
                b"authors" => in_authors = true,
                b"author" if in_authors => {
                    in_author = true;
                    author_buf.clear();
                }
                b"comment" => {
                    in_comment = true;
                    current_ref = None;
                    current_author = None;
                    comment_buf.clear();
                    for attr in e.attributes().flatten() {
                        match local_name(attr.key.as_ref()) {
                            b"ref" => current_ref = Some(attr_value(&attr)),
                            b"authorId" => current_author = attr_value(&attr).parse().ok(),
                            _ => {}
                        }
                    }
                }
                b"t" if in_comment => in_t = true,
                _ => {}
            },
            Ok(Event::Text(ref t)) => {
                let Ok(text) = t.unescape() else {
                    continue;
                };
                if in_author && in_authors {
                    author_buf.push_str(&text);
                } else if in_comment && in_t {
                    comment_buf.push_str(&text);
                }
            }
            Ok(Event::CData(ref t)) => {
                if in_comment && in_t {
                    comment_buf.push_str(&String::from_utf8_lossy(&t[..]));
                }
            }
            Ok(Event::End(ref e)) => match local_name(e.name().as_ref()) {
                b"author" => {
                    authors.push(author_buf.clone());
                    in_author = false;
                }
                b"authors" => in_authors = false,
                b"t" => in_t = false,
                b"comment" => {
                    in_comment = false;
                    if let Some(cell_ref) = &current_ref {
                        if let Some((row, col)) = super::xlsx_layout::parse_cell_ref(cell_ref) {
                            let author_name = current_author
                                .and_then(|i| authors.get(i))
                                .cloned()
                                .unwrap_or_default();
                            map.insert(
                                format!("{row},{col}"),
                                CellComment {
                                    id: Uuid::new_v4().to_string(),
                                    author_id: author_name.clone(),
                                    author_name,
                                    content: comment_buf.clone(),
                                    created_at: Utc::now(),
                                    updated_at: Utc::now(),
                                    replies: vec![],
                                    resolved: false,
                                },
                            );
                        }
                    }
                }
                _ => {}
            },
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

/// `xl/worksheets/sheet1.xml` -> `xl/worksheets/_rels/sheet1.xml.rels`.
fn rels_path_for(sheet_path: &str) -> Option<String> {
    let (dir, file) = sheet_path.rsplit_once('/')?;
    Some(format!("{dir}/_rels/{file}.rels"))
}

/// Returns the `Target` of the relationship whose `Type` ends in `/comments`.
fn comments_target(rels_xml: &[u8]) -> Option<String> {
    let mut reader = quick_xml::Reader::from_reader(rels_xml);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if local_name(e.name().as_ref()) == b"Relationship" =>
            {
                let mut target = None;
                let mut is_comments = false;
                for attr in e.attributes().flatten() {
                    match local_name(attr.key.as_ref()) {
                        b"Target" => target = Some(attr_value(&attr)),
                        b"Type" => is_comments = attr_value(&attr).ends_with("/comments"),
                        _ => {}
                    }
                }
                if is_comments {
                    return target;
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    None
}

/// Returns the ordered `(sheet_index, sheet_path)` pairs from workbook.xml.
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
