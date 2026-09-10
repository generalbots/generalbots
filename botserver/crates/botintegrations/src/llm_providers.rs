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
/// The Kilo gateway — keyless free-tier OpenAI-compatible models.
pub const KILO_BASE_URL: &str = "https://api.kilo.ai/api/gateway/chat/completions";
/// The LLM7 gateway — OpenAI-compatible, single-concurrent free tier.
pub const LLM7_BASE_URL: &str = "https://api.llm7.io/v1/chat/completions";

/// Default (chat-completions url, model) for an LLM provider slug.
/// Returns `None` for unknown slugs so callers can reject non-LLM
/// connections.
pub fn llm_provider_defaults(provider_slug: &str) -> Option<(&'static str, &'static str)> {
    let url_model = match provider_slug {
        "kilo-dots" => (KILO_BASE_URL, "dots-studio/dots-3-note-preview:free"),
        "kilo-nex" => (KILO_BASE_URL, "nex-agi/nex-n2.5-pro:free"),
        "kilo-ling" => (KILO_BASE_URL, "inclusionai/ling-3.0-flash-fin:free"),
        "kilo-openrouter" => (KILO_BASE_URL, "openrouter/free"),
        "kilo-nemotron-ultra" => (KILO_BASE_URL, "nvidia/nemotron-3-ultra-550b-a55b:free"),
        "kilo-nemotron-super" => (KILO_BASE_URL, "nvidia/nemotron-3-super-120b-a12b:free"),
        "kilo-nemotron-lightning" => (KILO_BASE_URL, "nvidia/nemotron-3.5-lightning:free"),
        "kilo-cohere" => (KILO_BASE_URL, "cohere/north-mini-code:free"),
        "kilo-stepfun" => (KILO_BASE_URL, "stepfun/step-3.7-flash:free"),
        "llm7-minimax" => (LLM7_BASE_URL, "minimax-m2.7"),
        _ => return None,
    };
    Some(url_model)
}

/// True when the provider requires no API key (keyless gateway models).
pub fn llm_provider_keyless(provider_slug: &str) -> bool {
    matches!(
        provider_slug,
        "kilo-dots"
            | "kilo-nex"
            | "kilo-ling"
            | "kilo-openrouter"
            | "kilo-nemotron-ultra"
            | "kilo-nemotron-super"
            | "kilo-nemotron-lightning"
            | "kilo-cohere"
            | "kilo-stepfun"
            | "llm7-minimax"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_providers_have_defaults() {
        for slug in [
            "kilo-dots", "kilo-nex", "kilo-ling", "kilo-openrouter", "kilo-nemotron-ultra",
            "kilo-nemotron-super", "kilo-nemotron-lightning", "kilo-cohere", "kilo-stepfun",
            "llm7-minimax",
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
            "kilo-dots", "kilo-nex", "kilo-ling", "kilo-openrouter", "kilo-nemotron-ultra",
            "kilo-nemotron-super", "kilo-nemotron-lightning", "kilo-cohere", "kilo-stepfun",
            "llm7-minimax",
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
}