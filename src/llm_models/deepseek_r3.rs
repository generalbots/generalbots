use super::ModelHandler;
use regex;
pub struct DeepseekR3Handler;
impl ModelHandler for DeepseekR3Handler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        buffer.contains("</think>")
    }
    fn process_content(&self, content: &str) -> String {
        let re = regex::Regex::new(r"(?s)<think>.*?</think>").unwrap();
        re.replace_all(content, "").to_string()
    }
    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains("<think>")
    }
}
