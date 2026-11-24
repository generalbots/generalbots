//! Persistence seam for the in-memory KB consumer (issue #2499).
//!
//! [`super::InMemoryKbIndex`] accepts [`UpsertPoint`] values and answers
//! permission-aware searches, but it is process-local. This module maps the
//! very same points onto a real vector store **without** adding a Qdrant
//! dependency to `botcore`: [`KbVectorStore`] is the transport-agnostic trait,
//! [`QdrantPoint`] is the wire shape, and [`QdrantPayload`] is the payload a
//! Qdrant collection stores next to the embedding.
//!
//! # Contract a real adapter must preserve
//!
//! 1. Call [`validate_points`] (or [`QdrantPoint::try_from`], which validates a
//!    single point) before any network I/O. Validation is delegated to
//!    [`InMemoryKbIndex::insert`], so id emptiness, finite non-empty vectors,
//!    dimension consistency, required payload fields, `stable_id` agreement and
//!    tenant/ACL agreement behave **exactly** as in the in-process consumer.
//!    The only adapter-specific precondition is [`VectorSlot::is_ready`]: the
//!    runtime must replace the reserved slot with a real embedding first.
//! 2. Replace the reserved [`VectorSlot`] with an embedding produced outside
//!    `botcore` (e.g. `botqdrant::embedding::EmbeddingGenerator::
//!    generate_text_embedding`, fed with the chunk's `content` from the
//!    source document). The embedding is the only place text crosses the
//!    boundary; [`QdrantPayload`] contains **no content field**.
//!
//! 3. Upsert one point per converted record. With the native client already
//!    used by this repo (`botqdrant`, `qdrant-client = "1.20"` is pinned at the
//!    workspace root but `botqdrant` talks REST via `reqwest`) the exact call
//!    is:
//!
//!    ```text
//!    let structs: Vec<botqdrant::PointStruct> = records.into_iter().map(|r| {
//!        let map: serde_json::Map<String, serde_json::Value> =
//!            serde_json::to_value(r.payload)?.as_object().cloned().unwrap_or_default();
//!        botqdrant::PointStruct::new(r.id, r.vector, map)
//!    }).collect();
//!    botqdrant::UpsertPointsBuilder::new(COLLECTION, structs).build(&client).await?;
//!    ```
//!
//!    which issues `PUT {qdrant_url}/collections/{COLLECTION}/points/upsert`
//!    with `{"points":[{"id":..,"vector":..,"payload":..}]}`.
//! 4. Delete is administrative and **tenant-scoped by the caller**: the prefix
//!    must come from a tenant-owned document path. It matches either the
//!    deterministic point id or the payload's `document_path`, mirroring
//!    [`InMemoryKbIndex::remove`]. `botqdrant::QdrantClient::delete_points`
//!    currently accepts explicit ids only, so a real adapter must first scroll
//!    with a filter on `stable_id` / `document_path` and pass the resulting ids,
//!    or add a filter-based delete to `botqdrant` (owned elsewhere).
//! 5. Search must keep the in-memory filter order: scope by tenant, apply
//!    `AclMetadata::allows`, then score; return citation-only hits with no
//!    document text.

use super::super::chunk::{stable_id, AclMetadata};
use super::super::citation::Citation;
use super::super::upsert::{UpsertPoint, VectorSlot};
use super::{read_string, read_value, InMemoryKbIndex, IndexError};
use serde::{Deserialize, Serialize};

/// Failures surfaced by a vector-store adapter.
///
/// All messages are fixed templates: an HTTP/client error is logged locally by
/// the adapter and reduced to [`StoreError::Backend`] at this boundary, so no
/// raw external text reaches a caller (same rule as [`IndexError`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// Shared validation failed; the exact cause is the in-memory one.
    Invalid(IndexError),
    /// The runtime has not filled the reserved embedding slot yet.
    VectorNotReady { id: String },
    /// The vector store rejected the operation (details logged locally).
    Backend,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(error) => write!(formatter, "invalid KB vector point: {error}"),
            Self::VectorNotReady { .. } => {
                formatter.write_str("KB vector point embedding slot is not ready")
            }
            Self::Backend => formatter.write_str("KB vector store operation failed"),
        }
    }
}

impl std::error::Error for StoreError {}

/// Transport-agnostic persistence seam for projected KB points.
///
/// Implementations own a collection and a client; they must call
/// [`validate_points`] before touching the backend and must never persist
/// document content.
#[async_trait::async_trait]
pub trait KbVectorStore: Send + Sync {
    /// Upserts a batch of points. Validation is atomic: a rejected batch leaves
    /// the store untouched, exactly like [`InMemoryKbIndex::insert`].
    async fn upsert(&self, points: Vec<UpsertPoint>) -> Result<(), StoreError>;

    /// Removes points whose id or `document_path` starts with `source_prefix`.
    /// An empty prefix is a no-op, as in [`InMemoryKbIndex::remove`].
    async fn delete(&self, source_prefix: &str) -> Result<(), StoreError>;
}

