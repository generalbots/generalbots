//! Engine-type classification for usage grouping.
//!
//! Each assistant row in `message_history` carries an `llm_engine` and
//! `llm_provider` that say how the turn was served.  New rows are tagged at
//! write time; old rows (NULL engine) are classified by a model-name fallback
//! when reading the usage summary.

use crate::model_config::{self, ModelMode};

/// Engine type constants used in `message_history.llm_engine`.
pub const ENGINE_SUBSCRIPTION: &str = "subscription";
pub const ENGINE_API_KEY: &str = "api_key";
pub const ENGINE_LOCAL: &str = "local";
pub const ENGINE_FREE: &str = "free";
pub const ENGINE_UNKNOWN: &str = "unknown";

/// Classify a turn into `(engine, provider)` at write time.
///
/// Returns `None` when nothing can be decided (the columns should stay NULL).
///
/// Resolution order:
///  1. `sub:<provider>[/<model>]`  ->  subscription / provider
///  2. `acct:<id>/<model>`         ->  api_key / id
///  3. Bot whose `llm-provider` is a subscription name  ->  subscription / provider
///  4. Otherwise, from the effective URL and model that the turn actually uses
///     (after Vault config + env overrides):
///       Local    ->  local / local
///       Free     ->  free / <free provider id>
///       OwnKey   ->  api_key / configured provider or host-derived id
///  5. When nothing matches, return `None` (never store "unknown").
pub fn classify_preference(
    model_preference: &str,
    bot_provider: &str,
    effective_url: &str,
    effective_model: &str,
) -> Option<(&'static str, String)> {
    let pref = model_preference.trim();

    if let Some(rest) = pref.strip_prefix("sub:") {
        let provider = rest.split('/').next().unwrap_or(rest).trim();
        if !provider.is_empty() {
            return Some((ENGINE_SUBSCRIPTION, provider.to_string()));
        }
    }

    if let Some(rest) = pref.strip_prefix("acct:") {
        let provider_id = rest.split('/').next().unwrap_or(rest).trim();
        if !provider_id.is_empty() {
            return Some((ENGINE_API_KEY, provider_id.to_string()));
        }
    }

    let bp = bot_provider.trim();
    if is_subscription_name(bp) {
        return Some((ENGINE_SUBSCRIPTION, bp.to_string()));
    }

    let mode = model_config::infer_mode(effective_url, effective_model);
    match mode {
        ModelMode::Local => Some((ENGINE_LOCAL, "local".to_string())),
        ModelMode::Free => Some((ENGINE_FREE, model_config::FREE_PROVIDER.to_string())),
        ModelMode::OwnKey => {
            if !bp.is_empty() {
                return Some((ENGINE_API_KEY, bp.to_string()));
            }
            provider_id_from_host(effective_url)
                .map(|id| (ENGINE_API_KEY, id))
        }
    }
}

/// Extract a provider id from an endpoint URL's host.
///
/// `api.anthropic.com` -> `anthropic`, `api.openai.com` -> `openai`,
/// `*.googleapis.com` -> `google`, `api.deepseek.com` -> `deepseek`.
/// Returns `None` for unparseable addresses.
fn provider_id_from_host(url: &str) -> Option<String> {
    // Extract the host from the URL without depending on the `url` crate.
    let after_scheme = url.split("://").nth(1)?;
    let authority = after_scheme.split('/').next()?;
    // Strip optional port and userinfo.
    let host_part = authority.rsplit('@').next()?;
    let host = host_part.split(':').next()?;
    if host.is_empty() {
        return None;
    }
    let lower = host.to_ascii_lowercase();

    if lower.ends_with(".googleapis.com") || lower == "googleapis.com" {
        return Some("google".to_string());
    }

    // Strip the common `api.` prefix and the public suffix.
    let without_api = lower.strip_prefix("api.").unwrap_or(&lower);
    let name = without_api.split('.').next()?;
    if name.is_empty() {
        return None;
    }
    Some(name.to_string())
}

fn is_subscription_name(name: &str) -> bool {
    matches!(
        name,
        "claude-code" | "chatgpt-plan" | "codex-cli" | "antigravity"
    )
}

/// Fallback classifier for rows written before the engine columns existed.
/// Based on the `llm_model` name alone — honest about what it cannot know.
pub fn classify_model_fallback(model: &str) -> (&'static str, &'static str) {
    let m = model.trim().to_ascii_lowercase();

    if m.ends_with(".gguf") {
        return (ENGINE_LOCAL, "local");
    }
    if m.ends_with(":free") {
        return (ENGINE_FREE, ENGINE_FREE);
    }
    if m.starts_with("codex-") {
        return (ENGINE_SUBSCRIPTION, "codex-cli");
    }

    (ENGINE_UNKNOWN, ENGINE_UNKNOWN)
}

