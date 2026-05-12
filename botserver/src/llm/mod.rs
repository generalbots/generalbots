use std::sync::Arc;
pub use botllm::*;


pub struct BotlibLLMProviderWrapper(pub Arc<dyn crate::llm::LLMProvider>);

impl std::fmt::Debug for BotlibLLMProviderWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BotlibLLMProviderWrapper").finish_non_exhaustive()
    }
}

impl botlib::traits::LLMProvider for BotlibLLMProviderWrapper {
    fn generate(&self, prompt: &str, config: &serde_json::Value, model: &str, key: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, Box<dyn std::error::Error + Send + Sync>>> + Send>> {
        let prompt = prompt.to_string();
        let config = config.clone();
        let model = model.to_string();
        let key = key.to_string();
        let inner = self.0.clone();
        Box::pin(async move {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            rt.block_on(async { inner.generate(&prompt, &config, &model, &key).await.map_err(|e| Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error + Send + Sync>) })
        })
    }
    fn generate_simple(&self, prompt: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>> {
        let prompt = prompt.to_string();
        let inner = self.0.clone();
        Box::pin(async move {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| e.to_string())?;
            rt.block_on(async { inner.generate(&prompt, &serde_json::Value::Null, "", "").await.map_err(|e| e.to_string()) })
        })
    }
    fn generate_with_context(&self, prompt: &str, _context: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>> {
        self.generate_simple(prompt)
    }
}

