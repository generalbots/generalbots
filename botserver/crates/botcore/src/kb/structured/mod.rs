//! Structure-aware document knowledge indexing and permission-aware retrieval
//! for Office documents (issue #2499).
//!
//! This module is deliberately self-contained and free of Qdrant/HTTP/DB calls
//! so that the indexing semantics (stable IDs, page/clause/section/slide/cell
//! addressing, ACL filtering, tenant isolation and backfill accounting) can be
//! verified offline with inline unit tests. The runtime indexer can project the
//! produced [`StructuredChunk`] values onto vector points by passing
//! [`Citation`] and ACL fields into the point payload.

mod citation;
mod chunk;
mod chunker;
mod index;
mod index_types;

pub use citation::{Citation, SourceKind, SourceRef};
pub use chunk::{stable_id, AclMetadata, StructuredChunk};
pub use chunker::{ChunkOptions, StructureAwareChunker};
pub use index::{retrieve_with_citations, StructuredKbIndex};
pub use index_types::{
    BackfillEntry, BackfillReport, DocumentRecord, IndexDocument, IndexStatus, RetrievedChunk,
};
