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

/// Splits `<think>` reasoning out of a model answer.
///
/// Returns `(answer, reasoning)`. An unterminated block — which happens when a
/// response is cut off mid-reasoning — is treated as reasoning too, so a
/// truncated answer never leaks the model's private chain of thought.
pub(super) fn split_think_blocks(content: &str) -> (String, String) {
    let mut answer = String::new();
    let mut reasoning = String::new();
    let mut rest = content;

    while let Some(start) = rest.find(OPEN) {
        answer.push_str(&rest[..start]);
        let after = &rest[start + OPEN.len()..];
        match after.find(CLOSE) {
            Some(end) => {
                reasoning.push_str(&after[..end]);
                // Strip at most the newline(s) directly after </think> so the
                // answer text does not start with a blank line, but spaces and
                // other whitespace (which may be meaningful formatting) survive.
                let after_close = &after[end + CLOSE.len()..];
                rest = after_close.trim_start_matches(|c: char| c == '\n' || c == '\r');
            }
            None => {
                reasoning.push_str(after);
                rest = "";
                break;
            }
        }
    }
    answer.push_str(rest);

    // Return raw text: callers that want trimmed ends (non-streaming) do it
    // themselves.  Streaming must preserve inter-chunk whitespace byte-for-byte.
    (answer, reasoning)
}

