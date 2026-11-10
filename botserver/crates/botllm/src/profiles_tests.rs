//! LLM profile selection: capability filtering and the `prefer_cheap`
//! ordering that decides which provider serves a request.
use super::*;

fn profile(id: &str, caps: &[&str], cost_per_1k: f64, priority: i32) -> LlmProfile {
    LlmProfile {
        id: id.to_string(),
        url: format!("https://{id}.example/v1/chat/completions"),
        model: "model".to_string(),
        key: "key".to_string(),
        caps: caps.iter().map(|cap| (*cap).to_string()).collect(),
        cost_per_1k,
        priority,
    }
}

#[test]
fn a_profile_without_capabilities_matches_only_an_empty_request() {
    let plain = profile("plain", &[], 0.0, 0);

    assert!(plain.matches_caps(&[]));
    assert!(!plain.matches_caps(&["vision"]));
}

#[test]
fn every_requested_capability_must_be_declared() {
    let capable = profile("capable", &["vision", "tools"], 0.0, 0);

    assert!(capable.matches_caps(&["vision"]));
    assert!(capable.matches_caps(&["vision", "tools"]));
    assert!(!capable.matches_caps(&["vision", "audio"]));
}

#[test]
fn capability_matching_ignores_letter_case() {
    let capable = profile("capable", &["Vision"], 0.0, 0);

    assert!(capable.matches_caps(&["vision"]));
    assert!(capable.matches_caps(&["VISION"]));
}

#[test]
fn priority_is_used_as_is_when_cost_is_not_preferred() {
    let cheap = profile("cheap", &[], 0.001, 5);
    let expensive = profile("expensive", &[], 2.0, 5);

    assert_eq!(cheap.effective_priority(false), 5);
    assert_eq!(expensive.effective_priority(false), 5);
}

#[test]
fn preferring_cheap_lowers_the_rank_of_costly_profiles() {
    let cheap = profile("cheap", &[], 0.001, 5);
    let expensive = profile("expensive", &[], 2.0, 5);

    assert!(
        cheap.effective_priority(true) > expensive.effective_priority(true),
        "the cheaper profile must sort first"
    );
}

#[test]
fn a_free_profile_keeps_its_priority_even_when_cost_is_preferred() {
    let free = profile("free", &[], 0.0, 3);

    assert_eq!(free.effective_priority(true), 3);
}

#[test]
fn the_default_profile_is_named_default_and_empty() {
    let default = LlmProfile::default();

    assert_eq!(default.id, "default");
    assert!(default.url.is_empty());
    assert!(default.caps.is_empty());
    assert_eq!(default.cost_per_1k, 0.0);
    assert_eq!(default.priority, 0);
}
