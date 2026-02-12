use anyhow::{Context, Result};
use log::{info, trace, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

use crate::core::shared::DbPool;
use crate::core::shared::memory_monitor::{log_jemalloc_stats, MemoryStats};
use super::document_processor::TextChunk;

static EMBEDDING_SERVER_READY: AtomicBool = AtomicBool::new(false);

pub fn is_embedding_server_ready() -> bool {
    EMBEDDING_SERVER_READY.load(Ordering::SeqCst)
}

pub fn set_embedding_server_ready(ready: bool) {
    EMBEDDING_SERVER_READY.store(ready, Ordering::SeqCst);
    if ready {
        info!("[EMBEDDING] Embedding server marked as ready");
    }
}

#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub embedding_url: String,
    pub embedding_model: String,
    pub dimensions: usize,
    pub batch_size: usize,
    pub timeout_seconds: u64,
    pub max_concurrent_requests: usize,
    pub connect_timeout_seconds: u64,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            embedding_url: "http://localhost:8082".to_string(),
            embedding_model: "bge-small-en-v1.5".to_string(),
            dimensions: 384,
            batch_size: 16,
            timeout_seconds: 60,
            max_concurrent_requests: 1,
            connect_timeout_seconds: 10,
        }
    }
}

impl EmbeddingConfig {
    pub fn from_env() -> Self {
        Self::default()
    }

    /// Load embedding config from bot's config.csv (similar to llm-url, llm-model)
    /// This allows configuring embedding server per-bot in config.csv:
    /// embedding-url,http://localhost:8082
    /// embedding-model,bge-small-en-v1.5
    /// embedding-dimensions,384
    /// embedding-batch-size,16
    /// embedding-timeout,60
    pub fn from_bot_config(pool: &DbPool, _bot_id: &uuid::Uuid) -> Self {
        use crate::core::shared::models::schema::bot_configuration::dsl::*;
        use diesel::prelude::*;

        let embedding_url = match pool.get() {
            Ok(mut conn) => bot_configuration
                .filter(bot_id.eq(bot_id))
                .filter(config_key.eq("embedding-url"))
                .select(config_value)
                .first::<String>(&mut conn)
                .ok()
                .filter(|s| !s.is_empty()),
            Err(_) => None,
        }.unwrap_or_else(|| "http://localhost:8082".to_string());

        let embedding_model = match pool.get() {
            Ok(mut conn) => bot_configuration
                .filter(bot_id.eq(bot_id))
                .filter(config_key.eq("embedding-model"))
                .select(config_value)
                .first::<String>(&mut conn)
                .ok()
                .filter(|s| !s.is_empty()),
            Err(_) => None,
        }.unwrap_or_else(|| "bge-small-en-v1.5".to_string());

        let dimensions = match pool.get() {
            Ok(mut conn) => bot_configuration
                .filter(bot_id.eq(bot_id))
                .filter(config_key.eq("embedding-dimensions"))
                .select(config_value)
                .first::<String>(&mut conn)
                .ok()
                .and_then(|v| v.parse().ok()),
            Err(_) => None,
        }.unwrap_or_else(|| Self::detect_dimensions(&embedding_model));

        let batch_size = match pool.get() {
            Ok(mut conn) => bot_configuration
                .filter(bot_id.eq(bot_id))
                .filter(config_key.eq("embedding-batch-size"))
                .select(config_value)
                .first::<String>(&mut conn)
                .ok()
                .and_then(|v| v.parse().ok()),
            Err(_) => None,
        }.unwrap_or(16);

        let timeout_seconds = match pool.get() {
            Ok(mut conn) => bot_configuration
                .filter(bot_id.eq(bot_id))
                .filter(config_key.eq("embedding-timeout"))
                .select(config_value)
                .first::<String>(&mut conn)
                .ok()
                .and_then(|v| v.parse().ok()),
            Err(_) => None,
        }.unwrap_or(60);

        let max_concurrent_requests = match pool.get() {
            Ok(mut conn) => bot_configuration
                .filter(bot_id.eq(bot_id))
                .filter(config_key.eq("embedding-concurrent"))
                .select(config_value)
                .first::<String>(&mut conn)
                .ok()
                .and_then(|v| v.parse().ok()),
            Err(_) => None,
        }.unwrap_or(1);

        Self {
            embedding_url,
            embedding_model,
            dimensions,
            batch_size,
            timeout_seconds,
            max_concurrent_requests,
            connect_timeout_seconds: 10,
        }
    }

