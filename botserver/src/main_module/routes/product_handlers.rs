use axum::Json;

pub async fn get_product_config() -> Json<serde_json::Value> {
    // Optional routes that the suite probes at boot are advertised here so a
    // build without them can skip the request instead of logging a 404 every
    // load (#1693 follow-up). When neither feature is compiled the manifest is
    // returned untouched.
    #[cfg(any(feature = "consent", feature = "vibe"))]
    let value = {
        let mut value = crate::core::product::get_product_config_json();
        if let Some(object) = value.as_object_mut() {
            let entry = object
                .entry("compiled_features")
                .or_insert_with(|| serde_json::json!([]));
            if let Some(features) = entry.as_array_mut() {
                #[cfg(feature = "consent")]
                features.push(serde_json::json!("consent"));
                #[cfg(feature = "vibe")]
                features.push(serde_json::json!("vibe"));
            }
        }
        value
    };
    #[cfg(not(any(feature = "consent", feature = "vibe")))]
    let value = crate::core::product::get_product_config_json();
    Json(value)
}

pub async fn get_workspace_manifest() -> Json<serde_json::Value> {
    Json(crate::core::product::get_workspace_manifest())
}
