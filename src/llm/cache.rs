use async_trait::async_trait;
use log::{debug, info, trace};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

use super::LLMProvider;
use crate::shared::utils::estimate_token_count;

/// Configuration for semantic caching
#[derive(Clone)]
pub struct CacheConfig {
    /// TTL for cache entries in seconds
    pub ttl: u64,
    /// Whether to use semantic similarity matching
    pub semantic_matching: bool,
    /// Similarity threshold for semantic matching (0.0 to 1.0)
    pub similarity_threshold: f32,
    /// Maximum number of cache entries to check for similarity
    pub max_similarity_checks: usize,
    /// Cache key prefix
    pub key_prefix: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ttl: 3600, // 1 hour default
            semantic_matching: true,
            similarity_threshold: 0.95,
            max_similarity_checks: 100,
            key_prefix: "llm_cache".to_string(),
        }
    }
}

/// Cached LLM response with metadata
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedResponse {
    /// The actual response text
    pub response: String,
    /// The original prompt
    pub prompt: String,
    /// The messages/context used
    pub messages: Value,
    /// The model used
    pub model: String,
    /// Timestamp when cached
    pub timestamp: u64,
    /// Number of times this cache entry was hit
    pub hit_count: u32,
    /// Optional embedding vector for semantic similarity
    pub embedding: Option<Vec<f32>>,
}

/// LLM provider wrapper with caching capabilities
pub struct CachedLLMProvider {
    /// The underlying LLM provider
    provider: Arc<dyn LLMProvider>,
    /// Redis client for caching
    cache: Arc<redis::Client>,
    /// Cache configuration
    config: CacheConfig,
    /// Optional embedding service for semantic matching
    embedding_service: Option<Arc<dyn EmbeddingService>>,
}

/// Trait for embedding services
#[async_trait]
pub trait EmbeddingService: Send + Sync {
    async fn get_embedding(
        &self,
        text: &str,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>>;
    async fn compute_similarity(&self, embedding1: &[f32], embedding2: &[f32]) -> f32;
}

impl CachedLLMProvider {
    pub fn new(
        provider: Arc<dyn LLMProvider>,
        cache: Arc<redis::Client>,
        config: CacheConfig,
        embedding_service: Option<Arc<dyn EmbeddingService>>,
    ) -> Self {
        info!("Initializing CachedLLMProvider with semantic cache");
        info!(
            "Cache config: TTL={}s, Semantic={}, Threshold={}",
            config.ttl, config.semantic_matching, config.similarity_threshold
        );

        Self {
            provider,
            cache,
            config,
            embedding_service,
        }
    }

    /// Generate a cache key from prompt and context
    fn generate_cache_key(&self, prompt: &str, messages: &Value, model: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        hasher.update(messages.to_string().as_bytes());
        hasher.update(model.as_bytes());
        let hash = hasher.finalize();
        format!("{}:{}:{}", self.config.key_prefix, model, hex::encode(hash))
    }

    /// Check if caching is enabled based on config
    async fn is_cache_enabled(&self, bot_id: &str) -> bool {
        // Try to get llm-cache config from bot configuration
        // This would typically query the database, but for now we'll check Redis
        let mut conn = match self.cache.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                debug!("Cache connection failed: {}", e);
                return false;
            }
        };

