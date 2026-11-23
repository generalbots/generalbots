use super::*;

#[cfg(feature = "vectordb")]
use botqdrant::drive_vectordb::{chunk_is_authorized, AccessVerifier, VectorScope};
#[cfg(feature = "vectordb")]
use std::sync::OnceLock;

/// Process-wide current-ACL verifier for Drive vector search (#2820). Installed
/// once at startup from a trusted source (Drive/MinIO object + `drive_shares`).
/// The first installation wins; a second attempt is returned for logging.
#[cfg(feature = "vectordb")]
static ACCESS_VERIFIER: OnceLock<Arc<AccessVerifier>> = OnceLock::new();

#[cfg(feature = "vectordb")]
pub fn set_drive_access_verifier(verifier: Arc<AccessVerifier>) -> Result<(), Arc<AccessVerifier>> {
    ACCESS_VERIFIER.set(verifier)
}

impl UserDriveVectorDB {
    pub fn new(user_id: Uuid, bot_id: Uuid, db_path: PathBuf) -> Self {
        let collection_name = format!("drive_{}_{}", bot_id, user_id);

        Self {
            user_id,
            bot_id,
            collection_name,
            db_path,
            #[cfg(feature = "vectordb")]
            client: None,
        }
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn bot_id(&self) -> Uuid {
        self.bot_id
    }

    pub fn collection_name(&self) -> &str {
        &self.collection_name
    }

    pub fn db_path(&self) -> &std::path::Path {
        &self.db_path
    }

    #[cfg(feature = "vectordb")]
    pub async fn initialize(&mut self, qdrant_url: &str) -> Result<()> {
        log::trace!(
            "Initializing vectordb, fallback path: {}",
            self.db_path.display()
        );
        let client = Qdrant::from_url(qdrant_url).build()?;

        let collections = client.list_collections().await?;
        let exists = {
            let collections_guard = collections.collections;
            collections_guard
                .iter()
                .any(|c| c.name == self.collection_name)
        };

        if !exists {
            botqdrant::CreateCollectionBuilder::new(&self.collection_name)
                .vectors_config(VectorParams {
                    size: 1536,
                    distance: Distance::Cosine,
                })
                .build(&client)
                .await?;

            log::info!("Initialized vector DB collection: {}", self.collection_name);
        }

        self.client = Some(Arc::new(client));
        Ok(())
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn initialize(&mut self, _qdrant_url: &str) -> Result<()> {
        log::warn!("Vector DB feature not enabled, using fallback storage");
        fs::create_dir_all(&self.db_path).await?;
        Ok(())
    }

    #[cfg(feature = "vectordb")]
    pub async fn index_file(&self, file: &FileDocument, embedding: Vec<f32>) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

        let scope = VectorScope::new(self.bot_id, self.user_id);
        let payload = file_payload(file, &scope)?;

        let point = PointStruct::new(file.id.clone(), embedding, payload);

        botqdrant::UpsertPointsBuilder::new(&self.collection_name, vec![point])
            .build(client)
            .await?;

        log::debug!("Indexed file: {} - {}", file.id, file.file_name);
        Ok(())
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn index_file(&self, file: &FileDocument, _embedding: Vec<f32>) -> Result<()> {
        let file_path = self.db_path.join(format!("{}.json", file.id));
        let json = serde_json::to_string_pretty(file)?;
        fs::write(file_path, json).await?;
        Ok(())
    }

    pub async fn index_files_batch(&self, files: &[(FileDocument, Vec<f32>)]) -> Result<()> {
        #[cfg(feature = "vectordb")]
        {
            let client = self
                .client
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

            let scope = VectorScope::new(self.bot_id, self.user_id);
            let points: Vec<PointStruct> = files
                .iter()
                .filter_map(|(file, embedding)| {
                    file_payload(file, &scope).ok().map(|payload| {
                        PointStruct::new(file.id.clone(), embedding.clone(), payload)
                    })
                })
                .collect();

            if !points.is_empty() {
                botqdrant::UpsertPointsBuilder::new(&self.collection_name, points)
                    .build(client)
                    .await?;
            }
        }

        #[cfg(not(feature = "vectordb"))]
        {
            for (file, embedding) in files {
                self.index_file(file, embedding.clone()).await?;
            }
        }

        Ok(())
    }

    #[cfg(feature = "vectordb")]
    pub async fn search(
        &self,
        query: &FileSearchQuery,
        query_embedding: Vec<f32>,
    ) -> Result<Vec<FileSearchResult>> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

        // One effective Qdrant filter: trusted tenant scope (#2820) plus the
        // caller's per-file conditions (#2819) in a single `must`. The native
        // `match.value` shape is required by Qdrant; `Condition::matches` used
        // to emit a flat `value` that never matched, cancelling the scope and
        // ACL rechecks below by returning zero points.
        let scope = VectorScope::new(self.bot_id, self.user_id);
        let filter = build_effective_filter(&scope, query);
        let points = client
            .search_points(&self.collection_name, &query_embedding, query.limit, Some(filter))
            .await?;

        let mut results = Vec::new();
        for point in points {
            let payload = point
                .get("payload")
                .and_then(|p| p.as_object())
                .cloned()
                .unwrap_or_default();
            if !payload.is_empty() {
                let get_str = |key: &str| -> String {
                    payload
                        .get(key)
                        .and_then(|v: &serde_json::Value| v.as_str())
                        .map(|s: &str| s.to_string())
                        .unwrap_or_default()
                };

                let file = FileDocument {
                    id: get_str("id"),
                    file_path: get_str("file_path"),
                    file_name: get_str("file_name"),
                    file_type: get_str("file_type"),
                    file_size: payload
                        .get("file_size")
                        .and_then(|v: &serde_json::Value| v.as_i64())
                        .unwrap_or(0) as u64,
                    bucket: get_str("bucket"),
                    content_text: get_str("content_text"),
                    content_summary: payload
                        .get("content_summary")
                        .and_then(|v: &serde_json::Value| v.as_str())
                        .map(|s: &str| s.to_string()),
                    created_at: chrono::Utc::now(),
                    modified_at: chrono::Utc::now(),
                    indexed_at: chrono::Utc::now(),
                    mime_type: payload
                        .get("mime_type")
                        .and_then(|v: &serde_json::Value| v.as_str())
                        .map(|s: &str| s.to_string()),
                    tags: vec![],
                };

                // Re-check the trusted stamp and the current ACL before any
                // embedded text leaves the index (#2820).
                let access_verifier = ACCESS_VERIFIER.get().map(|v| v.as_ref());
                if !chunk_is_authorized(
                    &scope,
                    &payload,
                    &file.file_path,
                    &file.bucket,
                    access_verifier,
                ) {
                    log::warn!(
                        "Dropping Qdrant point '{}': server scope or current ACL rejected it",
                        file.id
                    );
                    continue;
                }

                let snippet = Self::create_snippet(&file.content_text, &query.query_text, 200);
                let highlights = Self::extract_highlights(&file.content_text, &query.query_text, 3);

                let score = point.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                results.push(FileSearchResult {
                    file,
                    score,
                    snippet,
                    highlights,
                });
            }
        }

        Ok(results)
    }
}

/// Build a point payload with native JSON values and the trusted scope stamp.
/// The old `Value::String(v.to_string())` double-encoded every field (`"x"` ->
/// `"\"x\""`), so no per-file Qdrant filter could ever match (#2819).
#[cfg(feature = "vectordb")]
fn file_payload(file: &FileDocument, scope: &VectorScope) -> Result<botqdrant::Payload> {
    let mut payload: botqdrant::Payload = serde_json::to_value(file)?
        .as_object()
        .cloned()
        .unwrap_or_default();
    scope.stamp(&mut payload);
    Ok(payload)
}

/// Build the single Qdrant filter for a Drive search (#2819): the trusted
/// server-derived scope plus the caller's per-file conditions, in the `must`
/// shape Qdrant's REST API accepts. Pure so the filter is testable offline.
#[cfg(feature = "vectordb")]
fn build_effective_filter(scope: &VectorScope, query: &FileSearchQuery) -> serde_json::Value {
    let mut filter = scope.as_qdrant_filter();
    if let Some(must) = filter.get_mut("must").and_then(serde_json::Value::as_array_mut) {
        if let Some(v) = &query.bucket { must.push(serde_json::json!({"key":"bucket","match":{"value":v}})); }
        if let Some(v) = &query.file_type { must.push(serde_json::json!({"key":"file_type","match":{"value":v}})); }
        for v in &query.tags { must.push(serde_json::json!({"key":"tags","match":{"value":v}})); }
    }
    filter
}

#[cfg(test)]
mod vector_collection_tests {
    use super::*;

