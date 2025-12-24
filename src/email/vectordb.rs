use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
#[cfg(not(feature = "vectordb"))]
use tokio::fs;
use uuid::Uuid;

#[cfg(feature = "vectordb")]
use qdrant_client::{
    qdrant::{Distance, PointStruct, VectorParams},
    Qdrant,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDocument {
    pub id: String,
    pub account_id: String,
    pub from_email: String,
    pub from_name: String,
    pub to_email: String,
    pub subject: String,
    pub body_text: String,
    pub date: DateTime<Utc>,
    pub folder: String,
    pub has_attachments: bool,
    pub thread_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSearchQuery {
    pub query_text: String,
    pub account_id: Option<String>,
    pub folder: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSearchResult {
    pub email: EmailDocument,
    pub score: f32,
    pub snippet: String,
}

pub struct UserEmailVectorDB {
    user_id: Uuid,
    bot_id: Uuid,
    collection_name: String,
    db_path: PathBuf,
    #[cfg(feature = "vectordb")]
    client: Option<Arc<Qdrant>>,
}

impl UserEmailVectorDB {
    pub fn new(user_id: Uuid, bot_id: Uuid, db_path: PathBuf) -> Self {
        let collection_name = format!("emails_{}_{}", bot_id, user_id);

        Self {
            user_id,
            bot_id,
            collection_name,
            db_path,
            #[cfg(feature = "vectordb")]
            client: None,
        }
    }

    #[cfg(feature = "vectordb")]
    pub async fn initialize(&mut self, qdrant_url: &str) -> Result<()> {
        let client = Qdrant::from_url(qdrant_url).build()?;

        let collections = client.list_collections().await?;
        let exists = collections
            .collections
            .iter()
            .any(|c| c.name == self.collection_name);

        if !exists {
            client
                .create_collection(
                    qdrant_client::qdrant::CreateCollectionBuilder::new(&self.collection_name)
                        .vectors_config(VectorParams {
                            size: 1536,
                            distance: Distance::Cosine.into(),
                            ..Default::default()
                        }),
                )
                .await?;

            log::info!("Created email vector collection: {}", self.collection_name);
        }

        self.client = Some(Arc::new(client));
        Ok(())
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn initialize(&mut self, _qdrant_url: &str) -> Result<()> {
        log::warn!("Vector DB feature not enabled, using fallback storage");
        Ok(())
    }

    #[cfg(feature = "vectordb")]
    pub async fn index_email(&self, email: &EmailDocument, embedding: Vec<f32>) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

        let payload: qdrant_client::Payload = serde_json::to_value(email)?
            .as_object()
            .map(|m| m.clone())
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| (k, qdrant_client::qdrant::Value::from(v.to_string())))
            .collect::<std::collections::HashMap<_, _>>()
            .into();

        let point = PointStruct::new(email.id.clone(), embedding, payload);

        client
            .upsert_points(qdrant_client::qdrant::UpsertPointsBuilder::new(
                &self.collection_name,
                vec![point],
            ))
            .await?;

        log::debug!("Indexed email: {} - {}", email.id, email.subject);
        Ok(())
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn index_email(&self, email: &EmailDocument, _embedding: Vec<f32>) -> Result<()> {
        let file_path = self.db_path.join(format!("{}.json", email.id));
        let json = serde_json::to_string_pretty(email)?;
        fs::write(file_path, json).await?;
        Ok(())
    }

    pub async fn index_emails_batch(&self, emails: &[(EmailDocument, Vec<f32>)]) -> Result<()> {
        for (email, embedding) in emails {
            self.index_email(email, embedding.clone()).await?;
        }
        Ok(())
    }

    #[cfg(feature = "vectordb")]
    pub async fn search(
        &self,
        query: &EmailSearchQuery,
        query_embedding: Vec<f32>,
    ) -> Result<Vec<EmailSearchResult>> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

        let filter = if query.account_id.is_some() || query.folder.is_some() {
            let mut conditions = vec![];

            if let Some(account_id) = &query.account_id {
                conditions.push(qdrant_client::qdrant::Condition::matches(
                    "account_id",
                    account_id.clone(),
                ));
            }

            if let Some(folder) = &query.folder {
                conditions.push(qdrant_client::qdrant::Condition::matches(
                    "folder",
                    folder.clone(),
                ));
            }

            Some(qdrant_client::qdrant::Filter::must(conditions))
        } else {
            None
        };

        let mut search_builder = qdrant_client::qdrant::SearchPointsBuilder::new(
            &self.collection_name,
            query_embedding,
            query.limit as u64,
        )
        .with_payload(true);

        if let Some(f) = filter {
            search_builder = search_builder.filter(f);
        }

        let search_result = client.search_points(search_builder).await?;

        let mut results = Vec::new();
        for point in search_result.result {
            let payload = &point.payload;
            if !payload.is_empty() {
                let get_str = |key: &str| -> String {
                    payload
                        .get(key)
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_default()
                };

                let email = EmailDocument {
                    id: get_str("id"),
                    account_id: get_str("account_id"),
                    from_email: get_str("from_email"),
                    from_name: get_str("from_name"),
                    to_email: get_str("to_email"),
                    subject: get_str("subject"),
                    body_text: get_str("body_text"),
                    date: chrono::Utc::now(),
                    folder: get_str("folder"),
                    has_attachments: false,
                    thread_id: payload
                        .get("thread_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                };

                let snippet = if email.body_text.len() > 200 {
                    format!("{}...", &email.body_text[..200])
                } else {
                    email.body_text.clone()
                };

                results.push(EmailSearchResult {
                    email,
                    score: point.score,
                    snippet,
                });
            }
        }

        Ok(results)
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn search(
        &self,
        query: &EmailSearchQuery,
        _query_embedding: Vec<f32>,
    ) -> Result<Vec<EmailSearchResult>> {
        let mut results = Vec::new();
        let mut entries = fs::read_dir(&self.db_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(entry.path()).await?;
                if let Ok(email) = serde_json::from_str::<EmailDocument>(&content) {
                    let query_lower = query.query_text.to_lowercase();
                    if email.subject.to_lowercase().contains(&query_lower)
                        || email.body_text.to_lowercase().contains(&query_lower)
                        || email.from_email.to_lowercase().contains(&query_lower)
                    {
                        let snippet = if email.body_text.len() > 200 {
                            format!("{}...", &email.body_text[..200])
                        } else {
                            email.body_text.clone()
                        };

                        results.push(EmailSearchResult {
                            email,
                            score: 1.0,
                            snippet,
                        });
                    }
                }

                if results.len() >= query.limit {
                    break;
                }
            }
        }

        Ok(results)
    }

    #[cfg(feature = "vectordb")]
    pub async fn delete_email(&self, email_id: &str) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

        client
            .delete_points(
                qdrant_client::qdrant::DeletePointsBuilder::new(&self.collection_name).points(
                    vec![qdrant_client::qdrant::PointId::from(email_id.to_string())],
                ),
            )
            .await?;

        log::debug!("Deleted email from index: {}", email_id);
        Ok(())
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn delete_email(&self, email_id: &str) -> Result<()> {
        let file_path = self.db_path.join(format!("{}.json", email_id));
        if file_path.exists() {
            fs::remove_file(file_path).await?;
        }
        Ok(())
    }

    #[cfg(feature = "vectordb")]
    pub async fn get_count(&self) -> Result<u64> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

        let info = client.collection_info(self.collection_name.clone()).await?;

        Ok(info.result.unwrap().points_count.unwrap_or(0))
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn get_count(&self) -> Result<u64> {
        let mut count = 0;
        let mut entries = fs::read_dir(&self.db_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                count += 1;
            }
        }

        Ok(count)
    }

    #[cfg(feature = "vectordb")]
    pub async fn clear(&self) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Vector DB not initialized"))?;

        client.delete_collection(&self.collection_name).await?;

        client
            .create_collection(
                qdrant_client::qdrant::CreateCollectionBuilder::new(&self.collection_name)
                    .vectors_config(VectorParams {
                        size: 1536,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    }),
            )
            .await?;

        log::info!("Cleared email vector collection: {}", self.collection_name);
        Ok(())
    }

    #[cfg(not(feature = "vectordb"))]
    pub async fn clear(&self) -> Result<()> {
        if self.db_path.exists() {
            fs::remove_dir_all(&self.db_path).await?;
            fs::create_dir_all(&self.db_path).await?;
        }
        Ok(())
    }
}

pub struct EmailEmbeddingGenerator {
    pub llm_endpoint: String,
}

impl EmailEmbeddingGenerator {
    pub fn new(llm_endpoint: String) -> Self {
        Self { llm_endpoint }
    }

    pub async fn generate_embedding(&self, email: &EmailDocument) -> Result<Vec<f32>> {
        let text = format!(
            "From: {} <{}>\nSubject: {}\n\n{}",
            email.from_name, email.from_email, email.subject, email.body_text
        );

        let text = if text.len() > 8000 {
            &text[..8000]
        } else {
            &text
        };

        self.generate_text_embedding(text).await
    }

    pub async fn generate_text_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let embedding_url = "http://localhost:8082".to_string();
        self.generate_local_embedding(text, &embedding_url).await
    }

    async fn generate_openai_embedding(&self, text: &str, api_key: &str) -> Result<Vec<f32>> {
        use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
        use serde_json::json;

        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", api_key))?,
        );

        let body = json!({
            "input": text,
            "model": "text-embedding-3-small"
        });

        let response = client
            .post("https://api.openai.com/v1/embeddings")
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("OpenAI API error: {}", response.status()));
        }

        let result: serde_json::Value = response.json().await?;
        let embedding = result["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid OpenAI response format"))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(embedding)
    }

    async fn generate_local_embedding(&self, text: &str, embedding_url: &str) -> Result<Vec<f32>> {
        use serde_json::json;

        let client = reqwest::Client::new();
        let body = json!({
            "text": text,
            "model": "sentence-transformers/all-MiniLM-L6-v2"
        });

        let response = client.post(embedding_url).json(&body).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Local embedding service error: {}",
                response.status()
            ));
        }

        let result: serde_json::Value = response.json().await?;
        let embedding = result["embedding"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid embedding response format"))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(embedding)
    }

    fn generate_hash_embedding(&self, text: &str) -> Result<Vec<f32>> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        const EMBEDDING_DIM: usize = 1536;
        let mut embedding = vec![0.0f32; EMBEDDING_DIM];

        let words: Vec<&str> = text.split_whitespace().collect();

        for (i, chunk) in words.chunks(10).enumerate() {
            let mut hasher = DefaultHasher::new();
            chunk.join(" ").hash(&mut hasher);
            let hash = hasher.finish();

            for j in 0..64 {
                let idx = (i * 64 + j) % EMBEDDING_DIM;
                let value = ((hash >> j) & 1) as f32;
                embedding[idx] += value;
            }
        }

        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }

        Ok(embedding)
    }
}
