
use super::ModelHandler;
use std::sync::LazyLock;

static THINK_TAG_REGEX: LazyLock<Result<regex::Regex, regex::Error>> = LazyLock::new(|| {
    regex::Regex::new(r"(?s)<think>.*?</think>")
});

#[derive(Debug)]
pub struct DeepseekR3Handler;
impl ModelHandler for DeepseekR3Handler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        buffer.contains("</think>")
    }
    fn process_content(&self, content: &str) -> String {
        if let Ok(re) = &*THINK_TAG_REGEX {
            re.replace_all(content, "").to_string()
        } else {
            content.to_string()
        }
    }
    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains("<think>")
    }
}
