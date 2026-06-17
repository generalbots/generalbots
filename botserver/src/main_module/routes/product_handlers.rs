use axum::Json;

pub async fn get_product_config() -> Json<serde_json::Value> {
    Json(crate::core::product::get_product_config_json())
}

pub async fn get_workspace_manifest() -> Json<serde_json::Value> {
    Json(crate::core::product::get_workspace_manifest())
}
