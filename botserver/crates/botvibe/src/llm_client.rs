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
/// environment (`LLM_URL` / `LLM_MODEL` / `LLM_KEY`), matching the
/// fallback chain used by the rest of the Vibe agent loop (#795).
pub fn resolve_llm(
    override_url: Option<&str>,
    override_model: Option<&str>,
    override_key: Option<&str>,
) -> LlmSettings {
    let url = override_url
        .map(str::to_string)
        .or_else(|| std::env::var("LLM_URL").ok())
        .unwrap_or_default();
    let model = override_model
        .map(str::to_string)
        .or_else(|| std::env::var("LLM_MODEL").ok())
        .unwrap_or_else(|| "gpt-4o-mini".to_string());
    let key = override_key
        .map(str::to_string)
        .or_else(|| std::env::var("LLM_KEY").ok())
        .unwrap_or_default();
    LlmSettings { url, model, key }
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
        "max_tokens": 4096,
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
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or_default()
        .trim()
        .to_string();
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
