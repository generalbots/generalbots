use std::io::Cursor;

pub struct OoxmlPresentation {
    pub original_bytes: Vec<u8>,
    pub slides: Vec<SlideInfo>,
}

pub struct SlideInfo {
    pub index: usize,
    pub texts: Vec<String>,
}

pub fn load_pptx_preserving(bytes: &[u8]) -> Result<OoxmlPresentation, String> {
    use ooxmlsdk::parts::presentation_document::PresentationDocument;

    let reader = Cursor::new(bytes);
    let pptx = PresentationDocument::new(reader)
        .map_err(|e| format!("Failed to parse PPTX: {e}"))?;

    let mut slides = Vec::new();

    for (idx, slide_part) in pptx.presentation_part.slide_parts.iter().enumerate() {
        let xml_str = slide_part.root_element.to_xml().unwrap_or_default();

        let texts = extract_texts_from_slide(&xml_str);
        slides.push(SlideInfo { index: idx, texts });
    }

    Ok(OoxmlPresentation {
        original_bytes: bytes.to_vec(),
        slides,
    })
}

fn extract_texts_from_slide(xml: &str) -> Vec<String> {
    let mut texts = Vec::new();
    let mut pos = 0;

    while let Some(p_start) = xml[pos..].find("<a:p") {
        let abs_start = pos + p_start;

        if let Some(p_end_rel) = xml[abs_start..].find("</a:p>") {
            let abs_end = abs_start + p_end_rel + 6;
            let para_content = &xml[abs_start..abs_end];

            let text = extract_text_from_paragraph(para_content);
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

fn extract_text_from_paragraph(para_xml: &str) -> String {
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

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn save_pptx_preserving(original_bytes: &[u8]) -> Result<Vec<u8>, String> {
    use ooxmlsdk::parts::presentation_document::PresentationDocument;

    let reader = Cursor::new(original_bytes);
    let pptx = PresentationDocument::new(reader)
        .map_err(|e| format!("Failed to parse PPTX: {e}"))?;

    let mut output = Cursor::new(Vec::new());
    pptx.save(&mut output)
        .map_err(|e| format!("Failed to save PPTX: {e}"))?;

    Ok(output.into_inner())
}

pub fn update_pptx_text(
    original_bytes: &[u8],
    new_slide_texts: &[Vec<String>],
) -> Result<Vec<u8>, String> {
    use std::io::{Read, Write};
    use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

    let reader = Cursor::new(original_bytes);
    let mut archive =
        ZipArchive::new(reader).map_err(|e| format!("Failed to open PPTX archive: {e}"))?;

    let mut output_buf = Cursor::new(Vec::new());
    {
        let mut zip_writer = ZipWriter::new(&mut output_buf);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read archive entry: {e}"))?;

            let name = file.name().to_string();

            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                let slide_num = extract_slide_number(&name);

                let mut content = String::new();
                file.read_to_string(&mut content)
                    .map_err(|e| format!("Failed to read slide xml: {e}"))?;

                let modified_content = if slide_num > 0 && slide_num <= new_slide_texts.len() {
                    replace_slide_texts(&content, &new_slide_texts[slide_num - 1])
                } else {
                    content
                };

                zip_writer
                    .start_file(&name, options)
                    .map_err(|e| format!("Failed to start file in zip: {e}"))?;
                zip_writer
                    .write_all(modified_content.as_bytes())
                    .map_err(|e| format!("Failed to write slide xml: {e}"))?;
            } else {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)
                    .map_err(|e| format!("Failed to read file: {e}"))?;

                zip_writer
                    .start_file(&name, options)
                    .map_err(|e| format!("Failed to start file in zip: {e}"))?;
                zip_writer
                    .write_all(&buf)
                    .map_err(|e| format!("Failed to write file: {e}"))?;
            }
        }

        zip_writer
            .finish()
            .map_err(|e| format!("Failed to finish zip: {e}"))?;
    }

    Ok(output_buf.into_inner())
}

fn extract_slide_number(filename: &str) -> usize {
    let name = filename
        .trim_start_matches("ppt/slides/slide")
        .trim_end_matches(".xml");
    name.parse().unwrap_or(0)
}

fn replace_slide_texts(xml: &str, new_texts: &[String]) -> String {
    let mut result = xml.to_string();
    let mut text_idx = 0;
    let mut search_pos = 0;

    while let Some(p_start) = result[search_pos..]
        .find("<a:p>")
        .or_else(|| result[search_pos..].find("<a:p "))
    {
        let abs_start = search_pos + p_start;

        if let Some(p_end_rel) = result[abs_start..].find("</a:p>") {
            let abs_end = abs_start + p_end_rel + 6;
            let para_content = result[abs_start..abs_end].to_string();

            if para_content.contains("<a:t") {
                if text_idx < new_texts.len() {
                    let new_para = replace_first_text_run(&para_content, &new_texts[text_idx]);
                    let new_len = new_para.len();
                    result = format!("{}{}{}", &result[..abs_start], new_para, &result[abs_end..]);
                    search_pos = abs_start + new_len;
                } else {
                    search_pos = abs_end;
                }
                text_idx += 1;
            } else {
                search_pos = abs_end;
            }
        } else {
            break;
        }
    }

    result
}

fn replace_first_text_run(para_xml: &str, new_text: &str) -> String {
    let mut result = para_xml.to_string();
    let mut found_first = false;

    let mut search_pos = 0;
    while let Some(t_start) = result[search_pos..].find("<a:t") {
        let abs_start = search_pos + t_start;

        if let Some(tag_end_rel) = result[abs_start..].find('>') {
            let abs_content_start = abs_start + tag_end_rel + 1;

            if let Some(t_end_rel) = result[abs_content_start..].find("</a:t>") {
                let abs_content_end = abs_content_start + t_end_rel;

                if !found_first {
                    let escaped = escape_xml(new_text);
                    result = format!(
                        "{}{}{}",
                        &result[..abs_content_start],
                        escaped,
                        &result[abs_content_end..]
                    );
                    found_first = true;
                    search_pos = abs_content_start + escaped.len() + 6;
                } else {
                    result = format!("{}{}", &result[..abs_content_start], &result[abs_content_end..]);
                    search_pos = abs_content_start;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    result
}
