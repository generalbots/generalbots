//! High-fidelity PPTX export/import for Slides.
//!
//! Export builds a valid OOXML presentation (16:9) from the in-memory
//! `Presentation` model: text boxes, shapes, images (data-URI sources) and
//! speaker notes round-trip. Import extracts text, shapes and images back into
//! the same model. Both sides are pure zip + XML so they work without the
//! heavier `ooxmlsdk` path used elsewhere.
mod export;
mod import;

pub use export::export_to_pptx;
pub use import::load_pptx;