    fn detect_dimensions(model: &str) -> usize {
        if model.contains("small") || model.contains("MiniLM") {
            384
        } else if model.contains("base") || model.contains("mpnet") {
            768
        } else if model.contains("large") || model.contains("ada") {
            1536
        } else {
            384
        }
    }
}

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    input: Vec<String>,
    model: String,
}

// OpenAI/Claude/OpenAI-compatible format
#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<OpenAIEmbeddingData>,
    model: String,
    usage: Option<EmbeddingUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f32>,
}

// llama.cpp format
#[derive(Debug, Deserialize)]
struct LlamaCppEmbeddingItem {
    embedding: Vec<Vec<f32>>,
}

// Hugging Face/SentenceTransformers format (simple array)
type HuggingFaceEmbeddingResponse = Vec<Vec<f32>>;

// Generic embedding service format (object with embeddings key)
#[derive(Debug, Deserialize)]
struct GenericEmbeddingResponse {
    embeddings: Vec<Vec<f32>>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    usage: Option<EmbeddingUsage>,
}

// Universal response wrapper - tries formats in order of likelihood
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EmbeddingResponse {
    OpenAI(OpenAIEmbeddingResponse),           // Most common: OpenAI, Claude, etc.
    LlamaCpp(Vec<LlamaCppEmbeddingItem>),      // llama.cpp server
    HuggingFace(HuggingFaceEmbeddingResponse), // Simple array format
    Generic(GenericEmbeddingResponse),         // Generic services
}

#[derive(Debug, Deserialize)]
struct EmbeddingUsage {
    #[serde(default)]
    total_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub vector: Vec<f32>,
    pub dimensions: usize,
    pub model: String,
    pub tokens_used: Option<usize>,
}

pub struct KbEmbeddingGenerator {
    config: EmbeddingConfig,
    client: Client,
    semaphore: Arc<Semaphore>,
}

impl std::fmt::Debug for KbEmbeddingGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KbEmbeddingGenerator")
            .field("config", &self.config)
            .field("client", &"Client")
            .field("semaphore", &"Semaphore")
            .finish()
    }
}

