
pub mod deepseek_r3;
pub mod gpt_oss_120b;
pub mod gpt_oss_20b;

pub trait ModelHandler: Send + Sync {
    fn is_analysis_complete(&self, buffer: &str) -> bool;
    fn process_content(&self, content: &str) -> String;
    fn has_analysis_markers(&self, buffer: &str) -> bool;
}
pub fn get_handler(model_path: &str) -> Box<dyn ModelHandler> {
    let path = model_path.to_lowercase();
    if path.contains("deepseek") {
        Box::new(deepseek_r3::DeepseekR3Handler)
    } else if path.contains("120b") {
        Box::new(gpt_oss_120b::GptOss120bHandler::new())
    } else if path.contains("gpt-oss") || path.contains("gpt") {
        Box::new(gpt_oss_20b::GptOss20bHandler)
    } else {
        Box::new(gpt_oss_20b::GptOss20bHandler)
    }
}
