use super::ModelHandler;
use regex::Regex;
use std::sync::LazyLock;

static THINK_REGEX: LazyLock<Result<Regex, regex::Error>> =
    LazyLock::new(|| Regex::new(r"(?s)（分析）.*?（/分析）"));
static THINK_REGEX2: LazyLock<Result<Regex, regex::Error>> =
    LazyLock::new(|| Regex::new(r"(?s)<think>.*?</think>"));
static THINK_REGEX3: LazyLock<Result<Regex, regex::Error>> =
    LazyLock::new(|| Regex::new(r"(?s)【分析】.*?【/分析】"));

pub fn strip_think_tags(content: &str) -> String {
    let mut result = content.to_string();
    if let Ok(re) = &*THINK_REGEX {
        result = re.replace_all(&result, "").to_string();
    }
    if let Ok(re) = &*THINK_REGEX2 {
        result = re.replace_all(&result, "").to_string();
    }
    if let Ok(re) = &*THINK_REGEX3 {
        result = re.replace_all(&result, "").to_string();
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
