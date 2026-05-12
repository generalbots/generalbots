use super::ModelHandler;

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

    fn process_content_streaming(&self, chunk: &str, state: &mut String) -> String {
        let old_len = state.len();
        state.push_str(chunk);
        
        let mut clean_current = String::new();
        let mut in_think = false;
        
        let mut current_pos = 0;
        let full_text = state.as_str();
        
        while current_pos < full_text.len() {
            if !in_think {
                if full_text[current_pos..].starts_with("<think>") {
                    in_think = true;
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
        
        clean_current
    }

    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains("<think>")
    }
}
