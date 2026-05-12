#[cfg(feature = "directory")]
use std::sync::Arc;

#[cfg(feature = "llm")]
#[derive(Debug, Clone)]
pub struct MockLLMProvider {
    pub response: String,
}

#[cfg(feature = "llm")]
impl MockLLMProvider {
    pub fn new() -> Self {
        Self { response: String::new() }
    }
}

#[cfg(feature = "llm")]
impl botlib::traits::LLMProvider for MockLLMProvider {
    fn generate(&self, _prompt: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>> {
        let response = self.response.clone();
        Box::pin(async move { Ok(response) })
    }
    fn generate_with_context(&self, _prompt: &str, _context: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>> {
        let response = self.response.clone();
        Box::pin(async move { Ok(response) })
    }
}

#[cfg(feature = "directory")]
pub fn create_mock_auth_service() -> Arc<tokio::sync::Mutex<dyn botlib::traits::AuthServiceTrait>> {
    unimplemented!()
}
