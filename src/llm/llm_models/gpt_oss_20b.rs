use super::deepseek_r3::strip_think_tags;
use super::ModelHandler;
use std::sync::LazyLock;
use regex::Regex;

static ANALYSIS_MARKER_REGEX: LazyLock<Result<Regex, regex::Error>> = LazyLock::new(|| {
    Regex::new(r"analysis<\|message\|>")
});

#[derive(Debug)]
pub struct GptOss20bHandler;

impl ModelHandler for GptOss20bHandler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        buffer.contains("final") || buffer.contains("</think>")
    }

    fn process_content(&self, content: &str) -> String {
        let without_think = strip_think_tags(content);
        if without_think.is_empty() {
            return String::new();
        }
        match without_think.find("final") {
            Some(pos) => without_think[..pos].to_string(),
            None => without_think,
        }
    }

    fn has_analysis_markers(&self, buffer: &str) -> bool {
        (if let Ok(re) = &*ANALYSIS_MARKER_REGEX {
            re.is_match(buffer)
        } else {
            buffer.contains("analysis<|message|>")
        }) || buffer.contains("<think>")
    }
}