/// Citation-only payload stored beside an embedding in Qdrant.
///
/// Deliberately has no `content` field: retrieval needs embeddings, ACLs and
/// citations, never document bytes. `section` is the deepest heading, kept as
/// a convenience filter alongside the full `citation.heading_path`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QdrantPayload {
    pub stable_id: String,
    pub tenant_id: String,
    pub document_path: String,
    pub section: Option<String>,
    pub citation: Citation,
    pub acl: AclMetadata,
}

/// Wire shape matching `botqdrant::PointStruct` and the Qdrant upsert REST body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QdrantPoint {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: QdrantPayload,
}

impl TryFrom<&UpsertPoint> for QdrantPoint {
    type Error = StoreError;

    fn try_from(point: &UpsertPoint) -> Result<Self, Self::Error> {
        validate_single(point)?;
        let tenant_id = read_string(&point.payload, "tenant_id", &point.id)
            .map_err(StoreError::Invalid)?
            .to_string();
        let stable_id_value = read_string(&point.payload, "stable_id", &point.id)
            .map_err(StoreError::Invalid)?
            .to_string();
        let citation = read_value::<Citation>(&point.payload, "citation", &point.id)
            .map_err(StoreError::Invalid)?;
        let acl = read_value::<AclMetadata>(&point.payload, "acl", &point.id)
            .map_err(StoreError::Invalid)?;

        Ok(Self {
            // FNV-1a of (stable_id, tenant): same id for an unchanged reindex,
            // a new one on edit, and no collision between tenants.
            id: stable_id(&[&stable_id_value, &tenant_id]),
            vector: point.vector.values.clone(),
            payload: QdrantPayload {
                stable_id: stable_id_value,
                tenant_id,
                document_path: citation.file.clone(),
                section: citation.heading_path.last().cloned(),
                citation,
                acl,
            },
        })
    }
}

/// Validates a batch with the exact semantics of the in-process consumer.
pub fn validate_points(points: &[UpsertPoint]) -> Result<(), StoreError> {
    let mut probe = InMemoryKbIndex::new();
    probe.insert(points.to_vec()).map_err(StoreError::Invalid)?;
    for point in points {
        require_ready(point)?;
    }
    Ok(())
}

fn validate_single(point: &UpsertPoint) -> Result<(), StoreError> {
    let mut probe = InMemoryKbIndex::new();
    probe
        .insert(vec![point.clone()])
        .map_err(StoreError::Invalid)?;
    require_ready(point)
}

