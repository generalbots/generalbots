//! #2537 — wire model and validation for the recursive segment editor.
//!
//! Mirrors `botui/ui/suite/lists/modules/01_field_catalog.js`: a segment is
//! `{ match, groups: [...] }` where a child is a leaf `{ field, op, value }`
//! or a nested group (max depth 3). Everything here is pure and side-effect
//! free; evaluation lives in `ui_preview`.

use serde::Deserialize;
use serde_json::Value;

use super::model::SegmentError;

/// Maximum nesting accepted by both the editor and this validator.
pub(crate) const MAX_WIRE_DEPTH: usize = 3;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum WireNode {
    Group(WireGroup),
    Condition(WireCondition),
}

#[derive(Debug, Clone, Deserialize)]
pub struct WireGroup {
    #[serde(rename = "match")]
    pub match_mode: String,
    pub groups: Vec<WireNode>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WireCondition {
    pub field: String,
    pub op: String,
    pub value: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WireSegment {
    #[serde(rename = "match")]
    pub match_mode: String,
    pub groups: Vec<WireNode>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WirePreviewRequest {
    pub segment: WireSegment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Field {
    FirstName,
    LastName,
    Email,
    Company,
    City,
    Country,
    Tags,
    Owner,
    LifecycleStatus,
    DealValue,
    DealStage,
    LastActivity,
    ConsentEmail,
    ConsentSms,
    ConsentPush,
    ConsentWhatsapp,
    ListMembership,
}

impl Field {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "first_name" => Some(Self::FirstName),
            "last_name" => Some(Self::LastName),
            "email" => Some(Self::Email),
            "company" => Some(Self::Company),
            "city" => Some(Self::City),
            "country" => Some(Self::Country),
            "tags" => Some(Self::Tags),
            "owner" => Some(Self::Owner),
            "lifecycle_status" => Some(Self::LifecycleStatus),
            "deal_value" => Some(Self::DealValue),
            "deal_stage" => Some(Self::DealStage),
            "last_activity" => Some(Self::LastActivity),
            "consent_email" => Some(Self::ConsentEmail),
            "consent_sms" => Some(Self::ConsentSms),
            "consent_push" => Some(Self::ConsentPush),
            "consent_whatsapp" => Some(Self::ConsentWhatsapp),
            "list_membership" => Some(Self::ListMembership),
            _ => None,
        }
    }

    fn ops(self) -> &'static [&'static str] {
        match self {
            Self::FirstName | Self::LastName | Self::Email | Self::Company
            | Self::City | Self::Country => &["eq", "ne", "contains", "not_contains"],
            Self::Tags => &["contains", "not_contains"],
            Self::Owner => &["eq", "ne"],
            Self::LifecycleStatus | Self::DealStage => &["eq", "ne", "in", "not_in"],
            Self::DealValue => &["eq", "ne", "gt", "gte", "lt", "lte"],
            Self::LastActivity => &["before", "after", "within_days"],
            Self::ConsentEmail | Self::ConsentSms | Self::ConsentPush
            | Self::ConsentWhatsapp => &["eq", "ne"],
            Self::ListMembership => &["in", "not_in"],
        }
    }

    fn expects_number(self) -> bool {
        matches!(self, Self::DealValue | Self::LastActivity)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Op {
    Eq,
    Ne,
    Contains,
    NotContains,
    In,
    NotIn,
    Gt,
    Gte,
    Lt,
    Lte,
    Before,
    After,
    WithinDays,
}

impl Op {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "eq" => Some(Self::Eq),
            "ne" => Some(Self::Ne),
            "contains" => Some(Self::Contains),
            "not_contains" => Some(Self::NotContains),
            "in" => Some(Self::In),
            "not_in" => Some(Self::NotIn),
            "gt" => Some(Self::Gt),
            "gte" => Some(Self::Gte),
            "lt" => Some(Self::Lt),
            "lte" => Some(Self::Lte),
            "before" => Some(Self::Before),
            "after" => Some(Self::After),
            "within_days" => Some(Self::WithinDays),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ValidCondition {
    pub(crate) field: Field,
    pub(crate) op: Op,
    pub(crate) value: Value,
}

#[derive(Debug, Clone)]
pub(crate) enum ValidNode {
    Group(ValidGroup),
    Condition(ValidCondition),
}

#[derive(Debug, Clone)]
pub(crate) struct ValidGroup {
    pub(crate) all: bool,
    pub(crate) children: Vec<ValidNode>,
}

/// Reads a number from either a JSON number or a numeric string. The editor
/// normalizes every condition value to a string, so both forms must be valid.
pub(crate) fn numeric_value(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(raw) => raw.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn validate_match(raw: &str) -> Result<bool, SegmentError> {
    match raw {
        "all" => Ok(true),
        "any" => Ok(false),
        _ => Err(SegmentError::InvalidValue {
            field: "match".to_string(),
            expected: "all|any",
        }),
    }
}

fn validate_condition(raw: &WireCondition) -> Result<ValidCondition, SegmentError> {
    let field = Field::parse(&raw.field).ok_or_else(|| SegmentError::UnknownField {
        field: raw.field.clone(),
    })?;
    let op = Op::parse(&raw.op).ok_or_else(|| SegmentError::UnknownOp {
        op: raw.op.clone(),
    })?;
    if !field.ops().contains(&raw.op.as_str()) {
        return Err(SegmentError::OpNotAllowed {
            field: raw.field.clone(),
            op: raw.op.clone(),
        });
    }
    // The editor normalizes every value to a string, so numeric fields must
    // accept both a JSON number and a numeric string.
    if field.expects_number() && numeric_value(&raw.value).is_none() {
        return Err(SegmentError::InvalidValue {
            field: raw.field.clone(),
            expected: "number",
        });
    }
    Ok(ValidCondition {
        field,
        op,
        value: raw.value.clone(),
    })
}

fn validate_node(node: &WireNode, depth: usize) -> Result<ValidNode, SegmentError> {
    match node {
        WireNode::Condition(raw) => Ok(ValidNode::Condition(validate_condition(raw)?)),
        WireNode::Group(group) => Ok(ValidNode::Group(validate_group(group, depth + 1)?)),
    }
}

fn validate_group(group: &WireGroup, depth: usize) -> Result<ValidGroup, SegmentError> {
    if depth > MAX_WIRE_DEPTH {
        return Err(SegmentError::InvalidValue {
            field: "groups".to_string(),
            expected: "depth<=3",
        });
    }
    if group.groups.is_empty() {
        return Err(SegmentError::EmptySegment);
    }
    let all = validate_match(&group.match_mode)?;
    let children = group
        .groups
        .iter()
        .map(|node| validate_node(node, depth))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ValidGroup { all, children })
}

/// Validates a wire segment into the internal tree the evaluator consumes.
pub(crate) fn validate_wire(segment: &WireSegment) -> Result<ValidGroup, SegmentError> {
    validate_group(
        &WireGroup {
            match_mode: segment.match_mode.clone(),
            groups: segment.groups.clone(),
        },
        1,
    )
}
