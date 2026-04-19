use std::io::Cursor;

pub struct OoxmlDocument {
    pub original_bytes: Vec<u8>,
    pub paragraphs: Vec<ParagraphInfo>,
}

pub struct ParagraphInfo {
    pub text: String,
    pub index: usize,
}

pub fn load_docx_preserving(bytes: &[u8]) -> Result<OoxmlDocument, String> {
    use ooxmlsdk::parts::wordprocessing_document::WordprocessingDocument;

    let reader = Cursor::new(bytes);
    let docx = WordprocessingDocument::new(reader)
        .map_err(|e| format!("Failed to parse DOCX: {e}"))?;

    let xml_str = docx
        .main_document_part
        .root_element
        .to_xml()
        .unwrap_or_default();

    let paragraphs = extract_paragraphs(&xml_str);

    Ok(OoxmlDocument {
        original_bytes: bytes.to_vec(),
        paragraphs,
    })
}

fn extract_paragraphs(xml: &str) -> Vec<ParagraphInfo> {
    let mut paragraphs = Vec::new();
    let mut para_index = 0;

    let mut pos = 0;
    while let Some(p_start) = xml[pos..].find("<w:p") {
        let abs_start = pos + p_start;

        if let Some(p_end_rel) = xml[abs_start..].find("</w:p>") {
            let abs_end = abs_start + p_end_rel + 6;
            let para_content = &xml[abs_start..abs_end];

            let text = extract_text_from_paragraph(para_content);
            if !text.trim().is_empty() {
                paragraphs.push(ParagraphInfo {
                    text,
                    index: para_index,
                });
            }
            para_index += 1;
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

pub fn save_docx_preserving(original_bytes: &[u8]) -> Result<Vec<u8>, String> {
    use ooxmlsdk::parts::wordprocessing_document::WordprocessingDocument;

    let reader = Cursor::new(original_bytes);
    let docx = WordprocessingDocument::new(reader)
        .map_err(|e| format!("Failed to parse DOCX: {e}"))?;

    let mut output = Cursor::new(Vec::new());
    docx.save(&mut output)
        .map_err(|e| format!("Failed to save DOCX: {e}"))?;

    Ok(output.into_inner())
}

pub fn update_docx_text(
    original_bytes: &[u8],
    new_paragraphs: &[String],
) -> Result<Vec<u8>, String> {
    use std::io::{Read, Write};
    use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

    let reader = Cursor::new(original_bytes);
    let mut archive =
        ZipArchive::new(reader).map_err(|e| format!("Failed to open DOCX archive: {e}"))?;

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

            if name == "word/document.xml" {
                let mut content = String::new();
                file.read_to_string(&mut content)
                    .map_err(|e| format!("Failed to read document.xml: {e}"))?;

                let modified_content = replace_paragraph_texts(&content, new_paragraphs);

                zip_writer
                    .start_file(&name, options)
                    .map_err(|e| format!("Failed to start file in zip: {e}"))?;
                zip_writer
                    .write_all(modified_content.as_bytes())
                    .map_err(|e| format!("Failed to write document.xml: {e}"))?;
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

fn replace_paragraph_texts(xml: &str, new_paragraphs: &[String]) -> String {
    let mut result = xml.to_string();
    let mut para_idx = 0;
    let mut search_pos = 0;

    while let Some(p_start) = result[search_pos..]
        .find("<w:p ")
        .or_else(|| result[search_pos..].find("<w:p>"))
    {
        let abs_start = search_pos + p_start;

        if let Some(p_end_rel) = result[abs_start..].find("</w:p>") {
            let abs_end = abs_start + p_end_rel + 6;
            let para_content = result[abs_start..abs_end].to_string();

            if para_content.contains("<w:t") {
                if para_idx < new_paragraphs.len() {
                    let new_para = replace_first_text_run(&para_content, &new_paragraphs[para_idx]);
                    let new_len = new_para.len();
                    result = format!("{}{}{}", &result[..abs_start], new_para, &result[abs_end..]);
                    search_pos = abs_start + new_len;
                } else {
                    search_pos = abs_end;
                }
                para_idx += 1;
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
    while let Some(t_start) = result[search_pos..].find("<w:t") {
        let abs_start = search_pos + t_start;

        if let Some(tag_end_rel) = result[abs_start..].find('>') {
            let abs_content_start = abs_start + tag_end_rel + 1;

            if let Some(t_end_rel) = result[abs_content_start..].find("</w:t>") {
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
