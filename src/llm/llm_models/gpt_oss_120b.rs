use super::ModelHandler;

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

fn strip_think_tags(content: &str) -> String {
    let result = content
        .replace("<think>", "")
        .replace("</think>", "")
        .replace("**start**", "")
        .replace("**end**", "");
    if result.is_empty() && !content.is_empty() {
        content.to_string()
    } else {
        result
    }
}

impl ModelHandler for GptOss120bHandler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        buffer.contains("**end**") || buffer.contains("</think>")
    }
    fn process_content(&self, content: &str) -> String {
        strip_think_tags(content)
    }
    fn process_content_streaming(&self, chunk: &str, state: &mut String) -> String {
        let old_len = state.len();
        state.push_str(chunk);

        let mut clean_current = String::new();
        let mut in_think = false;

        let full_text = state.as_str();
        let mut current_pos = 0;

        while current_pos < full_text.len() {
            if !in_think {
                if full_text[current_pos..].starts_with("<think>") {
                    in_think = true;
                    current_pos += 7;
                } else if full_text[current_pos..].starts_with("**start**") {
                    current_pos += 10;
                } else if full_text[current_pos..].starts_with("**end**") {
                    current_pos += 7;
                } else {
                    let c = full_text[current_pos..].chars().next().unwrap();
                    if current_pos >= old_len {
                        clean_current.push(c);
                    }
                    current_pos += c.len_utf8();
                }
            } else {
                if full_text[current_pos..].starts_with("</think>") {
                    in_think = false;
                    current_pos += 8;
                } else {
                    let c = full_text[current_pos..].chars().next().unwrap();
                    current_pos += c.len_utf8();
                }
            }
        }

        if clean_current.is_empty() && chunk.len() > 0 {
            chunk.to_string()
        } else {
            clean_current
        }
    }
    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains("**start**") || buffer.contains("<think>")
    }
}
