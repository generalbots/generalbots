//! Host detection and limit resolution for the provider catalog (issue #1467).
//!
//! Split from [`provider_catalog`] to keep both files inside the project size
//! limit. The catalog holds *what* each provider is; this module answers *which*
//! provider a URL or a config name refers to and what limits follow from that.

use crate::provider_catalog::{
    DEFAULT_MAX_OUTPUT_TOKENS, LLMProviderType, PROVIDER_PROFILES, ProviderProfile,
    profile_for_type,
};
use crate::rate_limiter::RateLimits;

/// Every catalog row that would be selected by `url`, in table order.
///
/// A host can appear more than once — `azure` plus `claude` both select the
/// Azure Claude row — which is what makes the "most specific host wins" rule
/// observable in a test.
#[must_use]
pub fn profiles_for_url(url: &str) -> Vec<&'static ProviderProfile> {
    let lower = url.to_lowercase();
    PROVIDER_PROFILES
        .iter()
        .filter(|p| p.host_patterns.iter().any(|pat| lower.contains(pat)))
        .collect()
}

/// Backends whose base URL identifies them, most specific first.
///
/// Returns `None` when nothing matches, which is the caller's signal to fall
/// back to the generic `openai` client.
#[must_use]
pub fn provider_type_from_url(url: &str) -> Option<LLMProviderType> {
    profiles_for_url(url).first().map(|p| p.provider_type)
}

/// Resolves the `llm-provider` bot-config value to a backend.
///
/// Accepts both the canonical [`LLMProviderType::config_name`] and the provider
/// brand, so existing values (`openai`, `nvidia`, `groq`, `anthropic`,
/// `azure`) keep resolving. Returns `None` for an empty or unknown name so the
/// caller falls back to URL detection instead of guessing.
#[must_use]
pub fn provider_type_from_name(name: &str) -> Option<LLMProviderType> {
    let lower = name.trim().to_lowercase();
    if lower.is_empty() {
        return None;
    }
    // `kiro` is tested first: `kiro-claude` names an entitlement, not a model.
    if lower.contains("kiro") || lower.contains("codewhisperer") {
        return Some(LLMProviderType::Kiro);
    }
    if lower.contains("azure") && lower.contains("claude") {
        return Some(LLMProviderType::AzureClaude);
    }
    if lower.contains("azure") {
        return Some(LLMProviderType::AzureGPT5);
    }
    if lower.contains("claude") || lower.contains("anthropic") {
        return Some(LLMProviderType::Claude);
    }
    if lower.contains("glm") || lower.contains("z.ai") {
        return Some(LLMProviderType::GLM);
    }
    if lower.contains("bedrock") {
        return Some(LLMProviderType::Bedrock);
    }
    if lower.contains("vertex") || lower.contains("gemini") || lower.contains("google") {
        return Some(LLMProviderType::Vertex);
    }
    for profile in PROVIDER_PROFILES {
        if profile.host_patterns.iter().any(|pat| lower.contains(pat)) {
            return Some(profile.provider_type);
        }
    }
    match lower.as_str() {
        "openai" | "openai-compatible" => Some(LLMProviderType::OpenAI),
        "nvidia" | "nim" | "groq" => Some(LLMProviderType::OpenAI),
        "cerebras" | "cerebras-ai" => Some(LLMProviderType::Cerebras),
        "deepinfra" | "deep-infra" => Some(LLMProviderType::DeepInfra),
        "fireworks" | "fireworks-ai" => Some(LLMProviderType::Fireworks),
        "together" | "together-ai" | "togetherai" => Some(LLMProviderType::TogetherAI),
        "openrouter" | "open-router" => Some(LLMProviderType::OpenRouter),
        "requesty" => Some(LLMProviderType::Requesty),
        _ => None,
    }
}

/// Host fragments that identify a self-hosted or in-network inference server.
/// These have no per-request quota and no billing, so capping them only adds
/// latency — a local llama.cpp endpoint must never be throttled by the API
/// limiter.
const PRIVATE_HOST_MARKERS: &[&str] = &[
    "localhost",
    "127.0.0.1",
    "0.0.0.0",
    "[::1]",
    "host.docker.internal",
    "10.0.",
    "172.16.",
    "192.168.",
    ".local",
];

/// True when the URL points at a self-hosted or private-network endpoint.
#[must_use]
pub fn is_private_host(url: &str) -> bool {
    let lower = url.to_lowercase();
    PRIVATE_HOST_MARKERS.iter().any(|marker| lower.contains(marker))
}

/// Operating caps for a base URL.
///
/// A private host gets [`RateLimits::unlimited`] — there is no quota to protect.
/// A recognised public host gets its catalog row. Anything else gets
/// [`RateLimits::openai_compatible_default`], which is bounded: a mistyped URL
/// must not become an unbounded retry storm.
#[must_use]
pub fn rate_limits_for_url(url: &str) -> RateLimits {
    if is_private_host(url) {
        return RateLimits::unlimited();
    }
    profiles_for_url(url)
        .first()
        .map(|p| p.rate_limits)
        .unwrap_or_else(RateLimits::openai_compatible_default)
}