impl KbEmbeddingGenerator {
    pub fn new(config: EmbeddingConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .connect_timeout(Duration::from_secs(config.connect_timeout_seconds))
            .pool_max_idle_per_host(2)
            .pool_idle_timeout(Duration::from_secs(30))
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)
            .build()
            .unwrap_or_else(|e| {
                warn!("Failed to create HTTP client with timeout: {}, using default", e);
                Client::new()
            });

        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_requests));

        Self {
            config,
            client,
            semaphore,
        }
    }

    pub async fn check_health(&self) -> bool {
        let health_url = format!("{}/health", self.config.embedding_url);

        match tokio::time::timeout(
            Duration::from_secs(5),
            self.client.get(&health_url).send()
        ).await {
            Ok(Ok(response)) => {
                let is_healthy = response.status().is_success();
                if is_healthy {
                    set_embedding_server_ready(true);
                }
                is_healthy
            }
            Ok(Err(e)) => {
                let alt_url = &self.config.embedding_url;
                match tokio::time::timeout(
                    Duration::from_secs(5),
                    self.client.get(alt_url).send()
                ).await {
                    Ok(Ok(response)) => {
                        let is_healthy = response.status().is_success();
                        if is_healthy {
                            set_embedding_server_ready(true);
                        }
                        is_healthy
                    }
                    _ => {
                        warn!("[EMBEDDING] Health check failed: {}", e);
                        false
                    }
                }
            }
            Err(_) => {
                warn!("[EMBEDDING] Health check timed out");
                false
            }
        }
    }

    pub async fn wait_for_server(&self, max_wait_secs: u64) -> bool {
        let start = std::time::Instant::now();
        let max_wait = Duration::from_secs(max_wait_secs);

        info!("[EMBEDDING] Waiting for embedding server at {} (max {}s)...",
              self.config.embedding_url, max_wait_secs);

        while start.elapsed() < max_wait {
            if self.check_health().await {
                info!("[EMBEDDING] Embedding server is ready after {:?}", start.elapsed());
                return true;
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }

        warn!("[EMBEDDING] Embedding server not available after {}s", max_wait_secs);
        false
    }

    pub async fn generate_embeddings(
        &self,
        chunks: &[TextChunk],
    ) -> Result<Vec<(TextChunk, Embedding)>> {
        if chunks.is_empty() {
            return Ok(Vec::new());
        }

        if !is_embedding_server_ready() {
            info!("[EMBEDDING] Server not marked ready, checking health...");
            if !self.wait_for_server(30).await {
                return Err(anyhow::anyhow!(
                    "Embedding server not available at {}. Skipping embedding generation.",
                    self.config.embedding_url
                ));
            }
        }

        let start_mem = MemoryStats::current();
        trace!("[EMBEDDING] Generating embeddings for {} chunks, RSS={}",
              chunks.len(), MemoryStats::format_bytes(start_mem.rss_bytes));

        let mut results = Vec::with_capacity(chunks.len());
        let total_batches = chunks.len().div_ceil(self.config.batch_size);

        for (batch_num, batch) in chunks.chunks(self.config.batch_size).enumerate() {
            let batch_start = MemoryStats::current();
            trace!("[EMBEDDING] Processing batch {}/{} ({} items), RSS={}",
                  batch_num + 1,
                  total_batches,
                  batch.len(),
                  MemoryStats::format_bytes(batch_start.rss_bytes));

            let batch_embeddings = match tokio::time::timeout(
                Duration::from_secs(self.config.timeout_seconds),
                self.generate_batch_embeddings(batch)
            ).await {
                Ok(Ok(embeddings)) => embeddings,
                Ok(Err(e)) => {
                    warn!("[EMBEDDING] Batch {} failed: {}", batch_num + 1, e);
                    break;
                }
                Err(_) => {
                    warn!("[EMBEDDING] Batch {} timed out after {}s",
                          batch_num + 1, self.config.timeout_seconds);
                    break;
                }
            };

            let batch_end = MemoryStats::current();
            let delta = batch_end.rss_bytes.saturating_sub(batch_start.rss_bytes);
            trace!("[EMBEDDING] Batch {} complete: {} embeddings, RSS={} (delta={})",
                  batch_num + 1,
                  batch_embeddings.len(),
                  MemoryStats::format_bytes(batch_end.rss_bytes),
                  MemoryStats::format_bytes(delta));

            if delta > 100 * 1024 * 1024 {
                warn!("[EMBEDDING] Excessive memory growth detected ({}), stopping early",
                      MemoryStats::format_bytes(delta));
                for (chunk, embedding) in batch.iter().zip(batch_embeddings.iter()) {
                    results.push((chunk.clone(), embedding.clone()));
                }
                break;
            }

            for (chunk, embedding) in batch.iter().zip(batch_embeddings.iter()) {
                results.push((chunk.clone(), embedding.clone()));
            }

            if batch_num + 1 < total_batches {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        let end_mem = MemoryStats::current();
        trace!("[EMBEDDING] Generated {} embeddings, RSS={} (total delta={})",
              results.len(),
              MemoryStats::format_bytes(end_mem.rss_bytes),
              MemoryStats::format_bytes(end_mem.rss_bytes.saturating_sub(start_mem.rss_bytes)));
        log_jemalloc_stats();

        Ok(results)
    }

    async fn generate_batch_embeddings(&self, chunks: &[TextChunk]) -> Result<Vec<Embedding>> {
        let _permit = self.semaphore.acquire().await
            .map_err(|e| anyhow::anyhow!("Failed to acquire semaphore: {}", e))?;

        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();
        let total_chars: usize = texts.iter().map(|t| t.len()).sum();

        info!("[EMBEDDING] generate_batch_embeddings: {} texts, {} total chars",
              texts.len(), total_chars);

        match self.generate_local_embeddings(&texts).await {
            Ok(embeddings) => {
                info!("[EMBEDDING] Local embeddings succeeded: {} vectors", embeddings.len());
                Ok(embeddings)
            }
            Err(e) => {
                warn!("[EMBEDDING] Local embedding service failed: {}", e);
                Err(e)
            }
        }
    }

    async fn generate_local_embeddings(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        // Apply token-aware truncation to each text before creating request
        let truncated_texts: Vec<String> = texts.iter()
            .map(|text| crate::core::shared::utils::truncate_text_for_model(text, &self.config.embedding_model, 600))
            .collect();

        let request = EmbeddingRequest {
            input: truncated_texts,
            model: self.config.embedding_model.clone(),
        };

        let request_size = serde_json::to_string(&request)
            .map(|s| s.len())
            .unwrap_or(0);
        info!("[EMBEDDING] Sending request to {} (size: {} bytes)",
              self.config.embedding_url, request_size);

        let response = self
            .client
            .post(format!("{}/embedding", self.config.embedding_url))
            .json(&request)
            .send()
            .await
            .context("Failed to send request to embedding service")?;

        let status = response.status();
        if !status.is_success() {
            let error_bytes = response.bytes().await.unwrap_or_default();
            let error_text = String::from_utf8_lossy(&error_bytes[..error_bytes.len().min(1024)]);
            return Err(anyhow::anyhow!(
                "Embedding service error {}: {}",
                status,
                error_text
            ));
        }

        let response_bytes = response.bytes().await
            .context("Failed to read embedding response bytes")?;

        info!("[EMBEDDING] Received response: {} bytes", response_bytes.len());

        if response_bytes.len() > 50 * 1024 * 1024 {
            return Err(anyhow::anyhow!(
                "Embedding response too large: {} bytes (max 50MB)",
                response_bytes.len()
            ));
        }

        let embedding_response: EmbeddingResponse = serde_json::from_slice(&response_bytes)
            .with_context(|| {
                let preview = std::str::from_utf8(&response_bytes)
                    .map(|s| if s.len() > 200 { &s[..200] } else { s })
                    .unwrap_or("<invalid utf8>");
                format!("Failed to parse embedding response. Preview: {}", preview)
            })?;

        drop(response_bytes);

        let embeddings = match embedding_response {
            EmbeddingResponse::OpenAI(openai_response) => {
                let mut embeddings = Vec::with_capacity(openai_response.data.len());
                for data in openai_response.data {
                    embeddings.push(Embedding {
                        vector: data.embedding,
                        dimensions: self.config.dimensions,
                        model: openai_response.model.clone(),
                        tokens_used: openai_response.usage.as_ref().map(|u| u.total_tokens),
                    });
                }
                embeddings
            }
            EmbeddingResponse::LlamaCpp(llama_response) => {
                let mut embeddings = Vec::new();
                for item in llama_response {
                    for embedding_vec in item.embedding {
                        embeddings.push(Embedding {
                            vector: embedding_vec,
                            dimensions: self.config.dimensions,
                            model: self.config.embedding_model.clone(),
                            tokens_used: None,
                        });
                    }
                }
                embeddings
            }
            EmbeddingResponse::HuggingFace(hf_response) => {
                let mut embeddings = Vec::with_capacity(hf_response.len());
                for embedding_vec in hf_response {
                    embeddings.push(Embedding {
                        vector: embedding_vec,
                        dimensions: self.config.dimensions,
                        model: self.config.embedding_model.clone(),
                        tokens_used: None,
                    });
                }
                embeddings
            }
            EmbeddingResponse::Generic(generic_response) => {
                let mut embeddings = Vec::with_capacity(generic_response.embeddings.len());
                for embedding_vec in generic_response.embeddings {
                    embeddings.push(Embedding {
                        vector: embedding_vec,
                        dimensions: self.config.dimensions,
                        model: generic_response.model.clone().unwrap_or_else(|| self.config.embedding_model.clone()),
                        tokens_used: generic_response.usage.as_ref().map(|u| u.total_tokens),
                    });
                }
                embeddings
            }
        };

        Ok(embeddings)
    }

    pub async fn generate_single_embedding(&self, text: &str) -> Result<Embedding> {
        if !is_embedding_server_ready() && !self.check_health().await {
            return Err(anyhow::anyhow!(
                "Embedding server not available at {}",
                self.config.embedding_url
            ));
        }

        let embeddings = self
            .generate_batch_embeddings(&[TextChunk {
                content: text.to_string(),
                metadata: super::document_processor::ChunkMetadata {
                    document_path: "query".to_string(),
                    document_title: None,
                    chunk_index: 0,
                    total_chunks: 1,
                    start_char: 0,
                    end_char: text.len(),
                    page_number: None,
                },
            }])
            .await?;

        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No embedding generated"))
    }
}

pub struct EmbeddingGenerator {
    kb_generator: KbEmbeddingGenerator,
}

impl std::fmt::Debug for EmbeddingGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddingGenerator")
            .field("kb_generator", &self.kb_generator)
            .finish()
    }
}

