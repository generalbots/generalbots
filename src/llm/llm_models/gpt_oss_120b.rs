use super::deepseek_r3::strip_think_tags;
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
        buffer.contains("**end**") || buffer.contains("</think>")
    }
    fn process_content(&self, content: &str) -> String {
        strip_think_tags(content)
            .replace("**start**", "")
            .replace("**end**", "")
    }
    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains("**start**") || buffer.contains("<think>")
    }
}
