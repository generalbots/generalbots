use super::{ModelHandler, ProcessedChunk};

/// Extract content outside think tags, returning (content, reasoning).
/// If everything is inside think tags, content is empty and reasoning has the full text.
pub fn extract_think_tags(content: &str) -> (String, String) {
    let mut result = String::new();
    let mut reasoning = String::new();
    let mut in_think = false;
    let chars: Vec<char> = content.chars().collect();
    let mut pos = 0;

    while pos < chars.len() {
        if !in_think {
            if pos + 9 <= chars.len() && chars[pos..pos+9].iter().collect::<String>() == " thinking" {
                in_think = true;
                pos += 9;
                continue;
            }
            result.push(chars[pos]);
            pos += 1;
        } else {
            if pos + 9 <= chars.len() && chars[pos..pos+9].iter().collect::<String>() == " response" {
                in_think = false;
                pos += 9;
                continue;
            }
            reasoning.push(chars[pos]);
            pos += 1;
        }
    }

    (result.trim().to_string(), reasoning.trim().to_string())
}

#[derive(Debug)]
pub struct DeepseekV4Handler;

impl ModelHandler for DeepseekV4Handler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        buffer.contains(" response")
    }

    fn process_content(&self, content: &str) -> String {
        let (cleaned, _reasoning) = extract_think_tags(content);
        cleaned
    }

    fn process_content_streaming(&self, chunk: &str, _state: &mut String) -> ProcessedChunk {
        // Deepseek API sends reasoning via "reasoning_content" SSE field,
        // NOT in the "content" field. Forward content chunks directly.
        // The process_content cleanup handles any stray think tags from edge cases.
        ProcessedChunk { content: chunk.to_string(), reasoning: String::new() }
    }

    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains("lldsdkx")
    }

    fn skip_reasoning_content(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_think_tags() {
        let (c, r) = extract_think_tags("Hello  thinkingthinking... response World");
        assert_eq!(c, "Hello  World");
        assert_eq!(r, "thinking...");

        let (c, r) = extract_think_tags(" thinkinghmm responseAns");
        assert_eq!(c, "Ans");
        assert_eq!(r, "hmm");

        let (c, r) = extract_think_tags("Start  thinkingthinking...");
        assert_eq!(c, "Start");
        assert_eq!(r, "thinking...");
    }

    #[test]
    fn test_process_content_streaming() {
        let handler = DeepseekV4Handler;
        let mut state = String::new();

        let r1 = handler.process_content_streaming("He", &mut state);
        assert_eq!(r1.content, "He");
        assert_eq!(r1.reasoning, "");
        assert!(state.is_empty());

        let r2 = handler.process_content_streaming("llo World", &mut state);
        assert_eq!(r2.content, "llo World");
        assert_eq!(r2.reasoning, "");
        assert!(state.is_empty());
    }
}