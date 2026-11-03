//! LLM provider metadata used by the Vibe LLM selection.
//!
//! When a user connects an LLM integration in Settings and marks it as the
//! Vibe LLM, the connection handler copies the provider's default endpoint
//! and model (this table) plus the Vault-stored API key into the per-bot
//! `vibe-llm-*` config keys. The Vibe agent loop then resolves those keys
//! through `ConfigManager` with no further integration lookups.

/// Subscription providers are per-user accounts, not integration connections.
pub const CLAUDE_CODE_SLUG: &str = "claude-code";
pub const CHATGPT_PLAN_SLUG: &str = "chatgpt-plan";

pub fn is_subscription_provider(provider_slug: &str) -> bool {
    matches!(provider_slug, CLAUDE_CODE_SLUG | CHATGPT_PLAN_SLUG)
}

/// Default (chat-completions url, model) for an LLM provider slug.
/// Returns `None` for unknown slugs so callers can reject non-LLM
/// connections.
/// The Kilo gateway — keyless free-tier OpenAI-compatible models.
pub const KILO_BASE_URL: &str = "https://api.kilo.ai/api/gateway/chat/completions";

/// Default (chat-completions url, model) for an LLM provider slug.
/// Returns `None` for unknown slugs so callers can reject non-LLM
/// connections.
pub fn llm_provider_defaults(provider_slug: &str) -> Option<(&'static str, &'static str)> {
    let url_model = match provider_slug {
        "kilo-dots" => (KILO_BASE_URL, "dots-studio/dots-3-note-preview:free"),
        "kilo-openrouter" => (KILO_BASE_URL, "openrouter/free"),
        "kilo-nemotron-ultra" => (KILO_BASE_URL, "nvidia/nemotron-3-ultra-550b-a55b:free"),
        "kilo-nemotron-super" => (KILO_BASE_URL, "nvidia/nemotron-3.5-super-vl:free"),
        "kilo-nemotron-lightning" => (KILO_BASE_URL, "nvidia/nemotron-3.5-lightning:free"),
        "kilo-nano" => (KILO_BASE_URL, "nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free"),
        "kilo-cohere" => (KILO_BASE_URL, "cohere/north-mini-code:free"),
        "kilo-flint" => (KILO_BASE_URL, "recursive/flint-v1.0:free"),
        "kilo-xing" => (KILO_BASE_URL, "xingchen-agi/xing4.0-29b-a4b:free"),
        CLAUDE_CODE_SLUG => ("https://api.anthropic.com/v1/messages", "claude-sonnet-4-5"),
        CHATGPT_PLAN_SLUG => ("https://chatgpt.com/backend-api/codex", "gpt-5-codex"),
        _ => return None,
    };
    Some(url_model)
}

/// True when the provider requires no API key (keyless gateway models).
pub fn llm_provider_keyless(provider_slug: &str) -> bool {
    matches!(
        provider_slug,
        "kilo-dots"
            | "kilo-openrouter"
            | "kilo-nemotron-ultra"
            | "kilo-nemotron-super"
            | "kilo-nemotron-lightning"
            | "kilo-cohere"
            | "kilo-nano"
            | "kilo-flint"
            | "kilo-xing"
            | CLAUDE_CODE_SLUG
            | CHATGPT_PLAN_SLUG
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_providers_have_defaults() {
        for slug in [
            "kilo-dots", "kilo-openrouter", "kilo-nemotron-ultra",
            "kilo-nemotron-super", "kilo-nemotron-lightning", "kilo-nano",
            "kilo-cohere", "kilo-flint", "kilo-xing",
        ] {
            let (url, model) =
                llm_provider_defaults(slug).unwrap_or_else(|| panic!("{slug} must have defaults"));
            assert!(url.starts_with("https://"), "{slug} url must be https");
            assert!(!model.is_empty(), "{slug} model must not be empty");
        }
    }

    #[test]
    fn verified_providers_are_keyless() {
        for slug in [
            "kilo-dots", "kilo-openrouter", "kilo-nemotron-ultra",
            "kilo-nemotron-super", "kilo-nemotron-lightning", "kilo-nano",
            "kilo-cohere", "kilo-flint", "kilo-xing",
        ] {
            assert!(llm_provider_keyless(slug), "{slug} must be keyless");
        }
        assert!(!llm_provider_keyless("github"));
    }

    #[test]
    fn unknown_providers_have_no_defaults() {
        assert!(llm_provider_defaults("github").is_none());
        assert!(llm_provider_defaults("").is_none());
    }

    #[test]
    fn subscription_providers_are_keyless_and_present() {
        for slug in [CLAUDE_CODE_SLUG, CHATGPT_PLAN_SLUG] {
            let (_, model) = llm_provider_defaults(slug).expect("subscription defaults");
            assert!(!model.is_empty());
            assert!(llm_provider_keyless(slug));
            assert!(is_subscription_provider(slug));
        }
        assert!(!is_subscription_provider("kilo-dots"));
    }
}