        let config_key = format!("bot_config:{}:llm-cache", bot_id);
        match conn.get::<_, String>(config_key).await {
            Ok(value) => value.to_lowercase() == "true",
            Err(_) => {
                // Default to enabled if not specified
                true
            }
        }
    }

    /// Try to get a cached response
    async fn get_cached_response(
        &self,
        prompt: &str,
        messages: &Value,
        model: &str,
    ) -> Option<CachedResponse> {
        let cache_key = self.generate_cache_key(prompt, messages, model);

        let mut conn = match self.cache.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                debug!("Failed to connect to cache: {}", e);
                return None;
            }
        };

        // Try exact match first
        if let Ok(cached_json) = conn.get::<_, String>(&cache_key).await {
            if let Ok(mut cached) = serde_json::from_str::<CachedResponse>(&cached_json) {
                // Update hit count
                cached.hit_count += 1;
                let _ = conn
                    .set_ex::<_, _, ()>(
                        &cache_key,
                        serde_json::to_string(&cached).unwrap_or_default(),
                        self.config.ttl,
                    )
                    .await;

                info!(
                    "Cache hit (exact match) for prompt: ~{} tokens",
                    estimate_token_count(prompt)
                );
                return Some(cached);
            }
        }

        // Try semantic similarity if enabled
        if self.config.semantic_matching && self.embedding_service.is_some() {
            if let Some(similar) = self.find_similar_cached(prompt, messages, model).await {
                info!(
                    "Cache hit (semantic match) for prompt: ~{} tokens",
                    estimate_token_count(prompt)
                );
                return Some(similar);
            }
        }

        debug!(
            "Cache miss for prompt: ~{} tokens",
            estimate_token_count(prompt)
        );
        None
    }

    /// Find semantically similar cached responses
    async fn find_similar_cached(
        &self,
        prompt: &str,
        messages: &Value,
        model: &str,
    ) -> Option<CachedResponse> {
        let embedding_service = self.embedding_service.as_ref()?;

        // Combine prompt with messages for more accurate matching
        let combined_context = format!("{}\n{}", prompt, messages.to_string());

        // Get embedding for current prompt
        let prompt_embedding = match embedding_service.get_embedding(&combined_context).await {
            Ok(emb) => emb,
            Err(e) => {
                debug!("Failed to get embedding for prompt: {}", e);
                return None;
            }
        };

        let mut conn = match self.cache.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                debug!("Failed to connect to cache for semantic search: {}", e);
                return None;
            }
        };

        // Get recent cache keys for this model
        let pattern = format!("{}:{}:*", self.config.key_prefix, model);
        let keys: Vec<String> = match conn.keys(pattern).await {
            Ok(k) => k,
            Err(e) => {
                debug!("Failed to get cache keys: {}", e);
                return None;
            }
        };

        let mut best_match: Option<(CachedResponse, f32)> = None;
        let check_limit = keys.len().min(self.config.max_similarity_checks);

        for key in keys.iter().take(check_limit) {
            if let Ok(cached_json) = conn.get::<_, String>(key).await {
                if let Ok(cached) = serde_json::from_str::<CachedResponse>(&cached_json) {
                    if let Some(ref cached_embedding) = cached.embedding {
                        let similarity = embedding_service
                            .compute_similarity(&prompt_embedding, cached_embedding)
                            .await;

                        if similarity >= self.config.similarity_threshold {
                            if best_match.is_none() || best_match.as_ref().unwrap().1 < similarity {
                                best_match = Some((cached.clone(), similarity));
                            }
                        }
                    }
                }
            }
        }

        if let Some((mut cached, similarity)) = best_match {
            debug!("Found semantic match with similarity: {}", similarity);
            // Update hit count
            cached.hit_count += 1;
            let cache_key =
                self.generate_cache_key(&cached.prompt, &cached.messages, &cached.model);
            let _ = conn
                .set_ex::<_, _, ()>(
                    &cache_key,
                    serde_json::to_string(&cached).unwrap_or_default(),
                    self.config.ttl,
                )
                .await;
            return Some(cached);
        }

        None
    }

    /// Store a response in cache
    async fn cache_response(&self, prompt: &str, messages: &Value, model: &str, response: &str) {
        let cache_key = self.generate_cache_key(prompt, messages, model);

        let mut conn = match self.cache.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                debug!("Failed to connect to cache for storing: {}", e);
                return;
            }
        };

        // Get embedding if service is available
        let embedding = if let Some(ref service) = self.embedding_service {
            service.get_embedding(prompt).await.ok()
        } else {
            None
        };

        let cached_response = CachedResponse {
            response: response.to_string(),
            prompt: prompt.to_string(),
            messages: messages.clone(),
            model: model.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            hit_count: 0,
            embedding,
        };

        match serde_json::to_string(&cached_response) {
            Ok(json) => {
                if let Err(e) = conn
                    .set_ex::<_, _, ()>(&cache_key, json, self.config.ttl)
                    .await
                {
                    debug!("Failed to cache response: {}", e);
                } else {
                    trace!(
                        "Cached response for prompt: ~{} tokens",
                        estimate_token_count(prompt)
                    );
                }
            }
            Err(e) => {
                debug!("Failed to serialize cached response: {}", e);
            }
        }
    }

    /// Get cache statistics
    pub async fn get_cache_stats(
        &self,
    ) -> Result<CacheStats, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.cache.get_multiplexed_async_connection().await?;

        let pattern = format!("{}:*", self.config.key_prefix);
        let keys: Vec<String> = conn.keys(pattern).await?;

        let mut total_hits = 0u32;
        let mut total_size = 0usize;
        let mut model_stats: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();

        for key in keys.iter() {
            if let Ok(cached_json) = conn.get::<_, String>(key).await {
                total_size += cached_json.len();
                if let Ok(cached) = serde_json::from_str::<CachedResponse>(&cached_json) {
                    total_hits += cached.hit_count;
                    *model_stats.entry(cached.model.clone()).or_insert(0) += 1;
                }
            }
        }

        Ok(CacheStats {
            total_entries: keys.len(),
            total_hits,
            total_size_bytes: total_size,
            model_distribution: model_stats,
        })
    }

    /// Clear cache for a specific model or all models
    pub async fn clear_cache(
        &self,
        model: Option<&str>,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.cache.get_multiplexed_async_connection().await?;

        let pattern = if let Some(m) = model {
            format!("{}:{}:*", self.config.key_prefix, m)
        } else {
            format!("{}:*", self.config.key_prefix)
        };

        let keys: Vec<String> = conn.keys(pattern).await?;
        let count = keys.len();

        for key in keys {
            let _: Result<(), _> = conn.del(&key).await;
        }

        info!("Cleared {} cache entries", count);
        Ok(count)
    }
}

