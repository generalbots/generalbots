use super::*;

#[test]
fn caller_supplied_llm_endpoint_is_ignored() {
    let malicious_url = "http://attacker.example.com/exfiltrate";
    let config = LlmConfig::for_request(
        Some(malicious_url),
        Some("attacker-supplied-key"),
        Some("attacker-model"),
    );
    assert!(
        !config.url.contains("attacker.example.com"),
        "caller-supplied LLM URL must never be used, got {}",
        config.url
    );
    assert_ne!(config.key, "attacker-supplied-key");
    assert_ne!(config.model, "attacker-model");
}

#[test]
fn over_budget_step_count_is_capped() {
    assert_eq!(bounded_max_steps(Some(usize::MAX)), MAX_AGENT_STEPS);
    assert_eq!(bounded_max_steps(Some(1_000)), MAX_AGENT_STEPS);
    assert_eq!(bounded_max_steps(Some(3)), 3);
    assert_eq!(bounded_max_steps(Some(0)), 1);
    assert_eq!(bounded_max_steps(None), DEFAULT_AGENT_STEPS);
}

#[test]
fn non_http_llm_endpoint_is_rejected() {
    assert!(validate_llm_endpoint("file:///etc/passwd").is_err());
    assert!(validate_llm_endpoint("ftp://attacker.example.com/x").is_err());
    assert!(validate_llm_endpoint("not-a-url").is_err());
    assert!(validate_llm_endpoint("http://127.0.0.1:8081/v1/chat/completions").is_ok());
}
