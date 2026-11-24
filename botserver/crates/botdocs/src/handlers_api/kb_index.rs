//! Structure-aware KB projection for saved Docs documents (issue #2499).
//!
//! A Docs save owns the document bytes; the vector store does not live in
//! [`crate::state::DocState`]. This module is the safe seam between the two: it
//! turns a saved document into an [`IndexDocument`] (for the in-memory
//! structured index) and storage-agnostic [`UpsertPoint`]s (for the runtime
//! vector indexer).
//!
//! Nothing here claims persistence. Callers own the decision of where the
//! produced points are written, so an unavailable store can never turn a
//! successful document save into an HTTP error.

use crate::utils_markdown::html_to_markdown;
use botcore::kb::structured::{
    AclMetadata, ChunkOptions, IndexDocument, StructureAwareChunker, StructuredChunk, UpsertPoint,
    DEFAULT_VECTOR_DIMENSION,
};

/// The fields a Docs save can hand to the KB projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentIndexInput<'a> {
    pub document_id: &'a str,
    pub title: &'a str,
    pub source_ref: &'a str,
    pub version: u64,
    pub content: &'a str,
    pub tenant_id: &'a str,
    pub owner_id: &'a str,
}

/// Bucket-derived tenant plus the document owner. No grant is invented: role
/// and group lists stay empty so retrieval fails closed until real grants are
/// projected by the runtime indexer.
fn derived_acl(input: &DocumentIndexInput<'_>) -> AclMetadata {
    AclMetadata::private(input.tenant_id).with_users(vec![input.owner_id.to_string()])
}

fn citation_file<'a>(input: &DocumentIndexInput<'a>) -> &'a str {
    if !input.source_ref.trim().is_empty() {
        input.source_ref
    } else if !input.title.trim().is_empty() {
        input.title
    } else {
        input.document_id
    }
}

/// Prepares the saved document for the in-memory structured index.
pub fn build_index_document(input: &DocumentIndexInput<'_>) -> IndexDocument {
    IndexDocument::new(
        input.document_id,
        input.tenant_id,
        citation_file(input),
        &input.version.to_string(),
        &html_to_markdown(input.content),
        derived_acl(input),
    )
}

/// Structure-aware chunks for a saved document, with stable ids and ACL.
pub fn build_structured_chunks(input: &DocumentIndexInput<'_>) -> Vec<StructuredChunk> {
    let document = build_index_document(input);
    if document.text.trim().is_empty() {
        return Vec::new();
    }
    StructureAwareChunker::new(ChunkOptions::default()).chunk_document(
        &document.document_id,
        &document.tenant_id,
        &document.file,
        &document.version,
        &document.text,
        &document.acl,
    )
}

/// Storage-agnostic upsert requests ready for the runtime vector indexer.
pub fn build_upsert_points(input: &DocumentIndexInput<'_>) -> Vec<UpsertPoint> {
    build_structured_chunks(input)
        .iter()
        .map(|chunk| chunk.to_upsert_point(DEFAULT_VECTOR_DIMENSION))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TENANT: &str = "11111111-1111-1111-1111-111111111111";
    const OTHER_TENANT: &str = "22222222-2222-2222-2222-222222222222";
    const OWNER: &str = "33333333-3333-3333-3333-333333333333";
    const HTML: &str = "<h1>Employment</h1><h2>Termination</h2><p>7.3 Notice</p>\
                        <p>The notice period is ninety days.</p>";

    fn input<'a>(content: &'a str, version: u64, tenant: &'a str) -> DocumentIndexInput<'a> {
        DocumentIndexInput {
            document_id: "contract",
            title: "Employment Agreement",
            source_ref: "employment.docx",
            version,
            content,
            tenant_id: tenant,
            owner_id: OWNER,
        }
    }

    #[test]
    fn saved_document_yields_addressed_chunks_with_derived_acl() {
        let chunks = build_structured_chunks(&input(HTML, 3, TENANT));
        let addressed = chunks.iter().any(|chunk| {
            chunk.source.locator == "7.3"
                && chunk.tenant_id == TENANT
                && chunk.file == "employment.docx"
                && chunk.version == "3"
                // The chunker treats the clause title ("Notice") as the
                // leaf heading, so assert the real heading path shape.
                && chunk.heading_path.len() >= 2
                && chunk.heading_path.iter().any(|heading| heading == "Notice")
                && chunk.acl.tenant_id == TENANT
                && !chunk.acl.is_public
                && chunk.acl.allowed_users == vec![OWNER.to_string()]
                && chunk.acl.allowed_roles.is_empty()
                && chunk.acl.allowed_groups.is_empty()
        });
        assert!(addressed);
    }

    #[test]
    fn upsert_points_carry_citation_acl_and_tenant() {
        let points = build_upsert_points(&input(HTML, 3, TENANT));
        assert!(!points.is_empty());
        for point in &points {
            assert_eq!(point.payload["tenant_id"], serde_json::json!(TENANT));
            assert_eq!(
                point.payload["document_path"],
                serde_json::json!("employment.docx")
            );
            assert_eq!(point.payload["acl"]["tenant_id"], serde_json::json!(TENANT));
            assert_eq!(point.payload["acl"]["is_public"], serde_json::json!(false));
            assert_eq!(
                point.payload["acl"]["allowed_users"],
                serde_json::json!([OWNER])
            );
            assert_eq!(
                point.payload["citation"]["document_id"],
                serde_json::json!("contract")
            );
            assert_eq!(point.vector.dimension, DEFAULT_VECTOR_DIMENSION);
            assert!(!point.vector.is_ready());
        }
    }

    #[test]
    fn empty_document_is_skipped_not_indexed() {
        assert!(build_structured_chunks(&input("<p>  </p>", 1, TENANT)).is_empty());
        assert!(build_upsert_points(&input("", 1, TENANT)).is_empty());
    }

    #[test]
    fn ids_are_stable_on_resave_and_change_on_edit() {
        let first = build_upsert_points(&input(HTML, 3, TENANT));
        let again = build_upsert_points(&input(HTML, 3, TENANT));
        let edited = build_upsert_points(&input(HTML, 4, TENANT));
        let ids = |points: &[UpsertPoint]| -> Vec<String> {
            points.iter().map(|point| point.id.clone()).collect()
        };
        assert!(!first.is_empty());
        assert_eq!(ids(&first), ids(&again));
        assert_ne!(ids(&first), ids(&edited));
    }

    #[test]
    fn projection_never_crosses_tenant_boundaries() {
        const MARKER_A: &str = "ALPHA_ONLY_CONFIDENTIAL";
        const MARKER_B: &str = "BETA_ONLY_CONFIDENTIAL";
        let alpha = build_upsert_points(&input(
            &format!("<p>{MARKER_A} clause body.</p>"),
            1,
            TENANT,
        ));
        let beta = build_upsert_points(&input(
            &format!("<p>{MARKER_B} clause body.</p>"),
            1,
            OTHER_TENANT,
        ));
        assert!(!alpha.is_empty());
        assert!(!beta.is_empty());
        for (points, own, foreign, tenant) in [
            (&alpha, MARKER_A, MARKER_B, TENANT),
            (&beta, MARKER_B, MARKER_A, OTHER_TENANT),
        ] {
            for point in points {
                let serialized = match serde_json::to_string(&point.payload) {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                assert!(serialized.contains(own));
                assert!(!serialized.contains(foreign));
                assert_eq!(point.payload["tenant_id"], serde_json::json!(tenant));
                assert_eq!(point.payload["acl"]["tenant_id"], serde_json::json!(tenant));
            }
        }
    }
}
