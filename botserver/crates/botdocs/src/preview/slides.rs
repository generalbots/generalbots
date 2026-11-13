//! Presentation previews: slide paragraphs/tables in authored order, speaker
//! notes resolved through the slide relationship part, plus the media gallery.

use std::io::{Cursor, Read};

use super::shell::media_gallery;

enum SlideItem {
    Paragraph(String),
    Table(Vec<Vec<String>>),
}

#[derive(Default)]
struct SlidePreview {
    paragraphs: Vec<String>,
    tables: Vec<Vec<Vec<String>>>,
    notes: Vec<String>,
    /// Paragraph/table order as authored. The separate vectors above stay for
    /// the existing assertions, but rendering follows this sequence so a table
    /// between two paragraphs is not moved to the bottom of the slide.
    sequence: Vec<SlideItem>,
}

impl SlidePreview {
    /// Splits one slide XML into flow paragraphs and tables. Paragraphs inside
    /// `a:tbl` are cell content: rendering them as slide paragraphs as well
    /// duplicated every table cell above the table (#1714).
    fn parse(xml: &str) -> Self {
        use quick_xml::events::Event;
        let mut reader = quick_xml::Reader::from_str(xml);
        reader.config_mut().allow_dangling_amp = true;
        let mut preview = Self::default();
        let mut in_table = 0usize;
        let mut in_row = false;
        let mut in_cell = false;
        let mut in_paragraph = false;
        let mut in_text = false;
        let mut paragraph = String::new();
        let mut cell = String::new();
        let mut row: Vec<String> = Vec::new();
        let mut table: Vec<Vec<String>> = Vec::new();
        loop {
            match reader.read_event() {
                Ok(Event::Start(event)) => match event.name().as_ref() {
                    "a:tbl" => {
                        in_table += 1;
                        if in_table == 1 {
                            table.clear();
                        }
                    }
                    "a:tr" if in_table > 0 => {
                        in_row = true;
                        row.clear();
                    }
                    "a:tc" if in_table > 0 => {
                        in_cell = true;
                        cell.clear();
                    }
                    "a:p" if in_table == 0 => {
                        in_paragraph = true;
                        paragraph.clear();
                    }
                    "a:t" => in_text = true,
                    _ => {}
                },
                Ok(Event::Text(text)) if in_text => {
                    let decoded = text.xml10_content();
                    if in_cell {
                        cell.push_str(&decoded);
                    } else if in_paragraph {
                        paragraph.push_str(&decoded);
                    }
                }
                Ok(Event::GeneralRef(reference)) if in_text => {
                    let raw = format!("&{};", reference.as_ref());
                    if let Ok(decoded) = quick_xml::escape::unescape(&raw) {
                        if in_cell {
                            cell.push_str(&decoded);
                        } else if in_paragraph {
                            paragraph.push_str(&decoded);
                        }
                    }
                }
                Ok(Event::End(event)) => match event.name().as_ref() {
                    "a:t" => in_text = false,
                    "a:tc" if in_cell => {
                        row.push(std::mem::take(&mut cell));
                        in_cell = false;
                    }
                    "a:tr" if in_row => {
                        table.push(std::mem::take(&mut row));
                        in_row = false;
                    }
                    "a:tbl" => {
                        in_table = in_table.saturating_sub(1);
                        if in_table == 0 {
                            let finished = std::mem::take(&mut table);
                            preview.sequence.push(SlideItem::Table(finished.clone()));
                            preview.tables.push(finished);
                        }
                    }
                    "a:p" if in_paragraph => {
                        let finished = std::mem::take(&mut paragraph);
                        preview
                            .sequence
                            .push(SlideItem::Paragraph(finished.clone()));
                        preview.paragraphs.push(finished);
                        in_paragraph = false;
                    }
                    _ => {}
                },
                Ok(Event::Eof) | Err(_) => break,
                _ => {}
            }
        }
        preview
    }
}

