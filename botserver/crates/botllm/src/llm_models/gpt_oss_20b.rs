use super::deepseek_v4::strip_think_tags;
use super::ModelHandler;
use regex::Regex;
use std::sync::LazyLock;

static ANALYSIS_MARKER_REGEX: LazyLock<Result<Regex, regex::Error>> =
    LazyLock::new(|| Regex::new(r"analysis<\|message\|>"));

static FINAL_MARKER_REGEX: LazyLock<Result<Regex, regex::Error>> =
    LazyLock::new(|| Regex::new(r"<\|message\|>final<\|message\|>"));

static TEXT_NUMBER_REGEX: LazyLock<Result<Regex, regex::Error>> =
    LazyLock::new(|| Regex::new(r"(?<=\p{L})(?=\d)|(?<=\d)(?=\p{L})"));

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
        let without_think = strip_think_tags(content);
        if without_think.is_empty() {
            return String::new();
        }
        let cleaned = if let Ok(re) = &*FINAL_MARKER_REGEX {
            re.replace_all(&without_think, "").to_string()
        } else {
            without_think
        };
        // Safety net: fix text-number boundaries across the full text
        if let Ok(re) = &*TEXT_NUMBER_REGEX {
            re.replace_all(&cleaned, " ").to_string()
        } else {
            cleaned
        }
    }

    fn process_content_streaming(&self, chunk: &str, state: &mut String) -> String {
        if chunk.is_empty() {
            return String::new();
        }

        let mut processed = chunk.to_string();

        // 1. Cross-chunk: detect text-number boundary between accumulated state and new chunk
        if let (Some(last), Some(first)) = (state.chars().last(), chunk.chars().next()) {
            if (last.is_alphabetic() && first.is_ascii_digit())
                || (last.is_ascii_digit() && first.is_alphabetic())
            {
                processed.insert(0, ' ');
            }
        }

        // 2. Within-chunk: regex catches boundaries inside a single token (e.g. "em2021")
        if let Ok(re) = &*TEXT_NUMBER_REGEX {
            processed = re.replace_all(&processed, " ").to_string();
        }

        state.push_str(&processed);
        processed
    }

    fn has_analysis_markers(&self, buffer: &str) -> bool {
        (if let Ok(re) = &*ANALYSIS_MARKER_REGEX {
            re.is_match(buffer)
        } else {
            buffer.contains("analysis<|message|>")
        }) || buffer.contains("")
    }
}
