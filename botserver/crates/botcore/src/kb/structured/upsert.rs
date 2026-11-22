//! Storage-agnostic projection of a [`StructuredChunk`] into a vector-store
//! upsert request (issue #2499).
//!
//! The structured index never talks to Qdrant (or any other vector store) and
//! never computes embeddings. It only produces a stable point id, a reserved
//! slot where the runtime will place the embedding, and a payload that carries
//! the citation, the ACL and the tenant alongside the text. The runtime
//! indexer maps [`UpsertPoint`] onto its client's request type and replaces the
//! pending [`VectorSlot`] with the real embedding.

use super::chunk::StructuredChunk;
use std::collections::HashMap;

/// Default embedding width used by callers that do not manage dimensions.
/// The runtime may pass any width to [`StructuredChunk::to_upsert_point`].
pub const DEFAULT_VECTOR_DIMENSION: usize = 1536;

/// Reserved embedding slot. `values` is a zero-filled placeholder until the
/// runtime embeds the chunk text; no vector-store type is involved.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorSlot {
    pub dimension: usize,
    pub values: Vec<f32>,
    pub ready: bool,
}

impl VectorSlot {
    /// Creates a pending slot of `dimension` zeroed floats.
    pub fn pending(dimension: usize) -> Self {
        Self {
            dimension,
            values: vec![0.0_f32; dimension],
            ready: false,
        }
    }

    /// Creates a ready slot from an already computed embedding.
    pub fn filled(values: Vec<f32>) -> Self {
        let dimension = values.len();
        Self {
            dimension,
            values,
            ready: true,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }
}

/// A single, storage-agnostic vector-upsert request.
///
/// This deliberately contains only primitives, a [`VectorSlot`] and a JSON
/// payload. Converting it to a concrete Qdrant (or other) request is the
/// runtime indexer's responsibility, not the structured KB's.
#[derive(Debug, Clone, PartialEq)]
pub struct UpsertPoint {
    /// Stable, deterministic point id shared with the document model.
    pub id: String,
    /// Reserved embedding slot (pending until the runtime fills it).
    pub vector: VectorSlot,
    /// Citation + ACL + tenant + text, produced by
    /// [`StructuredChunk::to_payload`].
    pub payload: HashMap<String, serde_json::Value>,
}

impl StructuredChunk {
    /// Stable point id for the vector store.
    ///
    /// Reuses the chunk's stable id, which is derived from tenant, document,
    /// version, logical address, heading path, chunk index and content. An
    /// unchanged reindex therefore yields the same id (idempotent upsert),
    /// while an edit yields a new one. Tenant is already part of that id, so
    /// equal text in two tenants can never collide.
    pub fn point_id(&self) -> String {
        self.id.clone()
    }

