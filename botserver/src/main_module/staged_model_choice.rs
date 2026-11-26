//! Applies a model choice staged by the first-run wizard.
//!
//! The wizard writes `conf/system/model-choice.json` before Vault credentials
//! exist. The boot sequence calls [`apply`] here, before the LLM provider is
//! constructed, so the first chat message already uses the chosen model.

/// Copies `conf/system/model-choice.json` into the Vault LLM secret and removes
/// it. No-op when the file is absent or Vault is unavailable.
pub async fn apply() {
    let stack = botcore::shared::utils::get_stack_path();
    let path = std::path::PathBuf::from(&stack).join("conf/system/model-choice.json");
    let raw = match std::fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(_) => return,
    };
    let parsed: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(e) => {
            log::warn!("staged model choice is not valid JSON: {e}");
            return;
        }
    };
    let url = parsed
        .get("url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let model = parsed
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    if url.is_empty() || model.is_empty() {
        if let Err(e) = std::fs::remove_file(&path) {
            log::debug!("stale wizard model choice removal failed: {e}");
        }
        return;
    }
    let explicit_provider = parsed
        .get("provider")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let provider = if !explicit_provider.is_empty() {
        explicit_provider.to_string()
    } else if url.contains("api.kilo.ai") {
        match model {
            "dots-studio/dots-3-note-preview:free" => "kilo-dots".to_string(),
            "openrouter/free" => "kilo-openrouter".to_string(),
            "nvidia/nemotron-3-ultra-550b-a55b:free" => "kilo-nemotron-ultra".to_string(),
            "nvidia/nemotron-3.5-super-vl:free" => "kilo-nemotron-super".to_string(),
            "nvidia/nemotron-3.5-lightning:free" => "kilo-nemotron-lightning".to_string(),
            "nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free" => "kilo-nano".to_string(),
            "cohere/north-mini-code:free" => "kilo-cohere".to_string(),
            "recursive/flint-v1.0:free" => "kilo-flint".to_string(),
            "xingchen-agi/xing4.0-29b-a4b:free" => "kilo-xing".to_string(),
            _ => "kilo-dots".to_string(),
        }
    } else {
        String::new()
    };

    let manager = match botcoresecrets::SecretsManager::from_env() {
        Ok(manager) => manager,
        Err(e) => {
            log::warn!("could not initialise secrets to apply the model choice: {e}");
            return;
        }
    };
    if !manager.is_enabled() {
        log::warn!("Vault is not enabled yet; the staged model choice stays on disk");
        return;
    }
    let mut data = std::collections::HashMap::new();
    data.insert("url".to_string(), url.to_string());
    data.insert("model".to_string(), model.to_string());
    if !provider.is_empty() {
        data.insert("provider".to_string(), provider.clone());
    }
    if let Some(key) = parsed.get("api_key").and_then(|v| v.as_str()) {
        if !key.trim().is_empty() {
            data.insert("openai_key".to_string(), key.to_string());
        }
    }
    match manager.put_secret("gbo/llm", data).await {
        Ok(()) => {
            log::info!("Applied the wizard model choice: {model} at {url}");
            if let Err(e) = std::fs::remove_file(&path) {
                log::debug!("applied wizard model choice removal failed: {e}");
            }
        }
        Err(e) => log::warn!("could not apply the model choice: {e}"),
    }
}
