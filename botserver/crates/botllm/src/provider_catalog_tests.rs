//! Offline tests for the provider catalog (issue #2075, AI wave).
//!
//! The catalog is configuration data, so the tests pin the invariants every
//! consumer relies on: one row per backend identity, non-empty host patterns,
//! bounded caps, and a stable lookup for each identity.

use super::*;

#[test]
fn every_provider_type_has_a_catalog_row() {
    for provider_type in ALL_PROVIDER_TYPES {
        assert!(
            profile_for_type(*provider_type).is_some(),
            "{provider_type:?} has no catalog row"
        );
    }
}

#[test]
fn config_names_are_unique_and_stable() {
    let mut seen = Vec::new();
    for provider_type in ALL_PROVIDER_TYPES {
        let name = provider_type.config_name();
        assert!(!name.is_empty(), "{provider_type:?} has no config name");
        assert!(
            !seen.contains(&name),
            "duplicate config name {name} for {provider_type:?}"
        );
        seen.push(name);
    }
}

#[test]
fn config_name_and_display_name_are_published_values() {
    let cases = [
        (LLMProviderType::OpenAI, "openai", "OpenAI"),
        (LLMProviderType::Claude, "claude", "Claude"),
        (LLMProviderType::AzureClaude, "azure-claude", "Azure Claude"),
        (LLMProviderType::AzureGPT5, "azure-gpt5", "Azure GPT-5"),
        (LLMProviderType::GLM, "glm", "GLM"),
        (LLMProviderType::Bedrock, "bedrock", "Bedrock"),
        (LLMProviderType::Vertex, "vertex", "Vertex"),
        (LLMProviderType::Kiro, "kiro", "Kiro"),
        (LLMProviderType::DeepInfra, "deepinfra", "DeepInfra"),
        (LLMProviderType::Cerebras, "cerebras", "Cerebras"),
        (LLMProviderType::Fireworks, "fireworks", "Fireworks AI"),
        (LLMProviderType::TogetherAI, "together", "Together AI"),
        (LLMProviderType::OpenRouter, "openrouter", "OpenRouter"),
        (LLMProviderType::Requesty, "requesty", "Requesty"),
    ];

    for (provider_type, config_name, display_name) in cases {
        assert_eq!(provider_type.config_name(), config_name);
        assert_eq!(provider_type.display_name(), display_name);
    }
}

#[test]
fn only_wire_compatible_backends_are_marked_openai_compatible() {
    let compatible = [
        LLMProviderType::OpenAI,
        LLMProviderType::DeepInfra,
        LLMProviderType::Cerebras,
        LLMProviderType::Fireworks,
        LLMProviderType::TogetherAI,
        LLMProviderType::OpenRouter,
        LLMProviderType::Requesty,
    ];
    let native = [
        LLMProviderType::Claude,
        LLMProviderType::AzureClaude,
        LLMProviderType::AzureGPT5,
        LLMProviderType::GLM,
        LLMProviderType::Bedrock,
        LLMProviderType::Vertex,
        LLMProviderType::Kiro,
    ];

    for provider_type in compatible {
        assert!(provider_type.is_openai_compatible(), "{provider_type:?}");
    }
    for provider_type in native {
        assert!(!provider_type.is_openai_compatible(), "{provider_type:?}");
    }
}

#[test]
fn every_row_is_usable_for_detection_and_accounting() {
    for profile in PROVIDER_PROFILES {
        assert!(
            !profile.host_patterns.is_empty(),
            "{:?} has no host pattern",
            profile.provider_type
        );
        assert!(
            profile
                .host_patterns
                .iter()
                .all(|p| !p.is_empty() && p.to_lowercase() == *p),
            "{:?} must declare lowercase host patterns",
            profile.provider_type
        );
        assert!(
            profile.max_output_tokens > 0,
            "{:?} has a zero token ceiling",
            profile.provider_type
        );
        assert!(
            profile.usd_per_mtok_in >= 0.0 && profile.usd_per_mtok_out >= 0.0,
            "{:?} has negative pricing",
            profile.provider_type
        );
        assert!(
            !profile.reference_model.is_empty(),
            "{:?} has no reference model",
            profile.provider_type
        );
    }
}

#[test]
fn specific_hosts_precede_the_generic_openai_row() {
    // `provider_resolve` walks the table in order and the first host match wins.
    let openai_row = PROVIDER_PROFILES
        .iter()
        .position(|p| p.host_patterns.contains(&"api.openai.com"))
        .expect("generic openai row exists");

    for (host, expected) in [
        ("groq.com", LLMProviderType::OpenAI),
        ("build.nvidia.com", LLMProviderType::OpenAI),
        ("api.cerebras.ai", LLMProviderType::Cerebras),
        ("openrouter.ai", LLMProviderType::OpenRouter),
        ("api.deepinfra.com", LLMProviderType::DeepInfra),
    ] {
        let index = PROVIDER_PROFILES
            .iter()
            .position(|p| p.host_patterns.iter().any(|pattern| host.contains(pattern)))
            .unwrap_or_else(|| panic!("no row matches {host}"));
        assert_eq!(PROVIDER_PROFILES[index].provider_type, expected, "{host}");
        if expected == LLMProviderType::OpenAI {
            assert!(index < openai_row, "{host} must precede api.openai.com");
        }
    }
}

#[test]
fn token_ceilings_match_the_published_rows() {
    let cases = [
        (LLMProviderType::Claude, 8_192),
        (LLMProviderType::AzureClaude, 8_192),
        (LLMProviderType::AzureGPT5, 16_384),
        (LLMProviderType::GLM, 16_384),
        (LLMProviderType::Bedrock, 8_192),
        (LLMProviderType::Vertex, 8_192),
        (LLMProviderType::Kiro, 32_000),
        (LLMProviderType::Cerebras, 8_192),
        (LLMProviderType::DeepInfra, 8_192),
        (LLMProviderType::Fireworks, 16_384),
        (LLMProviderType::TogetherAI, 8_192),
        (LLMProviderType::OpenRouter, 16_384),
        (LLMProviderType::Requesty, 16_384),
    ];

    for (provider_type, expected) in cases {
        let profile = profile_for_type(provider_type).expect("row");
        assert_eq!(profile.max_output_tokens, expected, "{provider_type:?}");
    }
}

#[test]
fn the_openai_identity_covers_several_hosts_and_returns_the_first_row() {
    // `openai` deliberately groups groq.com, build.nvidia.com and api.openai.com.
    // The identity lookup returns the first row; only the URL can tell them
    // apart, which is why `provider_resolve::max_output_tokens_for_type` special
    // cases this identity.
    let rows: Vec<&ProviderProfile> = PROVIDER_PROFILES
        .iter()
        .filter(|p| p.provider_type == LLMProviderType::OpenAI)
        .collect();
    assert!(rows.len() >= 2, "the generic identity spans several hosts");

    let by_identity = profile_for_type(LLMProviderType::OpenAI).expect("row");
    // `PROVIDER_PROFILES` is a `const`, so each use site may materialise its own
    // copy: identity is compared by value, never by pointer.
    assert_eq!(by_identity.host_patterns, rows[0].host_patterns);

    let generic = rows
        .iter()
        .find(|p| p.host_patterns.contains(&"api.openai.com"))
        .expect("api.openai.com row");
    assert_eq!(generic.max_output_tokens, DEFAULT_MAX_OUTPUT_TOKENS);
}

#[test]
fn a_host_without_a_row_falls_back_to_the_documented_ceiling() {
    // The default matches the platform's long-standing value for `openai`.
    assert_eq!(DEFAULT_MAX_OUTPUT_TOKENS, 65_536);
}