/// `max_tokens` for a base URL.
#[must_use]
pub fn max_output_tokens_for_url(url: &str) -> u32 {
    profiles_for_url(url)
        .first()
        .map(|p| p.max_output_tokens)
        .unwrap_or(DEFAULT_MAX_OUTPUT_TOKENS)
}

/// Ceiling for an explicit backend identity, independent of the URL.
///
/// [`LLMProviderType::OpenAI`] is deliberately excluded: it covers both
/// `groq.com` (4 096) and `api.openai.com` (65 536), and only the URL can tell
/// them apart. The generic ceiling is the correct answer for the ambiguous
/// identity.
#[must_use]
pub fn max_output_tokens_for_type(provider_type: LLMProviderType) -> u32 {
    if provider_type == LLMProviderType::OpenAI {
        return DEFAULT_MAX_OUTPUT_TOKENS;
    }
    profile_for_type(provider_type)
        .map(|p| p.max_output_tokens)
        .unwrap_or(DEFAULT_MAX_OUTPUT_TOKENS)
}

impl From<&str> for LLMProviderType {
    /// URL-pattern detection, preserving the historical precedence so no
    /// deployed bot changes provider. Unknown hosts fall through to `OpenAI`,
    /// which is the correct generic OpenAI-compatible client.
    fn from(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("claude") || lower.contains("anthropic") {
            if lower.contains("azure") {
                Self::AzureClaude
            } else {
                Self::Claude
            }
        } else if lower.contains("azuregpt5")
            || lower.contains("gpt5")
            || (lower.contains("openai.azure.com") && lower.contains("responses"))
        {
            Self::AzureGPT5
        } else if lower.contains("z.ai") || lower.contains("glm") {
            Self::GLM
        } else if lower.contains("bedrock") {
            Self::Bedrock
        } else if lower.contains("googleapis.com")
            || lower.contains("vertex")
            || lower.contains("generativelanguage")
        {
            Self::Vertex
        } else if lower.contains("kiro")
            || lower.contains("q.us-east-1.amazonaws.com")
            || lower.contains("codewhisperer")
        {
            Self::Kiro
        } else {
            provider_type_from_url(&lower).unwrap_or(Self::OpenAI)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider_catalog::ALL_PROVIDER_TYPES;

    #[test]
    fn every_variant_has_a_catalog_row() {
        for provider_type in ALL_PROVIDER_TYPES {
            assert!(
                profile_for_type(*provider_type).is_some(),
                "{} has no catalog row",
                provider_type.config_name()
            );
        }
    }

    #[test]
    fn openai_compatible_hosts_are_detected_from_their_url() {
        let cases = [
            ("https://api.cerebras.ai/v1/chat/completions", LLMProviderType::Cerebras),
            ("https://api.deepinfra.com/v1/openai/chat/completions", LLMProviderType::DeepInfra),
            ("https://api.fireworks.ai/inference/v1/chat/completions", LLMProviderType::Fireworks),
            ("https://api.together.xyz/v1/chat/completions", LLMProviderType::TogetherAI),
            ("https://openrouter.ai/api/v1/chat/completions", LLMProviderType::OpenRouter),
            ("https://router.requesty.ai/v1/chat/completions", LLMProviderType::Requesty),
        ];
        for (url, expected) in cases {
            assert_eq!(LLMProviderType::from(url), expected, "url {url}");
            assert_eq!(provider_type_from_url(url), Some(expected), "url {url}");
        }
    }

    #[test]
    fn an_explicit_provider_name_beats_url_detection() {
        assert_eq!(
            provider_type_from_name("cerebras"),
            Some(LLMProviderType::Cerebras)
        );
        assert_eq!(provider_type_from_name("deepinfra"), Some(LLMProviderType::DeepInfra));
        assert_eq!(provider_type_from_name("Fireworks-AI"), Some(LLMProviderType::Fireworks));
        assert_eq!(provider_type_from_name("together"), Some(LLMProviderType::TogetherAI));
        assert_eq!(provider_type_from_name("openrouter"), Some(LLMProviderType::OpenRouter));
        assert_eq!(provider_type_from_name("requesty"), Some(LLMProviderType::Requesty));
    }

    #[test]
    fn legacy_provider_names_keep_their_old_resolution() {
        assert_eq!(provider_type_from_name("openai"), Some(LLMProviderType::OpenAI));
        assert_eq!(provider_type_from_name("nvidia"), Some(LLMProviderType::OpenAI));
        assert_eq!(provider_type_from_name("groq"), Some(LLMProviderType::OpenAI));
        assert_eq!(provider_type_from_name("anthropic"), Some(LLMProviderType::Claude));
        assert_eq!(provider_type_from_name("azure"), Some(LLMProviderType::AzureGPT5));
        assert_eq!(provider_type_from_name("kiro"), Some(LLMProviderType::Kiro));
        assert_eq!(provider_type_from_name("kiro-claude"), Some(LLMProviderType::Kiro));
        assert_eq!(provider_type_from_name(""), None);
        assert_eq!(provider_type_from_name("nonexistent"), None);
    }

    #[test]
    fn rate_limits_come_from_the_host_row() {
        let groq = rate_limits_for_url("https://api.groq.com/openai/v1/chat/completions");
        assert_eq!(groq, RateLimits::groq_free_tier());
        let openai = rate_limits_for_url("https://api.openai.com/v1/chat/completions");
        assert_eq!(openai, RateLimits::openai_free_tier());
        assert_eq!(rate_limits_for_url("https://api.cerebras.ai/v1"), RateLimits::cerebras_free_tier());
        assert_eq!(rate_limits_for_url("https://api.deepinfra.com/v1"), RateLimits::deepinfra_standard());
        assert_eq!(rate_limits_for_url("https://openrouter.ai/api/v1"), RateLimits::openrouter_standard());
    }

    #[test]
    fn unknown_hosts_are_bounded_rather_than_unlimited() {
        let limits = rate_limits_for_url("https://llm.internal.example/v1");
        assert_eq!(limits, RateLimits::openai_compatible_default());
        assert_ne!(limits.requests_per_minute, u32::MAX);
        assert_eq!(max_output_tokens_for_url("https://llm.internal.example/v1"), DEFAULT_MAX_OUTPUT_TOKENS);
    }

    #[test]
    fn self_hosted_endpoints_are_never_throttled() {
        for url in [
            "http://localhost:11434/v1",
            "http://127.0.0.1:8000/v1",
            "http://host.docker.internal:8080/v1",
            "http://10.0.3.10:5858/v1",
            "http://192.168.1.20:1234/v1",
        ] {
            assert!(is_private_host(url), "{url}");
            assert_eq!(rate_limits_for_url(url), RateLimits::unlimited(), "{url}");
        }
        assert!(!is_private_host("https://api.openai.com/v1"));
    }

    #[test]
    fn token_ceilings_are_per_host_data_not_a_groq_check() {
        assert_eq!(max_output_tokens_for_url("https://api.groq.com/openai/v1"), 4_096);
        assert_eq!(max_output_tokens_for_url("https://api.openai.com/v1"), 65_536);
        assert_eq!(max_output_tokens_for_url("https://api.cerebras.ai/v1"), 8_192);
        assert_eq!(
            max_output_tokens_for_type(LLMProviderType::Cerebras),
            8_192
        );
    }

    #[test]
    fn the_generic_openai_identity_never_claims_the_groq_ceiling() {
        // Two rows share the `openai` identity, so the type alone must resolve
        // to the generous ceiling and let the URL decide.
        assert_eq!(max_output_tokens_for_type(LLMProviderType::OpenAI), 65_536);
        assert_eq!(
            max_output_tokens_for_url("https://api.openai.com/v1"),
            max_output_tokens_for_type(LLMProviderType::OpenAI)
        );
    }

    #[test]
    fn most_specific_host_wins_over_the_generic_row() {
        let matches = profiles_for_url("https://api.anthropic.com/v1/messages");
        assert_eq!(matches[0].provider_type, LLMProviderType::Claude);

        let azure = profiles_for_url("https://x.openai.azure.com/claude/v1");
        assert_eq!(azure[0].provider_type, LLMProviderType::AzureClaude);
    }

    #[test]
    fn openai_compatible_flag_matches_the_client_that_serves_each_host() {
        assert!(LLMProviderType::Cerebras.is_openai_compatible());
        assert!(LLMProviderType::OpenAI.is_openai_compatible());
        assert!(!LLMProviderType::Claude.is_openai_compatible());
        assert!(!LLMProviderType::Bedrock.is_openai_compatible());
    }

    #[test]
    fn config_names_are_unique_per_variant() {
        // The catalog carries one extra row — the generic `openai` identity is
        // listed twice, for groq.com and api.openai.com — so uniqueness is a
        // property of the variants, not of the rows.
        let mut names: Vec<&str> = PROVIDER_PROFILES
            .iter()
            .map(|p| p.provider_type.config_name())
            .collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), ALL_PROVIDER_TYPES.len());
    }

    #[test]
    fn reference_pricing_is_positive_for_paid_hosts() {
        for profile in PROVIDER_PROFILES {
            if profile.provider_type == LLMProviderType::Kiro {
                continue;
            }
            assert!(profile.usd_per_mtok_in > 0.0, "{}", profile.provider_type.config_name());
            assert!(profile.usd_per_mtok_out > 0.0, "{}", profile.provider_type.config_name());
        }
    }
}
