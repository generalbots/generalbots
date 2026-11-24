use std::sync::Arc;

use botcore::kb::structured::UpsertPoint;
use botlib::traits::DriveRepository;

use crate::Document;
pub use botlib::db_pool::DbPool;

/// #2499 — seam between a successful Docs save and the runtime KB/vector
/// indexer. It is storage-agnostic: the runtime maps each [`UpsertPoint`] onto
/// its client request and fills the reserved embedding slot. The Docs save
/// owns the document bytes, so a missing or failing indexer must never turn a
/// successful save into an HTTP error.
pub type KbIndexer = Arc<dyn Fn(Vec<UpsertPoint>) -> Result<(), String> + Send + Sync>;

/// Default handle: accepts the projection and drops it. Used until a runtime
/// vector indexer is reachable from the Docs router (#2499).
pub fn noop_kb_indexer() -> KbIndexer {
    Arc::new(|_points| Ok(()))
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
    /// #2499 — optional projection of a saved document into the KB/vector
    /// index. Defaults to a no-op so saves never depend on index availability.
    pub kb_indexer: KbIndexer,
}

impl DocState {
    /// #2499 — project a successfully saved document into storage-agnostic
    /// upsert points and hand them to the runtime KB indexer.
    /// `build_upsert_points` is built on `build_structured_chunks`, so
    /// structure-aware addressing, the derived ACL and the bucket-scoped
    /// tenant all travel with the points.
    ///
    /// Strictly best-effort: the document is already durable when this runs,
    /// so a projection or index failure is logged and swallowed — it must
    /// never turn a successful save into an HTTP error.
    pub fn project_to_kb(&self, user_id: &str, doc_id: &str, title: &str, content: &str) {
        let source_ref = format!("{doc_id}.html");
        let input = crate::handlers_api::kb_index::DocumentIndexInput {
            document_id: doc_id,
            title,
            source_ref: &source_ref,
            version: 1,
            content,
            tenant_id: &self.bucket_name,
            owner_id: user_id,
        };
        let points = crate::handlers_api::kb_index::build_upsert_points(&input);
        if points.is_empty() {
            return;
        }
        if let Err(e) = (self.kb_indexer)(points) {
            log::warn!("botdocs: KB index projection for document {doc_id} failed: {e}");
        }
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
