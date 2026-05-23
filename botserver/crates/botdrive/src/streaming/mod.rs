//! Streaming processors for large file formats (Issue #493).
//! Each sub-module processes a specific format incrementally
//! to keep peak memory usage under control.

pub mod csv;
pub mod docx;
pub mod pdf;
pub mod xlsx;
