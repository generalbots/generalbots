
use super::ModelHandler;
use std::sync::LazyLock;

static THINK_TAG_REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?s)<think>.*?</think>").unwrap_or_else(|_| regex::Regex::new("").unwrap())
});

#[derive(Debug)]
pub struct DeepseekR3Handler;
impl ModelHandler for DeepseekR3Handler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        buffer.contains("</think>")
    }
    fn process_content(&self, content: &str) -> String {
        THINK_TAG_REGEX.replace_all(content, "").to_string()
    }
    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains("<think>")
    }
}