/// Processes a streaming chunk: splits `<think>` blocks, holds back partial
/// tags across chunk boundaries, returns the content and reasoning for this
/// chunk.  Shared by QwenHandler and PassthroughHandler (#2562).
///
/// When the accumulated buffer contains an open `<think>` without a matching
/// `</think>`, the entire buffer is held back and the new chunk's text is
/// emitted as reasoning. Once the closing tag arrives the answer text after
/// the block is emitted as content.
///
/// The `state` buffer format: when inside a think block, state holds the
/// entire accumulated text (including `<think>`). The caller must NOT clear
/// state between calls.
pub(super) fn stream_think_blocks(chunk: &str, state: &mut String) -> ProcessedChunk {
    // Whether the block was already open before this chunk: its text has then
    // gone out as deltas, and must not go out again when the block closes.
    let was_mid_reasoning = state.contains(OPEN) && !state.contains(CLOSE);
    state.push_str(chunk);

    let has_open = state.contains(OPEN);
    let has_close = state.contains(CLOSE);

    if has_open && !has_close {
        // Mid-reasoning: the new chunk is reasoning text. Emit it as
        // reasoning so the client sees a live thinking card. Keep the
        // full buffer so the closing tag can be detected later.
        //
        // The chunk itself is the reasoning delta (the caller accumulates).
        // Strip any `<think>` prefix from the chunk in case this is the
        // first chunk that contains the opening tag.
        let delta = chunk.replace(OPEN, "");
        return ProcessedChunk {
            content: String::new(),
            reasoning: delta,
        };
    }

    // The close tag is present or there is no open tag: process normally.
    let (answer, block_reasoning) = split_think_blocks(state);
    let reasoning = if was_mid_reasoning {
        // Only what this chunk adds before the closing tag is new. When the
        // tag itself was split across chunks, this chunk adds nothing.
        match chunk.split_once(CLOSE) {
            Some((before_close, _)) => before_close.to_string(),
            None => String::new(),
        }
    } else {
        block_reasoning
    };
    state.clear();

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
        split_think_blocks(content).0.trim().to_string()
    }

    fn process_content_streaming(&self, chunk: &str, state: &mut String) -> ProcessedChunk {
        stream_think_blocks(chunk, state)
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

    #[test]
    fn streaming_buffers_reasoning_across_many_chunks() {
        // Simulates a mock that streams word by word with <think> in the first chunk
        let mut state = String::new();

        let r1 = stream_think_blocks("<think> ", &mut state);
        assert!(r1.content.is_empty(), "no content while inside think block");
        assert_eq!(r1.reasoning, " ");

        let r2 = stream_think_blocks("Step one. ", &mut state);
        assert!(r2.content.is_empty());
        assert_eq!(r2.reasoning, "Step one. ");

        let r3 = stream_think_blocks("</think>Answer here.", &mut state);
        assert_eq!(r3.content, "Answer here.");
        // The block's text already went out as deltas: closing it adds nothing,
        // so a receiver that joins the deltas gets the reasoning exactly once.
        assert!(r3.reasoning.is_empty(), "the block must not be sent a second time");
        assert_eq!(format!("{}{}{}", r1.reasoning, r2.reasoning, r3.reasoning), " Step one. ");
        assert!(state.is_empty());
    }

    #[test]
    fn a_block_that_arrives_whole_is_sent_once() {
        let mut state = String::new();
        let r = stream_think_blocks("<think>all at once</think>Answer", &mut state);
        assert_eq!(r.reasoning, "all at once");
        assert_eq!(r.content, "Answer");
    }

    #[test]
    fn streaming_unclosed_think_at_end_emits_no_content() {
        let mut state = String::new();
        let r1 = stream_think_blocks("<think>partial", &mut state);
        assert!(r1.content.is_empty());
        assert_eq!(r1.reasoning, "partial");
        assert!(!state.is_empty());
    }

    // --- Whitespace-preservation tests (#2562 space-stripping fix) ---

    #[test]
    fn streaming_chunks_at_spaces_reassemble_exactly() {
        let original = "This is a plain text answer with spaces";
        let chunks = ["This ", "is ", "a ", "plain ", "text ", "answer ", "with ", "spaces"];
        let mut state = String::new();
        let mut assembled = String::new();
        for chunk in &chunks {
            let r = stream_think_blocks(chunk, &mut state);
            assembled.push_str(&r.content);
        }
        assert_eq!(assembled, original);
    }

    #[test]
    fn streaming_chunks_at_newlines_reassemble_exactly() {
        let original = "Line one\nLine two\n\n# Heading\nParagraph";
        let chunks = ["Line one\n", "Line two\n", "\n# Heading\n", "Paragraph"];
        let mut state = String::new();
        let mut assembled = String::new();
        for chunk in &chunks {
            let r = stream_think_blocks(chunk, &mut state);
            assembled.push_str(&r.content);
        }
        assert_eq!(assembled, original);
    }

    #[test]
    fn streaming_space_a_b_space_newline_heading() {
        // Exact chunks from the requirement: " a", "b ", "\n\n# T"
        let mut state = String::new();
        let mut assembled = String::new();
        for chunk in [" a", "b ", "\n\n# T"] {
            let r = stream_think_blocks(chunk, &mut state);
            assembled.push_str(&r.content);
        }
        assert_eq!(assembled, " ab \n\n# T");
    }

    #[test]
    fn streaming_think_block_then_text_starting_with_space() {
        // Think block split across chunks, answer starts with a space.
        let mut state = String::new();
        let r1 = stream_think_blocks("<think>reason", &mut state);
        assert!(r1.content.is_empty());
        let r2 = stream_think_blocks("ing</think> The answer", &mut state);
        assert_eq!(r2.content, " The answer", "leading space must survive");
        assert_eq!(format!("{}{}", r1.reasoning, r2.reasoning), "reasoning");
    }

    #[test]
    fn streaming_no_think_block_is_byte_for_byte() {
        let chunks = ["Hello ", "world! ", "No tags."];
        let mut state = String::new();
        let mut assembled = String::new();
        for chunk in &chunks {
            let r = stream_think_blocks(chunk, &mut state);
            assembled.push_str(&r.content);
        }
        assert_eq!(assembled, "Hello world! No tags.");
    }

    #[test]
    fn newlines_after_close_tag_are_stripped_once() {
        let (answer, _) = split_think_blocks("<think>r</think>\n\nThe answer");
        assert_eq!(answer, "The answer");
    }

    #[test]
    fn space_after_close_tag_is_preserved() {
        let (answer, _) = split_think_blocks("<think>r</think> The answer");
        assert_eq!(answer, " The answer");
    }

    #[test]
    fn non_streaming_process_content_trims_final_answer() {
        let handler = QwenHandler;
        // Non-streaming path may trim the final answer's ends.
        assert_eq!(handler.process_content("<think>r</think>\n\n The answer "), "The answer");
        assert_eq!(handler.process_content("  Plain answer  "), "Plain answer");
    }
}
