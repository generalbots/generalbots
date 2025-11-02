use super::ModelHandler;

pub struct GptOss120bHandler {
    model_name: String,
}

impl GptOss120bHandler {
    pub fn new(model_name: &str) -> Self {
        Self {
            model_name: model_name.to_string(),
        }
    }
}

impl ModelHandler for GptOss120bHandler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        // GPT-120B uses explicit end marker
        buffer.contains("**end**")
    }

    fn process_content(&self, content: &str) -> String {
        // Remove both start and end markers from final output
        content.replace("**start**", "")
              .replace("**end**", "")
    }

    fn has_analysis_markers(&self, buffer: &str) -> bool {
        // GPT-120B uses explicit start marker
        buffer.contains("**start**")
    }
}
