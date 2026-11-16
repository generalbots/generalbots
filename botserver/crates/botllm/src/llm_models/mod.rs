pub mod deepseek_v4;
pub mod gpt_oss_120b;
pub mod gpt_oss_20b;
pub mod minimax;
pub mod qwen;
pub mod stepfun;

#[derive(Debug, Clone, Default)]
pub struct ProcessedChunk {
    pub content: String,
    pub reasoning: String,
}

pub trait ModelHandler: Send + Sync {
    fn is_analysis_complete(&self, buffer: &str) -> bool;
    fn process_content(&self, content: &str) -> String;
    fn process_content_streaming(&self, content: &str, _state_buffer: &mut String) -> ProcessedChunk {
        let result = self.process_content(content);
        ProcessedChunk { content: result, reasoning: String::new() }
    }
    fn has_analysis_markers(&self, buffer: &str) -> bool;
    fn skip_reasoning_content(&self) -> bool {
        false
    }
}

/// Default handler that also extracts `<think>…</think>` blocks, so local
/// models whose name does not match any known family still get reasoning
/// separated from the answer (#2562).
#[derive(Debug)]
pub struct PassthroughHandler;

impl ModelHandler for PassthroughHandler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        // The passthrough handler always considers the analysis "complete"
        // once a closing tag appears, or immediately when there is no tag.
        !buffer.contains("<think>") || buffer.contains("</think>")
    }

    fn process_content(&self, content: &str) -> String {
        qwen::split_think_blocks(content).0.trim().to_string()
    }

    fn process_content_streaming(&self, chunk: &str, state: &mut String) -> ProcessedChunk {
        qwen::stream_think_blocks(chunk, state)
    }

    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains("<think>")
    }

    fn skip_reasoning_content(&self) -> bool {
        true
    }
}

pub fn get_handler(model_path: &str) -> Box<dyn ModelHandler> {
    let path = model_path.to_lowercase();
    if path.contains("deepseek") {
        Box::new(deepseek_v4::DeepseekV4Handler)
    } else if path.contains("qwen") {
        // Qwen is the most common open-weight model across the
        // OpenAI-compatible tier (#1467); it emits `<think>` blocks, which the
        // DeepSeek handler's `" thinking"` framing does not recognise.
        Box::new(qwen::QwenHandler)
    } else if path.contains("120b") {
        Box::new(gpt_oss_120b::GptOss120bHandler::new())
    } else if path.contains("20b") {
        Box::new(gpt_oss_20b::GptOss20bHandler)
    } else if path.contains("minimax") || path.contains("minimax-m") || path.contains("kimi") {
        Box::new(minimax::MinimaxHandler::new())
    } else if path.contains("stepfun") || path.contains("step-") {
        Box::new(stepfun::StepfunHandler::new())
    } else {
        Box::new(PassthroughHandler)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `<think>` block must be gone from `process_content`; the DeepSeek
    /// handler would leave it in place, so this also proves the two families
    /// are not routed to the same handler.
    fn thinking_is_stripped(model: &str) -> bool {
        let handler = get_handler(model);
        handler.process_content("<think>hidden</think>visible") == "visible"
    }

    #[test]
    fn qwen_is_routed_to_its_own_handler() {
        assert!(thinking_is_stripped("qwen3-235b-a22b"));
        assert!(thinking_is_stripped("Qwen/Qwen3-32B"));
    }

    #[test]
    fn the_deepseek_handler_is_not_used_for_qwen() {
        let handler = get_handler("qwen3-235b-a22b");
        let processed = handler.process_content("<think>hidden</think>visible");
        assert_eq!(processed, "visible");
        assert!(handler.has_analysis_markers("<think>open"));
    }

    #[test]
    fn unknown_models_extract_think_blocks() {
        let handler = get_handler("some-unreleased-model");
        // #2562 — the passthrough handler now strips `<think>` blocks so local
        // models that emit inline reasoning get thinking cards.
        assert_eq!(handler.process_content("<think>x</think>y"), "y");
    }

    #[test]
    fn passthrough_streaming_handles_split_tags() {
        let handler = get_handler("some-unreleased-model");
        let mut state = String::new();
        let first = handler.process_content_streaming("Hello<thin", &mut state);
        assert_eq!(first.content, "Hello");
        let second = handler.process_content_streaming("k>reason</think>Done", &mut state);
        assert_eq!(second.content, "Done");
        assert!(second.reasoning.contains("reason"));
    }

    #[test]
    fn passthrough_streaming_buffers_full_think_block() {
        let handler = get_handler("some-unreleased-model");
        let mut state = String::new();
        let r1 = handler.process_content_streaming("<think>Step 1. ", &mut state);
        assert!(r1.content.is_empty());
        assert_eq!(r1.reasoning, "Step 1. ");
        let r2 = handler.process_content_streaming("Step 2. ", &mut state);
        assert!(r2.content.is_empty());
        assert_eq!(r2.reasoning, "Step 2. ");
        let r3 = handler.process_content_streaming("</think>Final.", &mut state);
        assert_eq!(r3.content, "Final.");
    }
}
