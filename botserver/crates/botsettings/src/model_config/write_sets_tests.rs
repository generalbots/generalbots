//! Table-driven tests for [`super::compute`] — every mode transition and the
//! broken starting state from the defect report.

use std::collections::HashMap;
use super::{compute, is_inconsistent, key_hint, Target, LOCAL_ENGINE_URL};
use crate::model_config::{FREE_ENDPOINT, FREE_MODEL, FREE_PROVIDER};

// ── helpers ─────────────────────────────────────────────────────────────

fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

/// Build the starting state for a given mode.
fn state_for_mode(mode: &str) -> (HashMap<String, String>, HashMap<String, String>) {
    match mode {
        "free" => (
            map(&[("url", FREE_ENDPOINT), ("model", FREE_MODEL), ("provider", FREE_PROVIDER)]),
            map(&[("llm-url", FREE_ENDPOINT), ("llm-model", FREE_MODEL), ("llm-provider", FREE_PROVIDER)]),
        ),
        "local" => (
            map(&[("url", LOCAL_ENGINE_URL), ("model", "Qwen3.5-9B-Q4_K_M.gguf")]),
            map(&[("llm-url", LOCAL_ENGINE_URL), ("llm-model", "Qwen3.5-9B-Q4_K_M.gguf")]),
        ),
        "own_key" => (
            map(&[
                ("url", "https://api.openai.com/v1/chat/completions"),
                ("model", "gpt-4o"),
                ("openai_key", "sk-test"),
                ("provider", "openai"),
            ]),
            map(&[
                ("llm-url", "https://api.openai.com/v1/chat/completions"),
                ("llm-model", "gpt-4o"),
                ("llm-key", "sk-test"),
                ("llm-provider", "openai"),
            ]),
        ),
        _ => panic!("unknown mode"),
    }
}

fn local_target() -> Target {
    Target::Local { file: "Mistral-7B-Q4.gguf".into() }
}

fn free_target() -> Target {
    Target::Free
}

fn own_key_target() -> Target {
    Target::OwnKey {
        endpoint: "https://api.openai.com/v1/chat/completions".into(),
        model: "gpt-4o".into(),
        key: Some("sk-new".into()),
        provider: Some("openai".into()),
    }
}

// ── transition table ────────────────────────────────────────────────────

#[test]
fn free_to_local() {
    let (c_in, pb_in) = state_for_mode("free");
    let (c, pb) = compute(&c_in, &pb_in, &local_target());
    assert_eq!(c.get("url").unwrap(), LOCAL_ENGINE_URL);
    assert_eq!(c.get("model").unwrap(), "Mistral-7B-Q4.gguf");
    assert!(!c.contains_key("openai_key"), "key must be removed for local");
    assert!(!c.contains_key("provider"), "provider must be removed for local");
    assert_eq!(pb.get("llm-url").unwrap(), LOCAL_ENGINE_URL);
    assert_eq!(pb.get("llm-model").unwrap(), "Mistral-7B-Q4.gguf");
    assert!(!pb.contains_key("llm-key"));
    assert!(!pb.contains_key("llm-provider"));
}

#[test]
fn free_to_own_key() {
    let (c_in, pb_in) = state_for_mode("free");
    let (c, pb) = compute(&c_in, &pb_in, &own_key_target());
    assert_eq!(c.get("url").unwrap(), "https://api.openai.com/v1/chat/completions");
    assert_eq!(c.get("model").unwrap(), "gpt-4o");
    assert_eq!(c.get("openai_key").unwrap(), "sk-new");
    assert_eq!(c.get("provider").unwrap(), "openai");
    assert_eq!(pb.get("llm-url").unwrap(), "https://api.openai.com/v1/chat/completions");
    assert_eq!(pb.get("llm-model").unwrap(), "gpt-4o");
    assert_eq!(pb.get("llm-key").unwrap(), "sk-new");
    assert_eq!(pb.get("llm-provider").unwrap(), "openai");
}

