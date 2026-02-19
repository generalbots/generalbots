// Simple config reload endpoint
use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::core::shared::state::AppState;
use crate::core::config::ConfigManager;

pub async fn reload_config(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, StatusCode> {
    let config_manager = ConfigManager::new(state.conn.clone());
    
    // Get default bot
    let conn_arc = state.conn.clone();
    let (default_bot_id, _) = tokio::task::spawn_blocking(move || -> Result<(uuid::Uuid, String), String> {
        let mut conn = conn_arc
            .get()
            .map_err(|e| format!("failed to get db connection: {e}"))?;
        Ok(crate::core::bot::get_default_bot(&mut conn))
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get LLM config
    let llm_url = config_manager
        .get_config(&default_bot_id, "llm-url", Some("http://localhost:8081"))
        .unwrap_or_else(|_| "http://localhost:8081".to_string());
    
    let llm_model = config_manager
        .get_config(&default_bot_id, "llm-model", Some("local"))
        .unwrap_or_else(|_| "local".to_string());

    let llm_endpoint_path = config_manager
        .get_config(&default_bot_id, "llm-endpoint-path", Some("/v1/chat/completions"))
        .unwrap_or_else(|_| "/v1/chat/completions".to_string());

    // Update LLM provider
    if let Some(dynamic_llm) = &state.dynamic_llm_provider {
        dynamic_llm
            .update_from_config(&llm_url, Some(llm_model.clone()), Some(llm_endpoint_path.clone()))
            .await;
        
        Ok(Json(json!({
            "status": "success",
            "message": "LLM configuration reloaded",
            "config": {
                "llm_url": llm_url,
                "llm_model": llm_model,
                "llm_endpoint_path": llm_endpoint_path
            }
        })))
    } else {
        Ok(Json(json!({
            "status": "error",
            "message": "Dynamic LLM provider not available"
        })))
    }
}
