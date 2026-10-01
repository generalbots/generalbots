//! Qwen3 handler for the OpenAI-compatible tier (issue #1467).
//!
//! Qwen is the most common open-weight model across the OpenAI-compatible
//! hosts, and its reasoning mode emits a literal `<think>…</think>` block ahead
//! of the answer — the same marker DeepSeek sends, but with different tokens.
//! `deepseek_v4::extract_think_tags` looks for the DeepSeek API's `" thinking"`
//! / `" response"` framing and would pass a Qwen answer through untouched,
//! reasoning included, so the two families need separate handlers.

use super::{ModelHandler, ProcessedChunk};

const OPEN: &str = "<think>";
const CLOSE: &str = "</think>";

/// Splits `<think>` reasoning out of a Qwen answer.
///
/// Returns `(answer, reasoning)`. An unterminated block — which happens when a
/// response is cut off mid-reasoning — is treated as reasoning too, so a
/// truncated answer never leaks the model's private chain of thought.
fn split_think_blocks(content: &str) -> (String, String) {
    let mut answer = String::new();
    let mut reasoning = String::new();
    let mut rest = content;

    while let Some(start) = rest.find(OPEN) {
        answer.push_str(&rest[..start]);
        let after = &rest[start + OPEN.len()..];
        match after.find(CLOSE) {
            Some(end) => {
                reasoning.push_str(&after[..end]);
                rest = &after[end + CLOSE.len()..];
            }
            None => {
                reasoning.push_str(after);
                rest = "";
                break;
            }
        }
    }
    answer.push_str(rest);

    (answer.trim().to_string(), reasoning.trim().to_string())
}

/// Longest proper prefix of `tag` that `text` ends with, if any.
///
/// A `<think>` marker can be split across two SSE chunks, in which case the
/// buffer ends with `<thi`. Emitting that as answer text would put markup in the
/// chat. ASCII tags make the byte slicing safe: matching a short ASCII suffix
/// guarantees the split index lands on a char boundary.
fn pending_tag(text: &str) -> Option<String> {
    let mut best: Option<String> = None;
    for tag in [OPEN, CLOSE] {
        // The full tag cannot appear here — `split_think_blocks` consumes every
        // complete marker — so only proper prefixes are candidates.
        for len in 1..tag.len() {
            let prefix = &tag[..len];
            let is_longest = best.as_ref().is_none_or(|current| current.len() < len);
            if text.ends_with(prefix) && is_longest {
                best = Some(prefix.to_string());
            }
        }
    }
    best
}

#[derive(Debug, Default)]
pub struct QwenHandler;

impl ModelHandler for QwenHandler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        buffer.contains(CLOSE)
    }

    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains(OPEN)
    }

    fn skip_reasoning_content(&self) -> bool {
        true
    }

    fn process_content(&self, content: &str) -> String {
        split_think_blocks(content).0
    }

    fn process_content_streaming(&self, chunk: &str, state: &mut String) -> ProcessedChunk {
        state.push_str(chunk);
        let (answer, reasoning) = split_think_blocks(state);
        state.clear();

        // A marker split across SSE chunk boundaries must not be emitted as
        // answer text, so a trailing partial tag stays in the state buffer for
        // the next chunk to complete.
        match pending_tag(&answer) {
            Some(pending) => {
                let keep_from = answer.len() - pending.len();
                state.push_str(&answer[keep_from..]);
                ProcessedChunk {
                    content: answer[..keep_from].to_string(),
                    reasoning,
                }
            }
            None => ProcessedChunk {
                content: answer,
                reasoning,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_a_complete_reasoning_block() {
        let (answer, reasoning) = split_think_blocks("<think>weighing it</think>Final answer");
        assert_eq!(answer, "Final answer");
        assert_eq!(reasoning, "weighing it");
    }

    #[test]
    fn keeps_text_before_and_between_blocks() {
        let (answer, reasoning) =
            split_think_blocks("Pre<think>a</think>Mid<think>b</think>Post");
        assert_eq!(answer, "PreMidPost");
        assert_eq!(reasoning, "ab");
    }

    #[test]
    fn an_unterminated_block_is_reasoning_not_answer() {
        let (answer, reasoning) = split_think_blocks("Answer so far<think>still thinking");
        assert_eq!(answer, "Answer so far");
        assert_eq!(reasoning, "still thinking");
    }

    #[test]
    fn an_answer_without_think_tags_is_untouched() {
        let (answer, reasoning) = split_think_blocks("Plain answer");
        assert_eq!(answer, "Plain answer");
        assert!(reasoning.is_empty());
    }

    #[test]
    fn handler_reports_markers_and_completion() {
        let handler = QwenHandler;
        assert!(handler.has_analysis_markers("x<think>"));
        assert!(!handler.has_analysis_markers("x"));
        assert!(!handler.is_analysis_complete("x<think>still"));
        assert!(handler.is_analysis_complete("<think>done</think>"));
        assert!(handler.skip_reasoning_content());
        assert_eq!(handler.process_content("<think>r</think>a"), "a");
    }

    #[test]
    fn a_trailing_partial_marker_is_held_back() {
        assert_eq!(pending_tag("Answer<thin").as_deref(), Some("<thin"));
        assert_eq!(pending_tag("Answer</think").as_deref(), Some("</think"));
        assert_eq!(pending_tag("Complete answer"), None);
        assert_eq!(pending_tag("a < b"), None);
    }

    #[test]
    fn streaming_holds_back_a_tag_split_across_chunks() {
        let handler = QwenHandler;
        let mut state = String::new();

        let first = handler.process_content_streaming("Answer<thin", &mut state);
        assert_eq!(first.content, "Answer");
        assert!(!state.is_empty(), "the partial tag must be retained");

        let second = handler.process_content_streaming("k>reasoning</think>Done", &mut state);
        assert!(state.is_empty(), "the completed block must be consumed");
        assert_eq!(second.content, "Done");
        assert_eq!(second.reasoning, "reasoning");
    }

    #[test]
    fn streaming_passes_a_clean_chunk_straight_through() {
        let handler = QwenHandler;
        let mut state = String::new();
        let chunk = handler.process_content_streaming("Hello", &mut state);
        assert_eq!(chunk.content, "Hello");
        assert!(chunk.reasoning.is_empty());
        assert!(state.is_empty());
    }
}
