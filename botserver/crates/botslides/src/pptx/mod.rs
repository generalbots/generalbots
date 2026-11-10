//! High-fidelity PPTX export/import for Slides.
//!
//! Export builds a valid OOXML presentation (16:9) from the in-memory
//! `Presentation` model: text boxes, shapes, images (data-URI sources) and
//! speaker notes round-trip. Import extracts text, shapes and images back into
//! the same model. Both sides are pure zip + XML, so the crate needs no OOXML
//! SDK to read or write a deck.
//!
//! A deck imported from an uploaded `.pptx` is saved back into that same
//! package (see `preserve`): text edits patch the original slide XML, so theme,
//! masters, layouts, notes, charts and media survive a save (#2041).
mod export;
mod import;
mod merge;
pub(crate) mod order;
mod preserve;

pub use export::export_to_pptx;
pub use import::load_pptx;
pub use preserve::{
    export_presentation_pptx, preserve_pptx_text, slide_paragraphs, ExportedPptx,
    PreservedPptx, SourceTexts,
};
