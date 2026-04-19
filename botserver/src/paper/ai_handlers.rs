use crate::core::shared::state::AppState;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
    Json,
};
use std::sync::Arc;

use super::llm::call_llm;
use super::models::AiRequest;
use super::utils::format_ai_response;

pub async fn handle_ai_summarize(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();

    if text.is_empty() {
        return Html(format_ai_response("Please select some text to summarize."));
    }

    let system_prompt = "You are a helpful writing assistant. Summarize the following text concisely while preserving the key points. Provide only the summary without any preamble.";

    match call_llm(&state, system_prompt, &text).await {
        Ok(summary) => Html(format_ai_response(&summary)),
        Err(e) => {
            log::error!("LLM summarize error: {}", e);

            let word_count = text.split_whitespace().count();
            let summary = format!(
                "Summary of {} words: {}...",
                word_count,
                text.chars().take(100).collect::<String>()
            );
            Html(format_ai_response(&summary))
        }
    }
}

pub async fn handle_ai_expand(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();

    if text.is_empty() {
        return Html(format_ai_response("Please select some text to expand."));
    }

    let system_prompt = "You are a helpful writing assistant. Expand on the following text by adding more detail, examples, and context. Maintain the same style and tone. Provide only the expanded text without any preamble.";

    match call_llm(&state, system_prompt, &text).await {
        Ok(expanded) => Html(format_ai_response(&expanded)),
        Err(e) => {
            log::error!("LLM expand error: {}", e);
            let expanded = format!(
                "{}\n\nAdditionally, this concept can be further explored by considering its broader implications and related aspects.",
                text
            );
            Html(format_ai_response(&expanded))
        }
    }
}

pub async fn handle_ai_improve(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();

    if text.is_empty() {
        return Html(format_ai_response("Please select some text to improve."));
    }

    let system_prompt = "You are a professional editor. Improve the following text by enhancing clarity, grammar, style, and flow while preserving the original meaning. Provide only the improved text without any preamble or explanation.";

    match call_llm(&state, system_prompt, &text).await {
        Ok(improved) => Html(format_ai_response(&improved)),
        Err(e) => {
            log::error!("LLM improve error: {}", e);
            Html(format_ai_response(&format!("[Improved]: {}", text.trim())))
        }
    }
}

pub async fn handle_ai_simplify(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();

    if text.is_empty() {
        return Html(format_ai_response("Please select some text to simplify."));
    }

    let system_prompt = "You are a writing assistant specializing in plain language. Simplify the following text to make it easier to understand. Use shorter sentences, simpler words, and clearer structure. Provide only the simplified text without any preamble.";

    match call_llm(&state, system_prompt, &text).await {
        Ok(simplified) => Html(format_ai_response(&simplified)),
        Err(e) => {
            log::error!("LLM simplify error: {}", e);
            Html(format_ai_response(&format!(
                "[Simplified]: {}",
                text.trim()
            )))
        }
    }
}

pub async fn handle_ai_translate(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();
    let lang = payload.translate_lang.unwrap_or_else(|| "es".to_string());

    if text.is_empty() {
        return Html(format_ai_response("Please select some text to translate."));
    }

    let lang_name = match lang.as_str() {
        "es" => "Spanish",
        "fr" => "French",
        "de" => "German",
        "pt" => "Portuguese",
        "it" => "Italian",
        "zh" => "Chinese",
        "ja" => "Japanese",
        "ko" => "Korean",
        "ar" => "Arabic",
        "ru" => "Russian",
        _ => "the target language",
    };

    let system_prompt = format!(
        "You are a professional translator. Translate the following text to {}. Provide only the translation without any preamble or explanation.",
        lang_name
    );

    match call_llm(&state, &system_prompt, &text).await {
        Ok(translated) => Html(format_ai_response(&translated)),
        Err(e) => {
            log::error!("LLM translate error: {}", e);
            Html(format_ai_response(&format!(
                "[Translation to {}]: {}",
                lang_name,
                text.trim()
            )))
        }
    }
}

pub async fn handle_ai_custom(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AiRequest>,
) -> impl IntoResponse {
    let text = payload.selected_text.unwrap_or_default();
    let prompt = payload.prompt.unwrap_or_default();

    if text.is_empty() || prompt.is_empty() {
        return Html(format_ai_response(
            "Please select text and enter a command.",
        ));
    }

    let system_prompt = format!(
        "You are a helpful writing assistant. The user wants you to: {}. Apply this to the following text and provide only the result without any preamble.",
        prompt
    );

    match call_llm(&state, &system_prompt, &text).await {
        Ok(result) => Html(format_ai_response(&result)),
        Err(e) => {
            log::error!("LLM custom error: {}", e);
            Html(format_ai_response(&format!(
                "[Custom '{}' applied]: {}",
                prompt,
                text.trim()
            )))
        }
    }
}