    #[test]
    fn the_collection_name_scopes_bot_and_user() {
        let bot = Uuid::parse_str("11111111-1111-1111-1111-111111111111").expect("bot");
        let user = Uuid::parse_str("22222222-2222-2222-2222-222222222222").expect("user");
        let db = UserDriveVectorDB::new(user, bot, PathBuf::from("/tmp/vdb"));

        assert_eq!(db.collection_name(), format!("drive_{bot}_{user}"));
        assert_eq!(db.user_id(), user);
        assert_eq!(db.bot_id(), bot);
        assert_eq!(db.db_path(), std::path::Path::new("/tmp/vdb"));
    }

    #[test]
    fn two_users_of_the_same_bot_get_distinct_collections() {
        let bot = Uuid::nil();
        let first = UserDriveVectorDB::new(Uuid::new_v4(), bot, PathBuf::from("/tmp/a"));
        let second = UserDriveVectorDB::new(Uuid::new_v4(), bot, PathBuf::from("/tmp/b"));
        assert_ne!(first.collection_name(), second.collection_name());
    }

    #[test]
    fn the_same_user_on_two_bots_gets_distinct_collections() {
        let user = Uuid::new_v4();
        let first = UserDriveVectorDB::new(user, Uuid::new_v4(), PathBuf::from("/tmp/a"));
        let second = UserDriveVectorDB::new(user, Uuid::new_v4(), PathBuf::from("/tmp/b"));
        assert_ne!(first.collection_name(), second.collection_name());
    }

