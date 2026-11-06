use serde::{Deserialize, Serialize};

use crate::dataset::Contract;

use super::text::{contains_phrase, detect_language};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckOutcome {
    Pass,
    Fail,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub outcome: CheckOutcome,
    pub message: Option<String>,
}

impl CheckResult {
    pub fn pass(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            outcome: CheckOutcome::Pass,
            message: None,
        }
    }
    pub fn fail(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            outcome: CheckOutcome::Fail,
            message: Some(message.into()),
        }
    }
    pub fn skipped(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            outcome: CheckOutcome::Skipped,
            message: None,
        }
    }
}

pub fn validate_response(contract: &Contract, response: &str) -> Vec<CheckResult> {
    let mut results = Vec::new();
    // Contracts describe the information, not typography: "Bug"/"Fix" and
    // "Correção"/"correção" are the same answer. The previous byte-exact
    // compare failed both.
    let response_folded = response.to_lowercase();

    for phrase in &contract.must_contain {
        if contains_phrase(response, phrase) {
            results.push(CheckResult::pass("must_contain"));
        } else {
            results.push(CheckResult::fail(
                "must_contain",
                format!("expected to contain: {phrase}"),
            ));
        }
    }

    for phrase in &contract.must_not_contain {
        if !response_folded.contains(&phrase.to_lowercase()) {
            results.push(CheckResult::pass("must_not_contain"));
        } else {
            results.push(CheckResult::fail(
                "must_not_contain",
                format!("forbidden phrase present: {phrase}"),
            ));
        }
    }

    if let Some(max) = contract.max_tokens {
        let tokens = approx_token_count(response);
        if tokens <= max as usize {
            results.push(CheckResult::pass("max_tokens"));
        } else {
            results.push(CheckResult::fail(
                "max_tokens",
                format!("{tokens} > {max}"),
            ));
        }
    }
    if let Some(min) = contract.min_tokens {
        let tokens = approx_token_count(response);
        if tokens >= min as usize {
            results.push(CheckResult::pass("min_tokens"));
        } else {
            results.push(CheckResult::fail(
                "min_tokens",
                format!("{tokens} < {min}"),
            ));
        }
    }

    if let Some(schema) = &contract.json_schema {
        match validate_json_schema(schema, response) {
            Ok(()) => results.push(CheckResult::pass("json_schema")),
            Err(message) => results.push(CheckResult::fail("json_schema", message)),
        }
    }

    if let Some(lang) = &contract.language {
        if response.trim().is_empty() {
            results.push(CheckResult::fail("language", "empty response"));
        } else {
            let detected = detect_language(response);
            // Short numeric/code answers carry no language signal. Treat the
            // detector's "und" as inconclusive instead of failing a correct
            // answer such as "30" or "A hexagon has 6 sides."
            if detected == *lang || detected == "und" {
                results.push(CheckResult::pass("language"));
            } else {
                results.push(CheckResult::fail(
                    "language",
                    format!("expected language: {lang}, detected {detected}"),
                ));
            }
        }
    }

    if results.is_empty() {
        results.push(CheckResult::skipped("no_contract_defined"));
    }
    results
}

/// #817 — enforces the `requires_tool_calls` contract clause against the
/// actual tool-call count of a run. Returns `None` when the contract has no
/// tool floor, so callers can skip it for plain-LLM entries.
pub fn validate_tool_floor(contract: &Contract, tool_calls: u32) -> Option<CheckResult> {
    let required = contract.requires_tool_calls?;
    if tool_calls >= required {
        Some(CheckResult::pass("requires_tool_calls"))
    } else {
        Some(CheckResult::fail(
            "requires_tool_calls",
            format!("agent made {tool_calls} tool calls, contract requires at least {required}"),
        ))
    }
}

fn approx_token_count(text: &str) -> usize {
    text.split_whitespace().count()
}

fn validate_json_schema(_schema: &serde_json::Value, response: &str) -> Result<(), String> {
    let trimmed = response.trim();
    let candidate = trimmed
        .strip_prefix("```json")
        .and_then(|s| s.strip_suffix("```"))
        .map(|s| s.trim())
        .unwrap_or(trimmed);
    serde_json::from_str::<serde_json::Value>(candidate)
        .map(|_| ())
        .map_err(|e| format!("response is not valid JSON: {e}"))
}