/// Display label for a known provider id.
pub fn provider_label(provider: &str) -> Option<&'static str> {
    match provider {
        "claude-code" => Some("Claude Code"),
        "codex-cli" => Some("Codex"),
        "antigravity" => Some("Antigravity"),
        "anthropic" => Some("Anthropic"),
        "openai" => Some("OpenAI"),
        "chatgpt-plan" => Some("ChatGPT"),
        "google" => Some("Google"),
        "deepseek" => Some("DeepSeek"),
        "local" => Some("Local"),
        model_config::FREE_PROVIDER => Some("Kilo"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_subscription() {
        let r = classify_preference("sub:claude-code", "", "", "");
        assert_eq!(r, Some((ENGINE_SUBSCRIPTION, "claude-code".to_string())));
    }

    #[test]
    fn test_classify_subscription_with_model() {
        let r = classify_preference("sub:codex-cli/codex-mini-latest", "", "", "");
        assert_eq!(r, Some((ENGINE_SUBSCRIPTION, "codex-cli".to_string())));
    }

    #[test]
    fn test_classify_api_key() {
        let r = classify_preference("acct:anthropic/claude-sonnet-4-20250514", "", "", "");
        assert_eq!(r, Some((ENGINE_API_KEY, "anthropic".to_string())));
    }

    #[test]
    fn test_classify_bot_subscription_provider() {
        let r = classify_preference("auto", "claude-code", "", "");
        assert_eq!(r, Some((ENGINE_SUBSCRIPTION, "claude-code".to_string())));
    }

    #[test]
    fn test_classify_local_loopback() {
        let r = classify_preference("auto", "", "http://127.0.0.1:8081/v1/chat/completions", "llama-3.1-8b.gguf");
        assert_eq!(r, Some((ENGINE_LOCAL, "local".to_string())));
    }

    #[test]
    fn test_classify_local_localhost() {
        let r = classify_preference("balanced", "", "http://localhost:11434/v1/chat/completions", "phi-3");
        assert_eq!(r, Some((ENGINE_LOCAL, "local".to_string())));
    }

    #[test]
    fn test_classify_free_endpoint() {
        let r = classify_preference("auto", "", "https://api.kilo.ai/api/gateway/chat/completions", "dots-studio/dots-3-note-preview:free");
        assert_eq!(r, Some((ENGINE_FREE, "kilo".to_string())));
    }

    #[test]
    fn test_classify_free_empty_url_and_model() {
        let r = classify_preference("auto", "", "", "");
        assert_eq!(r, Some((ENGINE_FREE, "kilo".to_string())));
    }

    #[test]
    fn test_classify_own_key_configured_provider() {
        let r = classify_preference("balanced", "openai", "https://api.openai.com/v1/chat/completions", "gpt-4o");
        assert_eq!(r, Some((ENGINE_API_KEY, "openai".to_string())));
    }

    #[test]
    fn test_classify_own_key_by_host_anthropic() {
        let r = classify_preference("auto", "", "https://api.anthropic.com/v1/messages", "claude-sonnet-4-20250514");
        assert_eq!(r, Some((ENGINE_API_KEY, "anthropic".to_string())));
    }

    #[test]
    fn test_classify_own_key_by_host_openai() {
        let r = classify_preference("auto", "", "https://api.openai.com/v1/chat/completions", "gpt-4o");
        assert_eq!(r, Some((ENGINE_API_KEY, "openai".to_string())));
    }

    #[test]
    fn test_classify_own_key_by_host_google() {
        let r = classify_preference("auto", "", "https://generativelanguage.googleapis.com/v1/models", "gemini-pro");
        assert_eq!(r, Some((ENGINE_API_KEY, "google".to_string())));
    }

    #[test]
    fn test_classify_own_key_by_host_deepseek() {
        let r = classify_preference("auto", "", "https://api.deepseek.com/v1/chat/completions", "deepseek-chat");
        assert_eq!(r, Some((ENGINE_API_KEY, "deepseek".to_string())));
    }

    #[test]
    fn test_classify_undecidable_returns_none() {
        // OwnKey mode but no configured provider and unparseable URL
        let r = classify_preference("auto", "", "not-a-url", "some-model");
        assert_eq!(r, None);
    }

    #[test]
    fn test_fallback_gguf() {
        assert_eq!(classify_model_fallback("llama-3.1-8b.gguf"), (ENGINE_LOCAL, "local"));
    }

    #[test]
    fn test_fallback_free() {
        assert_eq!(classify_model_fallback("llama-3.1:free"), (ENGINE_FREE, ENGINE_FREE));
    }

    #[test]
    fn test_fallback_codex() {
        assert_eq!(classify_model_fallback("codex-mini-latest"), (ENGINE_SUBSCRIPTION, "codex-cli"));
    }

    #[test]
    fn test_fallback_unknown_claude() {
        assert_eq!(classify_model_fallback("claude-sonnet-4-20250514"), (ENGINE_UNKNOWN, ENGINE_UNKNOWN));
    }

    #[test]
    fn test_provider_label() {
        assert_eq!(provider_label("claude-code"), Some("Claude Code"));
        assert_eq!(provider_label("anthropic"), Some("Anthropic"));
        assert_eq!(provider_label("google"), Some("Google"));
        assert_eq!(provider_label("deepseek"), Some("DeepSeek"));
        assert_eq!(provider_label("local"), Some("Local"));
        assert_eq!(provider_label("kilo"), Some("Kilo"));
        assert_eq!(provider_label("random-thing"), None);
    }

    #[test]
    fn test_stored_unknown_falls_back() {
        // A stored engine of "unknown" should be treated like NULL by the read
        // side (build_engines_array), which applies classify_model_fallback.
        // This test verifies that classify_model_fallback produces something
        // other than unknown for recognisable models.
        let (e, _) = classify_model_fallback("llama-3.1-8b.gguf");
        assert_ne!(e, ENGINE_UNKNOWN);
        let (e, _) = classify_model_fallback("llama-3.1:free");
        assert_ne!(e, ENGINE_UNKNOWN);
    }

    #[test]
    fn test_provider_id_from_host() {
        assert_eq!(provider_id_from_host("https://api.anthropic.com/v1/messages"), Some("anthropic".to_string()));
        assert_eq!(provider_id_from_host("https://api.openai.com/v1/chat"), Some("openai".to_string()));
        assert_eq!(provider_id_from_host("https://generativelanguage.googleapis.com/v1"), Some("google".to_string()));
        assert_eq!(provider_id_from_host("https://api.deepseek.com/v1/chat"), Some("deepseek".to_string()));
        assert_eq!(provider_id_from_host("not-a-url"), None);
    }
}
