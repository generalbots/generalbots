use super::ModelHandler;

pub fn strip_think_tags(content: &str) -> String {
    let mut result = content.to_string();

    // Chinese: （分析）...（/分析） or unclosed （分析）...
    while let Some(start_idx) = result.find("（分析）") {
        if let Some(end_idx) = result[start_idx..].find("（/分析）") {
            result = format!(
                "{}{}",
                &result[..start_idx],
                &result[start_idx + end_idx + 4..]
            );
        } else {
            // Unclosed - strip to the end
            result = result[..start_idx].to_string();
            break;
        }
    }

    // English: <think>...</think> or unclosed <think>...
    while let Some(start_idx) = result.find("<think>") {
        if let Some(end_idx) = result[start_idx..].find("</think>") {
            result = format!(
                "{}{}",
                &result[..start_idx],
                &result[start_idx + end_idx + 8..]
            );
        } else {
            // Unclosed - strip to the end
            result = result[..start_idx].to_string();
            break;
        }
    }

    // Chinese alternative: 【分析】...【/分析】 or unclosed 【分析】...
    while let Some(start_idx) = result.find("【分析】") {
        if let Some(end_idx) = result[start_idx..].find("【/分析】") {
            result = format!(
                "{}{}",
                &result[..start_idx],
                &result[start_idx + end_idx + 5..]
            );
        } else {
            // Unclosed - strip to the end
            result = result[..start_idx].to_string();
            break;
        }
    }

    result
}

#[derive(Debug)]
pub struct MinimaxHandler;

impl MinimaxHandler {
    pub fn new() -> Self {
        Self
    }
}

impl ModelHandler for MinimaxHandler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        buffer.contains("（/分析）") || buffer.contains("</think>") || buffer.contains("【/分析】")
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
                } else if full_text[current_pos..].starts_with("（分析）") || full_text[current_pos..].starts_with("【分析】") {
                    in_think = true;
                    current_pos += 12; // UTF-8 for these 3-char Chinese tags
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
                } else if full_text[current_pos..].starts_with("（/分析）") || full_text[current_pos..].starts_with("【/分析】") {
                    in_think = false;
                    current_pos += 13; // UTF-8 for these 4-char Chinese tags
                } else {
                    let c = full_text[current_pos..].chars().next().unwrap();
                    current_pos += c.len_utf8();
                }
            }
        }
        
        clean_current
    }

    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains("（分析）") || buffer.contains("<think>") || buffer.contains("【分析】")
    }
}
