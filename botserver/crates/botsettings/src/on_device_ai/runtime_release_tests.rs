use super::*;

fn choices(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

const HOSTED: &[(&str, &str)] = &[
    ("llm-model", "nvidia/nemotron-3-ultra-550b-a55b:free"),
    ("llm-url", "https://api.example.com/api/gateway/chat"),
    ("llm-model-fast", ""),
    ("llm-model-balanced", ""),
    ("llm-model-quality", ""),
];

#[test]
fn a_hosted_choice_with_hosted_tiers_releases_the_engine() {
    assert!(should_release(&choices(HOSTED)));
    let mut with_tiers = choices(HOSTED);
    with_tiers.insert("llm-model-fast".into(), "gpt-4o-mini".into());
    with_tiers.insert("llm-model-quality".into(), "claude-sonnet-4-5".into());
    assert!(should_release(&with_tiers));
}

#[test]
fn a_local_tier_keeps_the_engine_even_when_the_main_choice_is_hosted() {
    for tier in ["llm-model-fast", "llm-model-balanced", "llm-model-quality"] {
        let mut set = choices(HOSTED);
        set.insert(tier.into(), "Qwen3.5-9B-Q4_K_M.gguf".into());
        assert!(!should_release(&set), "{tier}");
    }
}

#[test]
fn a_loopback_endpoint_keeps_the_engine() {
    let mut set = choices(HOSTED);
    set.insert("llm-url".into(), "http://localhost:8081/v1/chat/completions".into());
    assert!(!should_release(&set));
    let mut env = choices(HOSTED);
    env.insert("LLM_URL".into(), "http://127.0.0.1:8081".into());
    assert!(!should_release(&env), "the environment override wins");
    let mut env_model = choices(HOSTED);
    env_model.insert("LLM_MODEL".into(), "model.gguf".into());
    assert!(!should_release(&env_model));
}

#[test]
fn a_local_main_choice_or_no_choice_keeps_the_engine() {
    let mut local = choices(HOSTED);
    local.insert("llm-model".into(), "Qwen3.5-9B-Q4_K_M.gguf".into());
    local.insert("llm-url".into(), "http://localhost:8081/v1/chat/completions".into());
    assert!(!should_release(&local));
    assert!(!should_release(&choices(&[("llm-model", "")])), "first run, before the wizard");
    assert!(!should_release(&choices(&[("llm-model", "   ")])));
    assert!(!should_release(&HashMap::new()));
}

#[test]
fn a_model_name_mentioning_localhost_is_not_an_endpoint() {
    let set = choices(&[("llm-model", "localhost-tuned-7b"), ("llm-url", "https://api.example.com")]);
    assert!(should_release(&set));
}
