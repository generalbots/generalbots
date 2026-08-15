//! External-link extraction (#788, E6).
//!
//! umya-spreadsheet drops external-link parts on read, so this walks the raw
//! .xlsx package (workbook rels + `xl/externalLinks/*.xml`) to rebuild
//! `ExternalLink` records. Conservative: a malformed part is skipped, never an
//! error.

use crate::types::ExternalLink;
use chrono::Utc;
use quick_xml::events::Event;
use std::collections::HashMap;
use std::io::Cursor;
use uuid::Uuid;

/// Rebuilds external workbook links from the raw xlsx package.
pub fn extract_external_links(bytes: &[u8]) -> Option<Vec<ExternalLink>> {
    let files = read_zip_entries(bytes).ok()?;
    let workbook_xml = files.get("xl/workbook.xml")?;
    let rels = files
        .get("xl/_rels/workbook.xml.rels")
        .map(|r| parse_rels(r))
        .unwrap_or_default();

    let mut links = Vec::new();
    for rid in find_external_refs(workbook_xml) {
        let Some(target) = rels.get(&rid) else {
            continue;
        };
        let Some(part_path) = normalize_path("xl", target) else {
            continue;
        };
        let Some(part_xml) = files.get(&part_path) else {
            continue;
        };

        let target_sheet = first_sheet_name(part_xml);
        let part_name = part_path.rsplit('/').next().unwrap_or("");
        let source_path = files
            .get(&format!("xl/externalLinks/_rels/{part_name}.rels"))
            .and_then(|x| external_target(x))
            .unwrap_or_default();

        links.push(ExternalLink {
            id: Uuid::new_v4().to_string(),
            source_path,
            link_type: "external".to_string(),
            target_sheet,
            target_range: None,
            status: "active".to_string(),
            last_updated: Utc::now(),
        });
    }

    if links.is_empty() {
        None
    } else {
        Some(links)
    }
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

/// Parses an OOXML `.rels` document into a map of relationship id -> target.
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

/// Collects the `r:id` of every `<externalReference>` in workbook.xml.
fn find_external_refs(xml: &[u8]) -> Vec<String> {
    let mut refs = Vec::new();
    let mut reader = quick_xml::Reader::from_reader(xml);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if local_name(e.name().as_ref()) == b"externalReference" =>
            {
                for attr in e.attributes().flatten() {
                    if local_name(attr.key.as_ref()) == b"id" {
                        let value = attr_value(&attr);
                        if value.starts_with("rId") {
                            refs.push(value);
                        }
                    }
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    refs
}

/// Returns the first `<sheetName val="..."/>` in an externalLink part.
fn first_sheet_name(xml: &[u8]) -> Option<String> {
    let mut reader = quick_xml::Reader::from_reader(xml);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if local_name(e.name().as_ref()) == b"sheetName" =>
            {
                for attr in e.attributes().flatten() {
                    if local_name(attr.key.as_ref()) == b"val" {
                        return Some(attr_value(&attr));
                    }
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    None
}

/// Returns the `Target` of the relationship marked `TargetMode="External"`.
fn external_target(rels_xml: &[u8]) -> Option<String> {
    let mut reader = quick_xml::Reader::from_reader(rels_xml);
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if local_name(e.name().as_ref()) == b"Relationship" =>
            {
                let mut target = None;
                let mut external = false;
                for attr in e.attributes().flatten() {
                    match local_name(attr.key.as_ref()) {
                        b"Target" => target = Some(attr_value(&attr)),
                        b"TargetMode" => {
                            external = attr_value(&attr).eq_ignore_ascii_case("External");
                        }
                        _ => {}
                    }
                }
                if external {
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

/// Normalizes a package-relative target against a base directory into a zip
/// entry path (see `chart_read` for the same helper).
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
