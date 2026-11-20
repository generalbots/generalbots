//! #2537 — recursive, side-effect-free segment evaluation for the shipped UI.
//!
//! Validates are performed by `ui_wire`; this module evaluates the validated
//! tree against CRM-derived candidates and returns a count plus the first ten
//! non-suppressed matches — without materializing anything.

use chrono::{DateTime, Duration, Utc};
use diesel::PgConnection;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use super::evaluate::{is_suppressed, SegmentCandidate};
use super::store::{load_candidates, SegmentStoreError};
use super::ui_wire::{
    numeric_value, validate_wire, Field, Op, ValidCondition, ValidGroup, ValidNode, WireSegment,
};

fn as_text(value: &Value) -> String {
    match value {
        Value::String(raw) => raw.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        _ => String::new(),
    }
}

fn text_values(value: &Value) -> Vec<String> {
    match value {
        Value::Array(items) => items.iter().map(as_text).collect(),
        Value::String(raw) => raw.split(',').map(|part| part.trim().to_string()).collect(),
        _ => Vec::new(),
    }
}

fn text_matches(actual: Option<&str>, condition: &ValidCondition) -> bool {
    let needle = as_text(&condition.value).trim().to_lowercase();
    let haystack = actual
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_lowercase());
    match condition.op {
        Op::Eq => haystack.as_deref() == Some(needle.as_str()),
        Op::Ne => haystack.as_deref() != Some(needle.as_str()),
        Op::Contains => haystack.is_some_and(|value| value.contains(&needle)),
        Op::NotContains => haystack.is_some_and(|value| !value.contains(&needle)),
        Op::In => {
            let wanted: Vec<String> = text_values(&condition.value)
                .into_iter()
                .map(|value| value.to_lowercase())
                .collect();
            haystack.is_some_and(|value| wanted.iter().any(|item| item == &value))
        }
        Op::NotIn => {
            let wanted: Vec<String> = text_values(&condition.value)
                .into_iter()
                .map(|value| value.to_lowercase())
                .collect();
            haystack.map_or(true, |value| !wanted.iter().any(|item| item == &value))
        }
        _ => false,
    }
}

fn bool_matches(actual: Option<bool>, condition: &ValidCondition) -> bool {
    let target = match as_text(&condition.value).trim().to_lowercase().as_str() {
        "opted_in" | "true" | "1" => Some(true),
        "opted_out" | "false" | "0" => Some(false),
        "unknown" => None,
        _ => return false,
    };
    match condition.op {
        Op::Eq => actual == target,
        Op::Ne => actual != target,
        _ => false,
    }
}

fn number_matches(actual: Option<f64>, condition: &ValidCondition) -> bool {
    let (Some(value), Some(target)) = (actual, numeric_value(&condition.value)) else {
        return false;
    };
    match condition.op {
        Op::Eq => (value - target).abs() < f64::EPSILON,
        Op::Ne => (value - target).abs() >= f64::EPSILON,
        Op::Gt => value > target,
        Op::Gte => value >= target,
        Op::Lt => value < target,
        Op::Lte => value <= target,
        _ => false,
    }
}

fn date_matches(actual: &DateTime<Utc>, condition: &ValidCondition) -> bool {
    match condition.op {
        Op::WithinDays => numeric_value(&condition.value).is_some_and(|days| {
            days > 0.0 && *actual >= Utc::now() - Duration::seconds((days * 86_400.0) as i64)
        }),
        Op::Before | Op::After => {
            let Ok(target) = DateTime::parse_from_rfc3339(as_text(&condition.value).trim()) else {
                return false;
            };
            let stamp = actual.timestamp();
            if condition.op == Op::Before {
                stamp < target.timestamp()
            } else {
                stamp > target.timestamp()
            }
        }
        _ => false,
    }
}

fn list_matches(ids: &[Uuid], condition: &ValidCondition) -> bool {
    let wanted: Vec<Uuid> = text_values(&condition.value)
        .into_iter()
        .filter_map(|raw| Uuid::parse_str(&raw).ok())
        .collect();
    match condition.op {
        Op::In => ids.iter().any(|id| wanted.contains(id)),
        Op::NotIn => ids.iter().all(|id| !wanted.contains(id)),
        _ => false,
    }
}

fn condition_matches(candidate: &SegmentCandidate, condition: &ValidCondition) -> bool {
    match condition.field {
        Field::FirstName => text_matches(candidate.first_name.as_deref(), condition),
        Field::LastName => text_matches(candidate.last_name.as_deref(), condition),
        Field::Email => text_matches(candidate.email.as_deref(), condition),
        Field::Company => text_matches(candidate.company.as_deref(), condition),
        Field::City => text_matches(candidate.city.as_deref(), condition),
        Field::Country => text_matches(candidate.country.as_deref(), condition),
        Field::Tags => text_matches(Some(&candidate.tags.join(",")), condition),
        Field::Owner => text_matches(
            candidate.owner_id.map(|id| id.to_string()).as_deref(),
            condition,
        ),
        Field::LifecycleStatus => text_matches(Some(&candidate.status), condition),
        Field::DealValue => number_matches(candidate.deal_value, condition),
        Field::DealStage => text_matches(Some(&candidate.deal_stages.join(",")), condition),
        Field::LastActivity => date_matches(&candidate.last_activity, condition),
        Field::ConsentEmail => bool_matches(candidate.consent.email, condition),
        Field::ConsentSms => bool_matches(candidate.consent.sms, condition),
        Field::ConsentPush => bool_matches(candidate.consent.push, condition),
        Field::ConsentWhatsapp => bool_matches(candidate.consent.whatsapp, condition),
        Field::ListMembership => list_matches(&candidate.list_ids, condition),
    }
}

