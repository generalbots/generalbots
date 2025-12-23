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
use crate::config::ConfigManager;
use crate::shared::utils::{estimate_token_count, DbPool};


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


            let config_manager = ConfigManager::new(db_pool.clone());
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


    async fn get_bot_cache_config(&self, bot_id: &str) -> CacheConfig {
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

            let config_manager = ConfigManager::new(db_pool.clone());


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
                .get_config(&bot_uuid, "llm-cache-semantic", Some("true"))
                .unwrap_or_else(|_| "true".to_string())
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



    async fn find_similar_cached(
        &self,
        prompt: &str,
        messages: &Value,
        model: &str,
    ) -> Option<CachedResponse> {
        let embedding_service = self.embedding_service.as_ref()?;


        let actual_messages = if messages.get("messages").is_some() {
            messages.get("messages").unwrap_or(messages)
        } else {
            messages
        };


        let combined_context = format!("{}\n{}", prompt, actual_messages.to_string());


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

            let combined_context = format!("{}\n{}", prompt, actual_messages.to_string());
            service.get_embedding(&combined_context).await.ok()
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


        let bot_cache_config = self.get_bot_cache_config(bot_id).await;


        if let Some(cached) = self.get_cached_response(prompt, messages, model).await {
            info!("Cache hit (exact match) for bot {}", bot_id);
            return Ok(cached.response);
        }


        if bot_cache_config.semantic_matching && self.embedding_service.is_some() {
            if let Some(cached) = self.find_similar_cached(prompt, messages, model).await {
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
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

        let bot_id = "default";
        if !self.is_cache_enabled(bot_id).await {
            trace!("Cache disabled for streaming, bypassing");
            return self
                .provider
                .generate_stream(prompt, messages, tx, model, key)
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
            .generate_stream(prompt, messages, buffer_tx, model, key)
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
