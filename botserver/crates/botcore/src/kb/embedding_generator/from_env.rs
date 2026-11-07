use super::*;

const DEFAULT_EMBEDDING_MODEL: &str = "BAAI/bge-multilingual-gemma2";
const DEFAULT_LOCAL_EMBEDDING_MODEL: &str = "bge-small-en-v1.5-f32.gguf";

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            embedding_url: "".to_string(),
            embedding_model: "BAAI/bge-multilingual-gemma2".to_string(),
            embedding_key: None,
            dimensions: 384, // Default to BGE-Small dimensions, will be overridden by config
            batch_size: 2,
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

    pub fn from_bot_config(pool: &DbPool, _bot_id: &uuid::Uuid) -> Self {
        use crate::config::ConfigManager;

        let config_manager = ConfigManager::new(pool.clone());

        let embedding_model_raw = config_manager
            .get_config(_bot_id, "embedding-model", Some(""))
            .unwrap_or_default();
        let embedding_model = if embedding_model_raw.trim().is_empty() {
            DEFAULT_EMBEDDING_MODEL.to_string()
        } else {
            embedding_model_raw.clone()
        };

        let embedding_url_raw = config_manager
            .get_config(_bot_id, "embedding-url", Some(""))
            .unwrap_or_default();

        // The local llama.cpp embedding server always uses the bundled GGUF
        // when no model is configured (see `ensure_llama_servers_running`).
        let local_embedding_model = if embedding_model_raw.trim().is_empty() {
            DEFAULT_LOCAL_EMBEDDING_MODEL
        } else {
            embedding_model_raw.as_str()
        };
        let local_embedding_available =
            local_embedding::local_embedding_model_installed(local_embedding_model)
                || is_embedding_server_ready();

        // Only default to (or keep) the local llama.cpp server when it can
        // actually be started (model present) or is already answering. Without
        // this, a fresh desktop install with a hosted LLM pointed every
        // embedding request at localhost:8082 even though
        // `ensure_llama_servers_running` had just skipped that server for a
        // missing model (#1799).
        let embedding_url = if embedding_url_raw.trim().is_empty() {
            if local_embedding_available {
                let default_url = "http://localhost:8082/v1/embeddings".to_string();
                botlog::safe_info!(
                    "No embedding-url configured for bot {_bot_id}, using default: {default_url}"
                );
                default_url
            } else {
                botlog::safe_info!(
                    "No embedding-url configured for bot {_bot_id} and local embedding model '{}' is not installed; using the local hash fallback",
                    local_embedding_model
                );
                String::new()
            }
        } else if local_embedding::is_loopback_embedding_url(&embedding_url_raw)
            && !local_embedding_available
        {
            botlog::safe_warn!(
                "embedding-url for bot {_bot_id} points at loopback ({}) but the local embedding model '{}' is not installed; using the local hash fallback",
                botlib::text::truncate_at_char_boundary(&embedding_url_raw, 50),
                local_embedding_model
            );
            String::new()
        } else {
            embedding_url_raw
        };

        let embedding_key = config_manager
            .get_config(_bot_id, "embedding-key", Some(""))
            .ok()
            .filter(|s| !s.is_empty())
            // If value looks encrypted (1:nonce:ciphertext) but decryption failed,
            // it means the master key is ephemeral (dev mode) - reject it
            .filter(|s| !s.starts_with("1:"));

        let dimensions = config_manager
            .get_config(_bot_id, "embedding-dimensions", Some(""))
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or_else(|| Self::detect_dimensions(local_embedding_model));

        botlog::safe_info!("EmbeddingConfig::from_bot_config - bot_id: {}, embedding_url: {}, embedding_key: {}, dimensions: {}",
              _bot_id,
              botlib::text::truncate_at_char_boundary(&embedding_url, 50),
              if embedding_key.is_some() { "SET" } else { "NONE" },
              dimensions);

        let batch_size = config_manager
            .get_config(_bot_id, "embedding-batch-size", Some("16"))
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(16);

        let timeout_seconds = config_manager
            .get_config(_bot_id, "embedding-timeout", Some("60"))
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        let max_concurrent_requests = config_manager
            .get_config(_bot_id, "embedding-concurrent", Some("1"))
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        Self {
            embedding_url,
            embedding_model,
            embedding_key,
            dimensions,
            batch_size,
            timeout_seconds,
            max_concurrent_requests,
            connect_timeout_seconds: 10,
        }
    }

    pub(super) fn detect_dimensions(model: &str) -> usize {
        if model.contains("gemma") || model.contains("Gemma") {
            2048
        } else if model.contains("small") || model.contains("MiniLM") {
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
