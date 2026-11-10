//! Keeps the deck's own theme when a structural edit forces a rebuilt package.
//!
//! Patching text is the only save that keeps every source feature. When the
//! model added or removed a slide there is nothing to patch, so the deck is
//! re-authored — but the author's theme (colours, fonts, effects) is a
//! self-contained part and is carried over from the original package instead of
//! being replaced by the built-in default. That is the difference between a
//! rebuilt deck that still looks like the source and one that does not (#2041).

use std::io::{Cursor, Read};

use crate::ooxml::rewrite_package;
use zip::ZipArchive;

const THEME_DIR: &str = "ppt/theme/";
const THEME_SUFFIX: &str = ".xml";

/// Returns the rebuilt package with the original theme parts, or `None` when
/// there is nothing to carry over (no theme, or a theme that references media
/// the rebuilt package does not contain).
pub fn keep_original_theme(generated: &[u8], original: &[u8]) -> Option<Vec<u8>> {
    let themes = self_contained_themes(original)?;
    if themes.is_empty() {
        return None;
    }
    let generated_parts = part_names(generated)?;

    let carried: Vec<(String, Vec<u8>)> = themes
        .into_iter()
        .filter(|(name, _)| generated_parts.iter().any(|part| part == name))
        .collect();
    if carried.is_empty() {
        return None;
    }

    let targets: Vec<String> = carried.iter().map(|(name, _)| name.clone()).collect();
    let merged = rewrite_package(generated, &targets, |name, _content| {
        Ok(carried
            .iter()
            .find(|(part, _)| part == name)
            .map(|(_, bytes)| bytes.clone()))
    });
    match merged {
        Ok(bytes) => Some(bytes),
        Err(error) => {
            log::warn!("slides export: could not carry the original theme over: {error}");
            None
        }
    }
}

/// Theme parts of the original package that stand on their own: a theme that
/// embeds media needs its `.rels` and image parts, which a rebuilt package does
/// not have, so it is left behind.
fn self_contained_themes(original: &[u8]) -> Option<Vec<(String, Vec<u8>)>> {
    let mut archive = ZipArchive::new(Cursor::new(original.to_vec())).ok()?;
    let rels: Vec<String> = (0..archive.len())
        .filter_map(|index| {
            let file = archive.by_index(index).ok()?;
            let name = file.name().map(|name| name.to_string()).ok()?;
            name.contains("/_rels/").then_some(name)
        })
        .collect();

    let mut themes = Vec::new();
    for index in 0..archive.len() {
        let Ok(mut file) = archive.by_index(index) else {
            continue;
        };
        let Ok(name) = file.name().map(|name| name.to_string()) else {
            continue;
        };
        if !name.starts_with(THEME_DIR) || !name.ends_with(THEME_SUFFIX) {
            continue;
        }
        let mut content = Vec::new();
        if file.read_to_end(&mut content).is_err() {
            continue;
        }
        let xml = String::from_utf8_lossy(&content);
        if xml.contains("r:embed") || xml.contains("r:id") {
            continue;
        }
        let rels_name = name.replace(THEME_DIR, "ppt/theme/_rels/") + ".rels";
        if rels.iter().any(|part| part == &rels_name) {
            continue;
        }
        themes.push((name, content));
    }
    Some(themes)
}

fn part_names(package: &[u8]) -> Option<Vec<String>> {
    let mut archive = ZipArchive::new(Cursor::new(package.to_vec())).ok()?;
    Some(
        (0..archive.len())
            .filter_map(|index| {
                let file = archive.by_index(index).ok()?;
                file.name().map(|name| name.to_string()).ok()
            })
            .collect(),
    )
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

    fn part(bytes: &[u8], name: &str) -> String {
        let mut archive = ZipArchive::new(Cursor::new(bytes.to_vec())).expect("zip");
        let mut content = String::new();
        archive
            .by_name(name)
            .expect("part")
            .read_to_string(&mut content)
            .expect("read");
        content
    }

    #[test]
    fn a_rebuilt_package_keeps_the_original_theme() {
        let original = package(&[("ppt/theme/theme1.xml", "<a:theme name=\"source\"/>")]);
        let generated = package(&[
            ("ppt/theme/theme1.xml", "<a:theme name=\"default\"/>"),
            ("ppt/slides/slide1.xml", "<p:sld/>"),
        ]);
        let merged = keep_original_theme(&generated, &original).expect("merged");
        assert_eq!(
            part(&merged, "ppt/theme/theme1.xml"),
            "<a:theme name=\"source\"/>"
        );
        assert_eq!(part(&merged, "ppt/slides/slide1.xml"), "<p:sld/>");
    }

    #[test]
    fn a_theme_with_embedded_media_is_left_behind() {
        let original = package(&[(
            "ppt/theme/theme1.xml",
            "<a:theme><a:blipFill><a:blip r:embed=\"rId1\"/></a:blipFill></a:theme>",
        )]);
        let generated = package(&[("ppt/theme/theme1.xml", "<a:theme name=\"default\"/>")]);
        assert!(keep_original_theme(&generated, &original).is_none());
    }

    #[test]
    fn a_package_without_themes_has_nothing_to_merge() {
        let original = package(&[("ppt/presentation.xml", "<p:presentation/>")]);
        let generated = package(&[("ppt/theme/theme1.xml", "<a:theme name=\"default\"/>")]);
        assert!(keep_original_theme(&generated, &original).is_none());
    }
}
