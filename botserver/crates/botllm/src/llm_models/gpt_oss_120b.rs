use super::ModelHandler;
use log;

/// Handler for GPT-OSS 120B model with thinking tags filtering
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

/// Extract content outside thinking tags
/// If everything is inside thinking tags, extract from inside them
fn strip_think_tags(content: &str) -> String {
    if content.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut in_thinking = false;
    let mut thinking_content = String::new();
    let mut pos = 0;

    while pos < content.len() {
        let remaining = &content[pos..];
        if !in_thinking {
            if remaining.starts_with("<thinking>") {
                in_thinking = true;
                thinking_content.clear();
                pos += 9;
                continue;
            } else if remaining.starts_with("**start**") {
                in_thinking = true;
                thinking_content.clear();
                pos += 8;
                continue;
            }
        } else {
            if remaining.starts_with("</thinking>") {
                in_thinking = false;
                pos += 11;
                continue;
            } else if remaining.starts_with("**end**") {
                in_thinking = false;
                pos += 6;
                continue;
            } else {
                thinking_content.push(content.chars().nth(pos).unwrap());
                pos += 1;
                continue;
            }
        }

        if !in_thinking {
            result.push(content.chars().nth(pos).unwrap());
        }
        pos += 1;
    }

    // If we got content outside thinking tags, return it
    if !result.trim().is_empty() {
        return result;
    }

    // If everything was inside thinking tags, return that content
    if !thinking_content.trim().is_empty() {
        log::debug!("gpt_oss_120b: All content was in thinking tags, returning thinking content");
        return thinking_content;
    }

    // Fallback: try regex extraction
    if let Ok(re) = regex::Regex::new(r"<thinking>(.*?)</thinking>") {
        let mut extracted = String::new();
        for cap in re.captures_iter(content) {
            if let Some(m) = cap.get(1) {
                if !extracted.is_empty() {
                    extracted.push(' ');
                }
                extracted.push_str(m.as_str());
            }
        }
        if !extracted.is_empty() {
            return extracted;
        }
    }

    // Last resort: return original
    log::warn!("gpt_oss_120b: Could not extract meaningful content, returning original");
    content.to_string()
}

impl ModelHandler for GptOss120bHandler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        buffer.contains("**end**") || buffer.contains("</thinking>")
    }

    fn process_content(&self, content: &str) -> String {
        strip_think_tags(content)
    }

    fn process_content_streaming(&self, chunk: &str, state: &mut String) -> String {
        state.push_str(chunk);

        // Process accumulated state and return new content since last call
        let processed = strip_think_tags(state);

        // For streaming, we return the entire processed content
        // The caller should handle deduplication if needed
        processed
    }

    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains("**start**") || buffer.contains("<thinking>")
    }
}
