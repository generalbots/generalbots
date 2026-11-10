use super::*;

impl DlpMatch {
    pub fn new(
        data_type: SensitiveDataType,
        matched_text: &str,
        start: usize,
        end: usize,
    ) -> Self {
        Self {
            data_type,
            matched_text: matched_text.to_string(),
            start_position: start,
            end_position: end,
            confidence: 1.0,
            context: None,
        }
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }

    pub fn masked_value(&self) -> String {
        // #2066 — the value comes from a regex match over UTF-8 text (custom
        // patterns can match non-ASCII), so slicing by byte offset panicked
        // inside a multi-byte character. Mask by characters instead.
        let chars: Vec<char> = self.matched_text.chars().collect();
        let len = chars.len();
        if len <= 4 {
            "*".repeat(len)
        } else {
            format!(
                "{}{}{}{}",
                chars[0],
                chars[1],
                "*".repeat(len - 4),
                chars[len - 2..].iter().collect::<String>()
            )
        }
    }
}
