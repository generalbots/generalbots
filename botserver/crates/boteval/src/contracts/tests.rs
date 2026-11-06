use super::text::{contains_phrase, detect_language};
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
fn conversational_synonyms_satisfy_the_contract() {
    assert!(contains_phrase(
        "Hello! I'm doing well—thanks for asking. How are you today? Is there anything I can help you with?",
        "glad"
    ));
    assert!(contains_phrase("Posso ajudar com dúvidas e explicações.", "perguntas"));
    assert!(contains_phrase("O balanço mostra ativos e passivos.", "assets"));
}

#[test]
fn separators_inflections_and_numbers_are_equivalent() {
    assert!(contains_phrase("def binary_search(arr, target):", "binary search"));
    assert!(contains_phrase("A hexagon has 6 sides.", "six"));
    assert!(contains_phrase("Um hexágono tem 6 lados.", "seis"));
    assert!(contains_phrase("Hardware é a parte física do computador", "físico"));
    assert!(contains_phrase("use sacolas reutilizáveis", "reutilizar"));
    assert!(contains_phrase("a notação descreve a complexidade", "complexidade"));
    assert!(contains_phrase("[x * 2 for x in numbers]", "n * 2"));
    assert!(!contains_phrase("nothing relevant here", "binary search"));
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
fn must_contain_ignores_capitalization() {
    let c = Contract::must_contain_only(["bug", "fix"]);
    let response = "**The Bug**: `.upper()` on an int. **The Fix**: print(i).";
    let results = validate_response(&c, response);
    assert!(results.iter().all(|r| r.outcome == CheckOutcome::Pass));
}

#[test]
fn must_contain_accepts_fix_as_verb_or_noun() {
    let c = Contract::must_contain_only(["correção"]);
    let response = "O erro está em `i.upper()`. Como corrigir: use `str(i)`.";
    let results = validate_response(&c, response);
    assert!(results.iter().all(|r| r.outcome == CheckOutcome::Pass));
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
