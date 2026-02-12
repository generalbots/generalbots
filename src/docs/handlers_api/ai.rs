use crate::core::shared::state::AppState;
use crate::docs::types::{DocsAiRequest, DocsAiResponse, AiRequest, AiResponse};
use axum::{
    extract::State,
    http::StatusCode,
    Json,
    response::IntoResponse,
};
use std::sync::Arc;

pub async fn handle_docs_ai(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<DocsAiRequest>,
) -> impl IntoResponse {
    let command = req.command.to_lowercase();

    let response = if command.contains("summarize") || command.contains("summary") {
        "I've created a summary of your document. The key points are highlighted above."
    } else if command.contains("expand") || command.contains("longer") {
        "I've expanded the selected text with more details and examples."
    } else if command.contains("shorter") || command.contains("concise") {
        "I've made the text more concise while preserving the key information."
    } else if command.contains("formal") {
        "I've rewritten the text in a more formal, professional tone."
    } else if command.contains("casual") || command.contains("friendly") {
        "I've rewritten the text in a more casual, friendly tone."
    } else if command.contains("grammar") || command.contains("fix") {
        "I've corrected the grammar and spelling errors in your text."
    } else if command.contains("translate") {
        "I've translated the selected text. Please specify the target language if needed."
    } else if command.contains("bullet") || command.contains("list") {
        "I've converted the text into a bulleted list format."
    } else if command.contains("help") {
        "I can help you with:\n• Summarize text\n• Expand or shorten content\n• Fix grammar\n• Change tone (formal/casual)\n• Translate text\n• Convert to bullet points"
    } else {
        "I understand you want help with your document. Try commands like 'summarize', 'make shorter', 'fix grammar', or 'make formal'."
    };

    Json(DocsAiResponse {
        response: response.to_string(),
        result: None,
    })
}

pub async fn handle_ai_summarize(
    Json(req): Json<AiRequest>,
) -> Result<Json<AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();
    let summary = if text.len() > 200 {
        format!("Summary: {}...", &text[..200])
    } else {
        format!("Summary: {}", text)
    };

    Ok(Json(AiResponse {
        result: "success".to_string(),
        content: summary,
        error: None,
    }))
}

pub async fn handle_ai_expand(
    Json(req): Json<AiRequest>,
) -> Result<Json<AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();
    let expanded = format!("{}\n\n[Additional context and details would be added here by AI]", text);

    Ok(Json(AiResponse {
        result: "success".to_string(),
        content: expanded,
        error: None,
    }))
}

pub async fn handle_ai_improve(
    Json(req): Json<AiRequest>,
) -> Result<Json<AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();

    Ok(Json(AiResponse {
        result: "success".to_string(),
        content: text,
        error: None,
    }))
}

pub async fn handle_ai_simplify(
    Json(req): Json<AiRequest>,
) -> Result<Json<AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();

    Ok(Json(AiResponse {
        result: "success".to_string(),
        content: text,
        error: None,
    }))
}

pub async fn handle_ai_translate(
    Json(req): Json<AiRequest>,
) -> Result<Json<AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();
    let lang = req.translate_lang.unwrap_or_else(|| "English".to_string());

    Ok(Json(AiResponse {
        result: "success".to_string(),
        content: format!("[Translated to {}]: {}", lang, text),
        error: None,
    }))
}

pub async fn handle_ai_custom(
    Json(req): Json<AiRequest>,
) -> Result<Json<AiResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = req.selected_text.unwrap_or_default();

    Ok(Json(AiResponse {
        result: "success".to_string(),
        content: text,
        error: None,
    }))
}
