//! #2548 — typed revenue-attribution / contact-segment bridge.
//!
//! These types are the contract between persisted campaign evidence and CRM
//! records. Nothing here invents a number: counts come from rows, and every
//! derived metric that cannot be computed from the rows in scope is `None`
//! together with a machine-readable [`DataQualityNote`].

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Canonical event taxonomy. The raw strings from the campaign event stream
/// are normalized here; unknown strings are kept as [`Self::Unknown`] rather
/// than being silently folded into a known bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Send,
    Delivery,
    Open,
    Click,
    Reply,
    Unsubscribe,
    Bounce,
    Complaint,
    Skip,
    Unknown,
}

impl EventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Send => "send",
            Self::Delivery => "delivery",
            Self::Open => "open",
            Self::Click => "click",
            Self::Reply => "reply",
            Self::Unsubscribe => "unsubscribe",
            Self::Bounce => "bounce",
            Self::Complaint => "complaint",
            Self::Skip => "skip",
            Self::Unknown => "unknown",
        }
    }

    /// A suppression event is durable evidence that this address must not be
    /// contacted again on that channel (RFC 8058 unsub, hard bounce, abuse).
    pub fn is_suppression(self) -> bool {
        matches!(self, Self::Unsubscribe | Self::Bounce | Self::Complaint)
    }
}

/// Which persisted column produced a touch. `RecipientState` comes from a
/// status column, `RecipientTimestamp` from an actual sent/opened/clicked
/// timestamp, `CampaignEvent` from the append-only event stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TouchSource {
    RecipientState,
    RecipientTimestamp,
    CampaignEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributionTouch {
    pub id: Uuid,
    /// Distinguishes several touches derived from one row (status + opened_at).
    pub sequence: u8,
    pub source: TouchSource,
    pub kind: EventKind,
    /// Verbatim persisted value, kept for audit even when normalized.
    pub raw_event_type: String,
    pub channel: Option<String>,
    pub campaign_id: Option<Uuid>,
    /// Set only when an e-mail matched a `crm_contacts` row in the same branch.
    pub contact_id: Option<Uuid>,
    pub email: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

impl AttributionTouch {
    /// One contact is one recipient even if several addresses or legacy rows
    /// point at it. Unmatched rows are counted separately, never merged.
    pub fn recipient_key(&self) -> String {
        match self.contact_id {
            Some(id) => format!("contact:{id}"),
            None => match &self.email {
                Some(email) if !email.trim().is_empty() => {
                    format!("email:{}", super::evaluate::normalize_email(email))
                }
                _ => "unmatched".to_string(),
            },
        }
    }
}

/// Counts are *distinct recipients per event kind*, not raw event rows. One
/// contact that opens three times counts as one open, which makes the numbers
/// reproducible from the underlying rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventCounts {
    pub send: i64,
    pub delivery: i64,
    pub open: i64,
    pub click: i64,
    pub reply: i64,
    pub unsubscribe: i64,
    pub bounce: i64,
    pub complaint: i64,
    pub skip: i64,
    pub unknown: i64,
}

impl EventCounts {
    pub fn zero() -> Self {
        Self {
            send: 0,
            delivery: 0,
            open: 0,
            click: 0,
            reply: 0,
            unsubscribe: 0,
            bounce: 0,
            complaint: 0,
            skip: 0,
            unknown: 0,
        }
    }

    pub fn add(&mut self, kind: EventKind) {
        match kind {
            EventKind::Send => self.send += 1,
            EventKind::Delivery => self.delivery += 1,
            EventKind::Open => self.open += 1,
            EventKind::Click => self.click += 1,
            EventKind::Reply => self.reply += 1,
            EventKind::Unsubscribe => self.unsubscribe += 1,
            EventKind::Bounce => self.bounce += 1,
            EventKind::Complaint => self.complaint += 1,
            EventKind::Skip => self.skip += 1,
            EventKind::Unknown => self.unknown += 1,
        }
    }

    pub fn total(&self) -> i64 {
        self.send
            + self.delivery
            + self.open
            + self.click
            + self.reply
            + self.unsubscribe
            + self.bounce
            + self.complaint
            + self.skip
            + self.unknown
    }
}

