use super::*;

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

        let payload: botqdrant::Payload = serde_json::to_value(file)?
            .as_object()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v.to_string())))
            .collect::<serde_json::Map<String, serde_json::Value>>();

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

            let points: Vec<PointStruct> = files
                .iter()
                .filter_map(|(file, embedding)| {
                    serde_json::to_value(file).ok().and_then(|v| {
                        v.as_object().map(|m| {
                            let payload: botqdrant::Payload = m
                                .clone()
                                .into_iter()
                                .map(|(k, v)| (k, serde_json::Value::String(v.to_string())))
                                .collect::<serde_json::Map<String, serde_json::Value>>();
                            PointStruct::new(file.id.clone(), embedding.clone(), payload)
                        })
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

        let filter =
            if query.bucket.is_some() || query.file_type.is_some() || !query.tags.is_empty() {
                let mut conditions = Vec::new();

                if let Some(bucket) = &query.bucket {
                    conditions.push(botqdrant::Condition::matches(
                        "bucket",
                        serde_json::Value::String(bucket.clone()),
                    ));
                }

                if let Some(file_type) = &query.file_type {
                    conditions.push(botqdrant::Condition::matches(
                        "file_type",
                        serde_json::Value::String(file_type.clone()),
                    ));
                }

                for tag in &query.tags {
                    conditions.push(botqdrant::Condition::matches(
                        "tags",
                        serde_json::Value::String(tag.clone()),
                    ));
                }

                if conditions.is_empty() {
                    None
                } else {
                    Some(botqdrant::Filter::must(conditions))
                }
            } else {
                None
            };

        let mut search_builder = botqdrant::SearchPointsBuilder::new(
            &self.collection_name,
            query_embedding,
            query.limit,
        )
        .with_payload(true);

        if let Some(f) = filter {
            search_builder = search_builder.filter(Some(f));
        }

        let search_result = search_builder.build(client).await?;

        let mut results = Vec::new();
        for point in search_result.result {
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
