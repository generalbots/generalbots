//! Backwards-compatible entry point for PDF export.
//!
//! The renderer moved to [`crate::pdf`] so structure recovery, pagination and
//! the PDF writer can be tested independently. This facade keeps the existing
//! `crate::utils_pdf::html_to_pdf` call site (and any external consumer)
//! working unchanged.

pub use crate::pdf::html_to_pdf;