/// A money amount is reported only when every won deal in scope has a value
/// and the same non-empty currency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Money {
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InfluencedDeal {
    pub id: Uuid,
    pub name: Option<String>,
    pub stage: Option<String>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub won: bool,
    pub segment_id: Option<Uuid>,
    pub segment_name: Option<String>,
    pub campaign_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipSource {
    /// `marketing_list_contacts` — an audience/list built from CRM fields.
    AudienceList,
    /// `crm_deals.segment_id` — a deal tagged with a segment.
    DealSegment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentMembership {
    pub segment_id: Uuid,
    pub segment_name: Option<String>,
    pub source: MembershipSource,
    pub contact_id: Uuid,
    /// Channels with a durable suppression record for this contact.
    pub suppressed_channels: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuppressionSource {
    /// `marketing_suppressions` row (one-click unsubscribe, hard bounce, ...).
    DurableList,
    /// Derived from an unsubscribe/bounce/complaint event in the stream.
    CampaignEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuppressionRecord {
    pub address: String,
    pub channel: String,
    pub reason: String,
    pub source: SuppressionSource,
    pub contact_id: Option<Uuid>,
}

/// The per-contact bridge: one CRM row with all of its segments, touches,
/// suppressions and influenced deals — each carrying its source table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactRevenueBridge {
    pub contact_id: Uuid,
    pub email: Option<String>,
    pub touches: Vec<AttributionTouch>,
    pub memberships: Vec<SegmentMembership>,
    pub suppressions: Vec<SuppressionRecord>,
    pub deals: Vec<InfluencedDeal>,
}

/// Machine-readable reason a metric is `None` (no translated prose in the API).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataQualityNote {
    UnattributedTouches,
    UnknownEventTypes,
    IncompleteDealMoney,
    NoDealCohort,
    RoiNotReconciled,
}

impl DataQualityNote {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnattributedTouches => "unattributed_touches",
            Self::UnknownEventTypes => "unknown_event_types",
            Self::IncompleteDealMoney => "incomplete_deal_money",
            Self::NoDealCohort => "no_deal_cohort",
            Self::RoiNotReconciled => "roi_not_reconciled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DataQuality {
    pub unattributed_recipients: i64,
    pub unknown_event_types: i64,
    pub deals_without_money: i64,
    pub notes: Vec<DataQualityNote>,
}

/// Honest aggregate view. `conversion_rate`, `revenue_influenced` and `roi`
/// are `Option` on purpose: zero rows means "no denominator", not "0%".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RevenueAttributionReport {
    pub branch_id: Uuid,
    pub campaign_id: Option<Uuid>,
    pub segment_id: Option<Uuid>,
    pub contacts_scanned: i64,
    pub recipients_scanned: i64,
    pub touches: EventCounts,
    pub attributed_recipients: i64,
    pub unattributed_recipients: i64,
    pub suppressions: i64,
    pub segments_seen: i64,
    pub deals_influenced: i64,
    pub won_deals_influenced: i64,
    pub revenue_influenced: Option<Money>,
    /// Won / influenced deals *in this scope*; `None` when no deal is scoped.
    pub conversion_rate: Option<f64>,
    /// Always `None` until a budget with a matching currency is reconciled.
    pub roi: Option<f64>,
    pub data_quality: DataQuality,
}

/// Pure input for [`super::evaluate::build_report`], assembled by the store
/// from existing tables; no new migration is required.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AttributionInput {
    pub branch_id: Uuid,
    pub campaign_id: Option<Uuid>,
    pub segment_id: Option<Uuid>,
    pub contacts_scanned: i64,
    pub recipients_scanned: i64,
    pub touches: Vec<AttributionTouch>,
    pub suppressions: Vec<SuppressionRecord>,
    pub deals: Vec<InfluencedDeal>,
    pub segment_ids: Vec<Uuid>,
    pub memberships: Vec<SegmentMembership>,
}

/// Column values read from one `marketing_recipients` row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecipientFacts {
    pub id: Uuid,
    pub campaign_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub email: Option<String>,
    pub channel: Option<String>,
    pub status: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub opened_at: Option<DateTime<Utc>>,
    pub clicked_at: Option<DateTime<Utc>>,
    pub occurred_at: DateTime<Utc>,
}

/// Column values read from one `marketing_campaign_events` row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CampaignEventFacts {
    pub id: Uuid,
    pub campaign_id: Option<Uuid>,
    pub channel: Option<String>,
    pub event_type: String,
    pub recipient_email: Option<String>,
    pub contact_id: Option<Uuid>,
    pub occurred_at: DateTime<Utc>,
}
