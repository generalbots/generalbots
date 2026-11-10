//! One paragraph scan shared by text extraction and text patching (#2041).
//!
//! Extraction and patching must agree on what a paragraph is and where its text
//! runs live. When they disagree by a single `<a:p>` — an empty one, a table
//! cell, a paragraph with two runs — a patch lands in the wrong shape and the
//! saved deck is corrupted. Both sides therefore walk the XML with the same
//! code and the same rule: a paragraph counts only when it carries visible
//! text, and its runs are addressed by byte range.

pub struct Paragraph {
    /// Unescaped concatenation of every `<a:t>` run in the paragraph.
    pub text: String,
    /// Byte ranges of the run *contents* (not the tags) inside the scanned XML.
    pub runs: Vec<(usize, usize)>,
}

/// Every paragraph of a slide (or of a shape fragment) in document order.
pub fn paragraphs(xml: &str) -> Vec<Paragraph> {
    paragraphs_between(xml, 0, xml.len())
}

/// Paragraphs that live in shapes (`p:sp`) — the text the editor model can
/// represent. Table cells, chart frames and SmartArt are addressed by their own
/// parts, so their paragraphs are neither extracted nor patched: they stay
/// byte-identical in the saved package.
pub fn shape_paragraphs(xml: &str) -> Vec<Paragraph> {
    let mut out = Vec::new();
    for (start, end) in shape_blocks(xml) {
        out.extend(paragraphs_between(xml, start, end));
    }
    out
}

/// Byte ranges of every shape (`p:sp`) in the XML, in document order.
///
/// Import and the text patch both walk these blocks, so the model and the
/// package can never disagree about which shape a paragraph belongs to. A
/// plain `find("<p:sp")` would also match `p:spPr` and swallow the following
/// shapes into one fragment.
pub fn shape_blocks(xml: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut cursor = 0;
    while let Some(start) = find_open(xml, cursor, "<p:sp") {
        let Some(close) = xml[start..].find("</p:sp>") else {
            break;
        };
        let end = start + close + "</p:sp>".len();
        out.push((start, end));
        cursor = end;
    }
    out
}

/// The visible paragraph texts, trimmed, in document order. Empty paragraphs
/// are formatting artifacts the editor model cannot represent, so they are
/// dropped here — and skipped by the patch for the same reason.
pub fn paragraph_texts(xml: &str) -> Vec<String> {
    shape_paragraphs(xml)
        .into_iter()
        .map(|paragraph| paragraph.text.trim().to_string())
        .filter(|text| !text.is_empty())
        .collect()
}

fn paragraphs_between(xml: &str, from: usize, to: usize) -> Vec<Paragraph> {
    let mut out = Vec::new();
    let mut cursor = from;
    while let Some(start) = find_open(xml, cursor, "<a:p") {
        if start >= to {
            break;
        }
        let Some(close) = xml[start..].find("</a:p>") else {
            break;
        };
        let end = start + close + "</a:p>".len();
        out.push(scan_paragraph(xml, start, end));
        cursor = end;
    }
    out
}

/// Finds an opening tag, rejecting longer names that share the prefix
/// (`<a:pPr>` and `<a:pic>` are not paragraphs, `<a:tbl>`/`<a:tc>` are not
/// text runs).
fn find_open(xml: &str, from: usize, tag: &str) -> Option<usize> {
    let mut cursor = from;
    while let Some(offset) = xml[cursor..].find(tag) {
        let start = cursor + offset;
        let after = start + tag.len();
        match xml[after..].chars().next() {
            Some('>') | Some(' ') | Some('\t') | Some('\n') | Some('\r') => return Some(start),
            _ => cursor = after,
        }
    }
    None
}

fn scan_paragraph(xml: &str, start: usize, end: usize) -> Paragraph {
    let body = &xml[start..end];
    let mut runs = Vec::new();
    let mut text = String::new();
    let mut cursor = 0;

    while let Some(run) = find_open(body, cursor, "<a:t") {
        let Some(open_end) = body[run..].find('>') else {
            break;
        };
        let content_start = run + open_end + 1;
        let Some(close) = body[content_start..].find("</a:t>") else {
            break;
        };
        let content_end = content_start + close;

        runs.push((start + content_start, start + content_end));
        text.push_str(&unescape_xml(&body[content_start..content_end]));
        cursor = content_end + "</a:t>".len();
    }

    Paragraph { text, runs }
}

pub fn unescape_xml(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

pub fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape(paragraphs: &str) -> String {
        format!("<p:sp><p:txBody>{paragraphs}</p:txBody></p:sp>")
    }

    #[test]
    fn paragraphs_are_scanned_without_their_properties() {
        let xml = shape(
            "<a:p><a:pPr algn=\"ctr\"/><a:r><a:t>One</a:t></a:r></a:p>\
             <a:p><a:r><a:t>Two</a:t></a:r></a:p>",
        );
        assert_eq!(paragraph_texts(&xml), vec!["One".to_string(), "Two".to_string()]);
    }

    #[test]
    fn empty_paragraphs_are_dropped_consistently() {
        let xml = shape(
            "<a:p><a:r><a:t>Kept</a:t></a:r></a:p>\
             <a:p><a:r><a:t>   </a:t></a:r></a:p>",
        );
        assert_eq!(paragraph_texts(&xml), vec!["Kept".to_string()]);
        assert_eq!(shape_paragraphs(&xml).len(), 2);
    }

    #[test]
    fn table_text_is_not_part_of_the_model() {
        let xml = format!(
            "{}<p:graphicFrame><a:tbl><a:tr><a:tc><a:txBody>\
             <a:p><a:r><a:t>Cell</a:t></a:r></a:p>\
             </a:txBody></a:tc></a:tr></a:tbl></p:graphicFrame>",
            shape("<a:p><a:r><a:t>Body</a:t></a:r></a:p>")
        );
        assert_eq!(paragraph_texts(&xml), vec!["Body".to_string()]);
        assert_eq!(paragraphs(&xml).len(), 2, "the scan sees both paragraphs");
    }

    #[test]
    fn table_cells_are_not_mistaken_for_text_runs() {
        let xml = shape("<a:p><a:r><a:t>Cell</a:t></a:r></a:p>");
        let scanned = shape_paragraphs(&xml);
        assert_eq!(scanned.len(), 1);
        assert_eq!(scanned[0].runs.len(), 1);
        let (start, end) = scanned[0].runs[0];
        assert_eq!(&xml[start..end], "Cell");
    }

    #[test]
    fn entities_survive_a_round_trip() {
        let text = "Rice & beans <fresh> \"daily\"";
        assert_eq!(unescape_xml(&escape_xml(text)), text);
    }
}
