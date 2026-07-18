use super::deepseek_v4::extract_think_tags;
use super::{ModelHandler, ProcessedChunk};
use regex::Regex;
use std::sync::LazyLock;

static ANALYSIS_MARKER_REGEX: LazyLock<Result<Regex, regex::Error>> =
    LazyLock::new(|| Regex::new(r"analysis<\|message\|>"));

static FINAL_MARKER_REGEX: LazyLock<Result<Regex, regex::Error>> =
    LazyLock::new(|| Regex::new(r"<\|message\|>final<\|message\|>"));

fn separate_text_number(text: &str) -> String {
    let mut result = String::with_capacity(text.len() + 8);
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        result.push(c);
        if let Some(&next) = chars.peek() {
            if c.is_alphabetic() && next.is_ascii_digit()
                || c.is_ascii_digit() && next.is_alphabetic()
            {
                result.push(' ');
            }
        }
    }
    result
}

#[derive(Debug)]
pub struct GptOss20bHandler;

impl ModelHandler for GptOss20bHandler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        buffer.contains("")
            || (if let Ok(re) = &*FINAL_MARKER_REGEX {
                re.is_match(buffer)
            } else {
                false
            })
    }

    fn process_content(&self, content: &str) -> String {
        let (without_think, _reasoning) = extract_think_tags(content);
        if without_think.is_empty() {
            return String::new();
        }
        let cleaned = if let Ok(re) = &*FINAL_MARKER_REGEX {
            re.replace_all(&without_think, "").to_string()
        } else {
            without_think
        };
        // Safety net: fix text-number boundaries across the full text
        separate_text_number(&cleaned)
    }

    fn process_content_streaming(&self, chunk: &str, state: &mut String) -> ProcessedChunk {
        if chunk.is_empty() {
            return ProcessedChunk { content: String::new(), reasoning: String::new() };
        }

        state.push_str(chunk);

        const MIN_EMIT: usize = 1;

        if state.len() < MIN_EMIT {
            return ProcessedChunk { content: String::new(), reasoning: String::new() };
        }

        let (content, reasoning) = extract_think_tags(state);

        state.clear();
        // Keep any pending partial in state
        if reasoning.is_empty() && content.contains("<think>") {
            if let Some(think_pos) = content.rfind("<think>") {
                state.push_str(&content[think_pos..]);
            }
        }

        ProcessedChunk {
            content: separate_text_number(&content),
            reasoning,
        }
    }

    fn has_analysis_markers(&self, buffer: &str) -> bool {
        (if let Ok(re) = &*ANALYSIS_MARKER_REGEX {
            re.is_match(buffer)
        } else {
            buffer.contains("analysis<|message|>")
        }) || buffer.contains("")
    }
}
