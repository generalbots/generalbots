use super::ModelHandler;

/// Handler for Stepfun models.
/// Stepfun uses reasoning/content separation - reasoning is skipped at the stream level.
/// Content field contains the actual response (usually HTML).
#[derive(Debug)]
pub struct StepfunHandler;

impl Default for StepfunHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl StepfunHandler {
    pub fn new() -> Self {
        Self
    }
}

impl ModelHandler for StepfunHandler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        true
    }

    fn process_content(&self, content: &str) -> String {
        content.to_string()
    }

    fn process_content_streaming(&self, content: &str, _state: &mut String) -> String {
        content.to_string()
    }

    fn has_analysis_markers(&self, _buffer: &str) -> bool {
        false
    }
}
