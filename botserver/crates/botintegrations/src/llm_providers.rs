//! LLM provider metadata used by the Vibe LLM selection.
//!
//! When a user connects an LLM integration in Settings and marks it as the
//! Vibe LLM, the connection handler copies the provider's default endpoint
//! and model (this table) plus the Vault-stored API key into the per-bot
//! `vibe-llm-*` config keys. The Vibe agent loop then resolves those keys
//! through `ConfigManager` with no further integration lookups.

/// Default (chat-completions url, model) for an LLM provider slug.
/// Returns `None` for unknown slugs so callers can reject non-LLM
/// connections.
pub fn llm_provider_defaults(provider_slug: &str) -> Option<(&'static str, &'static str)> {
    match provider_slug {
        "groq" => Some((
            "https://api.groq.com/openai/v1/chat/completions",
            "openai/gpt-oss-120b",
        )),
        "siliconflow" => Some((
            "https://api.siliconflow.cn/v1/chat/completions",
            "Qwen/Qwen3-8B",
        )),
        "gemini" => Some((
            "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
            "gemini-3.5-flash-lite",
        )),
        "zai" | "zhipu" => Some((
            "https://open.bigmodel.cn/api/paas/v4/chat/completions",
            "glm-4.7-flash",
        )),
        "alibaba" => Some((
            "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions",
            "qwen-3.6-plus",
        )),
        "deepseek" => Some((
            "https://api.deepseek.com/chat/completions",
            "deepseek-v4-flash",
        )),
        "minimax" => Some((
            "https://api.minimaxi.com/v1/text/chatcompletion_v2",
            "MiniMax-Text-01",
        )),
        "yi" => Some((
            "https://api.lingyiwanwu.com/v1/chat/completions",
            "yi-lightning",
        )),
        "openai" => Some((
            "https://api.openai.com/v1/chat/completions",
            "gpt-5.4",
        )),
        "anthropic" => Some((
            "https://api.anthropic.com/v1/chat/completions",
            "claude-sonnet-4-5",
        )),
        "google" => Some((
            "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
            "gemini-3.5-flash-lite",
        )),
        "meta" => Some((
            "https://api.ai.meta.com/v1/chat/completions",
            "llama-4-maverick",
        )),
        "mistral" => Some((
            "https://api.mistral.ai/v1/chat/completions",
            "mistral-large-latest",
        )),
        "amazon" => Some((
            "https://bedrock-runtime.us-east-1.amazonaws.com/model/anthropic.claude-3-5-sonnet/v1/chat/completions",
            "anthropic.claude-3-5-sonnet",
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_providers_have_defaults() {
        for slug in [
            "groq", "siliconflow", "gemini", "zai", "zhipu", "alibaba", "deepseek", "minimax",
            "yi", "openai", "anthropic", "google", "meta", "mistral", "amazon",
        ] {
            let (url, model) =
                llm_provider_defaults(slug).unwrap_or_else(|| panic!("{slug} must have defaults"));
            assert!(url.starts_with("https://"), "{slug} url must be https");
            assert!(!model.is_empty(), "{slug} model must not be empty");
        }
    }

    #[test]
    fn unknown_providers_have_no_defaults() {
        assert!(llm_provider_defaults("github").is_none());
        assert!(llm_provider_defaults("").is_none());
    }
}