fn node_matches(candidate: &SegmentCandidate, node: &ValidNode) -> bool {
    match node {
        ValidNode::Condition(condition) => condition_matches(candidate, condition),
        ValidNode::Group(group) => group_matches(candidate, group),
    }
}

fn group_matches(candidate: &SegmentCandidate, group: &ValidGroup) -> bool {
    if group.all {
        group.children.iter().all(|node| node_matches(candidate, node))
    } else {
        group.children.iter().any(|node| node_matches(candidate, node))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WirePreviewMatch {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WirePreview {
    pub matched: i64,
    pub suppressed: i64,
    pub matches: Vec<WirePreviewMatch>,
}

/// Validates and evaluates a UI segment against CRM contacts in scope.
/// Read-only: no membership rows are written.
pub fn preview_wire_segment(
    conn: &mut PgConnection,
    branch_id: Uuid,
    wire: &WireSegment,
) -> Result<WirePreview, SegmentStoreError> {
    let validated = validate_wire(wire)?;
    let candidates = load_candidates(conn, branch_id)?;
    let mut matched = 0i64;
    let mut suppressed = 0i64;
    let mut matches: Vec<WirePreviewMatch> = Vec::new();
    for candidate in &candidates {
        if !group_matches(candidate, &validated) {
            continue;
        }
        if is_suppressed(candidate) {
            suppressed += 1;
            continue;
        }
        matched += 1;
        if matches.len() < 10 {
            matches.push(WirePreviewMatch {
                id: candidate.id,
                name: candidate.display_name(),
                email: candidate.email.clone(),
                status: candidate.status.clone(),
            });
        }
    }
    Ok(WirePreview {
        matched,
        suppressed,
        matches,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment::model::SegmentError;
    use serde_json::json;
    use uuid::Uuid;

    fn wire_group(match_mode: &str, groups: Vec<crate::segment::ui_wire::WireNode>) -> crate::segment::ui_wire::WireGroup {
        crate::segment::ui_wire::WireGroup {
            match_mode: match_mode.to_string(),
            groups,
        }
    }

    fn condition(field: &str, op: &str, value: Value) -> crate::segment::ui_wire::WireNode {
        crate::segment::ui_wire::WireNode::Condition(crate::segment::ui_wire::WireCondition {
            field: field.to_string(),
            op: op.to_string(),
            value,
        })
    }

    fn sample() -> SegmentCandidate {
        SegmentCandidate {
            id: Uuid::new_v4(),
            first_name: Some("Ada".to_string()),
            last_name: Some("Lovelace".to_string()),
            email: Some("ada@example.com".to_string()),
            status: "customer".to_string(),
            company: Some("Acme".to_string()),
            city: Some("Lisbon".to_string()),
            country: Some("PT".to_string()),
            tags: vec!["vip".to_string()],
            owner_id: None,
            last_activity: Utc::now(),
            deal_value: Some(1500.0),
            deal_stages: vec!["won".to_string()],
            list_ids: Vec::new(),
            consent: Default::default(),
            marketing_subscribed: None,
        }
    }

    #[test]
    fn single_condition_resolves() {
        let group = validate_wire(&WireSegment {
            match_mode: "all".to_string(),
            groups: vec![condition("city", "eq", json!("Lisbon"))],
        })
        .unwrap();
        assert!(group_matches(&sample(), &group));
        let miss = validate_wire(&WireSegment {
            match_mode: "all".to_string(),
            groups: vec![condition("city", "eq", json!("Porto"))],
        })
        .unwrap();
        assert!(!group_matches(&sample(), &miss));
    }

    #[test]
    fn nested_groups_resolve() {
        let nested = crate::segment::ui_wire::WireNode::Group(wire_group(
            "any",
            vec![
                condition("tags", "contains", json!("vip")),
                condition("deal_stage", "eq", json!("lost")),
            ],
        ));
        let group = validate_wire(&WireSegment {
            match_mode: "all".to_string(),
            groups: vec![condition("lifecycle_status", "eq", json!("customer")), nested],
        })
        .unwrap();
        assert!(group_matches(&sample(), &group));
    }

    #[test]
    fn empty_segment_is_rejected() {
        let err = validate_wire(&WireSegment {
            match_mode: "all".to_string(),
            groups: Vec::new(),
        })
        .unwrap_err();
        assert_eq!(err, SegmentError::EmptySegment);
    }

    #[test]
    fn invalid_field_is_rejected() {
        let err = validate_wire(&WireSegment {
            match_mode: "all".to_string(),
            groups: vec![condition("nope", "eq", json!("x"))],
        })
        .unwrap_err();
        assert_eq!(
            err,
            SegmentError::UnknownField {
                field: "nope".to_string()
            }
        );
    }

    #[test]
    fn consent_filter_resolves_unknown_as_unmatched() {
        let group = validate_wire(&WireSegment {
            match_mode: "all".to_string(),
            groups: vec![condition("consent_email", "eq", json!("opted_in"))],
        })
        .unwrap();
        assert!(!group_matches(&sample(), &group));
        let mut opted_in = sample();
        opted_in.consent.email = Some(true);
        assert!(group_matches(&opted_in, &group));
    }

    #[test]
    fn nesting_beyond_limit_is_rejected() {
        let deep = crate::segment::ui_wire::WireNode::Group(wire_group(
            "all",
            vec![crate::segment::ui_wire::WireNode::Group(wire_group(
                "all",
                vec![crate::segment::ui_wire::WireNode::Group(wire_group(
                    "all",
                    vec![condition("status", "eq", json!("customer"))],
                ))],
            ))],
        ));
        let err = validate_wire(&WireSegment {
            match_mode: "all".to_string(),
            groups: vec![deep],
        })
        .unwrap_err();
        assert!(matches!(err, SegmentError::InvalidValue { .. }));
    }
}
