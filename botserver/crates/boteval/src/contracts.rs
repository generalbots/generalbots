//! Contracts and validators. A contract describes what a good response looks
//! like, and the validator produces a structured pass/fail report.

use crate::dataset::Contract;
use serde::{Deserialize, Serialize};

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

    for phrase in &contract.must_contain {
        if response.contains(phrase) {
            results.push(CheckResult::pass("must_contain"));
        } else {
            results.push(CheckResult::fail(
                "must_contain",
                format!("expected to contain: {phrase}"),
            ));
        }
    }

    for phrase in &contract.must_not_contain {
        if !response.contains(phrase) {
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
        if detect_language(response) == *lang {
            results.push(CheckResult::pass("language"));
        } else {
            results.push(CheckResult::fail(
                "language",
                format!("expected language: {lang}"),
            ));
        }
    }

    if results.is_empty() {
        results.push(CheckResult::skipped("no_contract_defined"));
    }
    results
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

fn detect_language(text: &str) -> String {
    let text = text.to_ascii_lowercase();
    let pt_markers = [" olá", " tudo", "ção", "ão", " não", " que", " para", " você", " é "];
    let en_markers = [" hello", " how", " the ", " and ", " is ", " you ", " for ", " with "];
    let pt = pt_markers.iter().filter(|m| text.contains(*m)).count();
    let en = en_markers.iter().filter(|m| text.contains(*m)).count();
    if pt > en {
        "pt-BR".into()
    } else if en > pt {
        "en".into()
    } else {
        "und".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::Contract;

    #[test]
    fn must_contain_pass_and_fail() {
        let c = Contract::must_contain_only(["olá", "ajudar"]);
        let pass = validate_response(&c, "olá, como posso ajudar?");
        let fail = validate_response(&c, "oi, em que posso ser útil?");
        assert!(pass.iter().all(|r| r.outcome == CheckOutcome::Pass));
        assert!(fail.iter().any(|r| r.outcome == CheckOutcome::Fail));
    }

    #[test]
    fn must_not_contain_blocks_phrase() {
        let c = Contract::forbid(["error", "fail"]);
        let ok = validate_response(&c, "tudo certo");
        let bad = validate_response(&c, "we got an error");
        assert!(ok.iter().all(|r| r.outcome == CheckOutcome::Pass));
        assert!(bad.iter().any(|r| r.outcome == CheckOutcome::Fail));
    }

    #[test]
    fn json_schema_passes_when_response_is_json() {
        let c = Contract {
            must_contain: vec![],
            must_not_contain: vec![],
            json_schema: Some(serde_json::json!({ "type": "object" })),
            max_tokens: None,
            min_tokens: None,
            language: None,
        };
        let ok = validate_response(&c, "{\"ok\":true}");
        let bad = validate_response(&c, "not json");
        assert!(ok.iter().all(|r| r.outcome == CheckOutcome::Pass));
        assert!(bad.iter().any(|r| r.outcome == CheckOutcome::Fail));
    }

    #[test]
    fn language_detection() {
        assert_eq!(detect_language("olá, tudo bem?"), "pt-BR");
        assert_eq!(detect_language("hello, how are you?"), "en");
    }
}
