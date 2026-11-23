//! Structure-aware chunking of parsed Office block structures into KB chunks.
//!
//! `botdocs`, `botslides` and `botsheet` each parse an Office file into an
//! ordered list of blocks (paragraphs, headings, slide text, speaker notes,
//! sheet cells). Flattening those blocks into one string loses the addressing
//! that makes an enterprise answer citable: which clause, which slide, which
//! sheet/cell, which page, which version. This module turns the blocks
//! directly into KB chunks, keeping the active heading stack on every chunk
//! and attaching a [`SourceAnchor`] that renders as a citation.
//!
//! The module is pure: no I/O, no DB, no Qdrant, no clock. Callers map their
//! native parsed types into [`OfficeBlock`] / [`BlockLocator`], call
//! [`OfficeKbChunker::chunk`], and project the [`OfficeKbChunk`]s onto their
//! vector store. IDs are deterministic FNV-1a hashes of the document id,
//! version and structural address: re-chunking an unchanged document is
//! idempotent, while a version bump yields a fresh set of ids for reindexing.
//!
//! This complements the flat-text marker chunker in `botcore::kb::structured`:
//! that one recovers structure from `[[PAGE:n]]`-style markers after
//! extraction, whereas this one receives already-parsed block structures and
//! never throws the hierarchy away in the first place.

mod chunker;
mod types;

pub use chunker::{OfficeChunkOptions, OfficeKbChunker};
pub use types::{BlockLocator, OfficeBlock, OfficeDocument, OfficeKbChunk, SourceAnchor};
