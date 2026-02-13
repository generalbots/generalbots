use super::admin_types::*;
use crate::core::shared::state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use log::info;
use std::sync::Arc;

/// Get current configuration
pub async fn get_config(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Return default empty config for now
    let configs = vec![
        ConfigItem {
            key: "maintenance_mode".to_string(),
            value: "false".to_string(),
            description: "Enable/disable maintenance mode".to_string(),
        },
        ConfigItem {
            key: "max_users".to_string(),
            value: "1000".to_string(),
            description: "Maximum number of users allowed".to_string(),
        },
    ];

    (StatusCode::OK, Json(ConfigResponse { configs })).into_response()
}

/// Update configuration
pub async fn update_config(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<UpdateConfigRequest>,
) -> impl IntoResponse {
    info!("Updating config: {} = {}", request.key, request.value);

    // For now, just return success
    // In production, this would update the database
    (StatusCode::OK, Json(serde_json::json!({"success": true}))).into_response()
}
