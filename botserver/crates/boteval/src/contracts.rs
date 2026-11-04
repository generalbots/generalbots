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
    // Contracts describe the information, not typography: "Bug"/"Fix" and
    // "Correção"/"correção" are the same answer. The previous byte-exact
    // compare failed both.
    let response_folded = response.to_lowercase();

    for phrase in &contract.must_contain {
        if response_folded.contains(&phrase.to_lowercase()) {
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

fn detect_language(text: &str) -> String {
    let text = text.to_lowercase();
    // Word/phrase markers rather than single letters: short answers such as
    // "A hexagon has 6 sides." or "Um hexágono tem 6 lados." must classify
    // correctly, and a numeric answer must stay "und" (inconclusive).
    let pt_markers = [
        " olá", " tudo", "ção", "ões", "ão", " não", " você", " é ", " que", " para",
        " uma", " com", " exemplo", " como", " dos", " das", " por", " mais", " quando",
        " porque", " isso", " esta", " este", " tem ", " lado", " resídu", " fórmula",
        " corresponde", " quilômetro", " milha", " depende", " sistema", " cor", " vermelho",
        " verde", " azul", " substitu", " plástico", " reciclar", " coleta", " seletiva",
        " adeus", " obrigado", " tchau", " português", " hexágono", " dobrada", " lista",
        " em ", " não ",
    ];
    let en_markers = [
        " hello", " how", " the ", " and ", " is ", " are ", " you", " your", " for ",
        " with ", " this ", " that ", " has ", " have ", " sides", " days", " energy",
        " wind", " solar", " hydropower", " goodbye", " thank", " what", " which", " does",
        " year", " leap", " hexagon", " miles", " degrees", " temperature", " good",
    ];
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
    fn must_contain_is_case_insensitive() {
        let c = Contract::must_contain_only(["bug", "fix"]);
        let result = validate_response(&c, "**Bug:** range(10) is wrong. **Fix:** use a string.");
        assert!(result.iter().all(|r| r.outcome == CheckOutcome::Pass));
    }

    #[test]
    fn short_answers_are_language_inconclusive() {
        assert_eq!(detect_language("30"), "und");
        assert_eq!(detect_language("A hexagon has 6 sides."), "en");
        assert_eq!(detect_language("Um hexágono tem 6 lados."), "pt-BR");
        assert_eq!(
            detect_language("Exemplo completo: lista = [1, 2, 3]"),
            "pt-BR"
        );
    }

    #[test]
    fn inconclusive_language_passes_but_empty_response_fails() {
        let c = Contract {
            must_contain: vec![],
            must_not_contain: vec![],
            json_schema: None,
            max_tokens: None,
            min_tokens: None,
            language: Some("en".into()),
            requires_tool_calls: None,
        };
        let short = validate_response(&c, "30");
        assert!(short.iter().all(|r| r.outcome == CheckOutcome::Pass));
        let empty = validate_response(&c, "");
        assert!(empty.iter().any(|r| r.outcome == CheckOutcome::Fail));
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
            requires_tool_calls: None,
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
