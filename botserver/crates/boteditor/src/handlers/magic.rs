use super::*;

pub async fn handle_magic(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MagicRequest>,
) -> Result<Json<MagicResponse>, (StatusCode, String)> {
    let code = payload.code.unwrap_or_else(String::new);
    if code.trim().is_empty() {
        return Ok(Json(MagicResponse {
            improved_code: None,
            explanation: None,
            suggestions: Some(vec![MagicSuggestion {
                title: "Empty file".to_string(),
                description: "Write some code to get AI-powered suggestions.".to_string(),
            }]),
        }));
    }

    if let Some(llm) = state.llm_provider.clone() {
        let prompt = format!(
            "You are a senior code reviewer. Analyze the following code and return JSON with two keys: \
             'improved_code' (the code with improvements applied, or null if already good) and \
             'explanation' (a short explanation of the changes). Only respond with valid JSON.\n\n\
             ```\n{}\n```",
            code
        );
        match llm.generate_simple(&prompt).await {
            Ok(answer) => {
                let cleaned = answer
                    .trim_start_matches("```json")
                    .trim_start_matches("```")
                    .trim_end_matches("```")
                    .trim();
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(cleaned) {
                    let improved = value
                        .get("improved_code")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let explanation = value
                        .get("explanation")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    return Ok(Json(MagicResponse {
                        improved_code: improved,
                        explanation,
                        suggestions: None,
                    }));
                }
                Ok(Json(MagicResponse {
                    improved_code: None,
                    explanation: Some(answer),
                    suggestions: None,
                }))
            }
            Err(e) => {
                log::error!("Editor magic LLM error: {e}");
                Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    "AI service unavailable".to_string(),
                ))
            }
        }
    } else {
        Ok(Json(MagicResponse {
            improved_code: None,
            explanation: None,
            suggestions: Some(vec![MagicSuggestion {
                title: "No AI provider configured".to_string(),
                description: "Configure an LLM provider to get code improvement suggestions."
                    .to_string(),
            }]),
        }))
    }
}
