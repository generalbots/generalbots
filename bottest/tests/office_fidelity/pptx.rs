//! PPTX half of the fidelity contract (#2041/#2044): the attached deck carries
//! theme, masters, layouts, notes, charts and media the canvas model cannot
//! re-emit, so an untouched save must return it verbatim and an edit must touch
//! exactly one slide part.

use botslides::ooxml::{load_pptx_preserving, paragraph_texts};
use botslides::pptx::{
    load_pptx, preserve_pptx_text, slide_paragraphs, PreservedPptx, SourceTexts,
};

use crate::{changed_parts, fixture, part, part_hashes, PPTX_FIXTURE};

/// Rule 1+2: importing the real deck and saving it untouched returns the source
/// package byte for byte, so every part hash matches the original.
#[test]
fn pptx_untouched_save_returns_the_source_package() {
    let Some(original) = fixture(PPTX_FIXTURE) else {
        return; // fixture is optional in a trimmed checkout
    };
    let source = SourceTexts::of(&load_pptx_preserving(&original).expect("parse the deck"));
    let imported = load_pptx(&original).expect("import the deck");
    let model: Vec<Vec<String>> = imported.slides.iter().map(slide_paragraphs).collect();

    match preserve_pptx_text(&original, &source, &model).expect("preserve") {
        PreservedPptx::Unchanged(bytes) => {
            assert_eq!(
                bytes, original,
                "an untouched save must not rewrite the deck"
            );
            assert_eq!(
                part_hashes(&bytes),
                part_hashes(&original),
                "every part must hash identically"
            );
        }
        other => panic!("an untouched import must save the source, got {other:?}"),
    }
}

/// Rule 2: editing one text run rewrites only the slide part that owns it.
#[test]
fn pptx_single_text_edit_changes_only_the_edited_slide() {
    let Some(original) = fixture(PPTX_FIXTURE) else {
        return;
    };
    let source = SourceTexts::of(&load_pptx_preserving(&original).expect("parse the deck"));
    let imported = load_pptx(&original).expect("import the deck");
    let mut model: Vec<Vec<String>> = imported.slides.iter().map(slide_paragraphs).collect();

    let Some((index, edited)) = model
        .iter()
        .position(|slide| !slide.is_empty())
        .and_then(|index| {
            model[index].first_mut().map(|paragraph| {
                *paragraph = format!("{paragraph} (edited)");
                (index, paragraph.clone())
            })
        })
    else {
        return; // a deck without text cannot exercise the text patch
    };

    let patched = match preserve_pptx_text(&original, &source, &model).expect("patch") {
        PreservedPptx::Patched(bytes) => bytes,
        other => panic!("a text edit must patch the source, got {other:?}"),
    };

    assert_eq!(
        changed_parts(&original, &patched),
        vec![source.parts[index].clone()],
        "only the edited slide part may change"
    );
    // A paragraph can be split across several runs, so the patch is verified on
    // the scanned paragraph text rather than on a raw substring of the XML.
    let texts = paragraph_texts(&part(&patched, &source.parts[index]));
    assert!(
        texts.contains(&edited),
        "the edit must reach the slide XML, got {texts:?}"
    );
}

/// Rule 4: a structural edit asks for regeneration explicitly, and a corrupt
/// package is an error — neither case rebuilds silently.
#[test]
fn pptx_structural_and_corrupt_sources_are_explicit() {
    let Some(original) = fixture(PPTX_FIXTURE) else {
        return;
    };
    let source = SourceTexts::of(&load_pptx_preserving(&original).expect("parse the deck"));
    let mut model = source.paragraphs.clone();
    model.push(vec!["a slide the package cannot hold".to_string()]);
    assert_eq!(
        preserve_pptx_text(&original, &source, &model).expect("decision"),
        PreservedPptx::Regenerate,
        "an unrepresentable edit must ask for an explicit rebuild"
    );

    assert!(
        load_pptx_preserving(b"not a zip package").is_err(),
        "a corrupt source must be reported, not replaced by a fresh deck"
    );
}