#[test]
fn local_to_free() {
    let (c_in, pb_in) = state_for_mode("local");
    let (c, pb) = compute(&c_in, &pb_in, &free_target());
    assert_eq!(c.get("url").unwrap(), FREE_ENDPOINT);
    assert_eq!(c.get("model").unwrap(), FREE_MODEL);
    assert_eq!(c.get("provider").unwrap(), FREE_PROVIDER);
    assert!(!c.contains_key("openai_key"));
    assert_eq!(pb.get("llm-url").unwrap(), FREE_ENDPOINT);
    assert_eq!(pb.get("llm-model").unwrap(), FREE_MODEL);
    assert!(!pb.contains_key("llm-key"));
    assert_eq!(pb.get("llm-provider").unwrap(), FREE_PROVIDER);
}

#[test]
fn local_to_own_key() {
    let (c_in, pb_in) = state_for_mode("local");
    let (c, pb) = compute(&c_in, &pb_in, &own_key_target());
    assert_eq!(c.get("url").unwrap(), "https://api.openai.com/v1/chat/completions");
    assert_eq!(c.get("openai_key").unwrap(), "sk-new");
    assert_eq!(pb.get("llm-key").unwrap(), "sk-new");
}

#[test]
fn own_key_to_free() {
    let (c_in, pb_in) = state_for_mode("own_key");
    let (c, pb) = compute(&c_in, &pb_in, &free_target());
    assert_eq!(c.get("url").unwrap(), FREE_ENDPOINT);
    assert!(!c.contains_key("openai_key"), "key must be cleared on free");
    assert!(!pb.contains_key("llm-key"), "per-bot key must be cleared on free");
}

#[test]
fn own_key_to_local() {
    let (c_in, pb_in) = state_for_mode("own_key");
    let (c, pb) = compute(&c_in, &pb_in, &local_target());
    assert_eq!(c.get("url").unwrap(), LOCAL_ENGINE_URL);
    assert!(!c.contains_key("openai_key"), "key must be removed for local");
    assert!(!c.contains_key("provider"), "provider must be removed for local");
    assert!(!pb.contains_key("llm-key"));
    assert!(!pb.contains_key("llm-provider"));
}

// ── broken starting state from the defect report ────────────────────────

/// The defect: central had hosted free URL + local model file; per-bot had
/// llm-model + stale llm-url + stale llm-key. All three targets must fix it.
fn broken_state() -> (HashMap<String, String>, HashMap<String, String>) {
    (
        map(&[
            ("url", FREE_ENDPOINT),
            ("model", "Qwen3.5-9B-Q4_K_M.gguf"),
            ("provider", "kilo"),
        ]),
        map(&[
            ("llm-model", "Qwen3.5-9B-Q4_K_M.gguf"),
            ("llm-url", "https://api.example.invalid/v1/chat/completions"),
            ("llm-key", "sk-leftover"),
        ]),
    )
}

#[test]
fn broken_to_free() {
    let (c_in, pb_in) = broken_state();
    let (c, pb) = compute(&c_in, &pb_in, &free_target());
    assert_eq!(c.get("url").unwrap(), FREE_ENDPOINT);
    assert_eq!(c.get("model").unwrap(), FREE_MODEL);
    assert!(!c.contains_key("openai_key"));
    assert_eq!(pb.get("llm-url").unwrap(), FREE_ENDPOINT);
    assert_eq!(pb.get("llm-model").unwrap(), FREE_MODEL);
    assert!(!pb.contains_key("llm-key"), "stale key must be removed");
}