fn require_ready(point: &UpsertPoint) -> Result<(), StoreError> {
    if point.vector.is_ready() {
        Ok(())
    } else {
        Err(StoreError::VectorNotReady {
            id: point.id.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kb::structured::chunk::StructuredChunk;
    use crate::kb::structured::citation::SourceRef;
    use serde_json::Value;

    const TENANT_A: &str = "11111111-1111-1111-1111-111111111111";
    const TENANT_B: &str = "22222222-2222-2222-2222-222222222222";
    const SECRET: &str = "ACME_MERGER_CONFIDENTIAL_CLAUSE_7_3";

    fn point(tenant: &str, content: &str) -> UpsertPoint {
        let acl = AclMetadata::private(tenant)
            .with_users(vec!["user-1".to_string()])
            .with_roles(vec!["legal".to_string()])
            .with_groups(vec!["counsel".to_string()]);
        let chunk = StructuredChunk {
            id: stable_id(&[tenant, "contract.docx", "7.3", content]),
            document_id: "contract".to_string(),
            file: "contract.docx".to_string(),
            version: "3".to_string(),
            tenant_id: tenant.to_string(),
            heading_path: vec!["Employment".to_string(), "Termination".to_string()],
            source: SourceRef::clause("7.3").with_page(12),
            content: content.to_string(),
            acl,
        };
        let mut point = chunk.to_upsert_point(4);
        point.vector = VectorSlot::filled(vec![0.25_f32; 4]);
        point
    }

    fn convert(point: &UpsertPoint) -> QdrantPoint {
        let result = QdrantPoint::try_from(point);
        assert!(result.is_ok(), "conversion must succeed for a valid point");
        match result {
            Ok(record) => record,
            Err(_) => QdrantPoint {
                id: String::new(),
                vector: Vec::new(),
                payload: QdrantPayload {
                    stable_id: String::new(),
                    tenant_id: String::new(),
                    document_path: String::new(),
                    section: None,
                    citation: Citation::new("", "", "", SourceRef::page(1), Vec::new()),
                    acl: AclMetadata::private(""),
                },
            },
        }
    }

    #[test]
    fn payload_round_trips_source_section_acl_and_citation() {
        let point = point(TENANT_A, SECRET);
        let record = convert(&point);
        let payload = record.payload;

        let encoded = serde_json::to_string(&payload);
        assert!(encoded.is_ok(), "payload must be serializable");
        let decoded = encoded
            .ok()
            .and_then(|text| serde_json::from_str::<QdrantPayload>(&text).ok());
        assert!(decoded.is_some(), "payload must round-trip through JSON");
        if let Some(decoded) = decoded {
            assert_eq!(decoded, payload);

            let citation = read_value::<Citation>(&point.payload, "citation", &point.id);
            assert!(citation.is_ok());
            if let Ok(original) = citation {
                assert_eq!(decoded.citation, original);
                assert_eq!(
                    decoded.citation.source,
                    SourceRef::clause("7.3").with_page(12)
                );
                assert_eq!(decoded.citation.document_id, "contract");
                assert_eq!(decoded.citation.file, "contract.docx");
                assert_eq!(decoded.citation.version, "3");
                assert_eq!(decoded.section, original.heading_path.last().cloned());
                assert_eq!(decoded.section.as_deref(), Some("Termination"));
                assert_eq!(decoded.document_path, original.file);
            }

            let acl = read_value::<AclMetadata>(&point.payload, "acl", &point.id);
            assert!(acl.is_ok());
            if let Ok(original) = acl {
                assert_eq!(decoded.acl, original);
                assert_eq!(decoded.acl.tenant_id, TENANT_A);
                assert!(!decoded.acl.is_public);
                assert_eq!(decoded.acl.allowed_users, vec!["user-1".to_string()]);
                assert_eq!(decoded.acl.allowed_roles, vec!["legal".to_string()]);
                assert_eq!(decoded.acl.allowed_groups, vec!["counsel".to_string()]);
            }
            assert_eq!(decoded.stable_id, point.id);
            assert_eq!(decoded.tenant_id, TENANT_A);
        }
    }

    #[test]
    fn payload_has_no_document_content_field() {
        let point = point(TENANT_A, SECRET);
        let record = convert(&point);

        let encoded = serde_json::to_string(&record);
        assert!(encoded.is_ok());
        if let Ok(text) = encoded {
            assert!(!text.contains(SECRET), "document content leaked: {text}");
            assert!(!text.contains("\"content\""), "content key leaked: {text}");
        }

        let value = serde_json::to_value(&record);
        assert!(value.is_ok());
        if let Ok(Value::Object(map)) = value {
            let mut keys: Vec<&str> = map.keys().map(String::as_str).collect();
            keys.sort_unstable();
            assert_eq!(keys, vec!["id", "payload", "vector"]);
            let payload = map.get("payload");
            assert!(payload.is_some());
            if let Some(Value::Object(fields)) = payload {
                let mut names: Vec<&str> = fields.keys().map(String::as_str).collect();
                names.sort_unstable();
                assert_eq!(
                    names,
                    vec![
                        "acl",
                        "citation",
                        "document_path",
                        "section",
                        "stable_id",
                        "tenant_id"
                    ]
                );
                assert!(fields.get("content").is_none());
            }
        }
    }

    #[test]
    fn point_id_is_deterministic_and_tenant_partitioned() {
        let first = convert(&point(TENANT_A, SECRET));
        let again = convert(&point(TENANT_A, SECRET));
        let other_tenant = convert(&point(TENANT_B, SECRET));
        let edited = convert(&point(TENANT_A, "amended notice"));

        assert_eq!(first.id, again.id);
        assert_eq!(first.id.len(), 16);
        assert_ne!(first.id, point(TENANT_A, SECRET).id);
        assert_ne!(first.id, other_tenant.id);
        assert_ne!(first.id, edited.id);
        assert_eq!(first.vector, vec![0.25_f32; 4]);
    }

    #[test]
    fn validation_keeps_in_memory_tenant_vector_and_field_semantics() {
        let valid = point(TENANT_A, SECRET);
        assert!(validate_points(std::slice::from_ref(&valid)).is_ok());

        let mut mismatched = valid.clone();
        mismatched
            .payload
            .insert("tenant_id".to_string(), Value::String(TENANT_B.to_string()));
        assert_eq!(
            validate_points(std::slice::from_ref(&mismatched)),
            Err(StoreError::Invalid(IndexError::TenantMismatch {
                id: valid.id.clone(),
                tenant_id: TENANT_B.to_string(),
                acl_tenant_id: TENANT_A.to_string(),
            }))
        );

        let mut pending = valid.clone();
        pending.vector = VectorSlot::pending(4);
        assert_eq!(
            QdrantPoint::try_from(&pending),
            Err(StoreError::VectorNotReady {
                id: valid.id.clone()
            })
        );

        let mut missing = valid.clone();
        missing.payload.remove("citation");
        assert_eq!(
            QdrantPoint::try_from(&missing),
            Err(StoreError::Invalid(IndexError::MissingPayloadField {
                id: valid.id.clone(),
                field: "citation",
            }))
        );
    }
}
