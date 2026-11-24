use std::sync::{Arc, OnceLock};

use botcore::kb::permissions::UserContext;
use botcore::kb::structured::{
    stable_id, InMemoryKbIndex, SearchHit, SourceRef, StructuredChunk, UpsertPoint, VectorSlot,
    DEFAULT_VECTOR_DIMENSION,
};
use botcore::shared::analytics::MetricsCollector;
use botlib::traits::DriveRepository;
use serde::Deserialize;

use crate::Document;
pub use botlib::db_pool::DbPool;

/// #2459 — runtime-wide metrics handle. The runtime sets it once at boot with
/// the collector the exporter scrapes, so every DocState shares it.
static METRICS: OnceLock<MetricsCollector> = OnceLock::new();

/// Install the shared office-metrics collector (idempotent, called at boot).
pub fn set_metrics_collector(collector: MetricsCollector) {
    if METRICS.set(collector).is_err() {
        log::warn!("botdocs: metrics collector already initialized");
    }
}

/// #2499 — seam between a successful Docs save and the runtime KB/vector
/// indexer. It is storage-agnostic: the runtime maps each [`UpsertPoint`] onto
/// its client request and fills the reserved embedding slot. The Docs save
/// owns the document bytes, so a missing or failing indexer must never turn a
/// successful save into an HTTP error.
pub type KbIndexer = Arc<dyn Fn(Vec<UpsertPoint>) -> Result<(), String> + Send + Sync>;

/// Test-only handle: accepts the projection and drops it. Production uses
/// [`in_memory_kb_indexer`] so saves are actually searchable (#2499).
pub fn noop_kb_indexer() -> KbIndexer {
    Arc::new(|_points| Ok(()))
}

/// #2499 — the document identity a citation points at: `Citation.document_id`,
/// `Citation.file` and `Citation.version`. `SearchHit` deliberately drops these
/// three fields, so retrieval reads them from the projection-time map instead.
/// Only metadata is kept here — document bytes never enter this struct.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KbCitationMeta {
    pub document_id: String,
    pub file: String,
    pub version: String,
}

/// Deserialized only from `payload["citation"]`; unknown keys (including the
/// full `source`/`heading_path`) are ignored, and the sibling `payload["content"]`
/// is never read. This is what keeps the metadata map citation-only.
#[derive(Debug, Deserialize)]
struct CitationFields {
    document_id: String,
    file: String,
    version: String,
}

impl KbCitationMeta {
    fn from_payload(payload: &std::collections::HashMap<String, serde_json::Value>) -> Option<Self> {
        let citation = payload.get("citation")?;
        let fields: CitationFields = serde_json::from_value(citation.clone()).ok()?;
        if fields.document_id.trim().is_empty()
            || fields.file.trim().is_empty()
            || fields.version.trim().is_empty()
        {
            return None;
        }
        Some(Self {
            document_id: fields.document_id,
            file: fields.file,
            version: fields.version,
        })
    }
}

/// Per-index citation identity, keyed by stable point id. It is keyed by the
/// shared index's `Arc` address so two `DocState`s (e.g. two tenants in tests)
/// can never read each other's metadata, without changing `DocState`'s public
/// fields — an external integration test constructs it with a literal.
static KB_CITATIONS: OnceLock<std::sync::RwLock<std::collections::HashMap<usize, CitationMap>>> =
    OnceLock::new();

type CitationMap = std::collections::HashMap<String, KbCitationMeta>;

fn citation_map() -> &'static std::sync::RwLock<std::collections::HashMap<usize, CitationMap>> {
    KB_CITATIONS.get_or_init(|| std::sync::RwLock::new(std::collections::HashMap::new()))
}

fn index_key(index: &Arc<tokio::sync::RwLock<InMemoryKbIndex>>) -> usize {
    Arc::as_ptr(index) as usize
}