    #[test]
    fn the_collection_name_uses_only_qdrant_safe_characters() {
        let db = UserDriveVectorDB::new(Uuid::new_v4(), Uuid::new_v4(), PathBuf::from("/tmp/v"));
        // Qdrant accepts alphanumerics, underscore and hyphen; the UUID hyphens
        // are part of the scoped name.
        assert!(db
            .collection_name()
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'));
    }
}

#[cfg(test)]
mod scope_recheck_tests {
    use botqdrant::drive_vectordb::{chunk_is_authorized, AccessVerifier, VectorScope};
    use serde_json::Map;
    use uuid::Uuid;

    #[test]
    fn a_revoked_document_chunk_is_no_longer_returned() {
        let scope = VectorScope::new(Uuid::from_u128(1), Uuid::from_u128(2));
        let mut payload = Map::new();
        scope.stamp(&mut payload);

        // Trusted source initially grants access to this path. The verifier is
        // `'static`, so it owns an Arc clone of the grant state.
        let granted = std::sync::Arc::new(std::sync::Mutex::new(true));
        let g = std::sync::Arc::clone(&granted);
        let verifier = move |_scope: &VectorScope, _path: &str, _bucket: &str| {
            g.lock().is_ok_and(|v| *v)
        };
        assert!(chunk_is_authorized(
            &scope,
            &payload,
            "docs/a.pdf",
            "bucket",
            Some(&verifier)
        ));

        // Permission revoked; the stale Qdrant payload is unchanged.
        if let Ok(mut g) = granted.lock() {
            *g = false;
        }
        assert!(!chunk_is_authorized(
            &scope,
            &payload,
            "docs/a.pdf",
            "bucket",
            Some(&verifier)
        ));
    }

