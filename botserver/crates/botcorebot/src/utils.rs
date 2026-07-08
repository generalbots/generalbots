use std::env;

pub fn get_work_path() -> String {
    env::var("WORK_PATH")
        .or_else(|_| env::var("GB_WORK_PATH"))
        .unwrap_or_else(|_| "/opt/gbo/work".to_string())
}

/// Returns the current org ID for bot file path isolation.
/// Always returns Uuid::nil() until real multi-tenant auth is implemented.
pub fn current_org_id() -> uuid::Uuid {
    uuid::Uuid::nil()
}

/// Build a relative bot path with org isolation (.gborg wrapping).
/// Returns: "{org_id}.gborg/{bot_bucket}.gbai/{sub_path}"
pub fn build_bot_path(org_id: impl std::fmt::Display, bot_bucket: &str, sub_path: &str) -> String {
    format!("{org_id}.gborg/{bot_bucket}.gbai/{sub_path}")
}

/// Build an absolute bot path with org isolation.
/// Returns: "{work_root}/{org_id}.gborg/{bot_bucket}.gbai/{sub_path}"
pub fn build_absolute_bot_path(
    work_root: &str,
    org_id: impl std::fmt::Display,
    bot_bucket: &str,
    sub_path: &str,
) -> String {
    format!("{work_root}/{org_id}.gborg/{bot_bucket}.gbai/{sub_path}")
}

/// Get work path with org isolation suffix.
/// Returns: "{work_path}/{org_id}.gborg/"
pub fn get_org_work_path(org_id: impl std::fmt::Display) -> String {
    format!("{}/{org_id}.gborg/", get_work_path())
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
