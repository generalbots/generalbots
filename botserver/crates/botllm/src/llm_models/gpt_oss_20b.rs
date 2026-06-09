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

        // Accumulate raw chunks for token-level look-ahead.
        // This prevents text-number boundaries from being split across
        // streaming chunks (e.g. "fevereiro20" + "21" = "fevereiro2021").
        state.push_str(chunk);

        // Wait until enough content is accumulated (~4-5 tokens)
        // before processing with the regex.
        const MIN_EMIT: usize = 50;

        if state.len() < MIN_EMIT {
            return String::new();
        }

        // Apply text-number regex to the full accumulated buffer
        let result = if let Ok(re) = &*TEXT_NUMBER_REGEX {
            re.replace_all(state, " ").to_string()
        } else {
            state.clone()
        };

        state.clear();
        result
    }

    fn has_analysis_markers(&self, buffer: &str) -> bool {
        (if let Ok(re) = &*ANALYSIS_MARKER_REGEX {
            re.is_match(buffer)
        } else {
            buffer.contains("analysis<|message|>")
        }) || buffer.contains("")
    }
}