    #[test]
    fn a_foreign_tenant_payload_is_rejected_even_with_a_verifier() {
        let owner = VectorScope::new(Uuid::from_u128(1), Uuid::from_u128(2));
        let attacker = VectorScope::new(Uuid::from_u128(3), Uuid::from_u128(4));
        let mut payload = Map::new();
        owner.stamp(&mut payload);
        let permissive = |_scope: &VectorScope, _path: &str, _bucket: &str| true;
        assert!(!chunk_is_authorized(
            &attacker,
            &payload,
            "docs/a.pdf",
            "bucket",
            Some(&permissive)
        ));
    }
}

#[cfg(test)]
#[cfg(feature = "vectordb")]
mod filter_tests {
    use super::*;
    use serde_json::{json, Value};

    fn query() -> FileSearchQuery {
        FileSearchQuery {
            query_text: "q".into(), bucket: Some("finance".into()), file_type: Some("pdf".into()),
            date_from: None, date_to: None, tags: vec!["hr".into()], limit: 10,
        }
    }

    fn document() -> FileDocument {
        FileDocument {
            id: "f1".into(), file_path: "docs/a.pdf".into(), file_name: "a.pdf".into(),
            file_type: "pdf".into(), file_size: 1, bucket: "finance".into(),
            content_text: "x".into(), content_summary: None, mime_type: None,
            created_at: chrono::Utc::now(), modified_at: chrono::Utc::now(),
            indexed_at: chrono::Utc::now(), tags: vec!["hr".into()],
        }
    }

    /// The real search filter must be present, non-empty and combine the
    /// trusted scope with every caller-supplied condition (#2819).
    #[test]
    fn effective_filter_is_present_and_non_empty() {
        let scope = VectorScope::new(Uuid::from_u128(1), Uuid::from_u128(2));
        let filter = build_effective_filter(&scope, &query());
        let must = filter["must"].as_array().expect("must");
        assert_eq!(must.len(), 5, "{filter}");
        assert!(must.iter().any(|c| c["key"] == "scope_bot_id"));
        assert!(must.iter().any(|c| c["key"] == "scope_user_id"));
        assert!(must.iter().any(|c| c["key"] == "bucket" && c["match"]["value"] == "finance"));
        assert!(must.iter().any(|c| c["key"] == "file_type" && c["match"]["value"] == "pdf"));
        assert!(must.iter().any(|c| c["key"] == "tags" && c["match"]["value"] == "hr"));
    }

    /// Simulate Qdrant's `must` evaluation: a foreign bucket/tag document
    /// cannot satisfy the filter, so the query returns no results (#2819).
    #[test]
    fn a_foreign_bucket_and_tag_yield_no_results() {
        let scope = VectorScope::new(Uuid::from_u128(1), Uuid::from_u128(2));
        let filter = build_effective_filter(&scope, &query());
        let mut payload = serde_json::Map::new();
        scope.stamp(&mut payload);
        payload.insert("bucket".into(), Value::String("other".into()));
        payload.insert("file_type".into(), Value::String("png".into()));
        payload.insert("tags".into(), json!(["other"]));

        let must = filter["must"].as_array().expect("must");
        let matches = must.iter().all(|c| match c["key"].as_str() {
            Some("tags") => payload["tags"].as_array().is_some_and(|t| t.contains(&c["match"]["value"])),
            Some(key) => payload.get(key) == Some(&c["match"]["value"]),
            None => false,
        });
        assert!(!matches, "a foreign bucket/file_type/tag must yield no results");
    }

    /// The stored values must be native JSON, not `v.to_string()` wrapped
    /// strings, otherwise the filter conditions above can never match.
    #[test]
    fn indexed_payload_values_are_native_not_double_encoded() {
        let scope = VectorScope::new(Uuid::from_u128(1), Uuid::from_u128(2));
        let payload = file_payload(&document(), &scope).expect("payload");
        assert_eq!(payload["bucket"], Value::String("finance".into()));
        assert_eq!(payload["file_type"], Value::String("pdf".into()));
        assert_eq!(payload["tags"], json!(["hr"]));
        assert_eq!(payload["scope_bot_id"], Value::String(scope.bot_id.to_string()));
    }
}
