use super::ModelHandler;
#[derive(Debug)]
pub struct GptOss20bHandler;
impl ModelHandler for GptOss20bHandler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        buffer.ends_with("final")
    }
    fn process_content(&self, content: &str) -> String {
        if let Some(pos) = content.find("final") {
            content[..pos].to_string()
        } else {
            content.to_string()
        }
    }
    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains("analysis<|message|>")
    }
}
