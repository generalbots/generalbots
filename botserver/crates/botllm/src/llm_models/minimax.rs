use super::{ModelHandler, ProcessedChunk};

/// Strips thinking/analysis markers and returns (content, reasoning).
/// Handles three marker types:
/// - Chinese: （分析）...（/分析）
/// - English: <think>...</think>
/// - Chinese alt: 【分析】...【/分析】
pub fn extract_think_tags(content: &str) -> (String, String) {
    let mut result = String::new();
    let mut reasoning = String::new();
    let chars: Vec<char> = content.chars().collect();
    let mut pos = 0;
    let mut in_think = false;

    while pos < chars.len() {
        if !in_think {
            let mut found = false;
            for (start_tag, _) in &[("（分析）", "（/分析）"), ("<think>", "</think>"), ("【分析】", "【/分析】")] {
                let tag_chars: Vec<char> = start_tag.chars().collect();
                if pos + tag_chars.len() <= chars.len() {
                    let slice: String = chars[pos..pos + tag_chars.len()].iter().collect();
                    if slice == *start_tag {
                        in_think = true;
                        pos += tag_chars.len();
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                result.push(chars[pos]);
                pos += 1;
            }
        } else {
            let mut found = false;
            for (_, end_tag) in &[("（分析）", "（/分析）"), ("<think>", "</think>"), ("【分析】", "【/分析】")] {
                let tag_chars: Vec<char> = end_tag.chars().collect();
                if pos + tag_chars.len() <= chars.len() {
                    let slice: String = chars[pos..pos + tag_chars.len()].iter().collect();
                    if slice == *end_tag {
                        in_think = false;
                        pos += tag_chars.len();
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                reasoning.push(chars[pos]);
                pos += 1;
            }
        }
    }

    (result.trim().to_string(), reasoning.trim().to_string())
}

#[derive(Debug)]
pub struct MinimaxHandler;

impl Default for MinimaxHandler {
    fn default() -> Self {
        Self::new()
    }
}

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
        let (content, _reasoning) = extract_think_tags(content);
        content
    }

    fn process_content_streaming(&self, chunk: &str, state: &mut String) -> ProcessedChunk {
        state.push_str(chunk);

        let (content, reasoning) = extract_think_tags(state);
        state.clear();

        // Keep unclosed partial tags in state
        let mut pending = String::new();
        for &start_tag in &["（分析）", "<think>", "【分析】"] {
            if let Some(pos) = content.rfind(start_tag) {
                let remaining = &content[pos..];
                if !remaining.contains("（/分析）") && !remaining.contains("</think>") && !remaining.contains("【/分析】") {
                    pending = remaining.to_string();
                    break;
                }
            }
        }
        if !pending.is_empty() {
            state.push_str(&content[content.len() - pending.len()..]);
        }

        ProcessedChunk { content, reasoning }
    }

    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains("（分析）") || buffer.contains("<think>") || buffer.contains("【分析】")
    }
}
