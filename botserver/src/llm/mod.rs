use std::sync::Arc;
pub use botllm::*;


pub struct BotlibLLMProviderWrapper {
    pub inner: Arc<dyn crate::llm::LLMProvider>,
    pub model: String,
    pub key: String,
}

impl std::fmt::Debug for BotlibLLMProviderWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BotlibLLMProviderWrapper").finish_non_exhaustive()
    }
}

impl BotlibLLMProviderWrapper {
    pub fn new(inner: Arc<dyn crate::llm::LLMProvider>, model: String, key: String) -> Self {
        Self { inner, model, key }
    }
}

impl botlib::traits::LLMProvider for BotlibLLMProviderWrapper {
    fn generate(&self, prompt: &str, config: &serde_json::Value, model: &str, key: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Box<dyn std::error::Error + Send + Sync>>> + Send>> {
        let prompt = prompt.to_string();
        let config = config.clone();
        let model = model.to_string();
        let key = key.to_string();
        let inner = self.inner.clone();
        Box::pin(async move {
            inner.generate(&prompt, &config, &model, &key).await.map_err(|e| Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error + Send + Sync>)
        })
    }
    fn generate_simple(&self, prompt: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>> {
        let prompt = prompt.to_string();
        let inner = self.inner.clone();
        let model = self.model.clone();
        let key = self.key.clone();
        Box::pin(async move {
            inner.generate(&prompt, &serde_json::Value::Null, &model, &key).await.map_err(|e| e.to_string())
        })
    }
    fn generate_with_context(&self, prompt: &str, _context: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>> {
        self.generate_simple(prompt)
    }
    fn generate_stream(
        &self,
        prompt: &str,
        config: &serde_json::Value,
        tx: tokio::sync::mpsc::Sender<String>,
        model: &str,
        key: &str,
        tools: Option<&Vec<serde_json::Value>>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> {
        let prompt = prompt.to_string();
        let config = config.clone();
        let model = model.to_string();
        let key = key.to_string();
        let tools = tools.map(|t| t.clone());
        let inner = self.inner.clone();
        Box::pin(async move {
            inner.generate_stream(&prompt, &config, tx, &model, &key, tools.as_ref()).await.map_err(|e| e.to_string())
        })
    }
}
