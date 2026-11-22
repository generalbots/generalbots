//! #2548 — pure evaluation of campaign evidence into honest numbers.
//!
//! Every function in this module is deterministic and DB-free so the
//! attribution rules can be unit-tested offline. The store only loads rows;
//! the classification and all derived metrics live here.

use std::collections::{HashMap, HashSet};

use super::model::{
    AttributionInput, AttributionTouch, CampaignEventFacts, DataQuality, DataQualityNote,
    EventCounts, EventKind, InfluencedDeal, Money, RecipientFacts, RevenueAttributionReport,
    SegmentMembership, SuppressionRecord, TouchSource,
};

pub fn normalize_email(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

pub fn classify_event(raw: &str) -> EventKind {
    match normalize_email(raw).as_str() {
        "send" | "sent" => EventKind::Send,
        "delivery" | "delivered" | "read" => EventKind::Delivery,
        "open" | "opened" => EventKind::Open,
        "click" | "clicked" => EventKind::Click,
        "reply" | "replied" => EventKind::Reply,
        "unsubscribe" | "unsubscribed" | "opt_out" => EventKind::Unsubscribe,
        "bounce" | "hard_bounce" | "soft_bounce" => EventKind::Bounce,
        "complaint" | "spam_complaint" | "abuse" => EventKind::Complaint,
        "skip" | "skipped" => EventKind::Skip,
        _ => EventKind::Unknown,
    }
}

pub fn classify_recipient_status(status: Option<&str>) -> EventKind {
    match normalize_email(status.unwrap_or("")).as_str() {
        "sent" => EventKind::Send,
        "delivered" | "read" => EventKind::Delivery,
        "opened" => EventKind::Open,
        "clicked" => EventKind::Click,
        "replied" => EventKind::Reply,
        "unsubscribed" => EventKind::Unsubscribe,
        "bounced" | "hard_bounce" | "soft_bounce" => EventKind::Bounce,
        "complaint" => EventKind::Complaint,
        "skipped" | "suppressed" => EventKind::Skip,
        _ => EventKind::Unknown,
    }
}

fn touch(
    facts: &RecipientFacts,
    sequence: u8,
    kind: EventKind,
    source: TouchSource,
    occurred_at: chrono::DateTime<chrono::Utc>,
) -> AttributionTouch {
    AttributionTouch {
        id: facts.id,
        sequence,
        source,
        kind,
        raw_event_type: facts
            .status
            .clone()
            .unwrap_or_else(|| kind.as_str().to_string()),
        channel: facts.channel.clone(),
        campaign_id: facts.campaign_id,
        contact_id: facts.contact_id,
        email: facts.email.clone(),
        occurred_at,
    }
}

/// Derive touches only from columns that actually exist. A status with no
/// timestamp is still evidence of a send, but it is stamped with the row's
/// `created_at` and marked `RecipientState` so the weaker evidence is visible.
pub fn recipient_touches(facts: &RecipientFacts) -> Vec<AttributionTouch> {
    let mut result = Vec::new();
    let status_kind = classify_recipient_status(facts.status.as_deref());
    match status_kind {
        EventKind::Send | EventKind::Delivery | EventKind::Bounce | EventKind::Complaint
        | EventKind::Unsubscribe => {
            let at = match status_kind {
                EventKind::Delivery => facts.delivered_at.unwrap_or(facts.occurred_at),
                _ => facts.sent_at.unwrap_or(facts.occurred_at),
            };
            result.push(touch(facts, 0, status_kind, TouchSource::RecipientState, at));
        }
        _ => {}
    }
    if let Some(at) = facts.opened_at {
        result.push(touch(facts, 1, EventKind::Open, TouchSource::RecipientTimestamp, at));
    }
    if let Some(at) = facts.clicked_at {
        result.push(touch(facts, 2, EventKind::Click, TouchSource::RecipientTimestamp, at));
    }
    result
}

pub fn event_touch(facts: &CampaignEventFacts) -> AttributionTouch {
    AttributionTouch {
        id: facts.id,
        sequence: 0,
        source: TouchSource::CampaignEvent,
        kind: classify_event(&facts.event_type),
        raw_event_type: facts.event_type.clone(),
        channel: facts.channel.clone(),
        campaign_id: facts.campaign_id,
        contact_id: facts.contact_id,
        email: facts.recipient_email.clone(),
        occurred_at: facts.occurred_at,
    }
}

pub fn build_event_counts(touches: &[AttributionTouch]) -> EventCounts {
    let mut seen = HashSet::new();
    for item in touches {
        seen.insert((item.kind, item.recipient_key()));
    }
    let mut counts = EventCounts::zero();
    for (kind, _) in seen {
        counts.add(kind);
    }
    counts
}

/// Attach durable/event suppressions to each membership. A membership with a
/// suppression stays visible — it is simply flagged, never silently dropped.
pub fn apply_suppressions(
    memberships: Vec<SegmentMembership>,
    suppressions: &[SuppressionRecord],
) -> Vec<SegmentMembership> {
    let by_contact: HashMap<uuid::Uuid, Vec<&str>> = {
        let mut map: HashMap<uuid::Uuid, Vec<&str>> = HashMap::new();
        for record in suppressions {
            if let Some(id) = record.contact_id {
                map.entry(id).or_default().push(record.channel.as_str());
            }
        }
        map
    };
    memberships
        .into_iter()
        .map(|mut member| {
            let mut channels: Vec<String> = match by_contact.get(&member.contact_id) {
                Some(values) => values.iter().map(|value| (*value).to_string()).collect(),
                None => Vec::new(),
            };
            channels.sort();
            channels.dedup();
            member.suppressed_channels = channels;
            member
        })
        .collect()
}

fn money_from_deals(deals: &[InfluencedDeal]) -> (Option<Money>, i64) {
    let won: Vec<&InfluencedDeal> = deals.iter().filter(|deal| deal.won).collect();
    if won.is_empty() {
        return (None, 0);
    }
    let mut missing = 0;
    let mut currencies: HashSet<String> = HashSet::new();
    let mut amount = 0.0;
    for deal in &won {
        match (&deal.value, &deal.currency) {
            (Some(value), Some(currency))
                if !currency.trim().is_empty() && value.is_finite() =>
            {
                currencies.insert(currency.trim().to_uppercase());
                amount += *value;
            }
            _ => missing += 1,
        }
    }
    if missing > 0 || currencies.len() != 1 {
        return (None, missing);
    }
    let currency = match currencies.into_iter().next() {
        Some(value) => value,
        None => return (None, missing),
    };
    (
        Some(Money {
            amount,
            currency,
        }),
        missing,
    )
}

pub fn build_report(input: &AttributionInput) -> RevenueAttributionReport {
    let counts = build_event_counts(&input.touches);
    let mut attributed = HashSet::new();
    let mut unattributed = HashSet::new();
    let mut unknown_event_types = 0;
    for item in &input.touches {
        if item.kind == EventKind::Unknown {
            unknown_event_types += 1;
        }
        if item.contact_id.is_some() {
            attributed.insert(item.recipient_key());
        } else {
            unattributed.insert(item.recipient_key());
        }
    }
    let mut suppression_keys = HashSet::new();
    for record in &input.suppressions {
        suppression_keys.insert((normalize_email(&record.address), record.channel.clone()));
    }

    let won_count = input.deals.iter().filter(|deal| deal.won).count();
    let (revenue_influenced, deals_without_money) = money_from_deals(&input.deals);
    let conversion_rate = if input.deals.is_empty() {
        None
    } else {
        Some(won_count as f64 / input.deals.len() as f64 * 100.0)
    };

    let mut notes = Vec::new();
    if !unattributed.is_empty() {
        notes.push(DataQualityNote::UnattributedTouches);
    }
    if unknown_event_types > 0 {
        notes.push(DataQualityNote::UnknownEventTypes);
    }
    if deals_without_money > 0 {
        notes.push(DataQualityNote::IncompleteDealMoney);
    }
    if input.deals.is_empty() {
        notes.push(DataQualityNote::NoDealCohort);
    }
    // A budget has no currency column, so dividing revenue by it would be a
    // fabricated ROI. The note makes the omission auditable.
    notes.push(DataQualityNote::RoiNotReconciled);

    let mut segments: HashSet<uuid::Uuid> = input.segment_ids.iter().copied().collect();
    for deal in &input.deals {
        if let Some(id) = deal.segment_id {
            segments.insert(id);
        }
    }

    RevenueAttributionReport {
        branch_id: input.branch_id,
        campaign_id: input.campaign_id,
        segment_id: input.segment_id,
        contacts_scanned: input.contacts_scanned,
        recipients_scanned: input.recipients_scanned,
        touches: counts,
        attributed_recipients: attributed.len() as i64,
        unattributed_recipients: unattributed.len() as i64,
        suppressions: suppression_keys.len() as i64,
        segments_seen: segments.len() as i64,
        deals_influenced: input.deals.len() as i64,
        won_deals_influenced: won_count as i64,
        revenue_influenced,
        conversion_rate,
        roi: None,
        data_quality: DataQuality {
            unattributed_recipients: unattributed.len() as i64,
            unknown_event_types,
            deals_without_money,
            notes,
        },
    }
}
