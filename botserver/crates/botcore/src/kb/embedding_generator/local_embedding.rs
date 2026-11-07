use super::*;
use std::path::{Path, PathBuf};

/// True when the embedding URL targets a loopback interface. The local
/// llama.cpp embedding server managed by `ensure_llama_servers_running`
/// always binds to loopback, so these URLs are the ones that become dead
/// when its model is not installed.
pub(crate) fn is_loopback_embedding_url(url: &str) -> bool {
    match url::Url::parse(url) {
        Ok(parsed) => matches!(
            parsed.host_str(),
            Some("localhost") | Some("127.0.0.1") | Some("::1")
        ),
        Err(_) => false,
    }
}

/// Reduce a model reference to its bare filename, matching how
/// `botllm::local::model_file_name` resolves the local llama.cpp model.
pub(crate) fn model_file_name(model: &str) -> &str {
    let model = model.trim();
    if model.is_empty() {
        return model;
    }
    model.rsplit(['/', '\\']).next().unwrap_or(model)
}

pub(crate) fn local_embedding_model_path(model: &str) -> PathBuf {
    Path::new(&crate::shared::utils::get_stack_path())
        .join("data")
        .join("llm")
        .join(model_file_name(model))
}

/// The local embedding server can only start when its GGUF file exists under
/// `<stack>/data/llm`. Fresh desktop installs that chose a hosted provider do
/// not install it, so callers must not assume `localhost:8082` is alive.
pub(crate) fn local_embedding_model_installed(model: &str) -> bool {
    let name = model_file_name(model);
    !name.is_empty() && local_embedding_model_path(model).is_file()
}

/// Deterministic FNV-1a hash embedding used when no embedding server is
/// reachable. Kept byte-for-byte compatible with the fallback in
/// `botbasic_data::keywords::think_kb` so indexing and querying agree.
pub fn hash_embedding(text: &str, dimensions: usize) -> Vec<f32> {
    let dimensions = dimensions.max(1);
    let mut embedding = Vec::with_capacity(dimensions);
    for i in 0..dimensions {
        let hash = fnv1a_hash(text, i);
        embedding.push((hash as f32) / (u64::MAX as f32) * 2.0 - 1.0);
    }
    embedding
}

fn fnv1a_hash(text: &str, salt: usize) -> u64 {
    let mut hash: u64 = 14695981039346656037;
    for &b in text.as_bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash ^= salt as u64;
    hash = hash.wrapping_mul(1099511628211);
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_file_name_strips_legacy_directories() {
        assert_eq!(model_file_name("../../data/llm/a.gguf"), "a.gguf");
        assert_eq!(model_file_name("C:\\models\\a.gguf"), "a.gguf");
        assert_eq!(model_file_name("a.gguf"), "a.gguf");
        assert_eq!(model_file_name("  "), "");
    }

    #[test]
    fn loopback_urls_are_detected() {
        assert!(is_loopback_embedding_url(
            "http://localhost:8082/v1/embeddings"
        ));
        assert!(is_loopback_embedding_url(
            "http://127.0.0.1:8082/v1/embeddings"
        ));
        assert!(is_loopback_embedding_url("http://[::1]:8082/v1/embeddings"));
        assert!(!is_loopback_embedding_url(
            "https://api.example.com/v1/embeddings"
        ));
        assert!(!is_loopback_embedding_url("not a url"));
    }

    #[test]
    fn hash_embedding_is_deterministic_and_sized() {
        let first = hash_embedding("hello world", 384);
        let second = hash_embedding("hello world", 384);
        assert_eq!(first.len(), 384);
        assert_eq!(first, second);
        assert_ne!(first, hash_embedding("another text", 384));
    }
}