/// Cache statistics
#[derive(Serialize, Deserialize, Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_hits: u32,
    pub total_size_bytes: usize,
    pub model_distribution: std::collections::HashMap<String, u32>,
}

#[async_trait]
impl LLMProvider for CachedLLMProvider {
    async fn generate(
        &self,
        prompt: &str,
        messages: &Value,
        model: &str,
        key: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Check if cache is enabled for this bot
        let bot_id = "default"; // This should be passed from context
        if !self.is_cache_enabled(bot_id).await {
            trace!("Cache disabled, bypassing");
            return self.provider.generate(prompt, messages, model, key).await;
        }

        // Try to get from cache
        if let Some(cached) = self.get_cached_response(prompt, messages, model).await {
            return Ok(cached.response);
        }

        // Generate new response
        let response = self.provider.generate(prompt, messages, model, key).await?;

        // Cache the response
        self.cache_response(prompt, messages, model, &response)
            .await;

        Ok(response)
    }

    async fn generate_stream(
        &self,
        prompt: &str,
        messages: &Value,
        tx: mpsc::Sender<String>,
        model: &str,
        key: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if cache is enabled
        let bot_id = "default"; // This should be passed from context
        if !self.is_cache_enabled(bot_id).await {
            trace!("Cache disabled for streaming, bypassing");
            return self
                .provider
                .generate_stream(prompt, messages, tx, model, key)
                .await;
        }

        // Try to get from cache
        if let Some(cached) = self.get_cached_response(prompt, messages, model).await {
            // Stream the cached response
            for chunk in cached.response.chars().collect::<Vec<_>>().chunks(50) {
                let chunk_str: String = chunk.iter().collect();
                if tx.send(chunk_str).await.is_err() {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
            return Ok(());
        }

        // For streaming, we need to buffer the response to cache it
        let (buffer_tx, mut buffer_rx) = mpsc::channel::<String>(100);
        let tx_clone = tx.clone();
        let mut full_response = String::new();

        // Forward stream and buffer
        let forward_task = tokio::spawn(async move {
            while let Some(chunk) = buffer_rx.recv().await {
                full_response.push_str(&chunk);
                if tx_clone.send(chunk).await.is_err() {
                    break;
                }
            }
            full_response
        });

        // Generate stream
        self.provider
            .generate_stream(prompt, messages, buffer_tx, model, key)
            .await?;

        // Get the full response and cache it
        let full_response = forward_task.await?;
        self.cache_response(prompt, messages, model, &full_response)
            .await;

        Ok(())
    }

    async fn cancel_job(
        &self,
        session_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.provider.cancel_job(session_id).await
    }
}

/// Basic embedding service implementation using local embeddings
pub struct LocalEmbeddingService {
    embedding_url: String,
    model: String,
}

impl LocalEmbeddingService {
    pub fn new(embedding_url: String, model: String) -> Self {
        Self {
            embedding_url,
            model,
        }
    }
}

#[async_trait]
impl EmbeddingService for LocalEmbeddingService {
    async fn get_embedding(
        &self,
        text: &str,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/embeddings", self.embedding_url))
            .json(&serde_json::json!({
                "input": text,
                "model": self.model,
            }))
            .send()
            .await?;

        let result: Value = response.json().await?;
        let embedding = result["data"][0]["embedding"]
            .as_array()
            .ok_or("Invalid embedding response")?
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect();

        Ok(embedding)
    }

    async fn compute_similarity(&self, embedding1: &[f32], embedding2: &[f32]) -> f32 {
        // Cosine similarity
        if embedding1.len() != embedding2.len() {
            return 0.0;
        }

        let dot_product: f32 = embedding1
            .iter()
            .zip(embedding2.iter())
            .map(|(a, b)| a * b)
            .sum();

        let norm1: f32 = embedding1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = embedding2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            return 0.0;
        }

        dot_product / (norm1 * norm2)
    }
}
