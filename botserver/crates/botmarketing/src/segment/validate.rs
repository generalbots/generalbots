//! Segment validation, legacy-query compilation and drift fingerprinting.

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::model::{
    DriftReport, FieldSpec, Segment, SegmentCondition, SegmentError, SegmentMatch, SegmentMeta,
    ValueKind, FIELD_CATALOG,
};

/// Internal, validated field identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentField {
    Status,
    Company,
    Tags,
    ConsentEmail,
    ConsentSms,
    ConsentWhatsapp,
    Owner,
    DealValue,
    DealStage,
    LastActivity,
    ListMembership,
}

impl SegmentField {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Company => "company",
            Self::Tags => "tags",
            Self::ConsentEmail => "consent_email",
            Self::ConsentSms => "consent_sms",
            Self::ConsentWhatsapp => "consent_whatsapp",
            Self::Owner => "owner",
            Self::DealValue => "deal_value",
            Self::DealStage => "deal_stage",
            Self::LastActivity => "last_activity",
            Self::ListMembership => "list_membership",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        FIELD_CATALOG
            .iter()
            .find(|spec| spec.field == raw)
            .and_then(|spec| match spec.field {
                "status" => Some(Self::Status),
                "company" => Some(Self::Company),
                "tags" => Some(Self::Tags),
                "consent_email" => Some(Self::ConsentEmail),
                "consent_sms" => Some(Self::ConsentSms),
                "consent_whatsapp" => Some(Self::ConsentWhatsapp),
                "owner" => Some(Self::Owner),
                "deal_value" => Some(Self::DealValue),
                "deal_stage" => Some(Self::DealStage),
                "last_activity" => Some(Self::LastActivity),
                "list_membership" => Some(Self::ListMembership),
                _ => None,
            })
    }

    pub fn spec(self) -> &'static FieldSpec {
        FIELD_CATALOG
            .iter()
            .find(|spec| spec.field == self.as_str())
            .unwrap_or(&FIELD_CATALOG[0])
    }

    pub fn value_kind(self) -> ValueKind {
        match self.spec().value_kind {
            "text" => ValueKind::Text,
            "text_list" => ValueKind::TextList,
            "bool" => ValueKind::Bool,
            "number" => ValueKind::Number,
            "date" => ValueKind::Date,
            "uuid" => ValueKind::Uuid,
            "uuid_list" => ValueKind::UuidList,
            _ => ValueKind::Text,
        }
    }
}

/// Internal, validated operator identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentOp {
    Eq,
    Ne,
    Contains,
    In,
    NotIn,
    Gt,
    Gte,
    Lt,
    Lte,
    Before,
    After,
    IsEmpty,
    IsNotEmpty,
}

impl SegmentOp {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Eq => "eq",
            Self::Ne => "ne",
            Self::Contains => "contains",
            Self::In => "in",
            Self::NotIn => "not_in",
            Self::Gt => "gt",
            Self::Gte => "gte",
            Self::Lt => "lt",
            Self::Lte => "lte",
            Self::Before => "before",
            Self::After => "after",
            Self::IsEmpty => "is_empty",
            Self::IsNotEmpty => "is_not_empty",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "eq" => Some(Self::Eq),
            "ne" => Some(Self::Ne),
            "contains" => Some(Self::Contains),
            "in" => Some(Self::In),
            "not_in" => Some(Self::NotIn),
            "gt" => Some(Self::Gt),
            "gte" => Some(Self::Gte),
            "lt" => Some(Self::Lt),
            "lte" => Some(Self::Lte),
            "before" => Some(Self::Before),
            "after" => Some(Self::After),
            "is_empty" => Some(Self::IsEmpty),
            "is_not_empty" => Some(Self::IsNotEmpty),
            _ => None,
        }
    }
}

/// A condition after validation, with typed field and operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidCondition {
    pub field: SegmentField,
    pub op: SegmentOp,
    pub value: Value,
}

/// A segment after validation; the evaluator only accepts this form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidSegment {
    pub match_mode: SegmentMatch,
    pub conditions: Vec<ValidCondition>,
}

fn all_strings(value: &Value) -> bool {
    value.as_array().is_some_and(|items| items.iter().all(Value::is_string))
}