/// Dependency-free lexical embedding used to fill the reserved vector slot of
/// a projected point. It is deliberately local: no network or paid model call,
/// and the query side uses the same function, so a saved chunk is searchable
/// in-process. A runtime semantic embedder can replace the slot later.
pub fn lexical_embedding(text: &str, dimension: usize) -> Vec<f32> {
    let mut vector = vec![0.0_f32; dimension];
    if dimension == 0 {
        return vector;
    }
    for token in text.split_whitespace() {
        let token = token.trim_matches(|character: char| !character.is_alphanumeric());
        if token.is_empty() {
            continue;
        }
        let hash = fnv1a(&token.to_lowercase());
        vector[(hash as usize) % dimension] += 1.0;
    }
    if vector.iter().all(|value| *value == 0.0) {
        vector[0] = 1.0;
    }
    vector
}

fn fnv1a(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// #2499 — real consumer of the Docs projection: fills each reserved vector
/// slot with a local embedding and inserts the points into the shared
/// in-memory index. Insertion is best effort; a busy lock is deferred to the
/// async runtime instead of dropping the projection, and a validation failure
/// is reported to `project_to_kb` (which logs and swallows it).
pub fn in_memory_kb_indexer(kb_index: Arc<tokio::sync::RwLock<InMemoryKbIndex>>) -> KbIndexer {
    let key = index_key(&kb_index);
    Arc::new(move |points| {
        // Capture the citation identity from the payload BEFORE the index
        // consumes the point. `InMemoryKbIndex` keeps only `file` (as a private
        // removal key) and drops `document_id`/`version` from every `SearchHit`.
        for point in &points {
            if let Some(meta) = KbCitationMeta::from_payload(&point.payload) {
                let mut all = match citation_map().write() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                all.entry(key).or_default().insert(point.id.clone(), meta);
            }
        }
        let mut filled = Vec::with_capacity(points.len());
        for mut point in points {
            if !point.vector.is_ready() {
                let text = point
                    .payload
                    .get("content")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| point.id.clone());
                let embedding = lexical_embedding(&text, point.vector.dimension);
                point.vector = VectorSlot::filled(embedding);
            }
            filled.push(point);
        }
        match kb_index.try_write() {
            Ok(mut index) => index
                .insert(filled)
                .map(|_| ())
                .map_err(|e| format!("in-memory KB index rejected projection: {e}")),
            Err(_) => match tokio::runtime::Handle::try_current() {
                Ok(handle) => {
                    let index = kb_index.clone();
                    drop(handle.spawn(async move {
                        match index.write().await.insert(filled) {
                            Ok(_) => {}
                            Err(e) => log::warn!("botdocs: deferred KB index insert failed: {e}"),
                        }
                    }));
                    Ok(())
                }
                Err(_) => {
                    Err("in-memory KB index is busy and no async runtime is available".into())
                }
            },
        }
    })
}

/// Per-document undo/redo snapshot stacks (session-lifetime, capped).
#[derive(Default)]
pub struct DocHistory {
    current: Option<Document>,
    undo: std::collections::VecDeque<Document>,
    redo: std::collections::VecDeque<Document>,
}

const DOC_HISTORY_CAP: usize = 20;

impl DocHistory {
    /// Called on every save: archives the previous version.
    pub fn capture(&mut self, doc: Document) {
        let unchanged = self
            .current
            .as_ref()
            .map(|c| c.id == doc.id && c.content == doc.content)
            .unwrap_or(false);
        if unchanged {
            self.current = Some(doc);
            return;
        }
        if let Some(previous) = self.current.take() {
            if self.undo.len() == DOC_HISTORY_CAP {
                self.undo.pop_front();
            }
            self.undo.push_back(previous);
            self.redo.clear();
        }
        self.current = Some(doc);
    }

    pub fn undo(&mut self) -> Option<Document> {
        let previous = self.undo.pop_back()?;
        let restored = self.current.replace(previous.clone());
        if let Some(current) = restored {
            if self.redo.len() == DOC_HISTORY_CAP {
                self.redo.pop_front();
            }
            self.redo.push_back(current);
        }
        self.current.clone()
    }