impl EmbeddingGenerator {
    pub fn new(llm_endpoint: String) -> Self {
        let config = EmbeddingConfig {
            embedding_url: llm_endpoint,
            ..Default::default()
        };

        Self {
            kb_generator: KbEmbeddingGenerator::new(config),
        }
    }

    pub async fn generate_text_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let embedding = self.kb_generator.generate_single_embedding(text).await?;
        Ok(embedding.vector)
    }

    /// Check if the embedding server is healthy
    pub async fn check_health(&self) -> bool {
        self.kb_generator.check_health().await
    }
}

pub struct EmailEmbeddingGenerator {
    generator: EmbeddingGenerator,
}

impl std::fmt::Debug for EmailEmbeddingGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmailEmbeddingGenerator")
            .field("generator", &self.generator)
            .finish()
    }
}

impl EmailEmbeddingGenerator {
    pub fn new(llm_endpoint: String) -> Self {
        Self {
            generator: EmbeddingGenerator::new(llm_endpoint),
        }
    }

    pub async fn generate_embedding(&self, email: &impl EmailLike) -> Result<Vec<f32>> {
        let text = format!(
            "Subject: {}\nFrom: {}\nTo: {}\n\n{}",
            email.subject(),
            email.from(),
            email.to(),
            email.body()
        );

        self.generator.generate_text_embedding(&text).await
    }

    pub async fn generate_text_embedding(&self, text: &str) -> Result<Vec<f32>> {
        self.generator.generate_text_embedding(text).await
    }
}

pub trait EmailLike {
    fn subject(&self) -> &str;
    fn from(&self) -> &str;
    fn to(&self) -> &str;
    fn body(&self) -> &str;
}

#[derive(Debug)]
pub struct SimpleEmail {
    pub id: String,
    pub subject: String,
    pub from: String,
    pub to: String,
    pub body: String,
}

impl EmailLike for SimpleEmail {
    fn subject(&self) -> &str {
        &self.subject
    }
    fn from(&self) -> &str {
        &self.from
    }
    fn to(&self) -> &str {
        &self.to
    }
    fn body(&self) -> &str {
        &self.body
    }
}