fn value_is_valid(kind: ValueKind, op: SegmentOp, value: &Value) -> bool {
    match op {
        SegmentOp::IsEmpty | SegmentOp::IsNotEmpty => true,
        SegmentOp::In | SegmentOp::NotIn => all_strings(value),
        _ => match kind {
            ValueKind::Text | ValueKind::TextList | ValueKind::Date | ValueKind::Uuid => {
                value.is_string()
            }
            ValueKind::UuidList => all_strings(value),
            ValueKind::Bool => value.is_boolean(),
            ValueKind::Number => value.as_f64().is_some(),
        },
    }
}

/// Validates every condition against the documented catalog. Unknown fields
/// and operators are rejected with typed, machine-readable errors.
pub fn parse_segment(segment: &Segment) -> Result<ValidSegment, SegmentError> {
    if segment.groups.is_empty() {
        return Err(SegmentError::EmptySegment);
    }
    let mut conditions = Vec::with_capacity(segment.groups.len());
    for group in &segment.groups {
        let field = SegmentField::parse(&group.field).ok_or_else(|| SegmentError::UnknownField {
            field: group.field.clone(),
        })?;
        let op = SegmentOp::parse(&group.op).ok_or_else(|| SegmentError::UnknownOp {
            op: group.op.clone(),
        })?;
        let spec = field.spec();
        if !spec.ops.contains(&op.as_str()) {
            return Err(SegmentError::OpNotAllowed {
                field: group.field.clone(),
                op: group.op.clone(),
            });
        }
        let kind = field.value_kind();
        if !value_is_valid(kind, op, &group.value) {
            return Err(SegmentError::InvalidValue {
                field: group.field.clone(),
                expected: kind.as_str(),
            });
        }
        conditions.push(ValidCondition {
            field,
            op,
            value: group.value.clone(),
        });
    }
    Ok(ValidSegment {
        match_mode: segment.match_mode,
        conditions,
    })
}

/// Compiles the legacy `status=…` / `company=…` text syntax into the
/// structured model so existing dynamic lists keep working. Free text has no
/// name/email field in the catalog, so it is preserved as an approximate
/// case-insensitive company substring condition.
pub fn compile_legacy_query(query_text: &str) -> Segment {
    let lowered = query_text.to_lowercase();
    let mut groups: Vec<SegmentCondition> = Vec::new();
    for (prefix, field, op) in [("status=", "status", "eq"), ("company=", "company", "contains")] {
        if let Some(start) = lowered.find(prefix) {
            let remainder = &query_text[start + prefix.len()..];
            let value = remainder
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_matches(|c| c == '"' || c == '\'');
            if !value.is_empty() {
                groups.push(SegmentCondition {
                    field: field.to_string(),
                    op: op.to_string(),
                    value: Value::String(value.to_string()),
                });
            }
        }
    }
    if groups.is_empty() {
        let term = query_text.trim();
        if !term.is_empty() {
            groups.push(SegmentCondition {
                field: "company".to_string(),
                op: "contains".to_string(),
                value: Value::String(term.to_string()),
            });
        }
    }
    if groups.is_empty() {
        groups.push(SegmentCondition {
            field: "status".to_string(),
            op: "is_not_empty".to_string(),
            value: Value::Null,
        });
    }
    Segment {
        match_mode: SegmentMatch::All,
        groups,
        meta: None,
    }
}

/// SHA-256 fingerprint of the meaningful part of a segment (metadata excluded)
/// so a stored snapshot can detect query drift.
pub fn segment_fingerprint(segment: &Segment) -> String {
    let canonical = serde_json::json!({
        "match": segment.match_mode.as_str(),
        "groups": &segment.groups,
    });
    let digest = Sha256::digest(canonical.to_string().into_bytes());
    hex::encode(digest)
}

/// Compares stored refresh metadata with the current segment and membership.
pub fn detect_drift(
    segment: &Segment,
    stored: Option<&SegmentMeta>,
    actual_member_count: i64,
) -> DriftReport {
    let hash = segment_fingerprint(segment);
    let query_hash_changed = stored
        .and_then(|meta| meta.query_hash.as_deref())
        .map_or(true, |stored_hash| stored_hash != hash);
    let member_count_changed = stored
        .and_then(|meta| meta.member_count)
        .is_some_and(|stored_count| stored_count != actual_member_count);
    DriftReport {
        query_hash_changed,
        member_count_changed,
    }
}
