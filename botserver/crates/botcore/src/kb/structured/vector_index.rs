//! In-process consumer for the [`UpsertPoint`] values produced by the Docs KB
//! projection (issue #2499).
//!
//! The projection in the sibling `upsert` module is deliberately
//! storage-agnostic: it emits stable ids, reserved vector slots and JSON
//! payloads, but never talks
//! to a vector store. `InMemoryKbIndex` is the smallest real consumer of that
//! output — it accepts filled points, keeps them in process and answers
//! permission-aware cosine searches. It lets the whole projection be exercised
//! in unit tests without Qdrant, and it documents the exact contract a real
//! Qdrant adapter must preserve: ACL post-filtering and citation-only results.
//!
//! Document text is deliberately **not** stored. Search needs only embeddings,
//! ACLs and citation metadata, so neither [`SearchHit`] nor the index's `Debug`
//! can expose document content.

use super::chunk::AclMetadata;
use super::citation::{Citation, SourceRef};
use super::upsert::{UpsertPoint, VectorSlot};
use crate::kb::permissions::UserContext;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

/// Validation failures for [`InMemoryKbIndex::insert`].
///
/// Messages are fixed templates: no external/raw error text is ever propagated
/// to a caller (a real adapter should do the same at its HTTP boundary).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexError {
    EmptyId,
    EmptyVector {
        id: String,
    },
    NonFiniteVector {
        id: String,
        position: usize,
    },
    DimensionMismatch {
        id: String,
        expected: usize,
        actual: usize,
    },
    MissingPayloadField {
        id: String,
        field: &'static str,
    },
    InvalidPayloadField {
        id: String,
        field: &'static str,
    },
    IdMismatch {
        id: String,
        payload_id: String,
    },
    TenantMismatch {
        id: String,
        tenant_id: String,
        acl_tenant_id: String,
    },
}

impl fmt::Display for IndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId => formatter.write_str("upsert point id must not be empty"),
            Self::EmptyVector { .. } => {
                formatter.write_str("upsert point vector must not be empty")
            }
            Self::NonFiniteVector { .. } => {
                formatter.write_str("upsert point vector contains a non-finite value")
            }
            Self::DimensionMismatch { .. } => {
                formatter.write_str("upsert point vector dimension does not match the index")
            }
            Self::MissingPayloadField { .. } => {
                formatter.write_str("upsert point payload is missing a required field")
            }
            Self::InvalidPayloadField { .. } => {
                formatter.write_str("upsert point payload field has an invalid shape")
            }
            Self::IdMismatch { .. } => {
                formatter.write_str("upsert point id disagrees with its payload stable_id")
            }
            Self::TenantMismatch { .. } => {
                formatter.write_str("upsert point tenant disagrees with its ACL tenant")
            }
        }
    }
}

impl std::error::Error for IndexError {}

/// Citation-only search result. There is no content field, on purpose.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchHit {
    /// Stable point id produced by the projection (safe to log and to cite).
    pub id: String,
    /// Exact address inside the source document (clause, page, slide, cell…).
    pub source: SourceRef,
    /// Deepest heading of the chunk, when the document had one.
    pub section: Option<String>,
    /// Cosine similarity to the query embedding, in `(0, 1]`.
    pub score: f32,
}

#[derive(Debug, Clone)]
struct IndexedPoint {
    vector: Vec<f32>,
    tenant_id: String,
    acl: AclMetadata,
    source: SourceRef,
    section: Option<String>,
    source_path: String,
}

/// In-memory, permission-aware consumer of projected upsert points.
///
/// A real Qdrant adapter should reuse the same filtering order: scope by
/// tenant, apply `AclMetadata::allows`, then score. Dropping unauthorized
/// points before scoring prevents the caller from learning that a restricted
/// document exists.
#[derive(Debug, Default)]
pub struct InMemoryKbIndex {
    points: HashMap<String, IndexedPoint>,
    dimension: Option<usize>,
}

