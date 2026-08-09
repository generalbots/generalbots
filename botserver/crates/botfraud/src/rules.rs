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
            let threshold = cond.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
            actual
                .and_then(|v| v.as_f64())
                .map(|v| v > threshold)
                .unwrap_or(false)
        }
        "<" => {
            let threshold = cond.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
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
