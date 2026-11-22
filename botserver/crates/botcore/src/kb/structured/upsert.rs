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
    use std::collections::BTreeSet;

    const TENANT_A: &str = "11111111-1111-1111-1111-111111111111";
    const TENANT_B: &str = "22222222-2222-2222-2222-222222222222";

    /// Body line that acts as a unique tenant marker in `content`.
    fn clause_text(marker: &str) -> String {
        [
            "# Employment",
            "## Termination",
            "[[PAGE:12]]",
            "7.3 Termination Notice",
            marker,
        ]
        .join("\n")
    }

    fn chunks_for(
        tenant: &str,
        file: &str,
        version: &str,
        text: &str,
        acl: &AclMetadata,
    ) -> Vec<StructuredChunk> {
        StructureAwareChunker::default()
            .chunk_document("contract", tenant, file, version, text, acl)
    }

    /// The clause-7.3 chunk is the stable fixture used by payload assertions.
    fn clause_chunk(tenant: &str, file: &str, marker: &str, acl: &AclMetadata) -> StructuredChunk {
        chunks_for(tenant, file, "3", &clause_text(marker), acl)
            .into_iter()
            .find(|chunk| chunk.source.locator == "7.3")
            .expect("fixture must contain clause 7.3")
    }

    fn full_acl(tenant: &str) -> AclMetadata {
        AclMetadata::private(tenant)
            .with_users(vec!["user-1".to_string()])
            .with_roles(vec!["legal".to_string()])
            .with_groups(vec!["counsel".to_string()])
    }

    fn point_ids(chunks: &[StructuredChunk]) -> Vec<String> {
        chunks
            .iter()
            .map(|chunk| chunk.to_upsert_point(4).id)
            .collect()
    }

    // Criterion: to_upsert_point payload carries the documented fields and the
    // tenant, with no surprise fields that could smuggle data across tenants.
    #[test]
    fn upsert_payload_has_exactly_the_documented_fields() {
        let acl = full_acl(TENANT_A);
        let chunk = clause_chunk(TENANT_A, "contract.docx", "ninety days notice", &acl);
        let point = chunk.to_upsert_point(4);

        let keys: BTreeSet<&str> = point.payload.keys().map(String::as_str).collect();
        let expected: BTreeSet<&str> = [
            "content",
            "document_path",
            "stable_id",
            "tenant_id",
            "citation",
            "acl",
        ]
        .into_iter()
        .collect();
        assert_eq!(keys, expected);

        assert_eq!(point.payload["content"], serde_json::json!(chunk.content));
        assert_eq!(
            point.payload["document_path"],
            serde_json::json!("contract.docx")
        );
        assert_eq!(point.payload["stable_id"], serde_json::json!(chunk.id));
        assert_eq!(point.payload["tenant_id"], serde_json::json!(TENANT_A));
    }

    // Criterion: to_upsert_point carries the FULL ACL, including public flag
    // and every allow-list, and the tenant repeated inside the ACL.
    #[test]
    fn upsert_payload_carries_full_acl_including_public_flag() {
        let private = full_acl(TENANT_A);
        let chunk = clause_chunk(TENANT_A, "contract.docx", "ninety days", &private);
        let point = chunk.to_upsert_point(4);

        assert_eq!(
            point.payload["acl"],
            serde_json::to_value(&private).expect("serialize private ACL")
        );
        assert_eq!(
            point.payload["acl"]["tenant_id"],
            serde_json::json!(TENANT_A)
        );
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

        let public = AclMetadata::public(TENANT_A);
        let public_chunk = clause_chunk(TENANT_A, "contract.docx", "ninety days", &public);
        let public_point = public_chunk.to_upsert_point(4);
        assert_eq!(
            public_point.payload["acl"],
            serde_json::to_value(&public).expect("serialize public ACL")
        );
        assert_eq!(
            public_point.payload["acl"]["is_public"],
            serde_json::json!(true)
        );
    }

    // Criterion: to_upsert_point carries the FULL citation (document id, file,
    // version, logical address, page and heading path).
    #[test]
    fn upsert_payload_carries_full_citation() {
        let acl = full_acl(TENANT_A);
        let chunk = clause_chunk(TENANT_A, "contract.docx", "ninety days", &acl);
        let point = chunk.to_upsert_point(4);

        assert_eq!(
            point.payload["citation"],
            serde_json::to_value(&chunk.citation()).expect("serialize citation")
        );
        assert_eq!(
            point.payload["citation"]["document_id"],
            serde_json::json!("contract")
        );
        assert_eq!(
            point.payload["citation"]["file"],
            serde_json::json!("contract.docx")
        );
        assert_eq!(point.payload["citation"]["version"], serde_json::json!("3"));
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
            point.payload["citation"]["heading_path"],
            serde_json::to_value(&chunk.heading_path).expect("serialize heading path")
        );
        assert!(chunk
            .heading_path
            .iter()
            .any(|heading| heading == "Termination Notice"));
    }

    // Criterion: stable point id, partitioned by tenant/version/content.
    #[test]
    fn point_id_is_stable_and_partitioned_by_identity() {
        let acl_a = full_acl(TENANT_A);
        let text = clause_text("ninety days");
        let first = chunks_for(TENANT_A, "contract.docx", "3", &text, &acl_a);
        let reindexed = chunks_for(TENANT_A, "contract.docx", "3", &text, &acl_a);
        assert!(!first.is_empty());
        assert_eq!(point_ids(&first), point_ids(&reindexed));

        let chunk = &first[0];
        let point = chunk.to_upsert_point(4);
        assert_eq!(point.id, chunk.point_id());
        assert_eq!(point.id, chunk.id);
        assert_eq!(point.payload["stable_id"], serde_json::json!(point.id));

        let edited = chunks_for(
            TENANT_A,
            "contract.docx",
            "3",
            &clause_text("thirty days"),
            &acl_a,
        );
        let new_version = chunks_for(TENANT_A, "contract.docx", "4", &text, &acl_a);
        let other_tenant = chunks_for(TENANT_B, "contract.docx", "3", &text, &full_acl(TENANT_B));
        assert_ne!(point_ids(&first), point_ids(&edited));
        assert_ne!(point_ids(&first), point_ids(&new_version));
        assert_ne!(point_ids(&first), point_ids(&other_tenant));
    }

    // Criterion: VectorSlot starts pending on every projection, regardless of
    // the requested dimension, and is a zero-filled placeholder.
    #[test]
    fn to_upsert_point_always_starts_with_a_pending_vector() {
        let acl = full_acl(TENANT_A);
        let chunk = clause_chunk(TENANT_A, "contract.docx", "ninety days", &acl);
        for dimension in [1, 4, DEFAULT_VECTOR_DIMENSION] {
            let point = chunk.to_upsert_point(dimension);
            assert_eq!(point.vector.dimension, dimension);
            assert_eq!(point.vector.values.len(), dimension);
            assert!(point.vector.values.iter().all(|value| *value == 0.0));
            assert!(!point.vector.ready);
            assert!(!point.vector.is_ready());
            assert_eq!(point.vector, VectorSlot::pending(dimension));
        }
    }

    // Criterion: pending -> filled transition replaces the slot in place while
    // preserving identity fields of the UpsertPoint.
    #[test]
    fn vector_slot_transitions_from_pending_to_filled() {
        let acl = full_acl(TENANT_A);
        let chunk = clause_chunk(TENANT_A, "contract.docx", "ninety days", &acl);
        let mut point = chunk.to_upsert_point(4);
        let original_id = point.id.clone();
        let original_payload = point.payload.clone();
        assert!(!point.vector.is_ready());

        let embedding = vec![0.25_f32; 4];
        point.vector = VectorSlot::filled(embedding.clone());

        assert!(point.vector.is_ready());
        assert_eq!(point.vector.dimension, 4);
        assert_eq!(point.vector.values, embedding);
        assert_eq!(point.vector, VectorSlot::filled(embedding));
        assert_ne!(point.vector, VectorSlot::pending(4));
        assert_eq!(point.id, original_id);
        assert_eq!(point.payload, original_payload);

        let cloned = point.vector.clone();
        assert_eq!(cloned, point.vector);

        let empty_pending = VectorSlot::pending(0);
        assert_eq!(empty_pending.dimension, 0);
        assert!(empty_pending.values.is_empty());
        assert!(!empty_pending.is_ready());
        let empty_filled = VectorSlot::filled(Vec::new());
        assert_eq!(empty_filled.dimension, 0);
        assert!(empty_filled.values.is_empty());
        assert!(empty_filled.is_ready());
    }

    // Criterion: no content or payload field leaks across tenant boundaries.
    #[test]
    fn upsert_payloads_do_not_leak_fields_across_tenants() {
        let alpha_marker = "ALPHA_ONLY_CONFIDENTIAL";
        let beta_marker = "BETA_ONLY_CONFIDENTIAL";
        let acl_a = full_acl(TENANT_A);
        let acl_b = full_acl(TENANT_B);
        let alpha = chunks_for(
            TENANT_A,
            "alpha.docx",
            "3",
            &clause_text(alpha_marker),
            &acl_a,
        );
        let beta = chunks_for(
            TENANT_B,
            "beta.docx",
            "3",
            &clause_text(beta_marker),
            &acl_b,
        );
        assert!(!alpha.is_empty());
        assert!(!beta.is_empty());

        for (chunks, own, foreign, tenant, file) in [
            (&alpha, alpha_marker, beta_marker, TENANT_A, "alpha.docx"),
            (&beta, beta_marker, alpha_marker, TENANT_B, "beta.docx"),
        ] {
            for chunk in chunks {
                let point = chunk.to_upsert_point(4);
                let serialized = serde_json::to_string(&point.payload).expect("serialize payload");

                assert!(serialized.contains(own));
                assert!(
                    !serialized.contains(foreign),
                    "foreign tenant content leaked into payload: {serialized}"
                );
                assert_eq!(point.payload["content"], serde_json::json!(chunk.content));
                assert_eq!(point.payload["tenant_id"], serde_json::json!(tenant));
                assert_eq!(point.payload["acl"]["tenant_id"], serde_json::json!(tenant));
                assert_eq!(point.payload["document_path"], serde_json::json!(file));
                assert_eq!(point.payload["citation"]["file"], serde_json::json!(file));
            }
        }

        let point_a = alpha[0].to_upsert_point(4);
        let point_b = beta[0].to_upsert_point(4);
        assert_ne!(point_a.id, point_b.id);
        assert_ne!(
            point_a.payload["acl"], point_b.payload["acl"],
            "ACLs of different tenants must never be equal"
        );
    }
}
