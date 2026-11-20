//! Pure segment evaluation and suppression rules (#2537).

use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use super::validate::{SegmentField, SegmentOp, ValidCondition, ValidSegment};

/// CRM statuses that always suppress a recipient, regardless of segment match.
pub const SUPPRESSED_STATUSES: [&str; 5] = [
    "unsubscribed",
    "suppressed",
    "opted_out",
    "bounced",
    "complained",
];

/// Per-channel opt-in flags read from `crm_contacts.custom_fields`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ConsentFlags {
    pub email: Option<bool>,
    pub sms: Option<bool>,
    pub whatsapp: Option<bool>,
}

fn parse_bool(value: &Value) -> Option<bool> {
    match value {
        Value::Bool(flag) => Some(*flag),
        Value::String(raw) => match raw.trim().to_lowercase().as_str() {
            "true" | "1" | "yes" | "y" | "opt_in" | "opted_in" => Some(true),
            "false" | "0" | "no" | "n" | "opt_out" | "opted_out" => Some(false),
            _ => None,
        },
        Value::Number(number) => match number.as_i64() {
            Some(1) => Some(true),
            Some(0) => Some(false),
            _ => None,
        },
        _ => None,
    }
}

/// Reads consent flags from the documented custom-field keys. Both flat
/// (`consent_email`) and nested (`consent.email`) spellings are accepted.
pub fn consent_flags(custom_fields: Option<&Value>) -> ConsentFlags {
    let read = |channel: &str| -> Option<bool> {
        let root = custom_fields?;
        let flat = format!("consent_{channel}");
        let dotted = format!("consent.{channel}");
        let suffixed = format!("{channel}_consent");
        let opt_in = format!("{channel}_opt_in");
        let nested_path = format!("/consent/{channel}");
        [flat, dotted, suffixed, opt_in]
            .iter()
            .find_map(|key| root.get(key).or_else(|| root.pointer(nested_path.as_str())))
            .and_then(parse_bool)
    };
    ConsentFlags {
        email: read("email"),
        sms: read("sms"),
        whatsapp: read("whatsapp"),
    }
}

/// A denormalized contact prepared for evaluation. Built by `store::load_candidates`
/// from data already present in the people/marketing tables.
#[derive(Debug, Clone)]
pub struct SegmentCandidate {
    pub id: Uuid,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub status: String,
    pub company: Option<String>,
    pub tags: Vec<String>,
    pub owner_id: Option<Uuid>,
    pub last_activity: DateTime<Utc>,
    pub deal_value: Option<f64>,
    pub deal_stages: Vec<String>,
    pub list_ids: Vec<Uuid>,
    pub consent: ConsentFlags,
    pub marketing_subscribed: Option<bool>,
}

