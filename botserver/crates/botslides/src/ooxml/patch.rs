//! Write a package back out, replacing only the parts that changed (#2041).
//!
//! Untouched entries are copied as raw compressed data, so their bytes — and
//! their compression method, timestamps and permissions — survive exactly.
//! Only a targeted part is decompressed, edited and re-deflated.

use std::io::{Cursor, Read, Write};

use super::scan::{escape_xml, paragraph_texts, shape_paragraphs};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

/// Rebuilds a package, giving `replace` a chance to rewrite the named parts.
///
/// `replace` returns `None` to keep a part as it is. Every part that is not a
/// target is copied as raw compressed data.
pub fn rewrite_package(
    original: &[u8],
    targets: &[String],
    mut replace: impl FnMut(&str, &[u8]) -> Result<Option<Vec<u8>>, String>,
) -> Result<Vec<u8>, String> {
    let mut archive =
        ZipArchive::new(Cursor::new(original)).map_err(|e| format!("open pptx: {e}"))?;
    let comment = archive.comment().to_vec();

    let mut output = Cursor::new(Vec::new());
    {
        let mut writer = ZipWriter::new(&mut output);

        for index in 0..archive.len() {
            let mut file = archive
                .by_index(index)
                .map_err(|e| format!("read archive entry: {e}"))?;
            let name = file.name().map(|name| name.to_string()).unwrap_or_default();

            if !targets.iter().any(|target| target == &name) {
                writer
                    .raw_copy_file(file)
                    .map_err(|e| format!("copy {name}: {e}"))?;
                continue;
            }

            let mut content = Vec::new();
            file.read_to_end(&mut content)
                .map_err(|e| format!("read {name}: {e}"))?;
            let options = entry_options(&file);
            drop(file);

            let replacement = replace(&name, &content)?;
            let bytes = replacement.as_deref().unwrap_or(&content);

            writer
                .start_file(&name, options)
                .map_err(|e| format!("start {name}: {e}"))?;
            writer
                .write_all(bytes)
                .map_err(|e| format!("write {name}: {e}"))?;
        }

        writer
            .set_raw_comment(comment.into_boxed_slice())
            .map_err(|e| format!("comment: {e}"))?;
        writer.finish().map_err(|e| format!("finish zip: {e}"))?;
    }

    Ok(output.into_inner())
}

/// Keeps the entry's compression method, timestamp and unix mode so a rewritten
/// slide does not look like a different file to PowerPoint.
fn entry_options(file: &zip::read::ZipFile<'_, impl Read>) -> SimpleFileOptions {
    let mut options = SimpleFileOptions::default().compression_method(file.compression());
    if let Some(modified) = file.last_modified() {
        options = options.last_modified_time(modified);
    }
    if let Some(mode) = file.unix_mode() {
        options = options.unix_permissions(mode);
    }
    options
}

/// Patches edited paragraph text into the original package.
///
/// `changes` carries the slide part names — in presentation order — whose text
/// the model actually changed, with the paragraphs the model now holds. Only
/// those parts are rewritten; every other slide, the theme, the masters, the
/// notes and the media are copied byte for byte.
pub fn update_pptx_text(
    original: &[u8],
    changes: &[(String, Vec<String>)],
) -> Result<Vec<u8>, String> {
    if changes.is_empty() {
        // A save with no text edit must hand back the very same file.
        return Ok(original.to_vec());
    }
    let targets: Vec<String> = changes.iter().map(|(part, _)| part.clone()).collect();

    rewrite_package(original, &targets, |name, content| {
        let Some((_, texts)) = changes.iter().find(|(part, _)| part == name) else {
            return Ok(None);
        };
        let current = String::from_utf8_lossy(content);
        if &paragraph_texts(&current) == texts {
            return Ok(None);
        }
        Ok(Some(replace_slide_texts(&current, texts).into_bytes()))
    })
}

/// Rewrites the paragraphs of one slide part: the first text run of a paragraph
/// receives the new text, the remaining runs of that paragraph are emptied, and
/// every tag, attribute and empty paragraph stays as it was.
fn replace_slide_texts(xml: &str, new_texts: &[String]) -> String {
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    let mut text_index = 0;

    for paragraph in shape_paragraphs(xml) {
        let current = paragraph.text.trim();
        if current.is_empty() {
            continue;
        }
        let Some(new_text) = new_texts.get(text_index) else {
            break;
        };
        text_index += 1;
        if current == new_text.trim() {
            continue;
        }
        for (run_index, (start, end)) in paragraph.runs.iter().enumerate() {
            let replacement = if run_index == 0 {
                escape_xml(new_text)
            } else {
                String::new()
            };
            edits.push((*start, *end, replacement));
        }
    }

    let mut result = xml.to_string();
    for (start, end, replacement) in edits.into_iter().rev() {
        result.replace_range(start..end, &replacement);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use zip::CompressionMethod;

    fn slide(paragraphs: &str) -> String {
        format!(
            "<p:sld><p:cSld><p:spTree><p:sp><p:txBody>{paragraphs}</p:txBody></p:sp></p:spTree></p:cSld></p:sld>"
        )
    }

    fn package(entries: &[(&str, &str)]) -> Vec<u8> {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut buf);
            let options =
                SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
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
    fn an_empty_paragraph_does_not_shift_the_patch() {
        let xml = slide(
            "<a:p><a:r><a:t>Title</a:t></a:r></a:p>\
             <a:p><a:r><a:t></a:t></a:r></a:p>\
             <a:p><a:r><a:t>Body</a:t></a:r></a:p>",
        );
        let original = package(&[("ppt/slides/slide1.xml", &xml)]);
        let changes = vec![(
            "ppt/slides/slide1.xml".to_string(),
            vec!["Title".to_string(), "Body edited".to_string()],
        )];
        let patched = update_pptx_text(&original, &changes).expect("patch");
        let content = part(&patched, "ppt/slides/slide1.xml");
        assert!(content.contains("Body edited"), "{content}");
        assert!(!content.contains(">Body<"), "{content}");
        assert!(content.contains(">Title<"), "{content}");
    }

    #[test]
    fn a_package_without_changes_is_returned_verbatim() {
        let xml = slide("<a:p><a:r><a:t>Same</a:t></a:r></a:p>");
        let original = package(&[
            ("ppt/slides/slide1.xml", &xml),
            ("ppt/media/image1.png", "binary"),
        ]);
        let output = update_pptx_text(&original, &[]).expect("patch");
        assert_eq!(output, original, "nothing changed means the same package");
    }

    #[test]
    fn only_the_changed_part_changes() {
        let one = slide("<a:p><a:r><a:t>One</a:t></a:r></a:p>");
        let two = slide("<a:p><a:r><a:t>Two</a:t></a:r></a:p>");
        let original = package(&[
            ("ppt/slides/slide1.xml", &one),
            ("ppt/slides/slide2.xml", &two),
            ("ppt/theme/theme1.xml", "<a:theme/>"),
        ]);
        let changes = vec![(
            "ppt/slides/slide2.xml".to_string(),
            vec!["Two edited".to_string()],
        )];
        let output = update_pptx_text(&original, &changes).expect("patch");
        assert_eq!(part(&output, "ppt/slides/slide1.xml"), one);
        assert_eq!(part(&output, "ppt/theme/theme1.xml"), "<a:theme/>");
        assert!(part(&output, "ppt/slides/slide2.xml").contains("Two edited"));
    }
}
