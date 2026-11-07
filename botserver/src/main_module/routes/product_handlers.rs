use axum::Json;

pub async fn get_product_config() -> Json<serde_json::Value> {
    // Optional routes that the suite probes at boot are advertised here so a
    // build without them can skip the request instead of logging a 404 every
    // load (#1693 follow-up). The manifest is always returned.
    let mut value = crate::core::product::get_product_config_json();
    if let Some(object) = value.as_object_mut() {
        // #1793 — BotModels publishes Linux x86_64 and Windows x86_64 boxes;
        // arm64 has none. Report the real media capability (and the reason) so
        // the suite can hide or explain vision/STT/TTS instead of failing when
        // the optional service is not installed.
        object.insert(
            "media_capabilities".to_string(),
            crate::main_module::media_capabilities::media_capabilities_json(
                std::env::consts::OS,
                std::env::consts::ARCH,
            ),
        );
        // #1841 — the Terminal app needs both the PTY routes and the agent VM
        // that hosts the user's workspace. Advertise the real capability so
        // the launcher can hide the app instead of offering a shell that can
        // only answer "not_found".
        object.insert(
            "terminal_available".to_string(),
            serde_json::json!(cfg!(feature = "terminal") && cfg!(feature = "agent-vm")),
        );
        // `compiled_features` is built in botcore::features (#1811), including
        // consent/vibe/agent-vm. Do not append here: a second copy made the
        // manifest disagree with the Cargo feature set.
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
