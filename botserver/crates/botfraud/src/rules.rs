use crate::types::{FraudAssessmentRequest, FraudRule};
use diesel::prelude::*;

pub fn load_active_rules(conn: &mut PgConnection, branch: uuid::Uuid) -> Vec<FraudRule> {
    diesel::sql_query(
        "SELECT id, branch_id, name, description, rule_type, condition_json, \
         action, severity, is_active, created_at \
         FROM fraud_rules WHERE is_active = true AND branch_id = $1 ORDER BY severity DESC",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch)
    .load::<RuleDbRow>(conn)
    .unwrap_or_default()
    .into_iter()
    .map(|r| FraudRule {
        id: r.id,
        branch_id: r.branch_id,
        name: r.name,
        description: r.description,
        rule_type: r.rule_type,
        condition_json: r.condition_json,
        action: r.action,
        severity: r.severity,
        is_active: r.is_active,
        created_at: r.created_at,
    })
    .collect()
}

pub fn evaluate_rule(rule: &FraudRule, request: &FraudAssessmentRequest) -> bool {
    let cond = &rule.condition_json;

    let field = match cond.get("field").and_then(|v| v.as_str()) {
        Some(f) => f,
        None => return false,
    };
    let op = match cond.get("op").and_then(|v| v.as_str()) {
        Some(o) => o,
        None => return false,
    };

    let actual = request.details.get(field);

    match op {
        ">" => {
            // A malformed threshold must not silently become 0.0: that turned
            // `amount > "not-a-number"` into "any positive amount", firing a
            // rule whose condition was never actually expressed.
            let Some(threshold) = cond.get("value").and_then(|v| v.as_f64()) else {
                return false;
            };
            actual
                .and_then(|v| v.as_f64())
                .map(|v| v > threshold)
                .unwrap_or(false)
        }
        "<" => {
            let Some(threshold) = cond.get("value").and_then(|v| v.as_f64()) else {
                return false;
            };
            actual
                .and_then(|v| v.as_f64())
                .map(|v| v < threshold)
                .unwrap_or(false)
        }
        "==" => {
            let expected = cond.get("value");
            actual.is_some_and(|v| v == expected.unwrap_or(&serde_json::Value::Null))
        }
        "!=" => {
            let expected = cond.get("value");
            actual.is_some_and(|v| v != expected.unwrap_or(&serde_json::Value::Null))
        }
        "in" => {
            let list = cond.get("value").and_then(|v| v.as_array());
            match (actual, list) {
                (Some(val), Some(arr)) => arr.contains(val),
                _ => false,
            }
        }
        "contains" => {
            let substr = cond.get("value").and_then(|v| v.as_str()).unwrap_or("");
            actual
                .and_then(|v| v.as_str())
                .map(|s| s.contains(substr))
                .unwrap_or(false)
        }
        _ => false,
    }
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct RuleDbRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    rule_type: String,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    condition_json: serde_json::Value,
    #[diesel(sql_type = diesel::sql_types::Text)]
    action: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    severity: String,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_active: bool,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod rule_evaluation_tests {
    use super::*;
    use crate::types::{FraudAssessmentRequest, FraudRule};
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;

    fn rule(condition: serde_json::Value) -> FraudRule {
        FraudRule {
            id: Uuid::new_v4(),
            branch_id: Uuid::nil(),
            name: "test".to_string(),
            description: None,
            rule_type: "threshold".to_string(),
            condition_json: condition,
            action: "flag".to_string(),
            severity: "high".to_string(),
            is_active: true,
            created_at: Utc::now(),
        }
    }

    fn request(details: serde_json::Value) -> FraudAssessmentRequest {
        FraudAssessmentRequest {
            event_type: "payment".to_string(),
            entity_type: "order".to_string(),
            entity_id: Uuid::new_v4(),
            details,
            branch_id: None,
        }
    }

    #[test]
    fn numeric_comparisons_respect_the_threshold() {
        let amount = request(json!({"amount": 500}));
        assert!(evaluate_rule(&rule(json!({"field":"amount","op":">","value":100})), &amount));
        assert!(!evaluate_rule(&rule(json!({"field":"amount","op":">","value":500})), &amount));
        assert!(evaluate_rule(&rule(json!({"field":"amount","op":"<","value":1000})), &amount));
        assert!(!evaluate_rule(&rule(json!({"field":"amount","op":"<","value":500})), &amount));
    }

    #[test]
    fn equality_handles_strings_and_numbers() {
        let details = request(json!({"country": "BR", "attempts": 3}));
        assert!(evaluate_rule(&rule(json!({"field":"country","op":"==","value":"BR"})), &details));
        assert!(!evaluate_rule(&rule(json!({"field":"country","op":"==","value":"US"})), &details));
        assert!(evaluate_rule(&rule(json!({"field":"country","op":"!=","value":"US"})), &details));
        assert!(evaluate_rule(&rule(json!({"field":"attempts","op":"==","value":3})), &details));
    }

    #[test]
    fn membership_and_substring_operators_work() {
        let details = request(json!({"country": "BR", "email": "a@fraud.example"}));
        assert!(evaluate_rule(
            &rule(json!({"field":"country","op":"in","value":["BR","AR"]})),
            &details
        ));
        assert!(!evaluate_rule(
            &rule(json!({"field":"country","op":"in","value":["US"]})),
            &details
        ));
        assert!(evaluate_rule(
            &rule(json!({"field":"email","op":"contains","value":"fraud"})),
            &details
        ));
        assert!(!evaluate_rule(
            &rule(json!({"field":"email","op":"contains","value":"trusted"})),
            &details
        ));
    }

    #[test]
    fn a_missing_field_never_matches() {
        let details = request(json!({"amount": 500}));
        for condition in [
            json!({"field":"nope","op":">","value":1}),
            json!({"field":"nope","op":"<","value":1}),
            json!({"field":"nope","op":"==","value":1}),
            json!({"field":"nope","op":"!=","value":1}),
            json!({"field":"nope","op":"in","value":[1]}),
            json!({"field":"nope","op":"contains","value":"x"}),
        ] {
            assert!(
                !evaluate_rule(&rule(condition.clone()), &details),
                "{condition} must not match a missing field"
            );
        }
    }

    #[test]
    fn a_malformed_condition_is_false_not_a_panic() {
        let details = request(json!({"amount": 500}));
        for condition in [
            json!({"op": ">", "value": 1}),
            json!({"field": "amount"}),
            json!({"field": "amount", "op": "unknown-op", "value": 1}),
            json!({"field": "amount", "op": ">", "value": "not-a-number"}),
            json!({}),
        ] {
            assert!(
                !evaluate_rule(&rule(condition.clone()), &details),
                "{condition} must be false"
            );
        }
    }

    #[test]
    fn a_string_field_is_not_compared_numerically() {
        let details = request(json!({"amount": "500"}));
        assert!(!evaluate_rule(
            &rule(json!({"field":"amount","op":">","value":100})),
            &details
        ));
    }
}
