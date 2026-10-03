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

#[derive(Debug)]
pub struct PassthroughHandler;

impl ModelHandler for PassthroughHandler {
    fn is_analysis_complete(&self, _buffer: &str) -> bool {
        true
    }

    fn process_content(&self, content: &str) -> String {
        content.to_string()
    }

    fn has_analysis_markers(&self, _buffer: &str) -> bool {
        false
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
    fn unknown_models_still_pass_through() {
        let handler = get_handler("some-unreleased-model");
        assert_eq!(handler.process_content("<think>x</think>y"), "<think>x</think>y");
    }
}
