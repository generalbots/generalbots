//! Shared plain chat-completion helper for the Vibe AI-OS modules
//! (#1171 planner, #1172 agent API, #1173 mixture-of-agents, #1175
//! browser memory, #1185 proactivity). Performs non-streaming completions
//! against any OpenAI-compatible endpoint so every module speaks the same
//! LLM dialect instead of re-implementing HTTP calls.

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct LlmSettings {
    pub url: String,
    pub model: String,
    pub key: String,
}

/// Resolve LLM settings: explicit request overrides first, then the
/// environment (`LLM_URL` / `LLM_MODEL` / `LLM_KEY`), then the canonical
/// Vault secret `secret/gbo/llm` (`url` / `model` / `openai_key`), matching
/// the fallback chain used by the rest of the platform (config.rs
/// `central_llm_value`). Production runs botserver without LLM env vars;
/// the scaffold would otherwise silently degrade to the built-in template.
pub fn resolve_llm(
    override_url: Option<&str>,
    override_model: Option<&str>,
    override_key: Option<&str>,
) -> LlmSettings {
    let mut url = override_url
        .map(str::to_string)
        .or_else(|| std::env::var("LLM_URL").ok())
        .unwrap_or_default();
    let mut model = override_model
        .map(str::to_string)
        .or_else(|| std::env::var("LLM_MODEL").ok())
        .unwrap_or_else(|| "gpt-4o-mini".to_string());
    let mut key = override_key
        .map(str::to_string)
        .or_else(|| std::env::var("LLM_KEY").ok())
        .unwrap_or_default();

    if url.is_empty() || key.is_empty() {
        if let Some(secrets) = read_vault_llm() {
            if url.is_empty() {
                if let Some(v) = secrets.get("url") {
                    url = v.clone();
                }
            }
            if key.is_empty() {
                if let Some(v) = secrets.get("openai_key") {
                    key = v.clone();
                }
            }
            if model == "gpt-4o-mini" {
                if let Some(v) = secrets.get("model") {
                    model = v.clone();
                }
            }
        }
    }
    LlmSettings { url, model, key }
}

/// Reads `secret/gbo/llm` (url/model/openai_key) synchronously, or `None`
/// when Vault is unavailable. Mirrors the spawn-bridge pattern used by the
/// other synchronous Vault readers (botcoresecrets tenant.rs).
fn read_vault_llm() -> Option<std::collections::HashMap<String, String>> {
    let sm = botcoresecrets::SecretsManager::get_clone().ok()?;
    if !sm.is_enabled() {
        return None;
    }
    let self_owned = sm.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build();
        let result = if let Ok(rt) = rt {
            rt.block_on(async move { self_owned.get_secret(botcoresecrets::SecretPaths::LLM).await.ok() })
        } else {
            None
        };
        let _ = tx.send(result);
    });
    rx.recv_timeout(std::time::Duration::from_secs(5)).ok().flatten()
}

/// Runs a single non-streaming chat completion and returns the assistant
/// text. Errors are strings so callers can surface them in API responses
/// without panicking (security directive: no unwrap in production paths).
pub async fn chat_completion(
    settings: &LlmSettings,
    system: &str,
    prompt: &str,
) -> Result<String, String> {
    if settings.url.is_empty() {
        return Err("no LLM URL configured (set LLM_URL or pass llm_url)".to_string());
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .user_agent("VibeAgent/1.0")
        .build()
        .map_err(|e| format!("http client: {e}"))?;
    let body = serde_json::json!({
        "model": settings.model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.3,
        // Reasoning models (e.g. nvidia/nemotron-3-*) spend tokens on an
        // internal `reasoning_content` field BEFORE producing `content`; a
        // small budget leaves content empty/truncated and the reasoning pass
        // alone can exceed any sane scaffold timeout. Structured extraction
        // does not need the reasoning pass — disable it (no-op for models
        // that do not know the flag).
        "max_tokens": 16384,
        "chat_template_kwargs": { "thinking": false },
    });
    let resp = client
        .post(&settings.url)
        .header("Authorization", format!("Bearer {}", settings.key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("LLM HTTP {status}: {text}"));
    }
    let json: Value = resp.json().await.map_err(|e| format!("response json: {e}"))?;
    let message = &json["choices"][0]["message"];
    let mut content = message["content"]
        .as_str()
        .unwrap_or_default()
        .trim()
        .to_string();
    if content.is_empty() {
        // Reasoning models can return the whole answer inside
        // `reasoning_content` (with the final result at the end); fall back
        // to it so extract_json can still find the payload.
        if let Some(reasoning) = message["reasoning_content"].as_str() {
            content = reasoning.trim().to_string();
        }
    }
    if content.is_empty() {
        return Err("LLM returned empty content".to_string());
    }
    Ok(content)
}

/// Strips markdown fences and extracts the first `{...}` or `[...]` block
/// from an LLM answer so `serde_json` can parse it even when the model
/// wraps JSON in prose.
pub fn extract_json(raw: &str) -> String {
    let trimmed = raw.trim();
    for open in ['{', '['] {
        if let Some(start) = trimmed.find(open) {
            let rest = &trimmed[start..];
            if let Some(end) = rest.rfind(if open == '{' { '}' } else { ']' }) {
                let candidate = rest[..=end].to_string();
                if serde_json::from_str::<Value>(&candidate).is_ok() {
                    return candidate;
                }
            }
        }
    }
    trimmed.to_string()
}
