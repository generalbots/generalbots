use std::io::Cursor;

pub fn extract_docx_text_from_zip(bytes: &[u8]) -> Result<String, String> {
    use std::io::Read;
    use zip::ZipArchive;

    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader)
        .map_err(|e| format!("Failed to open DOCX as ZIP: {e}"))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry: {e}"))?;

        if file.name() == "word/document.xml" {
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| format!("Failed to read document.xml: {e}"))?;

            let paragraphs = extract_paragraphs(&content);
            let text: String = paragraphs.iter().map(|p| p.as_str()).collect::<Vec<_>>().join("\n");
            return Ok(text);
        }
    }

    Err("word/document.xml not found in DOCX archive".to_string())
}

fn extract_paragraphs(xml: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut pos = 0;

    while let Some(p_start) = xml[pos..].find("<w:p") {
        let abs_start = pos + p_start;

        if let Some(p_end_rel) = xml[abs_start..].find("</w:p>") {
            let abs_end = abs_start + p_end_rel + 6;
            let para_content = &xml[abs_start..abs_end];

            let text = extract_text_from_paragraph(para_content);
            if !text.trim().is_empty() {
                paragraphs.push(text);
            }
            pos = abs_end;
        } else {
            break;
        }
    }

    paragraphs
}

fn extract_text_from_paragraph(para_xml: &str) -> String {
    let mut text = String::new();
    let mut pos = 0;

    while let Some(t_start) = para_xml[pos..].find("<w:t") {
        let abs_start = pos + t_start;

        if let Some(content_start_rel) = para_xml[abs_start..].find('>') {
            let abs_content_start = abs_start + content_start_rel + 1;

            if let Some(t_end_rel) = para_xml[abs_content_start..].find("</w:t>") {
                let content = &para_xml[abs_content_start..abs_content_start + t_end_rel];
                text.push_str(content);
                pos = abs_content_start + t_end_rel + 6;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    unescape_xml(&text)
}

pub fn extract_pptx_text_from_zip(bytes: &[u8]) -> Result<String, String> {
    use std::io::Read;
    use zip::ZipArchive;

    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader)
        .map_err(|e| format!("Failed to open PPTX as ZIP: {e}"))?;

    let mut all_texts = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry: {e}"))?;

        let name = file.name().to_string();
        if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(|e| format!("Failed to read {name}: {e}"))?;

            let texts = extract_slide_texts(&content);
            all_texts.extend(texts);
        }
    }

    if all_texts.is_empty() {
        return Err("No slide text found in PPTX archive".to_string());
    }

    Ok(all_texts.join("\n"))
}

fn extract_slide_texts(xml: &str) -> Vec<String> {
    let mut texts = Vec::new();
    let mut pos = 0;

    while let Some(p_start) = xml[pos..].find("<a:p") {
        let abs_start = pos + p_start;

        if let Some(p_end_rel) = xml[abs_start..].find("</a:p>") {
            let abs_end = abs_start + p_end_rel + 6;
            let para_content = &xml[abs_start..abs_end];

            let text = extract_slide_text_from_paragraph(para_content);
            if !text.trim().is_empty() {
                texts.push(text);
            }
            pos = abs_end;
        } else {
            break;
        }
    }

    texts
}

fn extract_slide_text_from_paragraph(para_xml: &str) -> String {
    let mut text = String::new();
    let mut pos = 0;

    while let Some(t_start) = para_xml[pos..].find("<a:t") {
        let abs_start = pos + t_start;

        if let Some(tag_end_rel) = para_xml[abs_start..].find('>') {
            let abs_content_start = abs_start + tag_end_rel + 1;

            if let Some(t_end_rel) = para_xml[abs_content_start..].find("</a:t>") {
                let content = &para_xml[abs_content_start..abs_content_start + t_end_rel];
                text.push_str(content);
                pos = abs_content_start + t_end_rel + 6;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    unescape_xml(&text)
}

fn unescape_xml(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}