    /// Pure projection into a storage-agnostic upsert request.
    ///
    /// `dimension` reserves the embedding slot; the runtime later replaces it
    /// via [`VectorSlot::filled`]. The payload is the full projection returned
    /// by [`StructuredChunk::to_payload`] (citation, ACL, tenant and content).
    pub fn to_upsert_point(&self, dimension: usize) -> UpsertPoint {
        UpsertPoint {
            id: self.point_id(),
            vector: VectorSlot::pending(dimension),
            payload: self.to_payload(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kb::structured::chunk::AclMetadata;
    use crate::kb::structured::chunker::StructureAwareChunker;

    const TENANT_A: &str = "11111111-1111-1111-1111-111111111111";
    const TENANT_B: &str = "22222222-2222-2222-2222-222222222222";

    fn chunks(text: &str, acl: &AclMetadata) -> Vec<StructuredChunk> {
        StructureAwareChunker::default().chunk_document(
            "contract",
            TENANT_A,
            "contract.docx",
            "3",
            text,
            acl,
        )
    }

    fn clause_text() -> String {
        [
            "# Employment",
            "## Termination",
            "[[PAGE:12]]",
            "7.3 Termination Notice",
            "The termination notice period is ninety days from the effective date.",
        ]
        .join("\n")
    }

    #[test]
    fn upsert_point_id_is_stable_across_reindex() {
        let acl = AclMetadata::private(TENANT_A);
        let first = chunks(&clause_text(), &acl);
        let second = chunks(&clause_text(), &acl);
        assert!(!first.is_empty());
        if let (Some(original), Some(reindexed)) = (first.first(), second.first()) {
            assert_eq!(original.to_upsert_point(4).id, reindexed.to_upsert_point(4).id);
            assert_eq!(
                original.to_upsert_point(4).payload["stable_id"],
                serde_json::json!(original.id)
            );
        }

        let edited = chunks("7.3 Notice\nThe notice period is thirty days.", &acl);
        if let (Some(original), Some(changed)) = (first.first(), edited.first()) {
            assert_ne!(original.to_upsert_point(4).id, changed.to_upsert_point(4).id);
        }
    }

    #[test]
    fn upsert_point_payload_carries_all_acl_fields() {
        let acl = AclMetadata::private(TENANT_A)
            .with_users(vec!["user-1".to_string()])
            .with_roles(vec!["legal".to_string()])
            .with_groups(vec!["counsel".to_string()]);
        let chunks = chunks(&clause_text(), &acl);
        assert!(!chunks.is_empty());
        if let Some(chunk) = chunks.first() {
            let point = chunk.to_upsert_point(4);
            assert_eq!(point.payload["tenant_id"], serde_json::json!(TENANT_A));
            assert_eq!(point.payload["acl"]["tenant_id"], serde_json::json!(TENANT_A));
            assert_eq!(point.payload["acl"]["is_public"], serde_json::json!(false));
            assert_eq!(
                point.payload["acl"]["allowed_users"],
                serde_json::json!(["user-1"])
            );
            assert_eq!(
                point.payload["acl"]["allowed_roles"],
                serde_json::json!(["legal"])
            );
            assert_eq!(
                point.payload["acl"]["allowed_groups"],
                serde_json::json!(["counsel"])
            );
        }
    }

    #[test]
    fn upsert_point_preserves_citation_page_and_clause() {
        let acl = AclMetadata::private(TENANT_A);
        let chunks = chunks(&clause_text(), &acl);
        assert!(chunks.iter().any(|chunk| chunk.source.locator == "7.3"));
        if let Some(chunk) = chunks.iter().find(|chunk| chunk.source.locator == "7.3") {
            let point = chunk.to_upsert_point(4);
            assert_eq!(
                point.payload["citation"]["source"]["kind"],
                serde_json::json!("clause")
            );
            assert_eq!(
                point.payload["citation"]["source"]["locator"],
                serde_json::json!("7.3")
            );
            assert_eq!(
                point.payload["citation"]["source"]["page"],
                serde_json::json!(12)
            );
            assert_eq!(
                point.payload["citation"]["file"],
                serde_json::json!("contract.docx")
            );
            assert_eq!(point.payload["citation"]["version"], serde_json::json!("3"));
        }
    }

    #[test]
    fn upsert_points_from_different_tenants_are_separated() {
        let text = clause_text();
        let first = chunks(&text, &AclMetadata::private(TENANT_A));
        let second = StructureAwareChunker::default().chunk_document(
            "contract",
            TENANT_B,
            "contract.docx",
            "3",
            &text,
            &AclMetadata::private(TENANT_B),
        );
        assert!(!first.is_empty());
        assert!(!second.is_empty());
        if let (Some(a), Some(b)) = (first.first(), second.first()) {
            let point_a = a.to_upsert_point(4);
            let point_b = b.to_upsert_point(4);
            assert_ne!(point_a.id, point_b.id);
            assert_eq!(point_a.payload["tenant_id"], serde_json::json!(TENANT_A));
            assert_eq!(point_b.payload["tenant_id"], serde_json::json!(TENANT_B));
            assert_eq!(
                point_a.payload["acl"]["tenant_id"],
                serde_json::json!(TENANT_A)
            );
            assert_eq!(
                point_b.payload["acl"]["tenant_id"],
                serde_json::json!(TENANT_B)
            );
        }
    }

    #[test]
    fn vector_slot_is_a_pending_placeholder_of_requested_dimension() {
        let acl = AclMetadata::private(TENANT_A);
        let chunks = chunks(&clause_text(), &acl);
        assert!(!chunks.is_empty());
        if let Some(chunk) = chunks.first() {
            let point = chunk.to_upsert_point(8);
            assert_eq!(point.vector.dimension, 8);
            assert_eq!(point.vector.values.len(), 8);
            assert!(point.vector.values.iter().all(|value| *value == 0.0));
            assert!(!point.vector.is_ready());

            let filled = VectorSlot::filled(vec![0.1_f32; 2]);
            assert!(filled.is_ready());
            assert_eq!(filled.dimension, 2);
        }
    }
}