impl SegmentCandidate {
    /// Human-readable label used in the preview's first matches.
    pub fn display_name(&self) -> String {
        let full = [self.first_name.as_deref(), self.last_name.as_deref()]
            .into_iter()
            .flatten()
            .filter(|part| !part.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if full.is_empty() {
            self.email.clone().unwrap_or_else(|| self.id.to_string())
        } else {
            full
        }
    }
}

fn text_value(value: &Value) -> Option<String> {
    match value {
        Value::String(raw) => Some(raw.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

fn string_values(value: &Value) -> Vec<String> {
    match value {
        Value::Array(items) => items.iter().filter_map(text_value).collect(),
        Value::String(raw) => vec![raw.clone()],
        _ => Vec::new(),
    }
}

fn uuid_value(value: &Value) -> Option<Uuid> {
    text_value(value).and_then(|raw| Uuid::parse_str(raw.trim()).ok())
}

fn number_value(value: &Value) -> Option<f64> {
    value.as_f64().or_else(|| text_value(value).and_then(|raw| raw.parse::<f64>().ok()))
}

fn text_matches(actual: Option<&str>, condition: &ValidCondition) -> bool {
    let needle = text_value(&condition.value).unwrap_or_default().to_lowercase();
    let haystack = actual.map(str::trim).filter(|value| !value.is_empty());
    match condition.op {
        SegmentOp::Eq => haystack.is_some_and(|value| value.to_lowercase() == needle),
        SegmentOp::Ne => haystack.map_or(true, |value| value.to_lowercase() != needle),
        SegmentOp::Contains => haystack.is_some_and(|value| value.to_lowercase().contains(&needle)),
        SegmentOp::In => {
            let candidates: Vec<String> = string_values(&condition.value)
                .into_iter()
                .map(|value| value.to_lowercase())
                .collect();
            haystack.is_some_and(|value| candidates.iter().any(|item| item == &value.to_lowercase()))
        }
        SegmentOp::NotIn => {
            let candidates: Vec<String> = string_values(&condition.value)
                .into_iter()
                .map(|value| value.to_lowercase())
                .collect();
            haystack.map_or(true, |value| !candidates.iter().any(|item| item == &value.to_lowercase()))
        }
        SegmentOp::IsEmpty => haystack.is_none(),
        SegmentOp::IsNotEmpty => haystack.is_some(),
        _ => false,
    }
}

fn tag_matches(tags: &[String], condition: &ValidCondition) -> bool {
    let normalized: Vec<String> = tags.iter().map(|tag| tag.to_lowercase()).collect();
    match condition.op {
        SegmentOp::Contains => {
            let needle = text_value(&condition.value).unwrap_or_default().to_lowercase();
            normalized.iter().any(|tag| tag == &needle)
        }
        SegmentOp::In => string_values(&condition.value)
            .iter()
            .any(|value| normalized.iter().any(|tag| tag == &value.to_lowercase())),
        SegmentOp::IsEmpty => normalized.is_empty(),
        SegmentOp::IsNotEmpty => !normalized.is_empty(),
        _ => false,
    }
}

fn bool_matches(actual: Option<bool>, condition: &ValidCondition) -> bool {
    let target = parse_bool(&condition.value);
    match condition.op {
        SegmentOp::Eq => target.is_some() && actual == target,
        SegmentOp::Ne => target.is_some() && actual != target,
        _ => false,
    }
}

fn number_matches(actual: Option<f64>, condition: &ValidCondition) -> bool {
    let Some(value) = actual else {
        return false;
    };
    let Some(target) = number_value(&condition.value) else {
        return false;
    };
    match condition.op {
        SegmentOp::Eq => (value - target).abs() < f64::EPSILON,
        SegmentOp::Gt => value > target,
        SegmentOp::Gte => value >= target,
        SegmentOp::Lt => value < target,
        SegmentOp::Lte => value <= target,
        _ => false,
    }
}

fn date_matches(actual: &DateTime<Utc>, condition: &ValidCondition) -> bool {
    let Some(raw) = text_value(&condition.value) else {
        return false;
    };
    let Ok(target) = DateTime::parse_from_rfc3339(raw.trim()) else {
        return false;
    };
    let stamp = actual.timestamp();
    match condition.op {
        SegmentOp::Before => stamp < target.timestamp(),
        SegmentOp::After => stamp > target.timestamp(),
        _ => false,
    }
}

fn uuid_matches(actual: Option<Uuid>, condition: &ValidCondition) -> bool {
    match condition.op {
        SegmentOp::Eq => uuid_value(&condition.value).is_some_and(|target| actual == Some(target)),
        SegmentOp::Ne => uuid_value(&condition.value).is_some_and(|target| actual != Some(target)),
        SegmentOp::IsEmpty => actual.is_none(),
        SegmentOp::IsNotEmpty => actual.is_some(),
        _ => false,
    }
}

fn list_matches(ids: &[Uuid], condition: &ValidCondition) -> bool {
    match condition.op {
        SegmentOp::Eq => uuid_value(&condition.value).is_some_and(|target| ids.contains(&target)),
        SegmentOp::In => string_values(&condition.value)
            .iter()
            .filter_map(|raw| Uuid::parse_str(raw.trim()).ok())
            .any(|target| ids.contains(&target)),
        SegmentOp::NotIn => string_values(&condition.value)
            .iter()
            .filter_map(|raw| Uuid::parse_str(raw.trim()).ok())
            .all(|target| !ids.contains(&target)),
        SegmentOp::IsEmpty => ids.is_empty(),
        SegmentOp::IsNotEmpty => !ids.is_empty(),
        _ => false,
    }
}

fn condition_matches(candidate: &SegmentCandidate, condition: &ValidCondition) -> bool {
    match condition.field {
        SegmentField::Status => text_matches(Some(&candidate.status), condition),
        SegmentField::Company => text_matches(candidate.company.as_deref(), condition),
        SegmentField::Tags => tag_matches(&candidate.tags, condition),
        SegmentField::ConsentEmail => bool_matches(candidate.consent.email, condition),
        SegmentField::ConsentSms => bool_matches(candidate.consent.sms, condition),
        SegmentField::ConsentWhatsapp => bool_matches(candidate.consent.whatsapp, condition),
        SegmentField::Owner => uuid_matches(candidate.owner_id, condition),
        SegmentField::DealValue => number_matches(candidate.deal_value, condition),
        SegmentField::DealStage => text_list_or_scalar_matches(&candidate.deal_stages, condition),
        SegmentField::LastActivity => date_matches(&candidate.last_activity, condition),
        SegmentField::ListMembership => list_matches(&candidate.list_ids, condition),
    }
}

fn text_list_or_scalar_matches(values: &[String], condition: &ValidCondition) -> bool {
    match condition.op {
        SegmentOp::Eq | SegmentOp::Contains => {
            values.iter().any(|value| text_matches(Some(value), condition))
        }
        SegmentOp::Ne => {
            let needle = text_value(&condition.value).unwrap_or_default().to_lowercase();
            values.iter().all(|value| value.to_lowercase() != needle)
        }
        SegmentOp::In | SegmentOp::NotIn => {
            let wanted: Vec<String> = string_values(&condition.value)
                .into_iter()
                .map(|value| value.to_lowercase())
                .collect();
            let found = values
                .iter()
                .any(|value| wanted.iter().any(|item| item == &value.to_lowercase()));
            if condition.op == SegmentOp::In {
                found
            } else {
                !found
            }
        }
        SegmentOp::IsEmpty => values.is_empty(),
        SegmentOp::IsNotEmpty => !values.is_empty(),
        _ => false,
    }
}

/// AND/OR evaluation of a validated segment against one candidate.
pub fn candidate_matches(candidate: &SegmentCandidate, segment: &ValidSegment) -> bool {
    if segment.conditions.is_empty() {
        return false;
    }
    match segment.match_mode {
        super::model::SegmentMatch::All => segment
            .conditions
            .iter()
            .all(|condition| condition_matches(candidate, condition)),
        super::model::SegmentMatch::Any => segment
            .conditions
            .iter()
            .any(|condition| condition_matches(candidate, condition)),
    }
}

/// Suppression is the union of a blocked CRM status, an explicit marketing
/// `subscribed = false`, and an opt-out on any consent channel.
pub fn is_suppressed(candidate: &SegmentCandidate) -> bool {
    let status = candidate.status.trim().to_lowercase();
    if SUPPRESSED_STATUSES.contains(&status.as_str()) {
        return true;
    }
    if candidate.marketing_subscribed == Some(false) {
        return true;
    }
    [candidate.consent.email, candidate.consent.sms, candidate.consent.whatsapp]
        .iter()
        .any(|flag| *flag == Some(false))
}
