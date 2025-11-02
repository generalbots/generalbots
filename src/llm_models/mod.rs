//! Module for handling model-specific behavior and token processing

pub mod gpt_oss_20b;
pub mod deepseek_r3;
pub mod gpt_oss_120b;


/// Trait for model-specific token processing
pub trait ModelHandler: Send + Sync {
    /// Check if the analysis buffer indicates completion
    fn is_analysis_complete(&self, buffer: &str) -> bool;
    
    /// Process the content, removing any model-specific tokens
    fn process_content(&self, content: &str) -> String;

    /// Check if the buffer contains analysis start markers
    fn has_analysis_markers(&self, buffer: &str) -> bool;
}

/// Get the appropriate handler based on model path from bot configuration
pub fn get_handler(model_path: &str) -> Box<dyn ModelHandler> {
    let path = model_path.to_lowercase();
    
    if path.contains("deepseek") {
        Box::new(deepseek_r3::DeepseekR3Handler)
    } else if path.contains("120b") {
        Box::new(gpt_oss_120b::GptOss120bHandler::new("default"))
    } else if path.contains("gpt-oss") || path.contains("gpt") {
        Box::new(gpt_oss_20b::GptOss20bHandler)
    } else {
        Box::new(gpt_oss_20b::GptOss20bHandler)
    }
}