    pub fn redo(&mut self) -> Option<Document> {
        let next = self.redo.pop_back()?;
        let restored = self.current.replace(next.clone());
        if let Some(current) = restored {
            if self.undo.len() == DOC_HISTORY_CAP {
                self.undo.pop_front();
            }
            self.undo.push_back(current);
        }
        self.current.clone()
    }
}

pub struct DocState {
    pub pool: Arc<DbPool>,
    pub drive: Arc<dyn DriveRepository>,
    pub bucket_name: String,
    /// History keyed by document id; added for server-side undo/redo
    /// (#1138). Uses interior mutability so fragment handlers can share it.
    pub history: tokio::sync::Mutex<std::collections::HashMap<String, DocHistory>>,
    /// #2499 — projection into the KB index; a save never fails if it is absent.
    pub kb_indexer: KbIndexer,
    /// #2499 — the in-process index shared by the indexer and the search side.
    pub kb_index: Arc<tokio::sync::RwLock<InMemoryKbIndex>>,
    /// #2459 — shared office metrics (export duration).
    pub metrics_collector: Arc<MetricsCollector>,
}

impl DocState {
    /// Production constructor: projection and retrieval share one KB index (#2499).
    pub fn with_in_memory_kb_index(
        pool: Arc<DbPool>,
        drive: Arc<dyn DriveRepository>,
        bucket_name: String,
        metrics_collector: Arc<MetricsCollector>,
    ) -> Self {
        let kb_index = Arc::new(tokio::sync::RwLock::new(InMemoryKbIndex::new()));
        let kb_indexer = in_memory_kb_indexer(kb_index.clone());
        Self {
            pool,
            drive,
            bucket_name,
            history: tokio::sync::Mutex::new(std::collections::HashMap::new()),
            kb_indexer,
            kb_index,
            metrics_collector,
        }
    }

    /// #2499 — ACL-filtered cosine search over the index populated by saves.
    /// A busy lock fails closed: no result is better than an unchecked result.
    pub fn search_index(
        &self,
        query_embedding: &[f32],
        user: &UserContext,
        limit: usize,
    ) -> Vec<SearchHit> {
        match self.kb_index.try_read() {
            Ok(index) => index.search(query_embedding, user, limit),
            Err(_) => {
                log::warn!("botdocs: in-memory KB index busy during search; failing closed");
                Vec::new()
            }
        }
    }

    /// Convenience for in-process consumers: embed `query` with the same
    /// lexical embedding used when the points were indexed.
    pub fn search_text(&self, query: &str, user: &UserContext, limit: usize) -> Vec<SearchHit> {
        self.search_index(
            &lexical_embedding(query, DEFAULT_VECTOR_DIMENSION),
            user,
            limit,
        )
    }

    /// #2499 — document identity for an already permission-filtered hit, keyed
    /// by the stable point id. Missing metadata yields empty strings: never a
    /// guess, never document bytes, never another tenant's data. The caller
    /// reaches this only for hits the tenant/ACL filter already released.
    pub fn citation_meta(&self, id: &str) -> KbCitationMeta {
        let all = match citation_map().read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let map = match all.get(&index_key(&self.kb_index)) {
            Some(map) => map,
            None => return KbCitationMeta::default(),
        };
        match map.get(id) {
            Some(meta) => meta.clone(),
            None => KbCitationMeta::default(),
        }
    }

