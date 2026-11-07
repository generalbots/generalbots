use axum::Json;

pub async fn get_product_config() -> Json<serde_json::Value> {
    // Optional routes that the suite probes at boot are advertised here so a
    // build without them can skip the request instead of logging a 404 every
    // load (#1693 follow-up). The manifest is always returned.
    let mut value = crate::core::product::get_product_config_json();
    if let Some(object) = value.as_object_mut() {
        // #1793 — BotModels has no Windows/aarch64 pythonbox. Report the real
        // media capability (and the reason) so the suite can hide or explain
        // vision/STT/TTS instead of failing when the optional service is not
        // installed.
        object.insert(
            "media_capabilities".to_string(),
            crate::main_module::media_capabilities::media_capabilities_json(
                std::env::consts::OS,
                std::env::consts::ARCH,
            ),
        );
        // Advertise optional capabilities the suite probes at boot. `agent-vm`
        // gates /api/agent/drive/*, which the chat attachment actions call;
        // without it the client can hide Save/Open instead of eating a 404.
        #[cfg(any(feature = "consent", feature = "vibe", feature = "agent-vm"))]
        {
            let entry = object
                .entry("compiled_features")
                .or_insert_with(|| serde_json::json!([]));
            if let Some(features) = entry.as_array_mut() {
                #[cfg(feature = "consent")]
                features.push(serde_json::json!("consent"));
                #[cfg(feature = "vibe")]
                features.push(serde_json::json!("vibe"));
                #[cfg(feature = "agent-vm")]
                features.push(serde_json::json!("agent-vm"));
            }
        }
        // #1747 — tell the desktop/wizard why terminal, browser and VM actions
        // cannot run (missing Incus/WSL wrapper). `null` means the backend is
        // available; the reason is shown instead of a generic VM failure.
        #[cfg(feature = "agent-vm")]
        {
            object.insert(
                "agent_backend_unavailable".to_string(),
                serde_json::json!(botagent::agent_backend_unavailable()),
            );
        }
    }
    Json(value)
}

pub async fn get_workspace_manifest() -> Json<serde_json::Value> {
    Json(crate::core::product::get_workspace_manifest())
}