pub(super) fn presentation_html(bytes: &[u8]) -> Result<String, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("Failed to parse PPTX: {error}"))?;
    let mut slides: Vec<(u32, SlidePreview)> = Vec::new();
    for index in 0..archive.len() {
        let name = archive
            .by_index(index)
            .map_err(|error| format!("Failed to read PPTX entry: {error}"))?
            .name()
            .map_err(|error| format!("Failed to read PPTX entry name: {error}"))?
            .to_string();
        let Some(rest) = name.strip_prefix("ppt/slides/slide") else {
            continue;
        };
        let Some(number) = rest
            .strip_suffix(".xml")
            .and_then(|value| value.parse::<u32>().ok())
        else {
            continue;
        };
        let mut xml = String::new();
        archive
            .by_index(index)
            .map_err(|error| format!("Failed to read PPTX slide: {error}"))?
            .read_to_string(&mut xml)
            .map_err(|error| format!("Failed to read PPTX slide XML: {error}"))?;
        let mut preview = SlidePreview::parse(&xml);
        preview.notes = slide_notes(&mut archive, number);
        slides.push((number, preview));
    }
    slides.sort_by_key(|(number, _)| *number);
    if slides.is_empty() {
        return Err("no slides found in presentation".to_string());
    }
    let mut html = String::new();
    for (number, slide) in slides {
        html.push_str(&format!(r#"<div class="gb-slide"><h3>Slide {number}</h3>"#));
        for item in slide.sequence {
            match item {
                SlideItem::Paragraph(paragraph) => {
                    let text = paragraph.trim();
                    if !text.is_empty() {
                        html.push_str(&format!(
                            "<p>{}</p>",
                            crate::ui_fragments::html_escape(text)
                        ));
                    }
                }
                SlideItem::Table(table) => {
                    html.push_str("<table>");
                    for (row_index, row) in table.iter().enumerate() {
                        html.push_str("<tr>");
                        for cell in row {
                            let tag = if row_index == 0 { "th" } else { "td" };
                            html.push_str(&format!(
                                "<{tag}>{}</{tag}>",
                                crate::ui_fragments::html_escape(cell.trim())
                            ));
                        }
                        html.push_str("</tr>");
                    }
                    html.push_str("</table>");
                }
            }
        }
        if !slide.notes.is_empty() {
            html.push_str(r#"<div class="gb-slide-notes"><h4>Speaker notes</h4>"#);
            for note in slide.notes {
                html.push_str(&format!(
                    "<p>{}</p>",
                    crate::ui_fragments::html_escape(note.trim())
                ));
            }
            html.push_str("</div>");
        }
        html.push_str("</div>");
    }
    html.push_str(&media_gallery(bytes, "ppt/media/"));
    Ok(html)
}

fn zip_entry_text(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
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

/// Speaker notes for a slide. The note part is not guaranteed to share the
/// slide number, so the relationship target is resolved from the slide's
/// `.rels` file first (#1714).
fn slide_notes(archive: &mut zip::ZipArchive<Cursor<&[u8]>>, slide_number: u32) -> Vec<String> {
    let rels = format!("ppt/slides/_rels/slide{slide_number}.xml.rels");
    let Ok(rels_xml) = zip_entry_text(archive, &rels) else {
        return Vec::new();
    };
    let Some(target) = notes_target(&rels_xml) else {
        return Vec::new();
    };
    let path = rels_target_path("ppt/slides", &target);
    let Ok(xml) = zip_entry_text(archive, &path) else {
        return Vec::new();
    };
    notes_paragraphs(&xml)
}

fn notes_target(rels_xml: &str) -> Option<String> {
    let mut remaining = rels_xml;
    while let Some(position) = remaining.find("Target=\"") {
        let after = &remaining[position + 8..];
        let end = after.find('"')?;
        let value = &after[..end];
        if value.contains("notesSlide") && value.ends_with(".xml") {
            return Some(value.to_string());
        }
        remaining = &after[end..];
    }
    None
}

/// Resolves an OOXML relationship target (`../notesSlides/notesSlide1.xml`)
/// against the part directory (`ppt/slides`).
fn rels_target_path(base_dir: &str, target: &str) -> String {
    let mut parts: Vec<&str> = base_dir
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    for segment in target.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                parts.truncate(parts.len().saturating_sub(1));
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

fn notes_paragraphs(xml: &str) -> Vec<String> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(xml);
    reader.config_mut().allow_dangling_amp = true;
    let mut paragraphs = Vec::new();
    let mut current = String::new();
    let mut in_paragraph = false;
    let mut in_text = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => match event.name().as_ref() {
                "a:p" => {
                    in_paragraph = true;
                    current.clear();
                }
                "a:t" => in_text = true,
                _ => {}
            },
            Ok(Event::Text(text)) if in_text => {
                current.push_str(&text.xml10_content());
            }
            Ok(Event::GeneralRef(reference)) if in_text => {
                let raw = format!("&{};", reference.as_ref());
                if let Ok(decoded) = quick_xml::escape::unescape(&raw) {
                    current.push_str(&decoded);
                }
            }
            Ok(Event::End(event)) => match event.name().as_ref() {
                "a:t" => in_text = false,
                "a:p" => {
                    in_text = false;
                    if in_paragraph {
                        let text = std::mem::take(&mut current);
                        let trimmed = text.trim();
                        // Notes parts repeat the slide number and empty
                        // placeholders; neither is speaker note content.
                        if !trimmed.is_empty()
                            && !trimmed.chars().all(|character| character.is_ascii_digit())
                        {
                            paragraphs.push(text);
                        }
                        in_paragraph = false;
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    paragraphs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slide_paragraphs_collect_runs_per_paragraph() {
        let xml = r#"<p:sld xmlns:a="a"><a:p><a:r><a:t>Quarter</a:t></a:r><a:r><a:t> results</a:t></a:r></a:p><a:p><a:t>Revenue up</a:t></a:p></p:sld>"#;
        let paragraphs = SlidePreview::parse(xml).paragraphs;
        assert_eq!(paragraphs, vec!["Quarter results", "Revenue up"]);
    }

    #[test]
    fn slide_tables_keep_rows_and_cells_out_of_paragraphs() {
        let xml = r#"<p:sld xmlns:a="a"><a:p><a:t>Title</a:t></a:p><a:tbl><a:tr><a:tc><a:p><a:t>A</a:t></a:p></a:tc><a:tc><a:p><a:t>B</a:t></a:p></a:tc></a:tr><a:tr><a:tc><a:p><a:t>1</a:t></a:p></a:tc><a:tc><a:p><a:t>2</a:t></a:p></a:tc></a:tr></a:tbl></p:sld>"#;
        let slide = SlidePreview::parse(xml);
        assert_eq!(slide.paragraphs, vec!["Title"]);
        assert_eq!(slide.tables, vec![vec![vec!["A", "B"], vec!["1", "2"]]]);
    }

    #[test]
    fn notes_target_resolves_the_relationship() {
        let rels = r#"<Relationships><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide" Target="../notesSlides/notesSlide3.xml"/></Relationships>"#;
        let target = notes_target(rels).expect("notes relationship");
        assert_eq!(
            rels_target_path("ppt/slides", &target),
            "ppt/notesSlides/notesSlide3.xml"
        );
    }

    #[test]
    fn notes_paragraphs_skip_slide_number_placeholders() {
        let xml = r#"<p:notes xmlns:a="a"><a:p><a:t>Keep this note</a:t></a:p><a:p><a:t>7</a:t></a:p></p:notes>"#;
        assert_eq!(notes_paragraphs(xml), vec!["Keep this note"]);
    }
}