    /// #2499 — stable tenant UUID for a Drive bucket, so writer and reader
    /// always agree on named buckets (`default.gbai`) and real UUIDs.
    pub fn tenant_uuid(bucket: &str) -> uuid::Uuid {
        match uuid::Uuid::parse_str(bucket) {
            Ok(id) => id,
            Err(_) => uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, bucket.as_bytes()),
        }
    }

    /// Strictly best-effort: the document is already durable when this runs,
    /// so a projection or index failure is logged and swallowed — it must
    /// never turn a successful save into an HTTP error.
    pub fn project_to_kb(&self, user_id: &str, doc_id: &str, title: &str, content: &str) {
        // #2499 — an edit re-projects the document; drop the previous chunks first
        // so removed/renamed clauses stop matching (the browser UAT found stale hits).
        self.remove_from_kb(doc_id);
        let source_ref = format!("{doc_id}.html");
        let tenant = Self::tenant_uuid(&self.bucket_name).to_string();
        let input = crate::handlers_api::kb_index::DocumentIndexInput {
            document_id: doc_id,
            title,
            source_ref: &source_ref,
            version: 1,
            content,
            tenant_id: &tenant,
            owner_id: user_id,
        };
        let mut points = crate::handlers_api::kb_index::build_upsert_points(&input);
        if points.is_empty() {
            // A single "Clause 7.3: ..." paragraph is consumed as a heading by
            // the chunker; keep such non-empty documents searchable (#2499).
            let d = crate::handlers_api::kb_index::build_index_document(&input);
            if !d.text.trim().is_empty() {
                let id = stable_id(&[&d.tenant_id, &d.document_id, &d.version, "all", &d.text]);
                let chunk = StructuredChunk {
                    id,
                    document_id: d.document_id,
                    file: d.file,
                    version: d.version,
                    tenant_id: d.tenant_id,
                    heading_path: Vec::new(),
                    source: SourceRef::page(1),
                    content: d.text,
                    acl: d.acl,
                };
                points.push(chunk.to_upsert_point(DEFAULT_VECTOR_DIMENSION));
            }
        }
        if let Err(e) = (self.kb_indexer)(points) {
            log::warn!("botdocs: KB index projection for document {doc_id} failed: {e}");
        }
    }

    /// #2499 — drop a deleted document's `{doc_id}.html` points; busy locks are logged, not fatal.
    pub fn remove_from_kb(&self, doc_id: &str) {
        if doc_id.trim().is_empty() {
            return;
        }
        let source_prefix = format!("{doc_id}.html");
        match self.kb_index.try_write() {
            Ok(mut index) => {
                let removed = index.remove(&source_prefix);
                log::debug!("botdocs: removed {removed} KB points for {doc_id}");
            }
            Err(_) => log::warn!("botdocs: KB index busy; removal for {doc_id} skipped"),
        }
        // Drop stale citation identity too; a re-projected document gets fresh
        // ids, so old entries would otherwise be unreachable but retained.
        let mut all = match citation_map().write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(map) = all.get_mut(&index_key(&self.kb_index)) {
            map.retain(|_, meta| meta.file != source_prefix);
        }
    }

    /// #2499 — replay persisted documents from Drive into the process-local
    /// index on boot. Best effort; returns the number re-indexed.
    pub async fn reindex_persisted_documents(&self) -> usize {
        crate::handlers_api::kb_backfill::reindex_persisted_documents(self).await
    }
}

#[cfg(test)]
mod history_tests {
    use super::*;
    use crate::types_core::Document;

    fn doc(content: &str) -> Document {
        crate::handlers_api::history::test_document(content)
    }

    #[test]
    fn undo_redo_round_trip() {
        let mut h = DocHistory::default();
        h.capture(doc("v1"));
        h.capture(doc("v2"));
        assert_eq!(h.undo().unwrap().content, "v1");
        assert_eq!(h.redo().unwrap().content, "v2");
        assert!(h.redo().is_none());
    }

    #[test]
    fn unchanged_capture_is_noop() {
        let mut h = DocHistory::default();
        h.capture(doc("v1"));
        h.capture(doc("v1"));
        assert!(h.undo().is_none());
    }

    #[test]
    fn undo_clears_redo_branch() {
        let mut h = DocHistory::default();
        h.capture(doc("v1"));
        h.capture(doc("v2"));
        assert_eq!(h.undo().unwrap().content, "v1");
        h.capture(doc("v3")); // branch: redo stack must clear
        assert!(h.redo().is_none());
        assert_eq!(h.undo().unwrap().content, "v1");
    }
}

#[cfg(test)]
#[path = "state_kb_tests.rs"]
mod state_kb_tests;
