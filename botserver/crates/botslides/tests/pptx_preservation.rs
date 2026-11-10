//! #2041 — saving a deck must not lose the parts the editor model does not
//! represent (theme, masters, layouts, notes, charts, media).
//!
//! These tests exercise the whole round-trip with no external services: the
//! package goes in, the model comes out of the importer, and the save path
//! writes the model back into the package.

use std::io::{Cursor, Read, Write};

use botslides::ooxml::load_pptx_preserving;
use botslides::pptx::{
    load_pptx, preserve_pptx_text, slide_paragraphs, PreservedPptx, SourceTexts,
};

fn fixture() -> Option<Vec<u8>> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../../bottest/tests/fixtures/office-fidelity/Rice_and_Beans_Nutrition.pptx",
    );
    std::fs::read(path).ok()
}

fn parts(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes.to_vec())).expect("zip");
    let mut out = Vec::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).expect("entry");
        let name = file.name().map(|name| name.to_string()).unwrap_or_default();
        let mut content = Vec::new();
        file.read_to_end(&mut content).expect("read");
        out.push((name, content));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn part(bytes: &[u8], name: &str) -> String {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes.to_vec())).expect("zip");
    let mut content = String::new();
    archive
        .by_name(name)
        .expect("part")
        .read_to_string(&mut content)
        .expect("read");
    content
}

/// Minimal OPC-ish zip: the preserving path only needs slide parts, so a
/// single part is enough to exercise the decision logic and the patch.
fn sample_pptx(paragraph: &str) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buf);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip.start_file("ppt/slides/slide1.xml", options)
            .expect("entry");
        let xml = format!(
            "<p:sld><p:cSld><p:spTree><p:sp><p:txBody><a:p><a:r><a:t>{paragraph}</a:t></a:r></a:p></p:txBody></p:sp></p:spTree></p:cSld></p:sld>"
        );
        zip.write_all(xml.as_bytes()).expect("write");
        zip.finish().expect("finish");
    }
    buf.into_inner()
}

fn source_of(original: &[u8]) -> SourceTexts {
    SourceTexts::of(&load_pptx_preserving(original).expect("parse"))
}

#[test]
fn unchanged_text_returns_the_original_bytes() {
    let original = sample_pptx("Rice and Beans");
    let source = source_of(&original);
    let model = source.paragraphs.clone();
    match preserve_pptx_text(&original, &source, &model).expect("preserve") {
        PreservedPptx::Unchanged(bytes) => assert_eq!(bytes, original),
        other => panic!("expected unchanged, got {other:?}"),
    }
}

#[test]
fn edited_text_patches_only_the_slide_part() {
    let original = sample_pptx("Beans");
    let source = source_of(&original);
    let model = vec![vec!["Beans and rice".to_string()]];
    match preserve_pptx_text(&original, &source, &model).expect("preserve") {
        PreservedPptx::Patched(bytes) => {
            assert!(
                part(&bytes, "ppt/slides/slide1.xml").contains("Beans and rice"),
                "the edit must reach the slide XML"
            );
        }
        other => panic!("expected patched, got {other:?}"),
    }
}

#[test]
fn structural_changes_ask_for_regeneration() {
    let original = sample_pptx("One");
    let source = source_of(&original);
    // A second paragraph cannot be patched positionally.
    let model = vec![vec!["One".to_string(), "Two".to_string()]];
    assert_eq!(
        preserve_pptx_text(&original, &source, &model).expect("preserve"),
        PreservedPptx::Regenerate
    );
}

#[test]
fn unaligned_but_identical_text_keeps_the_original() {
    let original = sample_pptx("One");
    let source = source_of(&original);
    // The model reports the same visible text split differently (a blank line,
    // a wrapped run): nothing was edited, so the file must not be rebuilt.
    let model = vec![vec!["One".to_string(), String::new()]];
    match preserve_pptx_text(&original, &source, &model).expect("preserve") {
        PreservedPptx::Unchanged(bytes) => assert_eq!(bytes, original),
        other => panic!("expected the original back, got {other:?}"),
    }
}

/// The attached real deck (3.1 MB, charts, media, notes) must round-trip
/// untouched and change only the edited slide when text moves.
#[test]
fn real_deck_round_trips() {
    let Some(original) = fixture() else {
        return; // fixture is optional in a trimmed checkout
    };
    let parsed = load_pptx_preserving(&original).expect("fixture must parse");
    let source = SourceTexts::of(&parsed);

    match preserve_pptx_text(&original, &source, &source.paragraphs).expect("untouched") {
        PreservedPptx::Unchanged(bytes) => assert_eq!(bytes, original),
        other => panic!("untouched save must return the source, got {other:?}"),
    }

    let mut model = source.paragraphs.clone();
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

/// The canvas model comes from the importer, not from the preserving scan. The
/// two must agree on slide order and paragraph counts, otherwise the first save
/// of an untouched deck rebuilds it and the fidelity is lost again.
#[test]
fn the_imported_model_saves_without_a_rebuild() {
    let Some(original) = fixture() else {
        return;
    };
    let parsed = load_pptx_preserving(&original).expect("fixture must parse");
    let source = SourceTexts::of(&parsed);
    let imported = load_pptx(&original).expect("import must parse");
    let model: Vec<Vec<String>> = imported.slides.iter().map(slide_paragraphs).collect();
    assert_eq!(
        source.paragraphs.len(),
        model.len(),
        "the model and the package must hold the same slides"
    );
    let counts: Vec<String> = source
        .paragraphs
        .iter()
        .zip(model.iter())
        .enumerate()
        .filter(|(_, (source_slide, model_slide))| source_slide != model_slide)
        .map(|(index, (source_slide, model_slide))| {
            format!(
                "slide {index}: source {source_slide:?} model {model_slide:?}"
            )
        })
        .collect();
    match preserve_pptx_text(&original, &source, &model).expect("untouched") {
        PreservedPptx::Unchanged(bytes) => assert_eq!(bytes, original),
        other => panic!("an untouched import must save the source, got {other:?}\n{counts:#?}"),
    }
}

/// Editing one text run of an imported deck patches that slide and leaves the
/// rest of the package (theme, masters, notes, media) intact.
#[test]
fn an_edit_of_an_imported_deck_patches_one_slide() {
    let Some(original) = fixture() else {
        return;
    };
    let parsed = load_pptx_preserving(&original).expect("fixture must parse");
    let source = SourceTexts::of(&parsed);
    let imported = load_pptx(&original).expect("import must parse");
    let mut model: Vec<Vec<String>> = imported.slides.iter().map(slide_paragraphs).collect();

    let Some((slide_index, edited)) = model
        .iter()
        .position(|slide| slide.len() > 1)
        .and_then(|slide_index| {
            model[slide_index].last_mut().map(|paragraph| {
                *paragraph = format!("{paragraph} (edited)");
                (slide_index, paragraph.clone())
            })
        })
    else {
        return;
    };

    let patched = match preserve_pptx_text(&original, &source, &model).expect("patched") {
        PreservedPptx::Patched(bytes) => bytes,
        other => panic!("an edited import must patch, got {other:?}"),
    };

    let before = parts(&original);
    let after = parts(&patched);
    let changed: Vec<&str> = before
        .iter()
        .zip(after.iter())
        .filter(|((_, a), (_, b))| a != b)
        .map(|((name, _), _)| name.as_str())
        .collect();
    assert_eq!(
        changed,
        vec![source.parts[slide_index].as_str()],
        "only the edited slide part may change"
    );
    assert!(
        part(&patched, &source.parts[slide_index]).contains(&edited),
        "the edit must reach the slide XML"
    );
}
