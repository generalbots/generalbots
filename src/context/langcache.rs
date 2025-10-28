use crate::kb::qdrant_client::{ensure_collection_exists, VectorDBClient, QdrantPoint};
use std::error::Error;

/// LangCache client – currently a thin wrapper around the existing Qdrant client,
/// allowing future replacement with a dedicated LangCache SDK or API without
/// changing the rest of the codebase.
pub struct LLMCacheClient {
    inner: VectorDBClient,
}

impl LLMCacheClient {
    /// Create a new LangCache client.
    /// This client uses the internal Qdrant client with the default QDRANT_URL.
    /// No external environment variable is required.
    pub fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        // Use the same URL as the Qdrant client (default or from QDRANT_URL env)
        let qdrant_url = std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string());
        Ok(Self {
            inner: VectorDBClient::new(qdrant_url),
        })
    }
    

    /// Ensure a collection exists in LangCache.
    pub async fn ensure_collection_exists(
        &self,
        collection_name: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Reuse the Qdrant helper – LangCache uses the same semantics.
        ensure_collection_exists(&crate::shared::state::AppState::default(), collection_name).await
    }

    /// Search for similar vectors in a LangCache collection.
    pub async fn search(
        &self,
        collection_name: &str,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<QdrantPoint>, Box<dyn Error + Send + Sync>> {
        // Forward to the inner Qdrant client and map results to QdrantPoint.
        let results = self.inner.search(collection_name, query_vector, limit).await?;
        // Convert SearchResult to QdrantPoint (payload and vector may be None)
        let points = results
            .into_iter()
            .map(|res| QdrantPoint {
                id: res.id,
                vector: res.vector.unwrap_or_default(),
                payload: res.payload.unwrap_or_else(|| serde_json::json!({})),
            })
            .collect();
        Ok(points)
    }

    /// Upsert points (prompt/response pairs) into a LangCache collection.
    pub async fn upsert_points(
        &self,
        collection_name: &str,
        points: Vec<QdrantPoint>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.inner.upsert_points(collection_name, points).await
    }
}

/// Helper to obtain a LangCache client from the application state.
pub fn get_langcache_client() -> Result<LLMCacheClient, Box<dyn Error + Send + Sync>> {
    LLMCacheClient::new()
}
