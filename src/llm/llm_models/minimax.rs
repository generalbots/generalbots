use super::ModelHandler;

pub fn strip_think_tags(content: &str) -> String {
    let mut result = content.to_string();

    // Chinese: （分析）...（/分析）
    while let Some(start_idx) = result.find("（分析）") {
        if let Some(end_idx) = result[start_idx..].find("（/分析）") {
            result = format!(
                "{}{}",
                &result[..start_idx],
                &result[start_idx + end_idx + 4..]
            );
        } else {
            break;
        }
    }

    // English: <think>...</think>
    while let Some(start_idx) = result.find("<think>") {
        if let Some(end_idx) = result[start_idx..].find("</think>") {
            result = format!(
                "{}{}",
                &result[..start_idx],
                &result[start_idx + end_idx + 8..]
            );
        } else {
            break;
        }
    }

    // Chinese alternative: 【分析】...【/分析】
    while let Some(start_idx) = result.find("【分析】") {
        if let Some(end_idx) = result[start_idx..].find("【/分析】") {
            result = format!(
                "{}{}",
                &result[..start_idx],
                &result[start_idx + end_idx + 5..]
            );
        } else {
            break;
        }
    }

    result
}

#[derive(Debug)]
pub struct MinimaxHandler;

impl MinimaxHandler {
    pub fn new() -> Self {
        Self
    }
}

impl ModelHandler for MinimaxHandler {
    fn is_analysis_complete(&self, buffer: &str) -> bool {
        buffer.contains("（/分析）") || buffer.contains("</think>") || buffer.contains("【/分析】")
    }

    fn process_content(&self, content: &str) -> String {
        strip_think_tags(content)
    }

    fn has_analysis_markers(&self, buffer: &str) -> bool {
        buffer.contains("（分析）") || buffer.contains("<think>") || buffer.contains("【分析】")
    }
}
