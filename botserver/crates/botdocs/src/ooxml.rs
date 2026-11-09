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
    use ooxmlsdk::sdk::SdkType;

    let reader = Cursor::new(bytes);
    let docx = WordprocessingDocument::new(reader)
        .map_err(|e| format!("Failed to parse DOCX: {e}"))?;

    let xml_str = docx
        .main_document_part()
        .ok()
        .and_then(|part| part.root_element(&docx).ok().and_then(|root| root.to_xml().ok()))
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
                let content =
                    &para_xml[abs_content_start..abs_content_start + t_end_rel];
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

            let name = file.name().map(|n| n.to_string()).unwrap_or_default();

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
                    let new_para =
                        replace_first_text_run(&para_content, &new_paragraphs[para_idx]);
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
                        &result[..abs_content_start], escaped, &result[abs_content_end..]
                    );
                    found_first = true;
                    search_pos = abs_content_start + escaped.len() + 6;
                } else {
                    result = format!(
                        "{}{}",
                        &result[..abs_content_start], &result[abs_content_end..]
                    );
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

#[cfg(test)]
mod update_docx_tests {
    use super::*;

    /// #2042 — the patch path must keep every package part except the edited
    /// slide/document XML. A docx built by docx-rs is a valid OPC package, so
    /// it exercises the same zip walk a real Word file does.
    fn sample_docx(text: &str) -> Vec<u8> {
        use docx_rs::{Docx, Paragraph, Run};
        let docx = Docx::new().add_paragraph(
            Paragraph::new().add_run(Run::new().add_text(text)),
        );
        docx.build().expect("build docx")
    }

    fn part_hashes(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
        use std::io::Read;
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip");
        let mut parts = Vec::new();
        for index in 0..archive.len() {
            let mut file = archive.by_index(index).expect("entry");
            let name = file.name().map(str::to_string).unwrap_or_default();
            let mut content = Vec::new();
            file.read_to_end(&mut content).expect("read");
            parts.push((name, content));
        }
        parts.sort_by(|a, b| a.0.cmp(&b.0));
        parts
    }

    #[test]
    fn text_edit_changes_only_the_document_part() {
        let original = sample_docx("Old paragraph");
        let patched = update_docx_text(&original, &["New paragraph".to_string()]).expect("patch");

        let original_parts = part_hashes(&original);
        let patched_parts = part_hashes(&patched);
        assert_eq!(
            original_parts.len(),
            patched_parts.len(),
            "the package part list must not change"
        );

        for ((name, before), (patched_name, after)) in
            original_parts.iter().zip(patched_parts.iter())
        {
            assert_eq!(name, patched_name, "part order must be stable");
            if name == "word/document.xml" {
                assert_ne!(before, after, "the edited part must change");
                let xml = String::from_utf8_lossy(after);
                assert!(xml.contains("New paragraph"));
            } else {
                assert_eq!(
                    before, after,
                    "part {name} must be byte-identical after a text edit"
                );
            }
        }
    }

    #[test]
    fn unmatched_paragraph_count_still_returns_a_package() {
        let original = sample_docx("Only paragraph");
        // Fewer replacement paragraphs than source ones: the patcher keeps the
        // remaining source text instead of dropping content.
        let patched = update_docx_text(&original, &[]).expect("patch");
        assert!(!patched.is_empty());
    }
}

#[cfg(test)]
mod fixture_round_trip {
    use super::*;
    use std::io::Read;

    fn parts(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
        let mut archive =
            zip::ZipArchive::new(std::io::Cursor::new(bytes.to_vec())).expect("zip");
        let mut out = Vec::new();
        for index in 0..archive.len() {
            let mut file = archive.by_index(index).expect("entry");
            let name = file.name().map(str::to_string).unwrap_or_default();
            let mut content = Vec::new();
            file.read_to_end(&mut content).expect("read");
            out.push((name, content));
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    /// #2044 — the attached resume (styled runs, sections, headers/footers)
    /// keeps every part except word/document.xml when one paragraph is edited.
    #[test]
    fn real_docx_edit_changes_only_the_document_part() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../../bottest/tests/fixtures/office-fidelity/Alex_Rivera_Resume_Futuristic.docx",
        );
        let Ok(original) = std::fs::read(&path) else {
            return;
        };
        let parsed = load_docx_preserving(&original).expect("fixture parses");
        let mut paragraphs: Vec<String> = parsed.paragraphs.iter().map(|p| p.text.clone()).collect();
        let Some(first) = paragraphs.first_mut() else { return };
        *first = format!("{first} (edited)");

        let patched = update_docx_text(&original, &paragraphs).expect("patch");
        let before = parts(&original);
        let after = parts(&patched);
        assert_eq!(before.len(), after.len(), "part list must not change");
        for ((name, a), (patched_name, b)) in before.iter().zip(after.iter()) {
            assert_eq!(name, patched_name);
            if name == "word/document.xml" {
                assert_ne!(a, b, "the edited part must change");
            } else {
                assert_eq!(a, b, "part {name} must stay byte-identical");
            }
        }
    }
}

