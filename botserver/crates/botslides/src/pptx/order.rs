//! Slide part order, as the deck presents it (#2041).
//!
//! Writing text back into a package must address the same slide the editor
//! model holds. The presentation order (`p:sldIdLst` → relationship → part) is
//! authoritative; a part file name (`slide7.xml`) is only storage. Decks
//! reordered in PowerPoint, or produced by other tools, routinely disagree, so
//! import and the text patch both resolve the order here instead of sorting by
//! the digits in the file name.

use std::io::{Cursor, Read, Seek};
use zip::ZipArchive;

/// Slide part names (`ppt/slides/slideN.xml`) in presentation order.
///
/// Falls back to the numeric file-name order when `presentation.xml` or its
/// relationships are missing or unreadable, which is what the importer did
/// before; a package without a usable slide list still has to open.
pub fn slide_part_names(bytes: &[u8]) -> Result<Vec<String>, String> {
    let mut archive =
        ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("open pptx: {e}"))?;

    let mut names = presentation_order(&mut archive);
    names.retain(|name| archive.by_name(name).is_ok());
    if names.is_empty() {
        names = by_file_name(&mut archive);
    }
    Ok(names)
}

fn presentation_order<R: Read + Seek>(archive: &mut ZipArchive<R>) -> Vec<String> {
    let (Some(presentation), Some(rels)) = (
        read_part(archive, "ppt/presentation.xml"),
        read_part(archive, "ppt/_rels/presentation.xml.rels"),
    ) else {
        return Vec::new();
    };

    let targets = relationship_targets(&rels);
    let mut names = Vec::new();
    for id in slide_ids(&presentation) {
        let Some(target) = targets
            .iter()
            .find(|(rel_id, _)| *rel_id == id)
            .map(|(_, target)| target.clone())
        else {
            continue;
        };
        names.push(normalize_target(&target));
    }
    names
}

fn read_part<R: Read + Seek>(archive: &mut ZipArchive<R>, name: &str) -> Option<String> {
    let mut file = archive.by_name(name).ok()?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).ok()?;
    Some(String::from_utf8_lossy(&buf).into_owned())
}

/// `r:id` of every `<p:sldId>` in document order.
fn slide_ids(presentation: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut cursor = 0;
    while let Some(offset) = presentation[cursor..].find("<p:sldId ") {
        let start = cursor + offset;
        let Some(close) = presentation[start..].find('>') else {
            break;
        };
        let tag = &presentation[start..start + close];
        if let Some(id) = attribute(tag, "r:id") {
            ids.push(id);
        }
        cursor = start + close;
    }
    ids
}

/// `(Id, Target)` of every relationship in a `.rels` part.
fn relationship_targets(rels: &str) -> Vec<(String, String)> {
    let mut targets = Vec::new();
    let mut cursor = 0;
    while let Some(offset) = rels[cursor..].find("<Relationship ") {
        let start = cursor + offset;
        let Some(close) = rels[start..].find('>') else {
            break;
        };
        let tag = &rels[start..start + close];
        if let (Some(id), Some(target)) = (attribute(tag, "Id"), attribute(tag, "Target")) {
            targets.push((id, target));
        }
        cursor = start + close;
    }
    targets
}

fn attribute(tag: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    Some(tag[start..end].to_string())
}

/// A relationship target is relative to `ppt/` and may be absolute or carry a
/// fragment; parts are addressed by their full name inside the package.
fn normalize_target(target: &str) -> String {
    let target = target.split('#').next().unwrap_or(target);
    let target = target.trim_start_matches('/');
    if target.starts_with("ppt/") {
        target.to_string()
    } else {
        format!("ppt/{target}")
    }
}

fn by_file_name<R: Read + Seek>(archive: &mut ZipArchive<R>) -> Vec<String> {
    let mut names = Vec::new();
    for index in 0..archive.len() {
        let Ok(file) = archive.by_index(index) else {
            continue;
        };
        let name = file.name().map(|name| name.to_string()).unwrap_or_default();
        if name.starts_with("ppt/slides/slide")
            && name.ends_with(".xml")
            && !name.contains("_rels")
        {
            names.push(name);
        }
    }
    names.sort_by_key(|name| {
        name.trim_start_matches("ppt/slides/slide")
            .trim_end_matches(".xml")
            .parse::<u32>()
            .unwrap_or(0)
    });
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn package(entries: &[(&str, &str)]) -> Vec<u8> {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            for (name, content) in entries {
                zip.start_file(*name, options).expect("entry");
                zip.write_all(content.as_bytes()).expect("write");
            }
            zip.finish().expect("finish");
        }
        buf.into_inner()
    }

    fn deck(slide_ids: &[(&str, &str)]) -> Vec<u8> {
        let ids: String = slide_ids
            .iter()
            .map(|(id, _)| format!("<p:sldId id=\"256\" r:id=\"{id}\"/>"))
            .collect();
        let presentation = format!(
            "<p:presentation><p:sldIdLst>{ids}</p:sldIdLst></p:presentation>"
        );
        let rels: String = slide_ids
            .iter()
            .map(|(id, target)| {
                format!("<Relationship Id=\"{id}\" Type=\"slide\" Target=\"{target}\"/>")
            })
            .collect();
        package(&[
            ("ppt/presentation.xml", &presentation),
            ("ppt/_rels/presentation.xml.rels", &rels),
            ("ppt/slides/slide1.xml", "<p:sld>one</p:sld>"),
            ("ppt/slides/slide2.xml", "<p:sld>two</p:sld>"),
        ])
    }

    #[test]
    fn order_follows_the_slide_list_not_the_file_name() {
        let bytes = deck(&[("rId2", "slides/slide2.xml"), ("rId1", "slides/slide1.xml")]);
        assert_eq!(
            slide_part_names(&bytes).expect("order"),
            vec![
                "ppt/slides/slide2.xml".to_string(),
                "ppt/slides/slide1.xml".to_string()
            ]
        );
    }

    #[test]
    fn absolute_targets_are_normalised() {
        let bytes = deck(&[("rId1", "/ppt/slides/slide1.xml")]);
        assert_eq!(
            slide_part_names(&bytes).expect("order"),
            vec!["ppt/slides/slide1.xml".to_string()]
        );
    }

    #[test]
    fn a_package_without_a_slide_list_falls_back_to_the_file_names() {
        let bytes = package(&[
            ("ppt/slides/slide10.xml", "<p:sld/>"),
            ("ppt/slides/slide2.xml", "<p:sld/>"),
        ]);
        assert_eq!(
            slide_part_names(&bytes).expect("order"),
            vec![
                "ppt/slides/slide2.xml".to_string(),
                "ppt/slides/slide10.xml".to_string()
            ]
        );
    }
}
