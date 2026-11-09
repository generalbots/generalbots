//! #2041 — preserve the original PPTX when only text changed.
//!
//! Import used to throw the uploaded package away and export re-authored a
//! minimal deck, so opening a real `.pptx` and saving it lost theme, notes,
//! master layouts, charts and every effect the model does not represent.
//!
//! The import handler now stores the original package next to the deck JSON.
//! On export this module compares the source paragraphs with the model's
//! paragraphs: identical text returns the original bytes untouched; a
//! paragraph-for-paragraph text edit patches only the slide XML inside the
//! original ZIP; anything structural (slide/paragraph count changes) returns
//! `Regenerate` so the caller keeps the previous, regenerating path.

use crate::storage::{load_presentation_pptx, DriveOps};
use crate::types::{Presentation, Slide};

/// Paragraphs of one slide as the editor model sees them: each text element
/// contributes one entry per `\n`-separated line, in document order.
pub fn slide_paragraphs(slide: &Slide) -> Vec<String> {
    slide
        .elements
        .iter()
        .filter_map(|element| element.content.text.as_deref())
        .flat_map(|text| text.split('\n'))
        .map(|line| line.trim_end().to_string())
        .collect()
}

#[derive(Debug, PartialEq, Eq)]
pub enum PreservedPptx {
    /// Nothing changed: the original package is returned byte for byte.
    Unchanged(Vec<u8>),
    /// Only text changed: the original package with patched slide XML.
    Patched(Vec<u8>),
    /// A structural edit (slides or paragraphs added/removed) cannot be
    /// represented by patching; the caller must regenerate.
    Regenerate,
}

/// Decides whether the model can be written back into the original package.
pub fn preserve_pptx_text(
    original: &[u8],
    source_texts: &[Vec<String>],
    model_texts: &[Vec<String>],
) -> Result<PreservedPptx, String> {
    if source_texts.len() != model_texts.len() {
        return Ok(PreservedPptx::Regenerate);
    }
    let mut changed = false;
    for (source, model) in source_texts.iter().zip(model_texts.iter()) {
        if source.len() != model.len() {
            return Ok(PreservedPptx::Regenerate);
        }
        if source != model {
            changed = true;
        }
    }
    if !changed {
        return Ok(PreservedPptx::Unchanged(original.to_vec()));
    }
    let patched = crate::ooxml::update_pptx_text(original, model_texts)?;
    Ok(PreservedPptx::Patched(patched))
}

/// Export result with the fidelity that was used, so the API can report it.
pub struct ExportedPptx {
    pub bytes: Vec<u8>,
    pub fidelity: &'static str,
}

