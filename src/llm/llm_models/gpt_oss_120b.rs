
use super::ModelHandler;
#[derive(Debug)]
pub struct GptOss120bHandler {}
impl Default for GptOss120bHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl GptOss120bHandler {
    pub fn new() -> Self {
        Self {}
    }
}
impl ModelHandler for GptOss120bHandler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        buffer.contains("**end**")
    }
    fn process_content(&self, content: &str) -> String {
        content.replace("**start**", "").replace("**end**", "")
    }
    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains("**start**")
    }
}
