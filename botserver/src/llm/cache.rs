use async_trait::async_trait;
use log::{debug, info, trace};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::LLMProvider;
use crate::core::config::ConfigManager;
use crate::core::shared::utils::{estimate_token_count, DbPool};

#[derive(Clone, Debug)]

pub struct CacheConfig {
    pub ttl: u64,

    pub semantic_matching: bool,

    pub similarity_threshold: f32,

    pub max_similarity_checks: usize,

    pub key_prefix: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ttl: 3600,
            semantic_matching: true,
            similarity_threshold: 0.95,
            max_similarity_checks: 100,
            key_prefix: "llm_cache".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]

pub struct CachedResponse {
    pub response: String,

    pub prompt: String,

    pub messages: Value,

    pub model: String,

    pub timestamp: u64,

    pub hit_count: u32,

    pub embedding: Option<Vec<f32>>,
}

impl std::fmt::Debug for CachedLLMProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedLLMProvider")
            .field("provider", &"<dyn LLMProvider>")
            .field("cache", &self.cache)
            .field("config", &self.config)
            .field("embedding_service", &self.embedding_service.is_some())
            .field("db_pool", &self.db_pool.is_some())
            .finish()
    }
}

pub struct CachedLLMProvider {
    provider: Arc<dyn LLMProvider>,

    cache: Arc<redis::Client>,

    config: CacheConfig,

    embedding_service: Option<Arc<dyn EmbeddingService>>,