impl InMemoryKbIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Number of accepted points; re-inserting an existing id replaces it.
    /// Validation is atomic: a rejected point leaves the index untouched.
    pub fn insert(&mut self, points: Vec<UpsertPoint>) -> Result<usize, IndexError> {
        let mut staged = Vec::with_capacity(points.len());
        let mut expected = self.dimension;

        for point in points {
            let UpsertPoint {
                id,
                vector,
                payload,
            } = point;
            let VectorSlot {
                dimension, values, ..
            } = vector;
            if id.trim().is_empty() {
                return Err(IndexError::EmptyId);
            }
            if values.is_empty() {
                return Err(IndexError::EmptyVector { id });
            }
            if let Some(position) = values.iter().position(|value| !value.is_finite()) {
                return Err(IndexError::NonFiniteVector { id, position });
            }
            if dimension != values.len() {
                return Err(IndexError::DimensionMismatch {
                    id,
                    expected: dimension,
                    actual: values.len(),
                });
            }
            if let Some(current) = expected {
                if current != values.len() {
                    return Err(IndexError::DimensionMismatch {
                        id,
                        expected: current,
                        actual: values.len(),
                    });
                }
            } else {
                expected = Some(values.len());
            }

            let tenant_id = read_string(&payload, "tenant_id", &id)?.to_string();
            let stable_id = read_string(&payload, "stable_id", &id)?;
            if stable_id != id {
                return Err(IndexError::IdMismatch {
                    id,
                    payload_id: stable_id.to_string(),
                });
            }
            let acl = read_value::<AclMetadata>(&payload, "acl", &id)?;
            if acl.tenant_id != tenant_id {
                return Err(IndexError::TenantMismatch {
                    id,
                    tenant_id,
                    acl_tenant_id: acl.tenant_id,
                });
            }
            let citation = read_value::<Citation>(&payload, "citation", &id)?;
            let source_path = citation.file;
            let source = citation.source;
            let section = citation.heading_path.into_iter().last();
            staged.push((
                id,
                IndexedPoint {
                    vector: values,
                    tenant_id,
                    acl,
                    source,
                    section,
                    source_path,
                },
            ));
        }

        let inserted = staged.len();
        for (id, indexed) in staged {
            self.points.insert(id, indexed);
        }
        self.dimension = expected;
        Ok(inserted)
    }

    /// Permission-aware cosine search. Invalid queries, zero norms, denied
    /// ACLs and foreign tenants all yield no hits (never an existence hint).
    pub fn search(
        &self,
        query_embedding: &[f32],
        user: &UserContext,
        limit: usize,
    ) -> Vec<SearchHit> {
        if limit == 0
            || query_embedding.is_empty()
            || query_embedding.iter().any(|value| !value.is_finite())
        {
            return Vec::new();
        }
        if self
            .dimension
            .is_some_and(|dimension| dimension != query_embedding.len())
        {
            return Vec::new();
        }

        let organization = user.organization_id.map(|id| id.to_string());
        let mut hits: Vec<SearchHit> = self
            .points
            .iter()
            .filter(|(_, point)| organization.as_deref() == Some(point.tenant_id.as_str()))
            .filter(|(_, point)| point.acl.allows(user))
            .filter_map(|(id, point)| {
                let score = cosine(query_embedding, &point.vector);
                if score > 0.0 {
                    Some(SearchHit {
                        id: id.clone(),
                        source: point.source.clone(),
                        section: point.section.clone(),
                        score,
                    })
                } else {
                    None
                }
            })
            .collect();

        hits.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.id.cmp(&right.id))
        });
        hits.truncate(limit);
        hits
    }

    /// Removes points whose source path or stable id starts with the prefix.
    ///
    /// Administrative operation with no ACL filter: callers must pass a
    /// tenant-scoped prefix (e.g. the document path of a deleted/archived
    /// source). Returns the number of removed points.
    pub fn remove(&mut self, source_prefix: &str) -> usize {
        if source_prefix.is_empty() {
            return 0;
        }
        let before = self.points.len();
        self.points.retain(|id, point| {
            !id.starts_with(source_prefix) && !point.source_path.starts_with(source_prefix)
        });
        before - self.points.len()
    }
}

fn read_string<'a>(
    payload: &'a HashMap<String, serde_json::Value>,
    field: &'static str,
    id: &str,
) -> Result<&'a str, IndexError> {
    match payload.get(field).and_then(serde_json::Value::as_str) {
        Some(value) if !value.trim().is_empty() => Ok(value),
        _ => Err(IndexError::MissingPayloadField {
            id: id.to_string(),
            field,
        }),
    }
}

fn read_value<T: serde::de::DeserializeOwned>(
    payload: &HashMap<String, serde_json::Value>,
    field: &'static str,
    id: &str,
) -> Result<T, IndexError> {
    let value = payload
        .get(field)
        .ok_or_else(|| IndexError::MissingPayloadField {
            id: id.to_string(),
            field,
        })?;
    serde_json::from_value(value.clone()).map_err(|_| IndexError::InvalidPayloadField {
        id: id.to_string(),
        field,
    })
}

fn cosine(left: &[f32], right: &[f32]) -> f32 {
    if left.len() != right.len() {
        return 0.0;
    }
    let mut dot = 0.0_f64;
    let mut left_norm = 0.0_f64;
    let mut right_norm = 0.0_f64;
    for (left_value, right_value) in left.iter().zip(right.iter()) {
        let left_value = f64::from(*left_value);
        let right_value = f64::from(*right_value);
        dot += left_value * right_value;
        left_norm += left_value * left_value;
        right_norm += right_value * right_value;
    }
    if left_norm == 0.0 || right_norm == 0.0 {
        return 0.0;
    }
    (dot / (left_norm.sqrt() * right_norm.sqrt())) as f32
}

pub mod adapter;

#[cfg(test)]
mod tests;
