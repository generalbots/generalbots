//! High-fidelity PPTX export/import for Slides.
//!
//! Export builds a valid OOXML presentation (16:9) from the in-memory
//! `Presentation` model: text boxes, shapes, images (data-URI sources),
//! speaker notes, native tables and native charts (#2457). Import extracts
//! text, shapes and images back into the same model. Both sides are pure zip +
//! XML, so the crate needs no OOXML SDK to read or write a deck.
//!
//! A deck imported from an uploaded `.pptx` is saved back into that same
//! package (see `preserve`): text edits patch the original slide XML, so theme,
//! masters, layouts, notes, charts and media survive a save (#2041).
mod chart;
mod export;
mod import;
mod merge;
pub(crate) mod order;
mod preserve;
mod static_parts;
mod table;

#[cfg(test)]
#[path = "export_tests.rs"]
mod export_tests;

pub use export::export_to_pptx;
pub use import::load_pptx;
pub use preserve::{
    export_presentation_pptx, preserve_pptx_text, slide_paragraphs, ExportedPptx,
    PreservedPptx, SourceTexts,
};
