use super::ModelHandler;
use regex::Regex;
use std::sync::LazyLock;

static THINK_TAG_REGEX: LazyLock<Result<Regex, regex::Error>> =
    LazyLock::new(|| Regex::new(r"(?s)<think>.*?</think>"));

pub fn strip_think_tags(content: &str) -> String {
    // We want to strip <think>...</think> OR <think> until end of string (streaming)
    let mut result = content.to_string();
    if let Some(start_idx) = result.find("<think>") {
        if let Some(end_idx) = result[start_idx..].find("</think>") {
            // Case 1: Fully enclosed
            result = format!(
                "{}{}",
                &result[..start_idx],
                &result[start_idx + end_idx + 8..]
            );
            // Recursive call to catch multiple blocks
            return strip_think_tags(&result);
        } else {
            // Case 2: Unclosed (streaming)
            result = result[..start_idx].to_string();
        }
    }
    result
}

#[derive(Debug)]
pub struct DeepseekR3Handler;

impl ModelHandler for DeepseekR3Handler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        buffer.contains("</think>")
    }

    fn process_content(&self, content: &str) -> String {
        strip_think_tags(content)
    }

    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains("<think>")
    }
}
