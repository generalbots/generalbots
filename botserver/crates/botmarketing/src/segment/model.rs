//! Wire model and documented field catalog for list segments (#2537).
//!
//! The segment persisted in `marketing_lists.criteria` has this shape:
//!
//! ```json
//! { "match": "all", "groups": [ { "field": "status", "op": "eq", "value": "active" } ] }
//! ```
//!
//! `meta` is an internal, optional sibling key written on materialization; it
//! carries the refresh timestamp, member count and query fingerprint. The
//! `marketing_lists` table has no dedicated `last_refreshed_at` column, so the
//! timestamp lives here (and `updated_at` mirrors the refresh time).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// How the conditions in `groups` are combined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SegmentMatch {
    All,
    Any,
}

impl SegmentMatch {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Any => "any",
        }
    }
}

/// One validated segment condition as sent by the client. `field` and `op`
/// stay strings on the wire so unknown values can be rejected with a typed
/// [`SegmentError`] instead of a serde "unknown variant" message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentCondition {
    pub field: String,
    pub op: String,
    pub value: Value,
}

/// Refresh metadata stored alongside the segment in `criteria`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_refreshed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_count: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_hash: Option<String>,
}

/// Structured segment. `meta` is written by materialization and ignored by
/// evaluation, so a segment can be round-tripped without changing its meaning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Segment {
    #[serde(rename = "match")]
    pub match_mode: SegmentMatch,
    #[serde(default)]
    pub groups: Vec<SegmentCondition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<SegmentMeta>,
}

/// The kind of JSON value a field accepts. Used by validation to reject a
/// condition before it reaches the evaluator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Text,
    TextList,
    Bool,
    Number,
    Date,
    Uuid,
    UuidList,
}

impl ValueKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::TextList => "text_list",
            Self::Bool => "bool",
            Self::Number => "number",
            Self::Date => "date",
            Self::Uuid => "uuid",
            Self::UuidList => "uuid_list",
        }
    }
}

/// Typed validation error returned as HTTP 400 with a stable machine code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum SegmentError {
    EmptySegment,
    UnknownField { field: String },
    UnknownOp { op: String },
    OpNotAllowed { field: String, op: String },
    InvalidValue { field: String, expected: &'static str },
    InvalidMeta,
}

impl fmt::Display for SegmentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySegment => write!(formatter, "segment must contain at least one condition"),
            Self::UnknownField { field } => write!(formatter, "unknown field: {field}"),
            Self::UnknownOp { op } => write!(formatter, "unknown operator: {op}"),
            Self::OpNotAllowed { field, op } => {
                write!(formatter, "operator {op} is not allowed for field {field}")
            }
            Self::InvalidValue { field, expected } => {
                write!(formatter, "invalid value for field {field}: expected {expected}")
            }
            Self::InvalidMeta => write!(formatter, "invalid segment metadata"),
        }
    }
}

impl std::error::Error for SegmentError {}

impl SegmentError {
    pub fn code(&self) -> SegmentErrorCode {
        match self {
            Self::EmptySegment => SegmentErrorCode::EmptySegment,
            Self::UnknownField { .. } => SegmentErrorCode::UnknownField,
            Self::UnknownOp { .. } => SegmentErrorCode::UnknownOp,
            Self::OpNotAllowed { .. } => SegmentErrorCode::OpNotAllowed,
            Self::InvalidValue { .. } => SegmentErrorCode::InvalidValue,
            Self::InvalidMeta => SegmentErrorCode::InvalidMeta,
        }
    }
}

/// Stable, serializable error code (kept separate from the display message).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentErrorCode {
    EmptySegment,
    UnknownField,
    UnknownOp,
    OpNotAllowed,
    InvalidValue,
    InvalidMeta,
}

/// A row of the documented field catalog, also served to the UI so the
/// editor never invents field names or operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FieldSpec {
    pub field: &'static str,
    pub value_kind: &'static str,
    pub ops: &'static [&'static str],
    pub source: &'static str,
    pub description: &'static str,
}

const TEXT_OPS: &[&str] = &["eq", "ne", "contains", "in", "not_in", "is_empty", "is_not_empty"];
const TAG_OPS: &[&str] = &["contains", "in", "is_empty", "is_not_empty"];
const BOOL_OPS: &[&str] = &["eq", "ne"];
const NUMBER_OPS: &[&str] = &["eq", "gt", "gte", "lt", "lte"];
const STAGE_OPS: &[&str] = &["eq", "ne", "contains", "in", "not_in"];
const DATE_OPS: &[&str] = &["before", "after"];
const MEMBER_OPS: &[&str] = &["eq", "in", "not_in", "is_empty", "is_not_empty"];
const OWNER_OPS: &[&str] = &["eq", "ne", "is_empty", "is_not_empty"];

/// Authoritative field catalog. Every field below is evaluated from data
/// already owned by the people/marketing domain — no new dependency or column
/// is required.
pub const FIELD_CATALOG: &[FieldSpec] = &[
    FieldSpec {
        field: "status",
        value_kind: "text",
        ops: TEXT_OPS,
        source: "crm_contacts.status",
        description: "CRM lifecycle status (active, lead, customer, ...).",
    },
    FieldSpec {
        field: "company",
        value_kind: "text",
        ops: TEXT_OPS,
        source: "crm_contacts.company",
        description: "Company name, case-insensitive match or substring.",
    },
    FieldSpec {
        field: "tags",
        value_kind: "text_list",
        ops: TAG_OPS,
        source: "crm_contacts.tags",
        description: "CRM tags; contains tests membership of one tag.",
    },
    FieldSpec {
        field: "consent_email",
        value_kind: "bool",
        ops: BOOL_OPS,
        source: "crm_contacts.custom_fields",
        description: "Email opt-in flag (consent_email / consent.email).",
    },
    FieldSpec {
        field: "consent_sms",
        value_kind: "bool",
        ops: BOOL_OPS,
        source: "crm_contacts.custom_fields",
        description: "SMS opt-in flag (consent_sms / consent.sms).",
    },
    FieldSpec {
        field: "consent_whatsapp",
        value_kind: "bool",
        ops: BOOL_OPS,
        source: "crm_contacts.custom_fields",
        description: "WhatsApp opt-in flag (consent_whatsapp / consent.whatsapp).",
    },
    FieldSpec {
        field: "owner",
        value_kind: "uuid",
        ops: OWNER_OPS,
        source: "crm_contacts.owner_id",
        description: "Owner user id; is_empty matches unassigned contacts.",
    },
    FieldSpec {
        field: "deal_value",
        value_kind: "number",
        ops: NUMBER_OPS,
        source: "crm_leads.value, crm_opportunities.value",
        description: "Highest deal value linked to the contact in either table.",
    },
    FieldSpec {
        field: "deal_stage",
        value_kind: "text",
        ops: STAGE_OPS,
        source: "crm_leads.stage, crm_opportunities.stage",
        description: "Any deal stage linked to the contact.",
    },
    FieldSpec {
        field: "last_activity",
        value_kind: "date",
        ops: DATE_OPS,
        source: "crm_contacts.updated_at",
        description: "RFC3339 timestamp; before/after compare contact update time.",
    },
    FieldSpec {
        field: "list_membership",
        value_kind: "uuid",
        ops: MEMBER_OPS,
        source: "marketing_list_contacts.list_id",
        description: "Membership of another marketing list (by list id).",
    },
];

/// Drift between a stored refresh snapshot and the current segment/membership.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DriftReport {
    pub query_hash_changed: bool,
    pub member_count_changed: bool,
}
