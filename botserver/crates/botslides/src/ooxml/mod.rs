//! Package-level access to a `.pptx` for the preserving save path (#2041).
//!
//! Import builds the canvas model from the uploaded file, but the file itself
//! is what a save writes back: text edits are patched into the original parts,
//! everything else stays as the author saved it. This module reads the
//! paragraphs of each slide (in presentation order) and writes edited text back
//! into the package without re-authoring it.

pub mod patch;
pub mod scan;

pub use patch::{rewrite_package, update_pptx_text};
pub use scan::{paragraph_texts, shape_blocks};

use std::io::{Cursor, Read};

use crate::pptx::order::slide_part_names;
use zip::ZipArchive;

pub struct OoxmlPresentation {
    pub original_bytes: Vec<u8>,
    pub slides: Vec<SlideInfo>,
}

pub struct SlideInfo {
    pub index: usize,
    /// Part name inside the package (`ppt/slides/slide7.xml`), in presentation
    /// order — the address a patch must use.
    pub part_name: String,
    pub texts: Vec<String>,
}

/// Reads the original package and the visible paragraph text of every slide.
pub fn load_pptx_preserving(bytes: &[u8]) -> Result<OoxmlPresentation, String> {
    let parts = slide_part_names(bytes)?;
    let mut archive =
        ZipArchive::new(Cursor::new(bytes)).map_err(|e| format!("open pptx: {e}"))?;

    let mut slides = Vec::new();
    for (index, part_name) in parts.iter().enumerate() {
        let mut file = archive
            .by_name(part_name)
            .map_err(|e| format!("open {part_name}: {e}"))?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)
            .map_err(|e| format!("read {part_name}: {e}"))?;
        slides.push(SlideInfo {
            index,
            part_name: part_name.clone(),
            texts: paragraph_texts(&String::from_utf8_lossy(&content)),
        });
    }

    Ok(OoxmlPresentation {
        original_bytes: bytes.to_vec(),
        slides,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn package(entries: &[(&str, &str)]) -> Vec<u8> {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            for (name, content) in entries {
                zip.start_file(*name, options).expect("entry");
                zip.write_all(content.as_bytes()).expect("write");
            }
            zip.finish().expect("finish");
        }
        buf.into_inner()
    }

    #[test]
    fn slides_are_read_in_presentation_order() {
        let bytes = package(&[
            (
                "ppt/presentation.xml",
                "<p:presentation><p:sldIdLst><p:sldId r:id=\"rId2\"/>\
                 <p:sldId r:id=\"rId1\"/></p:sldIdLst></p:presentation>",
            ),
            (
                "ppt/_rels/presentation.xml.rels",
                "<Relationships><Relationship Id=\"rId1\" Target=\"slides/slide1.xml\"/>\
                 <Relationship Id=\"rId2\" Target=\"slides/slide2.xml\"/></Relationships>",
            ),
            (
                "ppt/slides/slide1.xml",
                "<p:sld><p:sp><p:txBody><a:p><a:r><a:t>First file</a:t></a:r></a:p></p:txBody></p:sp></p:sld>",
            ),
            (
                "ppt/slides/slide2.xml",
                "<p:sld><p:sp><p:txBody><a:p><a:r><a:t>Second file</a:t></a:r></a:p></p:txBody></p:sp></p:sld>",
            ),
        ]);

        let parsed = load_pptx_preserving(&bytes).expect("parse");
        assert_eq!(parsed.slides.len(), 2);
        assert_eq!(parsed.slides[0].part_name, "ppt/slides/slide2.xml");
        assert_eq!(parsed.slides[0].texts, vec!["Second file".to_string()]);
        assert_eq!(parsed.slides[1].texts, vec!["First file".to_string()]);
    }
}
