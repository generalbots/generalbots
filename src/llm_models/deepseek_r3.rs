use super::ModelHandler;

pub struct DeepseekR3Handler;

impl ModelHandler for DeepseekR3Handler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        buffer.contains("</think>")
    }

    fn process_content(&self, content: &str) -> String {
        content.replace("<think>", "").replace("</think>", "")
    }

    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains("<think>")
    }
}
