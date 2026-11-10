//! Offline tests for the heuristic classifier and the LLM reply parser
//! (issue #2072, AI-TASK-01…04). Both helpers are pure: no DB, no LLM.

use super::super::{IntentClassifier, IntentType, ScheduleType};

fn classify(intent: &str) -> (IntentType, f64) {
    let classified = IntentClassifier::classify_heuristic(intent).expect("heuristic never fails");
    (classified.intent_type, classified.confidence)
}

fn parse(response: &str, intent: &str) -> super::super::ClassifiedIntent {
    IntentClassifier::parse_classification_response(response, intent).expect("parse falls back")
}

const SCHEDULE_JSON: &str = r#"{"intent_type":"SCHEDULE","confidence":0.92,"subject":"vendas",
"time_spec":{"type":"DAILY","time":"09:00"},"features":["email"],"suggested_name":"vendas_diario"}"#;

#[test]
fn heuristic_routes_recurring_phrases_to_schedule() {
    for intent in ["fala todo dia que o comercial vendeu", "every day at noon", "a cada 2 horas"] {
        let (intent_type, confidence) = classify(intent);
        assert_eq!(intent_type, IntentType::Schedule, "intent {intent}");
        assert!((confidence - 0.85).abs() < 1e-9, "confidence {confidence}");
    }
}

#[test]
fn heuristic_routes_voice_commands_to_tool() {
    let (intent_type, confidence) = classify("quando eu disser relatório, envie o resumo");
    assert_eq!(intent_type, IntentType::Tool);
    assert!((confidence - 0.80).abs() < 1e-9);
}

#[test]
fn heuristic_routes_creation_requests_to_app_create() {
    let (intent_type, confidence) = classify("crie um crm para o comercial");
    assert_eq!(intent_type, IntentType::AppCreate);
    assert!((confidence - 0.75).abs() < 1e-9);
}

#[test]
fn heuristic_only_asks_for_clarification_when_unknown() {
    let unknown = IntentClassifier::classify_heuristic("blorf wibble").expect("heuristic");
    assert_eq!(unknown.intent_type, IntentType::Unknown);
    assert!((unknown.confidence - 0.30).abs() < 1e-9);
    assert!(unknown.requires_clarification);
    assert!(unknown.clarification_question.is_some());

    let known = IntentClassifier::classify_heuristic("crie um crm").expect("heuristic");
    assert!(!known.requires_clarification);
    assert!(known.clarification_question.is_none());
}

#[test]
fn heuristic_always_suggests_a_name_when_words_exist() {
    for intent in [
        "fala todo dia que o comercial vendeu",
        "quando eu disser relatório",
        "crie um crm",
        "blorf wibble",
    ] {
        let classified = IntentClassifier::classify_heuristic(intent).expect("heuristic");
        let name = classified.suggested_name.expect("name suggested");
        assert!(!name.is_empty(), "intent {intent}");
        assert!(
            name.chars().all(|c| c.is_alphanumeric() || c == '_'),
            "slug {name} must be resource-safe"
        );
    }
}

#[test]
fn parse_accepts_bare_fenced_and_prose_trailed_json() {
    let bare = parse(SCHEDULE_JSON, "fala todo dia");
    for response in [
        format!("```json\n{SCHEDULE_JSON}\n```"),
        format!("```json\n{SCHEDULE_JSON}\n```\nHope this classification helps."),
        format!("Here is the classification:\n{SCHEDULE_JSON}\nDone."),
    ] {
        let parsed = parse(&response, "fala todo dia");
        assert_eq!(parsed.intent_type, bare.intent_type, "response {response}");
        assert_eq!(parsed.confidence, bare.confidence);
        assert_eq!(parsed.entities.subject, bare.entities.subject);
        assert_eq!(parsed.suggested_name, bare.suggested_name);
    }
    assert_eq!(bare.intent_type, IntentType::Schedule);
    assert_eq!(bare.confidence, 0.92);
    assert_eq!(bare.entities.subject.as_deref(), Some("vendas"));
    assert_eq!(bare.entities.features, vec!["email".to_string()]);
    assert_eq!(bare.suggested_name.as_deref(), Some("vendas_diario"));
    assert_eq!(
        bare.entities.time_spec.map(|spec| spec.schedule_type),
        Some(ScheduleType::Daily)
    );
    assert_eq!(bare.original_text, "fala todo dia");
}

#[test]
fn parse_maps_every_time_spec_type() {
    let cases = [
        ("DAILY", ScheduleType::Daily),
        ("WEEKLY", ScheduleType::Weekly),
        ("MONTHLY", ScheduleType::Monthly),
        ("INTERVAL", ScheduleType::Interval),
        ("CRON", ScheduleType::Cron),
        ("YEARLY", ScheduleType::Once),
    ];
    for (raw, expected) in cases {
        let response =
            format!(r#"{{"intent_type":"SCHEDULE","confidence":0.9,"time_spec":{{"type":"{raw}"}}}}"#);
        let parsed = parse(&response, "agende");
        assert_eq!(
            parsed.entities.time_spec.map(|spec| spec.schedule_type),
            Some(expected),
            "type {raw}"
        );
    }
    let absent = parse(r#"{"intent_type":"SCHEDULE","confidence":0.9}"#, "agende");
    assert!(absent.entities.time_spec.is_none());
}

#[test]
fn parse_defaults_missing_optionals() {
    let parsed = parse(r#"{"intent_type":"TODO","confidence":0.5}"#, "me lembre amanhã");
    assert_eq!(parsed.intent_type, IntentType::Todo);
    assert!(parsed.entities.features.is_empty());
    assert!(parsed.entities.tables.is_empty());
    assert!(parsed.entities.trigger_phrases.is_empty());
    assert!(parsed.entities.subject.is_none());
    assert!(parsed.suggested_name.is_none());
    assert!(parsed.alternative_types.is_empty());
    assert!(!parsed.requires_clarification);
    assert!(parsed.clarification_question.is_none());
}

#[test]
fn parse_keeps_braces_inside_strings_and_alternatives() {
    let parsed = parse(
        r#"{"intent_type":"GOAL","confidence":0.6,"subject":"a } brace",
"alternatives":[{"type":"TODO","confidence":0.2,"reason":"maybe"}]}"#,
        "aumentar vendas",
    );
    assert_eq!(parsed.intent_type, IntentType::Goal);
    assert_eq!(parsed.entities.subject.as_deref(), Some("a } brace"));
    assert_eq!(parsed.alternative_types.len(), 1);
    assert_eq!(parsed.alternative_types[0].intent_type, IntentType::Todo);
    assert_eq!(parsed.alternative_types[0].confidence, 0.2);
}

#[test]
fn parse_falls_back_to_the_heuristic_without_error() {
    let parsed = parse("I could not classify that.", "crie um crm");
    assert_eq!(parsed.intent_type, IntentType::AppCreate);
    assert!(!parsed.requires_clarification);
    assert_eq!(parsed.suggested_name.as_deref(), Some("crie_crm"));
}
