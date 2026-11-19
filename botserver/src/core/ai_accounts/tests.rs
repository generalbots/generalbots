use super::*;

use super::{choose_llm, normalize_provider, ResolvedLlm};

#[test]
fn user_account_wins_over_bot_and_global() {
    let resolved = choose_llm(
        Some(("claude-code", "sonnet")),
        Some(("openai", "gpt-4o")),
        Some(("kilo", "dots")),
    )
    .expect("resolved");
    assert_eq!(
        resolved,
        ResolvedLlm {
            provider: "claude-code".into(),
            model: "sonnet".into(),
            source: "user"
        }
    );
}

#[test]
fn bot_config_wins_when_the_user_has_no_account() {
    let resolved =
        choose_llm(None, Some(("openai", "gpt-4o")), Some(("kilo", "dots"))).expect("resolved");
    assert_eq!(resolved.source, "bot");
    assert_eq!(resolved.provider, "openai");
}

#[test]
fn global_is_the_last_resort_and_empty_values_are_skipped() {
    let resolved = choose_llm(None, Some(("", "")), Some(("kilo", "dots"))).expect("resolved");
    assert_eq!(resolved.source, "global");
    assert!(choose_llm(None, None, None).is_none());
}

#[test]
fn provider_aliases_are_canonicalized() {
    assert_eq!(normalize_provider("Claude"), Some("claude-code"));
    assert_eq!(normalize_provider("chatgpt"), Some("chatgpt-plan"));
    assert_eq!(normalize_provider("openai-plan"), Some("chatgpt-plan"));
    assert_eq!(normalize_provider("codex"), Some("codex-cli"));
    assert_eq!(normalize_provider("codex-cli"), Some("codex-cli"));
    assert_eq!(normalize_provider("antigravity"), Some("antigravity"));
    assert_eq!(normalize_provider("kilo"), None);
}

#[test]
fn a_subscription_without_a_linked_account_is_an_error() {
    let error = super::choose_for_request(
        Some("claude-code"),
        None,
        Some(("openai", "gpt-4o")),
        Some(("kilo", "dots")),
    )
    .expect_err("must not fall back");
    assert!(error.contains("Connect your Claude Code account"));
}

#[test]
fn a_linked_account_wins_and_keeps_the_bot_model() {
    let account = super::LinkedAccount {
        provider: "chatgpt-plan".to_string(),
        account_email: Some("user@example.com".to_string()),
        plan: Some("plus".to_string()),
        vm_name: None,
        vault_path: Some("gbo/users/u/chatgpt-plan".to_string()),
    };
    let resolved = super::choose_for_request(
        Some("chatgpt-plan"),
        Some(&account),
        Some(("chatgpt-plan", "gpt-5-codex")),
        Some(("kilo", "dots")),
    )
    .expect("resolved")
    .expect("some");
    assert_eq!(resolved.provider, "chatgpt-plan");
    assert_eq!(resolved.model, "gpt-5-codex");
    assert_eq!(resolved.source, "user");
}

#[test]
fn default_models_are_provider_specific() {
    assert_eq!(super::default_model("claude"), "claude-sonnet-4-5");
    assert_eq!(super::default_model("chatgpt"), "gpt-5-codex");
    assert!(super::is_subscription_provider("claude-code"));
    assert!(!super::is_subscription_provider("openai"));
}
