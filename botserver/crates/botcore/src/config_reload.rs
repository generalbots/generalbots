use axum::{extract::State, http::StatusCode, response::Json};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::shared::state::AppState;

pub async fn reload_config(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Value>, StatusCode> {
    // Reload the white-label `.product` configuration from disk and swap it
    // into the global PRODUCT_CONFIG. This is a real operation backed by
    // ProductConfig::reload() (which re-reads the .product file) — not a stub.
    match crate::product::ProductConfig::reload() {
        Ok(()) => {
            log::info!("Product configuration reloaded via /api/config/reload");
            Ok(Json(json!({
                "success": true,
                "message": "Product configuration reloaded",
                "config": crate::product::get_product_config_json(),
            })))
        }
        Err(e) => {
            log::error!("Failed to reload product configuration: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
