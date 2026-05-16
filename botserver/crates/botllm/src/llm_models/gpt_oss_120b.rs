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
    let chars: Vec<char> = content.chars().collect();
    let mut pos = 0;

    while pos < chars.len() {
        if !in_thinking {
            if pos + 10 <= chars.len() && chars[pos..pos+10].iter().collect::<String>() == "<thinking>" {
                in_thinking = true;
                thinking_content.clear();
                pos += 10;
                continue;
            } else if pos + 9 <= chars.len() && chars[pos..pos+9].iter().collect::<String>() == "**start**" {
                in_thinking = true;
                thinking_content.clear();
                pos += 9;
                continue;
            }
        } else {
            if pos + 12 <= chars.len() && chars[pos..pos+12].iter().collect::<String>() == "</thinking>" {
                in_thinking = false;
                pos += 12;
                continue;
            } else if pos + 7 <= chars.len() && chars[pos..pos+7].iter().collect::<String>() == "**end**" {
                in_thinking = false;
                pos += 7;
                continue;
            } else {
                thinking_content.push(chars[pos]);
                pos += 1;
                continue;
            }
        }

        result.push(chars[pos]);
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
        // For streaming, we only receive actual content (reasoning is skipped by the caller).
        // Just pass through the chunk — no thinking tags in content field.
        state.push_str(chunk);
        chunk.to_string()
    }

    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains("**start**") || buffer.contains("<thinking>")
    }
}