    db_pool: Option<DbPool>,
}

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
            db_pool: None,
        }
    }

    pub fn with_db_pool(
        provider: Arc<dyn LLMProvider>,
        cache: Arc<redis::Client>,
        config: CacheConfig,
        embedding_service: Option<Arc<dyn EmbeddingService>>,
        db_pool: DbPool,
    ) -> Self {
        info!("Initializing CachedLLMProvider with semantic cache and DB pool");
        info!(
            "Cache config: TTL={}s, Semantic={}, Threshold={}",
            config.ttl, config.semantic_matching, config.similarity_threshold
        );

        Self {
            provider,
            cache,
            config,
            embedding_service,
            db_pool: Some(db_pool),
        }
    }

    fn generate_cache_key(&self, prompt: &str, messages: &Value, model: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        hasher.update(messages.to_string().as_bytes());
        hasher.update(model.as_bytes());
        let hash = hasher.finalize();
        format!("{}:{}:{}", self.config.key_prefix, model, hex::encode(hash))
    }

    async fn is_cache_enabled(&self, bot_id: &str) -> bool {
        if let Some(ref db_pool) = self.db_pool {
            let bot_uuid = match Uuid::parse_str(bot_id) {
                Ok(uuid) => uuid,
                Err(_) => {
                    if bot_id == "default" {
                        Uuid::nil()
                    } else {
                        return self.config.semantic_matching;
                    }
                }
            };

            let config_manager = ConfigManager::new(db_pool.clone().into());
            let cache_enabled = config_manager
                .get_config(&bot_uuid, "llm-cache", Some("true"))
                .unwrap_or_else(|_| "true".to_string());

            return cache_enabled.to_lowercase() == "true";
        }

        let mut conn = match self.cache.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                debug!("Cache connection failed: {}", e);
                return self.config.semantic_matching;
            }
        };

        let config_key = format!("bot_config:{}:llm-cache", bot_id);
        match conn.get::<_, String>(config_key).await {
            Ok(value) => value.to_lowercase() == "true",
            Err(_) => self.config.semantic_matching,
        }
    }

    fn get_bot_cache_config(&self, bot_id: &str) -> CacheConfig {
        if let Some(ref db_pool) = self.db_pool {
            let bot_uuid = match Uuid::parse_str(bot_id) {
                Ok(uuid) => uuid,
                Err(_) => {
                    if bot_id == "default" {
                        Uuid::nil()
                    } else {
                        return self.config.clone();
                    }
                }
            };

            let config_manager = ConfigManager::new(db_pool.clone().into());

            let ttl = config_manager
                .get_config(
                    &bot_uuid,
                    "llm-cache-ttl",
                    Some(&self.config.ttl.to_string()),
                )
                .unwrap_or_else(|_| self.config.ttl.to_string())
                .parse()
                .unwrap_or(self.config.ttl);

            let semantic_enabled = config_manager
                .get_config(&bot_uuid, "llm-cache-semantic", Some("false"))
                .unwrap_or_else(|_| "false".to_string())
                .to_lowercase()
                == "true";

            let threshold = config_manager
                .get_config(
                    &bot_uuid,
                    "llm-cache-threshold",
                    Some(&self.config.similarity_threshold.to_string()),
                )
                .unwrap_or_else(|_| self.config.similarity_threshold.to_string())
                .parse()
                .unwrap_or(self.config.similarity_threshold);

            CacheConfig {
                ttl,
                semantic_matching: semantic_enabled,
                similarity_threshold: threshold,
                max_similarity_checks: self.config.max_similarity_checks,
                key_prefix: self.config.key_prefix.clone(),
            }
        } else {
            self.config.clone()
        }
    }

    async fn get_cached_response(
        &self,
        prompt: &str,
        messages: &Value,
        model: &str,
    ) -> Option<CachedResponse> {
        let mut conn = match self.cache.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                debug!("Failed to connect to cache: {}", e);
                return None;
            }
        };

        let actual_messages = if messages.get("messages").is_some() {
            messages.get("messages").unwrap_or(messages)
        } else {
            messages
        };

        let cache_key = self.generate_cache_key(prompt, actual_messages, model);

        if let Ok(cached_json) = conn.get::<_, String>(&cache_key).await {
            if let Ok(mut cached) = serde_json::from_str::<CachedResponse>(&cached_json) {
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

        if self.config.semantic_matching && self.embedding_service.is_some() {
            if let Some(similar) = self.find_similar_cached(prompt, messages, model, self.config.similarity_threshold).await {
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

    async fn find_similar_cached(
        &self,
        prompt: &str,
        messages: &Value,
        model: &str,
        threshold: f32,
    ) -> Option<CachedResponse> {
        let embedding_service = self.embedding_service.as_ref()?;

        let actual_messages = if messages.get("messages").is_some() {
            messages.get("messages").unwrap_or(messages)
        } else {
            messages
        };

        // Extract ONLY the latest user question for semantic matching
        // This prevents false positives from matching on old conversation history
        let latest_user_question = if let Some(msgs) = actual_messages.as_array() {
            // Find the last message with role "user"
            msgs.iter()
                .rev()
                .find_map(|msg| {
                    if msg.get("role").and_then(|r| r.as_str()) == Some("user") {
                        msg.get("content").and_then(|c| c.as_str())
                    } else {
                        None
                    }
                })
                .unwrap_or("")
        } else {
            ""
        };

        // Use only the latest user question for semantic matching, not the full history
        // The prompt contains system context, so we combine with latest question
        let semantic_query = if latest_user_question.is_empty() {
            prompt.to_string()
        } else {
            format!("{}\n{}", prompt, latest_user_question)
        };

        // Debug: log the text being sent for embedding
        debug!(
            "Embedding request text (len={}, using latest user question): {}",
            semantic_query.len(),
            &semantic_query.chars().take(200).collect::<String>()
        );

        let prompt_embedding = match embedding_service.get_embedding(&semantic_query).await {
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

                        if similarity >= threshold
                            && best_match.as_ref().is_none_or(|(_, s)| *s < similarity)
                        {
                            best_match = Some((cached.clone(), similarity));
                        }
                    }
                }
            }
        }

        if let Some((mut cached, similarity)) = best_match {
            debug!("Found semantic match with similarity: {}", similarity);

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

    async fn cache_response(&self, prompt: &str, messages: &Value, model: &str, response: &str) {
        let actual_messages = if messages.get("messages").is_some() {
            messages.get("messages").unwrap_or(messages)
        } else {
            messages
        };

        let cache_key = self.generate_cache_key(prompt, actual_messages, model);

        let mut conn = match self.cache.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                debug!("Failed to connect to cache for storing: {}", e);
                return;
            }
        };

        let embedding = if let Some(ref service) = self.embedding_service {
            // Extract ONLY the latest user question for embedding
            // Same logic as find_similar_cached to ensure consistency
            let latest_user_question = if let Some(msgs) = actual_messages.as_array() {
                msgs.iter()
                    .rev()
                    .find_map(|msg| {
                        if msg.get("role").and_then(|r| r.as_str()) == Some("user") {
                            msg.get("content").and_then(|c| c.as_str())
                        } else {
                            None
                        }
                    })
                    .unwrap_or("")
            } else {
                ""
            };

            let semantic_query = if latest_user_question.is_empty() {
                prompt.to_string()
            } else {
                format!("{}\n{}", prompt, latest_user_question)
            };

            service.get_embedding(&semantic_query).await.ok()
        } else {
            None
        };

        let cached_response = CachedResponse {
            response: response.to_string(),
            prompt: prompt.to_string(),
            messages: actual_messages.clone(),
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

#[derive(Serialize, Deserialize, Clone, Debug)]

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
        let bot_id = messages
            .get("bot_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        if !self.is_cache_enabled(bot_id).await {
            trace!("Cache disabled for bot {}, bypassing", bot_id);
            return self.provider.generate(prompt, messages, model, key).await;
        }

        let bot_cache_config = self.get_bot_cache_config(bot_id);

        if let Some(cached) = self.get_cached_response(prompt, messages, model).await {
            info!("Cache hit (exact match) for bot {}", bot_id);
            return Ok(cached.response);
        }

        if bot_cache_config.semantic_matching && self.embedding_service.is_some() {
            if let Some(cached) = self.find_similar_cached(prompt, messages, model, bot_cache_config.similarity_threshold).await {
                info!(
                    "Cache hit (semantic match) for bot {} with similarity threshold {}",
                    bot_id, bot_cache_config.similarity_threshold
                );
                return Ok(cached.response);
            }
        }

        debug!("Cache miss for bot {}, generating new response", bot_id);
        let response = self.provider.generate(prompt, messages, model, key).await?;

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
        tools: Option<&Vec<Value>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let bot_id = "default";
        if !self.is_cache_enabled(bot_id).await {
            trace!("Cache disabled for streaming, bypassing");
            return self
                .provider
                .generate_stream(prompt, messages, tx, model, key, tools)
                .await;
        }

        if let Some(cached) = self.get_cached_response(prompt, messages, model).await {
            for chunk in cached.response.chars().collect::<Vec<_>>().chunks(50) {
                let chunk_str: String = chunk.iter().collect();
                if tx.send(chunk_str).await.is_err() {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
            return Ok(());
        }

        let (buffer_tx, mut buffer_rx) = mpsc::channel::<String>(100);
        let tx_clone = tx.clone();
        let mut full_response = String::new();

        let forward_task = tokio::spawn(async move {
            while let Some(chunk) = buffer_rx.recv().await {
                full_response.push_str(&chunk);
                if tx_clone.send(chunk).await.is_err() {
                    break;
                }
            }
            full_response
        });

        self.provider
            .generate_stream(prompt, messages, buffer_tx, model, key, tools)
            .await?;

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

#[derive(Debug)]

pub struct LocalEmbeddingService {
    embedding_url: String,
    model: String,
    api_key: Option<String>,
}

impl LocalEmbeddingService {
    pub fn new(embedding_url: String, model: String, api_key: Option<String>) -> Self {
        Self {
            embedding_url,
            model,
            api_key,
        }
    }

    /// Generate a deterministic hash-based embedding for fallback
    fn hash_embedding(&self, text: &str) -> Vec<f32> {
        const EMBEDDING_DIM: usize = 384; // Match common embedding dimensions
        let mut embedding = vec![0.0f32; EMBEDDING_DIM];

        let hash = Sha256::digest(text.as_bytes());

        // Use hash bytes to seed the embedding
        for (i, byte) in hash.iter().cycle().take(EMBEDDING_DIM * 4).enumerate() {
            let idx = i % EMBEDDING_DIM;
            let value = (*byte as f32 - 128.0) / 128.0;
            embedding[idx] += value * 0.1;
        }

        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }

        embedding
    }
}

#[async_trait]
impl EmbeddingService for LocalEmbeddingService {
    async fn get_embedding(
        &self,
        text: &str,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        // Determine if URL already includes endpoint path
        let url = if self.embedding_url.contains("/pipeline/") ||
                   self.embedding_url.contains("/v1/") ||
                   self.embedding_url.contains("/ai/run/") ||
                   self.embedding_url.ends_with("/embeddings") {
            self.embedding_url.clone()
        } else {
            format!("{}/embedding", self.embedding_url)
        };

        // Determine request body format based on URL
        let request_body = if self.embedding_url.contains("huggingface.co") {
            serde_json::json!({
                "inputs": text,
            })
        } else if self.embedding_url.contains("/ai/run/") {
            // Cloudflare AI format
            serde_json::json!({
                "text": text,
            })
        } else {
            serde_json::json!({
                "input": text,
                "model": self.model,
            })
        };

        // Retry logic with exponential backoff
        const MAX_RETRIES: u32 = 3;
        const INITIAL_DELAY_MS: u64 = 500;

        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                let delay_ms = INITIAL_DELAY_MS * (1 << (attempt - 1)); // 500, 1000, 2000
                debug!("Embedding service retry attempt {}/{} after {}ms", attempt + 1, MAX_RETRIES, delay_ms);
                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
            }

            let mut request = client.post(&url);

            // Add authorization header if API key is provided
            if let Some(ref api_key) = self.api_key {
                request = request.header("Authorization", format!("Bearer {}", api_key));
            }

            match request
                .json(&request_body)
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();
                    let response_text = match response.text().await {
                        Ok(t) => t,
                        Err(e) => {
                            debug!("Failed to read response body: {}", e);
                            continue;
                        }
                    };

                    if !status.is_success() {
                        debug!(
                            "Embedding service HTTP error {} (attempt {}/{}): {}",
                            status, attempt + 1, MAX_RETRIES, response_text
                        );
                        // Retry on 5xx errors
                        if status.as_u16() >= 500 {
                            continue;
                        }
                        // Non-retriable error
                        return Err(format!(
                            "Embedding service returned HTTP {}: {}",
                            status, response_text
                        ).into());
                    }

                    // Success - parse response
                    let result: Value = match serde_json::from_str(&response_text) {
                        Ok(r) => r,
                        Err(e) => {
                            debug!("Failed to parse embedding JSON: {} - Response: {}", e, response_text);
                            return Err(format!("Failed to parse embedding response JSON: {} - Response: {}", e, response_text).into());
                        }
                    };

                    if let Some(error) = result.get("error") {
                        debug!("Embedding service returned error: {}", error);
                        return Err(format!("Embedding service error: {}", error).into());
                    }

                    // Try multiple response formats
                    let embedding = if let Some(arr) = result.as_array() {
                        // HuggingFace format: direct array [0.1, 0.2, ...]
                        arr.iter()
                            .filter_map(|v| v.as_f64().map(|f| f as f32))
                            .collect()
                    } else if let Some(result_obj) = result.get("result") {
                        // Cloudflare AI format: {"result": {"data": [[...]]}}
                        if let Some(data) = result_obj.get("data") {
                            if let Some(data_arr) = data.as_array() {
                                if let Some(first) = data_arr.first() {
                                    if let Some(embedding_arr) = first.as_array() {
                                        embedding_arr
                                            .iter()
                                            .filter_map(|v| v.as_f64().map(|f| f as f32))
                                            .collect()
                                    } else {
                                        data_arr
                                            .iter()
                                            .filter_map(|v| v.as_f64().map(|f| f as f32))
                                            .collect()
                                    }
                                } else {
                                    return Err("Empty data array in Cloudflare response".into());
                                }
                            } else {
                                return Err(format!("Invalid Cloudflare response format - Expected result.data array, got: {}", response_text).into());
                            }
                        } else {
                            return Err(format!("Invalid Cloudflare response format - Expected result.data, got: {}", response_text).into());
                        }
                    } else if let Some(data) = result.get("data") {
                        // OpenAI/Standard format: {"data": [{"embedding": [...]}]}
                        data[0]["embedding"]
                            .as_array()
                            .ok_or_else(|| {
                                debug!("Invalid embedding response format. Expected data[0].embedding array. Got: {}", response_text);
                                format!("Invalid embedding response format - Expected data[0].embedding array, got: {}", response_text)
                            })?
                            .iter()
                            .filter_map(|v| v.as_f64().map(|f| f as f32))
                            .collect()
                    } else {
                        return Err(format!(
                            "Invalid embedding response format - Expected array or data[0].embedding, got: {}",
                            response_text
                        ).into());
                    };

                    return Ok(embedding);
                }
                Err(e) => {
                    // Network error - retry
                    debug!("Embedding service network error (attempt {}/{}): {}", attempt + 1, MAX_RETRIES, e);
                }
            }
        }

        // All retries exhausted - use hash-based fallback
        debug!("Embedding service failed after all retries, using hash-based fallback");
        Ok(self.hash_embedding(text))
    }

    async fn compute_similarity(&self, embedding1: &[f32], embedding2: &[f32]) -> f32 {
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
