//! PPTX → sanitized HTML: one section per slide, tables, speaker notes
//! resolved through the slide's relationship part.

use super::common::{document_shell, escape_html, media_gallery, zip_entry_text};
use std::io::Cursor;
use zip::ZipArchive;

pub(super) fn slides_html(file_name: &str, bytes: &[u8]) -> Result<String, String> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("Failed to open PPTX as ZIP: {error}"))?;
    let mut slides: Vec<(u32, String)> = Vec::new();
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
        slides.push((number, xml));
    }
    if slides.is_empty() {
        return Err("No slide text found in PPTX archive".to_string());
    }
    slides.sort_by_key(|(number, _)| *number);
    let mut body = String::new();
    for (number, xml) in slides {
        body.push_str(&format!("<section class=\"gb-slide\"><h3>Slide {number}</h3>"));
        body.push_str(&pptx_slide_html(&xml));
        let notes = pptx_slide_notes(&mut archive, number);
        if !notes.is_empty() {
            body.push_str("<div class=\"gb-slide-notes\"><h4>Speaker notes</h4>");
            for note in notes {
                body.push_str(&format!("<p>{}</p>", escape_html(&note)));
            }
            body.push_str("</div>");
        }
        body.push_str("</section>");
    }
    body.push_str(&media_gallery(bytes, "ppt/media/"));
    Ok(document_shell(file_name, &body))
}

/// Flow paragraphs and tables of one slide. Table cells are skipped by the
/// paragraph walker so their text is not duplicated above the table (#1714).
fn pptx_slide_html(xml: &str) -> String {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut html = String::new();
    let mut in_table = 0usize;
    let mut in_row = false;
    let mut in_cell = false;
    let mut in_paragraph = false;
    let mut in_text = false;
    let mut paragraph = String::new();
    let mut cell = String::new();
    let mut row: Vec<String> = Vec::new();
    let mut table_row = 0usize;
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => match event.name().as_ref() {
                b"a:tbl" => {
                    in_table += 1;
                    table_row = 0;
                    html.push_str("<table>");
                }
                b"a:tr" if in_table > 0 => {
                    in_row = true;
                    row.clear();
                }
                b"a:tc" if in_table > 0 => {
                    in_cell = true;
                    cell.clear();
                }
                b"a:p" if in_table == 0 => {
                    in_paragraph = true;
                    paragraph.clear();
                }
                b"a:t" => in_text = true,
                _ => {}
            },
            Ok(Event::Text(text)) if in_text => {
                if let Ok(decoded) = text.unescape() {
                    if in_cell {
                        cell.push_str(&decoded);
                    } else if in_paragraph {
                        paragraph.push_str(&decoded);
                    }
                }
            }
            Ok(Event::End(event)) => match event.name().as_ref() {
                b"a:t" => in_text = false,
                b"a:tc" if in_cell => {
                    row.push(std::mem::take(&mut cell));
                    in_cell = false;
                }
                b"a:tr" if in_row => {
                    html.push_str("<tr>");
                    for value in &row {
                        let tag = if table_row == 0 { "th" } else { "td" };
                        html.push_str(&format!("<{tag}>{}</{tag}>", escape_html(value.trim())));
                    }
                    html.push_str("</tr>");
                    table_row += 1;
                    row.clear();
                    in_row = false;
                }
                b"a:tbl" => {
                    in_table = in_table.saturating_sub(1);
                    html.push_str("</table>");
                }
                b"a:p" if in_paragraph => {
                    let text = paragraph.trim();
                    if !text.is_empty() {
                        html.push_str(&format!("<p>{}</p>", escape_html(text)));
                    }
                    in_paragraph = false;
                }
                _ => {}
            },
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }
    html
}

/// Speaker notes for a slide, resolved through its `.rels` part because the
/// notes part number is not guaranteed to match the slide number (#1714).
fn pptx_slide_notes(archive: &mut ZipArchive<Cursor<&[u8]>>, slide_number: u32) -> Vec<String> {
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

fn rels_target_path(base_dir: &str, target: &str) -> String {
    let mut parts: Vec<&str> = base_dir.split('/').filter(|part| !part.is_empty()).collect();
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
    let mut paragraphs = Vec::new();
    let mut current = String::new();
    let mut in_paragraph = false;
    let mut in_text = false;
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => match event.name().as_ref() {
                b"a:p" => {
                    in_paragraph = true;
                    current.clear();
                }
                b"a:t" => in_text = true,
                _ => {}
            },
            Ok(Event::Text(text)) if in_text => {
                if let Ok(decoded) = text.unescape() {
                    current.push_str(&decoded);
                }
            }
            Ok(Event::End(event)) => match event.name().as_ref() {
                b"a:t" => in_text = false,
                b"a:p" => {
                    in_text = false;
                    if in_paragraph {
                        let text = std::mem::take(&mut current);
                        let trimmed = text.trim();
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
    use super::{notes_paragraphs, notes_target, pptx_slide_html, rels_target_path};

    #[test]
    fn pptx_slide_renders_tables_without_duplicating_cells() {
        let xml = r#"<p:sld xmlns:a="a"><a:p><a:t>Title</a:t></a:p><a:tbl><a:tr><a:tc><a:p><a:t>A</a:t></a:p></a:tc><a:tc><a:p><a:t>B</a:t></a:p></a:tc></a:tr></a:tbl></p:sld>"#;
        let html = pptx_slide_html(xml);
        assert!(html.contains("<p>Title</p>"), "{html}");
        assert!(html.contains("<table><tr><th>A</th><th>B</th></tr></table>"), "{html}");
        assert_eq!(html.matches("Title").count(), 1);
    }

    #[test]
    fn pptx_notes_relationship_resolves_and_skips_slide_number() {
        let rels = r#"<Relationships><Relationship Id="rId2" Target="../notesSlides/notesSlide4.xml"/></Relationships>"#;
        let target = notes_target(rels).expect("notes relationship");
        assert_eq!(
            rels_target_path("ppt/slides", &target),
            "ppt/notesSlides/notesSlide4.xml"
        );
        let notes = r#"<p:notes xmlns:a="a"><a:p><a:t>Explain the table</a:t></a:p><a:p><a:t>4</a:t></a:p></p:notes>"#;
        assert_eq!(notes_paragraphs(notes), vec!["Explain the table"]);
    }
}
