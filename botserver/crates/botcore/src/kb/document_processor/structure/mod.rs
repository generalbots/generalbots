//! Structure-aware chunking of already-parsed document text.
//!
//! This module complements the flat-text extraction in the rest of
//! `document_processor`: callers pass an ordered stream of headings,
//! paragraphs and tables (see [`parse_structured_text`] for the line format),
//! and receive bounded chunks that keep the active heading breadcrumb on
//! every chunk, inherit page/clause locators, never split a table row, and
//! carry both a stable citation and the source ACL for permission-aware
//! retrieval.
//!
//! The pass is pure — no I/O, no clock, no database — so it can be unit tested
//! offline and used by Docs, Slides, Sheets and PDF ingestion alike.

mod chunker;
mod parser;
mod permissions;
mod types;

pub use chunker::StructuredChunker;
pub use parser::parse_structured_text;
pub use permissions::{AccessContext, can_read, filter_for_read};
pub use types::{
    ChunkOptions, ChunkPermissions, Element, Locator, StructuredChunk, StructuredDocument,
    Visibility,
};

#[cfg(test)]
mod tests;
