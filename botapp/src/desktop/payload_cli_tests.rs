//! Tests for the --agent-runtime-remove-silent argument matching.

use super::is_agent_runtime_remove_silent;

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn exact_switch_matches() {
    assert!(is_agent_runtime_remove_silent(&args(&[
        r"C:\Program Files\GeneralBots\generalbots.exe",
        "--agent-runtime-remove-silent",
    ])));
}

#[test]
fn wrong_switch_does_not_match() {
    assert!(!is_agent_runtime_remove_silent(&args(&[
        "generalbots.exe",
        "--version",
    ])));
}

#[test]
fn extra_arguments_do_not_match() {
    assert!(!is_agent_runtime_remove_silent(&args(&[
        "generalbots.exe",
        "--agent-runtime-remove-silent",
        "--extra",
    ])));
}

#[test]
fn no_arguments_does_not_match() {
    assert!(!is_agent_runtime_remove_silent(&args(&[
        "generalbots.exe",
    ])));
}

#[test]
fn empty_args_does_not_match() {
    assert!(!is_agent_runtime_remove_silent(&args(&[])));
}

#[test]
fn similar_prefix_does_not_match() {
    assert!(!is_agent_runtime_remove_silent(&args(&[
        "generalbots.exe",
        "--agent-runtime-remove-silent-and-more",
    ])));
}

#[test]
fn case_sensitive() {
    assert!(!is_agent_runtime_remove_silent(&args(&[
        "generalbots.exe",
        "--Agent-Runtime-Remove-Silent",
    ])));
}
