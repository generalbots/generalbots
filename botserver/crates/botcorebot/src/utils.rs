use std::env;

pub fn get_work_path() -> String {
    env::var("WORK_PATH")
        .or_else(|_| env::var("GB_WORK_PATH"))
        .unwrap_or_else(|_| "/opt/gbo/work".to_string())
}

pub fn get_stack_path() -> String {
    env::var("STACK_PATH")
        .or_else(|_| env::var("GB_STACK_PATH"))
        .unwrap_or_else(|_| "/opt/gbo".to_string())
}

pub fn estimate_token_count(text: &str) -> usize {
    text.len() / 4
}

pub fn truncate_text_for_model(text: &str, _model: &str, max_tokens: usize) -> String {
    let char_limit = max_tokens * 4;
    if text.len() <= char_limit {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(char_limit).collect();
        format!("{}...[truncated]", truncated)
    }
}

pub fn sanitize_utf16_surrogates(text: &str) -> String {
    text.chars()
        .filter(|c| {
            let cp = *c as u32;
            !(0xD800..=0xDBFF).contains(&cp) && !(0xDC00..=0xDFFF).contains(&cp)
        })
        .collect()
}
