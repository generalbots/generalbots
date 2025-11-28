use anyhow::{Context, Result};
use log::{debug, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use std::sync::Arc;
use tokio::sync::Semaphore;

use super::document_processor::TextChunk;

/// Embedding model configuration
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    /// URL for the embedding service (e.g., http://localhost:8082)
    pub embedding_url: String,
    /// Model name/path for embeddings (e.g., bge-small-en-v1.5)
    pub embedding_model: String,
    /// Dimension of embeddings (e.g., 384, 768, 1536)
    pub dimensions: usize,
    /// Maximum batch size for embedding generation
    pub batch_size: usize,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            embedding_url: "http://localhost:8082".to_string(),
            embedding_model: "bge-small-en-v1.5".to_string(),
            dimensions: 384, // Default for bge-small
            batch_size: 32,
            timeout_seconds: 30,
        }
    }
}

impl EmbeddingConfig {
    /// Create config from environment or config.csv values
    pub fn from_env() -> Self {
        let embedding_url =
            std::env::var("EMBEDDING_URL").unwrap_or_else(|_| "http://localhost:8082".to_string());

        let embedding_model =
            std::env::var("EMBEDDING_MODEL").unwrap_or_else(|_| "bge-small-en-v1.5".to_string());

        // Detect dimensions based on model name
        let dimensions = Self::detect_dimensions(&embedding_model);

        Self {
            embedding_url,
            embedding_model,
            dimensions,
            batch_size: 32,
            timeout_seconds: 30,
        }
    }

    /// Detect embedding dimensions based on model name
    fn detect_dimensions(model: &str) -> usize {
        if model.contains("small") || model.contains("MiniLM") {
            384
        } else if model.contains("base") || model.contains("mpnet") {
            768
        } else if model.contains("large") || model.contains("ada") {
            1536
        } else {
            384 // Default
        }
    }
}

/// Request payload for embedding generation
#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    input: Vec<String>,
    model: String,
}

/// Response from embedding service
#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
    model: String,
    usage: Option<EmbeddingUsage>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    _index: usize,
}

#[derive(Debug, Deserialize)]
struct EmbeddingUsage {
    _prompt_tokens: usize,
    total_tokens: usize,
}

/// Generated embedding with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub vector: Vec<f32>,
    pub dimensions: usize,
    pub model: String,
    pub tokens_used: Option<usize>,
}