#[test]
fn broken_to_local() {
    let (c_in, pb_in) = broken_state();
    let target = Target::Local { file: "Qwen3.5-9B-Q4_K_M.gguf".into() };
    let (c, pb) = compute(&c_in, &pb_in, &target);
    assert_eq!(c.get("url").unwrap(), LOCAL_ENGINE_URL);
    assert_eq!(c.get("model").unwrap(), "Qwen3.5-9B-Q4_K_M.gguf");
    assert!(!c.contains_key("openai_key"));
    assert!(!c.contains_key("provider"), "hosted provider must be removed");
    assert_eq!(pb.get("llm-url").unwrap(), LOCAL_ENGINE_URL, "stale hosted URL must be overwritten");
    assert!(!pb.contains_key("llm-key"), "stale key must be removed");
}

#[test]
fn broken_to_own_key() {
    let (c_in, pb_in) = broken_state();
    let (c, pb) = compute(&c_in, &pb_in, &own_key_target());
    assert_eq!(c.get("url").unwrap(), "https://api.openai.com/v1/chat/completions");
    assert_eq!(c.get("model").unwrap(), "gpt-4o");
    assert_eq!(c.get("openai_key").unwrap(), "sk-new");
    assert_eq!(pb.get("llm-url").unwrap(), "https://api.openai.com/v1/chat/completions");
    assert_eq!(pb.get("llm-key").unwrap(), "sk-new");
}

// ── unrelated keys are preserved ────────────────────────────────────────

#[test]
fn preserves_unrelated_keys() {
    let c_in = map(&[("url", FREE_ENDPOINT), ("model", FREE_MODEL), ("custom-thing", "keep")]);
    let pb_in = map(&[("llm-model", FREE_MODEL), ("other-setting", "preserved")]);
    let (c, pb) = compute(&c_in, &pb_in, &local_target());
    assert_eq!(c.get("custom-thing").unwrap(), "keep");
    assert_eq!(pb.get("other-setting").unwrap(), "preserved");
}

// ── inconsistency detection ─────────────────────────────────────────────

#[test]
fn detects_inconsistent_state() {
    // Hosted URL + local .gguf model → inconsistent
    assert!(is_inconsistent(FREE_ENDPOINT, "Qwen3.5-9B-Q4_K_M.gguf"));
    // Loopback URL + hosted model → inconsistent
    assert!(is_inconsistent("http://localhost:8081/v1", "dots-studio/dots-3-note-preview:free"));
    // Consistent states
    assert!(!is_inconsistent(FREE_ENDPOINT, FREE_MODEL));
    assert!(!is_inconsistent(LOCAL_ENGINE_URL, "model.gguf"));
    assert!(!is_inconsistent("https://api.openai.com/v1", "gpt-4o"));
    assert!(!is_inconsistent("", ""));
}

#[test]
fn own_key_without_a_typed_key_keeps_the_stored_one_for_the_same_host() {
    let mut central = HashMap::new();
    central.insert("url".to_string(), "https://api.anthropic.com/v1".to_string());
    central.insert("openai_key".to_string(), "sk-ant-stored".to_string());
    let same = Target::OwnKey {
        endpoint: "https://api.anthropic.com/v1".into(),
        model: "claude-opus-5-5".into(),
        key: None,
        provider: None,
    };
    let (c, b) = compute(&central, &HashMap::new(), &same);
    assert_eq!(c.get("openai_key").map(String::as_str), Some("sk-ant-stored"));
    assert_eq!(b.get("llm-key").map(String::as_str), Some("sk-ant-stored"));
    let other = Target::OwnKey {
        endpoint: "https://api.openai.com/v1".into(),
        model: "gpt-5".into(),
        key: None,
        provider: None,
    };
    let (c, b) = compute(&central, &HashMap::new(), &other);
    assert!(c.get("openai_key").is_none());
    assert!(b.get("llm-key").is_none());
}

#[test]
fn key_hint_shows_only_the_last_four_of_a_long_key() {
    assert_eq!(key_hint("sk-ant-api03-abcdefghWXYZ"), "WXYZ");
    assert_eq!(key_hint("short-key"), "");
    assert_eq!(key_hint(""), "");
}
