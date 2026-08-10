//! BASIC-only generation pipeline (#754/#755): turns a classified intent
//! into a single `.bas` script designed to persist on the bot's Drive bucket
//! (`{name}.gbai/{name}.gbdialog/{folder}/{file}.bas`) and be compiled by the
//! DriveMonitor — no AppGenerator, no local filesystem writes.

use crate::intent_classifier::{ClassifiedIntent, IntentType};
use crate::intent_compiler::CompiledIntent;

/// Cron expression chosen from the classification or a sane default
/// (09:00 daily). No LLM required.
pub fn schedule_expression(classification: &ClassifiedIntent) -> String {
    if let Some(ts) = classification.entities.time_spec.as_ref() {
        if let Some(ref cron) = ts.cron_expression {
            if !cron.is_empty() {
                return cron.clone();
            }
        }
    }
    "0 9 * * *".to_string()
}

/// Relative Drive path (`{folder}/{file}.bas`) for a classified intent.
pub fn script_path_for_intent(classification: &ClassifiedIntent) -> String {
    let folder = match classification.intent_type {
        IntentType::Schedule => "schedulers",
        IntentType::Monitor => "events",
        IntentType::Tool => "tools",
        IntentType::Goal => "goals",
        _ => "tools",
    };
    let name = classification
        .suggested_name
        .as_deref()
        .unwrap_or("autotask_script")
        .trim()
        .replace(' ', "_");
    let safe: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    let file = if safe.is_empty() { "autotask_script".to_string() } else { safe };
    format!("{folder}/{file}.bas")
}

/// The BASIC source that will be persisted and compiled (never null).
pub fn script_body(
    classification: &ClassifiedIntent,
    compiled: &CompiledIntent,
    fallback: &str,
) -> String {
    let body = compiled
        .basic_program
        .clone()
        .filter(|p| !p.trim().is_empty())
        .unwrap_or_else(|| fallback.to_string());
    match classification.intent_type {
        IntentType::Schedule => {
            let expr = schedule_expression(classification);
            format!("' Scheduler: {expr}\nSET SCHEDULE \"{expr}\"\n{body}\n")
        }
        IntentType::Monitor => {
            let subject = classification
                .entities
                .subject
                .clone()
                .unwrap_or_else(|| "data".to_string());
            format!(
                "' Monitor: {subject}\nON CHANGE \"{subject}\"\nTALK \"Alert: {subject} state changed\"\nEND ON\n{body}"
            )
        }
        IntentType::Tool => {
            let triggers = if classification.entities.trigger_phrases.is_empty() {
                vec![classification.suggested_name.clone().unwrap_or_default()]
            } else {
                classification.entities.trigger_phrases.clone()
            };
            let t = triggers
                .iter()
                .filter(|s| !s.trim().is_empty())
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<_>>()
                .join(", ");
            format!("' Tool triggers: {t}\n{body}")
        }
        IntentType::Goal => {
            format!(
                "' Goal: {}\nTALK \"Goal remains in execution\"\n{body}",
                classification
                    .entities
                    .target_value
                    .clone()
                    .unwrap_or_else(|| "goal".to_string())
            )
        }
        _ => body,
    }
}

/// Fallback BASIC source when the compiler has none (defensive; the compiler
/// already guarantees a program, but keep this as last resort).
pub fn fallback_script(intent: &str, name: &str) -> String {
    format!(
        "' AutoTask script: {name}\n' Intent: {intent}\nTALK \"Running: {intent}\"\n"
    )
}

/// Builds the (relative path, body) pair for a classification/compilation.
pub fn script_for(
    classification: &ClassifiedIntent,
    compiled: Option<&CompiledIntent>,
) -> (String, String) {
    let path = script_path_for_intent(classification);
    let fallback = fallback_script(
        &classification.original_text,
        &classification.suggested_name.clone().unwrap_or_default(),
    );
    let body = match compiled {
        Some(c) => script_body(classification, c, &fallback),
        None => fallback,
    };
    (path, body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intent_classifier::{ClassifiedIntent, IntentType};
    use chrono::Utc;
    use uuid::Uuid;

    fn intent(text: &str, kind: IntentType, name: &str) -> ClassifiedIntent {
        ClassifiedIntent {
            id: Uuid::new_v4().to_string(),
            original_text: text.to_string(),
            intent_type: kind,
            confidence: 0.85,
            entities: Default::default(),
            suggested_name: Some(name.to_string()),
            requires_clarification: false,
            clarification_question: None,
            alternative_types: Vec::new(),
            classified_at: Utc::now(),
        }
    }

    fn sample(text: &str, kind: IntentType, name: &str) -> ClassifiedIntent {
        intent(text, kind, name)
    }

    #[test]
    fn test_schedule_path_and_body_contain_set_schedule() {
        let c = sample(
            "fala todo dia que o comercial vendeu",
            IntentType::Schedule,
            "comercial_vendeu_diario",
        );
        let compiled = crate::intent_compiler::CompiledIntent {
            id: Uuid::new_v4().to_string(),
            intent_type: IntentType::Schedule,
            plan_name: "comercial_vendeu_diario".into(),
            plan_description: "daily report".into(),
            steps: Vec::new(),
            alternatives: Vec::new(),
            confidence: 0.85,
            risk_level: "low".into(),
            estimated_duration_minutes: 5,
            estimated_cost: 0.0,
            resource_estimate: crate::intent_compiler::ResourceEstimate {
                compute_hours: 0.0,
                storage_gb: 0.0,
                api_calls: 0,
                llm_tokens: 0,
                estimated_cost_usd: 0.0,
            },
            basic_program: Some("TALK \"relatorio gerado\"".into()),
            requires_approval: false,
            mcp_servers: Vec::new(),
            external_apis: Vec::new(),
            risks: Vec::new(),
        };
        let (path, body) = script_for(&c, Some(&compiled));
        assert!(path.contains("schedulers/"));
        assert!(!path.contains(' '));
        assert!(body.contains("SET SCHEDULE"));
        assert!(body.contains("TALK \"relatorio gerado\""));
    }

    #[test]
    fn test_tool_path_uses_tools_folder() {
        let c = sample("quando eu disser help", IntentType::Tool, "help");
        let (path, _) = script_for(&c, None);
        assert!(path.starts_with("tools/"));
    }

    #[test]
    fn test_schedule_expression_defaults_to_daily_nine() {
        let c = sample("todo dia", IntentType::Schedule, "x");
        assert_eq!(schedule_expression(&c), "0 9 * * *");
    }

    #[test]
    fn test_script_path_slugifies_pt_name() {
        let c = sample("todo dia", IntentType::Schedule, "Relatório de Vendas");
        let (path, _) = script_for(&c, None);
        assert!(path.ends_with("Relatório_de_Vendas.bas"));
    }
}