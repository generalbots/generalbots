use anyhow::Result;
use log::warn;

pub struct EmbeddingGenerator {
    pub llm_endpoint: String,
}

impl EmbeddingGenerator {
    pub fn new(llm_endpoint: String) -> Self {
        Self { llm_endpoint }
    }

    pub async fn generate_text_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let embedding_url = "http://localhost:8082".to_string();
        match self.generate_local_embedding(text, &embedding_url).await {
            Ok(embedding) => Ok(embedding),
            Err(e) => {
                warn!("Local embedding failed: {e}, falling back to hash embedding");
                Self::generate_hash_embedding(text)
            }
        }
    }

    pub async fn generate_text_embedding_with_openai(
        &self,
        text: &str,
        api_key: &str,
    ) -> Result<Vec<f32>> {
        self.generate_openai_embedding(text, api_key).await
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

        // Truncate text to fit within token limit (600 tokens for safety under 768 limit)
        let truncated_text = crate::core::shared::utils::truncate_text_for_model(text, "sentence-transformers/all-MiniLM-L6-v2", 600);

        let client = reqwest::Client::new();
        let body = json!({
            "text": truncated_text,
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

    fn generate_hash_embedding(text: &str) -> Result<Vec<f32>> {
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