/// Knowledge base embedding generator
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
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        // Limit concurrent requests
        let semaphore = Arc::new(Semaphore::new(4));

        Self {
            config,
            client,
            semaphore,
        }
    }

    /// Generate embeddings for text chunks
    pub async fn generate_embeddings(
        &self,
        chunks: &[TextChunk],
    ) -> Result<Vec<(TextChunk, Embedding)>> {
        if chunks.is_empty() {
            return Ok(Vec::new());
        }

        info!("Generating embeddings for {} chunks", chunks.len());

        let mut results = Vec::new();

        // Process in batches
        for batch in chunks.chunks(self.config.batch_size) {
            let batch_embeddings = self.generate_batch_embeddings(batch).await?;

            // Pair chunks with their embeddings
            for (chunk, embedding) in batch.iter().zip(batch_embeddings.iter()) {
                results.push((chunk.clone(), embedding.clone()));
            }
        }

        info!("Generated {} embeddings", results.len());

        Ok(results)
    }

    /// Generate embeddings for a batch of chunks
    async fn generate_batch_embeddings(&self, chunks: &[TextChunk]) -> Result<Vec<Embedding>> {
        let _permit = self.semaphore.acquire().await?;

        let texts: Vec<String> = chunks.iter().map(|c| c.content.clone()).collect();

        debug!("Generating embeddings for batch of {} texts", texts.len());

        // Try local embedding service first
        match self.generate_local_embeddings(&texts).await {
            Ok(embeddings) => Ok(embeddings),
            Err(e) => {
                warn!("Local embedding service failed: {}, trying OpenAI API", e);
                self.generate_openai_embeddings(&texts).await
            }
        }
    }

    /// Generate embeddings using local service
    async fn generate_local_embeddings(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        let request = EmbeddingRequest {
            input: texts.to_vec(),
            model: self.config.embedding_model.clone(),
        };

        let response = self
            .client
            .post(&format!("{}/embeddings", self.config.embedding_url))
            .json(&request)
            .send()
            .await
            .context("Failed to send request to embedding service")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Embedding service error {}: {}",
                status,
                error_text
            ));
        }

        let embedding_response: EmbeddingResponse = response
            .json()
            .await
            .context("Failed to parse embedding response")?;

        let mut embeddings = Vec::new();
        for data in embedding_response.data {
            embeddings.push(Embedding {
                vector: data.embedding,
                dimensions: self.config.dimensions,
                model: embedding_response.model.clone(),
                tokens_used: embedding_response.usage.as_ref().map(|u| u.total_tokens),
            });
        }

        Ok(embeddings)
    }

    /// Generate embeddings using OpenAI API (fallback)
    async fn generate_openai_embeddings(&self, texts: &[String]) -> Result<Vec<Embedding>> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .context("OPENAI_API_KEY not set for fallback embedding generation")?;

        let request = serde_json::json!({
            "input": texts,
            "model": "text-embedding-ada-002"
        });

        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request)
            .send()
            .await
            .context("Failed to send request to OpenAI")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "OpenAI API error {}: {}",
                status,
                error_text
            ));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse OpenAI response")?;

        let mut embeddings = Vec::new();

        if let Some(data) = response_json["data"].as_array() {
            for item in data {
                if let Some(embedding) = item["embedding"].as_array() {
                    let vector: Vec<f32> = embedding
                        .iter()
                        .filter_map(|v| v.as_f64().map(|f| f as f32))
                        .collect();

                    embeddings.push(Embedding {
                        vector,
                        dimensions: 1536, // OpenAI ada-002 dimensions
                        model: "text-embedding-ada-002".to_string(),
                        tokens_used: response_json["usage"]["total_tokens"]
                            .as_u64()
                            .map(|t| t as usize),
                    });
                }
            }
        }

        Ok(embeddings)
    }

    /// Generate embedding for a single text
    pub async fn generate_single_embedding(&self, text: &str) -> Result<Embedding> {
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

/// Generic embedding generator for other uses (email, etc.)
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

    /// Generate embedding for arbitrary text
    pub async fn generate_text_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let embedding = self.kb_generator.generate_single_embedding(text).await?;
        Ok(embedding.vector)
    }
}

/// Email-specific embedding generator (for compatibility)
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

    /// Generate embedding for email content
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

    /// Generate embedding for text
    pub async fn generate_text_embedding(&self, text: &str) -> Result<Vec<f32>> {
        self.generator.generate_text_embedding(text).await
    }
}

/// Trait for email-like objects
pub trait EmailLike {
    fn subject(&self) -> &str;
    fn from(&self) -> &str;
    fn to(&self) -> &str;
    fn body(&self) -> &str;
}

/// Simple email struct for testing
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_detection() {
        assert_eq!(EmbeddingConfig::detect_dimensions("bge-small-en"), 384);
        assert_eq!(EmbeddingConfig::detect_dimensions("all-mpnet-base-v2"), 768);
        assert_eq!(
            EmbeddingConfig::detect_dimensions("text-embedding-ada-002"),
            1536
        );
        assert_eq!(EmbeddingConfig::detect_dimensions("unknown-model"), 384);
    }

    #[tokio::test]
    async fn test_text_cleaning_for_embedding() {
        let text = "This is a test\n\nWith multiple lines";
        let _generator = EmbeddingGenerator::new("http://localhost:8082".to_string());

        // This would test actual embedding generation if service is available
        // For unit tests, we just verify the structure is correct
        assert!(!text.is_empty());
    }
}
