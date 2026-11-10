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
//! `Regenerate` so the caller rebuilds — carrying the original theme over.

use crate::ooxml::{load_pptx_preserving, update_pptx_text, OoxmlPresentation};
use crate::storage::{load_presentation_pptx, DriveOps};
use crate::types::{Presentation, Slide};

use super::merge::keep_original_theme;

/// Paragraphs of one slide as the editor model sees them: each text element
/// contributes one entry per `\n`-separated line, in document order. Blank
/// lines are formatting the model cannot represent, so they are dropped here —
/// the source scan drops them too, which is what keeps the two sides aligned.
pub fn slide_paragraphs(slide: &Slide) -> Vec<String> {
    slide
        .elements
        .iter()
        .filter_map(|element| element.content.text.as_deref())
        .flat_map(|text| text.split('\n'))
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

/// What the original package says, per slide, in presentation order.
pub struct SourceTexts {
    /// Paragraphs of each slide.
    pub paragraphs: Vec<Vec<String>>,
    /// Slide part names (`ppt/slides/slideN.xml`), same order — the address a
    /// patch must use, because file-name order is not presentation order.
    pub parts: Vec<String>,
}

impl SourceTexts {
    pub fn of(parsed: &OoxmlPresentation) -> Self {
        Self {
            paragraphs: parsed
                .slides
                .iter()
                .map(|slide| slide.texts.clone())
                .collect(),
            parts: parsed
                .slides
                .iter()
                .map(|slide| slide.part_name.clone())
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PreservedPptx {
    /// Nothing changed: the original package is returned byte for byte.
    Unchanged(Vec<u8>),
    /// Only text changed: the original package with patched slide XML.
    Patched(Vec<u8>),
    /// A structural edit (slides or paragraphs added/removed) cannot be
    /// represented by patching; the caller must rebuild the package.
    Regenerate,
}

/// Decides whether the model can be written back into the original package.
pub fn preserve_pptx_text(
    original: &[u8],
    source: &SourceTexts,
    model_texts: &[Vec<String>],
) -> Result<PreservedPptx, String> {
    let aligned = source.paragraphs.len() == model_texts.len()
        && source
            .paragraphs
            .iter()
            .zip(model_texts.iter())
            .all(|(source_slide, model_slide)| source_slide.len() == model_slide.len());

    if aligned {
        let changes: Vec<(String, Vec<String>)> = source
            .paragraphs
            .iter()
            .zip(model_texts.iter())
            .enumerate()
            .filter(|(_, (source_slide, model_slide))| source_slide != model_slide)
            .filter_map(|(index, (_, model_slide))| {
                source
                    .parts
                    .get(index)
                    .map(|part| (part.clone(), model_slide.clone()))
            })
            .collect();
        if changes.is_empty() {
            return Ok(PreservedPptx::Unchanged(original.to_vec()));
        }
        return Ok(PreservedPptx::Patched(update_pptx_text(original, &changes)?));
    }

    // The paragraph counts differ, so a positional patch would land in the
    // wrong shape. When the visible text is still the same the deck was not
    // edited (our scan simply counts differently — an empty placeholder, a
    // table cell, a line break), and the original file is the correct save.
    // Never rebuild a deck the user did not change.
    if same_visible_text(&source.paragraphs, model_texts) {
        return Ok(PreservedPptx::Unchanged(original.to_vec()));
    }

    Ok(PreservedPptx::Regenerate)
}

fn same_visible_text(source: &[Vec<String>], model: &[Vec<String>]) -> bool {
    if source.len() != model.len() {
        return false;
    }
    source
        .iter()
        .zip(model.iter())
        .all(|(source_slide, model_slide)| {
            flatten(source_slide) == flatten(model_slide)
        })
}

fn flatten(paragraphs: &[String]) -> String {
    paragraphs
        .iter()
        .map(|paragraph| paragraph.trim())
        .filter(|paragraph| !paragraph.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Export result with the fidelity that was used, so the API can report it.
pub struct ExportedPptx {
    pub bytes: Vec<u8>,
    /// `preserved` (original package, optionally text-patched), `regenerated`
    /// (rebuilt from the model) or `regenerated+theme` (rebuilt, but the
    /// author's theme was carried over).
    pub fidelity: &'static str,
}

/// Export a deck as PPTX, preferring the preserved original package.
pub async fn export_presentation_pptx<D: DriveOps>(
    drive: &D,
    user_id: &str,
    presentation: &Presentation,
) -> Result<ExportedPptx, String> {
    let original = load_presentation_pptx(drive, user_id, &presentation.id).await;

    if let Some(original) = original.as_deref() {
        match load_pptx_preserving(original) {
            Ok(parsed) => {
                let source = SourceTexts::of(&parsed);
                let model_texts: Vec<Vec<String>> = presentation
                    .slides
                    .iter()
                    .map(slide_paragraphs)
                    .collect();
                match preserve_pptx_text(original, &source, &model_texts) {
                    Ok(PreservedPptx::Unchanged(bytes))
                    | Ok(PreservedPptx::Patched(bytes)) => {
                        return Ok(ExportedPptx {
                            bytes,
                            fidelity: "preserved",
                        })
                    }
                    Ok(PreservedPptx::Regenerate) => {
                        log::info!(
                            "slides export: structural change in {} — rebuilding the package",
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

    let regenerated = crate::pptx::export_to_pptx(presentation)?;
    if let Some(original) = original.as_deref() {
        if let Some(merged) = keep_original_theme(&regenerated, original) {
            return Ok(ExportedPptx {
                bytes: merged,
                fidelity: "regenerated+theme",
            });
        }
    }
    Ok(ExportedPptx {
        bytes: regenerated,
        fidelity: "regenerated",
    })
}