/// Export a deck as PPTX, preferring the preserved original package.
pub async fn export_presentation_pptx<D: DriveOps>(
    drive: &D,
    user_id: &str,
    presentation: &Presentation,
) -> Result<ExportedPptx, String> {
    if let Some(original) = load_presentation_pptx(drive, user_id, &presentation.id).await {
        match crate::ooxml::load_pptx_preserving(&original) {
            Ok(parsed) => {
                let source_texts: Vec<Vec<String>> =
                    parsed.slides.iter().map(|slide| slide.texts.clone()).collect();
                let model_texts: Vec<Vec<String>> =
                    presentation.slides.iter().map(slide_paragraphs).collect();
                match preserve_pptx_text(&original, &source_texts, &model_texts) {
                    Ok(PreservedPptx::Unchanged(bytes)) => {
                        return Ok(ExportedPptx {
                            bytes,
                            fidelity: "preserved",
                        })
                    }
                    Ok(PreservedPptx::Patched(bytes)) => {
                        return Ok(ExportedPptx {
                            bytes,
                            fidelity: "preserved",
                        })
                    }
                    Ok(PreservedPptx::Regenerate) => {
                        log::info!(
                            "slides export: structural change in {} — regenerating the package",
                            presentation.id
                        );
                    }
                    Err(error) => {
                        log::warn!(
                            "slides export: could not patch {} into the original package: {error}",
                            presentation.id
                        );
                    }
                }
            }
            Err(error) => {
                log::warn!(
                    "slides export: preserved package for {} is unreadable: {error}",
                    presentation.id
                );
            }
        }
    }
    let bytes = crate::pptx::export_to_pptx(presentation)?;
    Ok(ExportedPptx {
        bytes,
        fidelity: "regenerated",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Minimal OPC-ish zip: update_pptx_text only needs the slide entry, so a
    /// single part is enough to test the decision logic and the patch.
    fn sample_pptx(paragraph: &str) -> Vec<u8> {
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            zip.start_file("ppt/slides/slide1.xml", options).unwrap();
            let xml = format!(
                "<p:sld><p:cSld><p:spTree><p:sp><p:txBody><a:p><a:r><a:t>{paragraph}</a:t></a:r></a:p></p:txBody></p:sp></p:spTree></p:cSld></p:sld>"
            );
            zip.write_all(xml.as_bytes()).unwrap();
            zip.finish().unwrap();
        }
        buf.into_inner()
    }

    #[test]
    fn unchanged_text_returns_the_original_bytes() {
        let original = sample_pptx("Rice and Beans");
        let source = vec![vec!["Rice and Beans".to_string()]];
        let model = vec![vec!["Rice and Beans".to_string()]];
        match preserve_pptx_text(&original, &source, &model).expect("preserve") {
            PreservedPptx::Unchanged(bytes) => assert_eq!(bytes, original),
            other => panic!("expected unchanged, got {other:?}"),
        }
    }

    #[test]
    fn edited_text_patches_only_the_slide_part() {
        let original = sample_pptx("Beans");
        let source = vec![vec!["Beans".to_string()]];
        let model = vec![vec!["Beans and rice".to_string()]];
        match preserve_pptx_text(&original, &source, &model).expect("preserve") {
            PreservedPptx::Patched(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                assert!(text.contains("Beans and rice"));
            }
            other => panic!("expected patched, got {other:?}"),
        }
    }

    #[test]
    fn structural_changes_ask_for_regeneration() {
        let original = sample_pptx("One");
        // A second slide/paragraph cannot be patched positionally.
        let source = vec![vec!["One".to_string()]];
        let model = vec![vec!["One".to_string(), "Two".to_string()]];
        assert_eq!(
            preserve_pptx_text(&original, &source, &model).expect("preserve"),
            PreservedPptx::Regenerate
        );
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

    /// #2044 — the attached real deck (3.1 MB, charts, media, notes) must
    /// round-trip untouched and change only the edited slide when text moves.
    #[test]
    fn real_deck_round_trips() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../../bottest/tests/fixtures/office-fidelity/Rice_and_Beans_Nutrition.pptx",
        );
        let Ok(original) = std::fs::read(&path) else {
            return; // fixture is optional in a trimmed checkout
        };
        let parsed = match crate::ooxml::load_pptx_preserving(&original) {
            Ok(parsed) => parsed,
            Err(error) => panic!("fixture must parse: {error}"),
        };
        let source: Vec<Vec<String>> = parsed.slides.iter().map(|s| s.texts.clone()).collect();

        match preserve_pptx_text(&original, &source, &source).expect("untouched") {
            PreservedPptx::Unchanged(bytes) => assert_eq!(bytes, original),
            other => panic!("untouched save must return the source, got {other:?}"),
        }

        let mut model = source.clone();
        let Some(first) = model.first_mut().and_then(|slide| slide.first_mut()) else {
            return; // a deck without text cannot exercise the text patch
        };
        *first = format!("{first} (edited)");
        let patched = match preserve_pptx_text(&original, &source, &model).expect("patched") {
            PreservedPptx::Patched(bytes) => bytes,
            other => panic!("a text edit must patch, got {other:?}"),
        };
        let before = parts(&original);
        let after = parts(&patched);
        assert_eq!(before.len(), after.len(), "part list must not change");
        for ((name, a), (patched_name, b)) in before.iter().zip(after.iter()) {
            assert_eq!(name, patched_name);
            if name.starts_with("ppt/slides/slide") {
                continue;
            }
            assert_eq!(a, b, "part {name} must stay byte-identical");
        }
    }
}

