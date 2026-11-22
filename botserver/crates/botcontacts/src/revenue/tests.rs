//! #2548 — offline unit tests for the attribution rules.
//! No database is required: every rule is exercised through the pure model.

use chrono::{TimeZone, Utc};
use uuid::Uuid;

use super::evaluate::{
    apply_suppressions, build_event_counts, build_report, classify_event,
    classify_recipient_status, event_touch, normalize_email, recipient_touches,
};
use super::model::{
    AttributionInput, AttributionTouch, CampaignEventFacts, DataQualityNote, EventKind,
    InfluencedDeal, MembershipSource, RecipientFacts, SegmentMembership, SuppressionRecord,
    SuppressionSource, TouchSource,
};

fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}

fn at(seconds: i64) -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(1_700_000_000 + seconds, 0)
        .single()
        .unwrap_or_else(Utc::now)
}

fn recipient(n: u128, status: &str, contact: Option<Uuid>, email: Option<&str>) -> RecipientFacts {
    RecipientFacts {
        id: id(n),
        campaign_id: Some(id(90)),
        contact_id: contact,
        email: email.map(str::to_string),
        channel: Some("email".to_string()),
        status: Some(status.to_string()),
        sent_at: Some(at(0)),
        delivered_at: Some(at(10)),
        opened_at: Some(at(20)),
        clicked_at: None,
        occurred_at: at(0),
    }
}

fn deal(n: u128, value: Option<f64>, currency: Option<&str>, won: bool) -> InfluencedDeal {
    InfluencedDeal {
        id: id(n),
        name: Some(format!("Deal {n}")),
        stage: Some("won".to_string()),
        value,
        currency: currency.map(str::to_string),
        won,
        segment_id: Some(id(77)),
        segment_name: Some("Segment".to_string()),
        campaign_id: Some(id(90)),
        contact_id: Some(id(1)),
    }
}

fn empty_input() -> AttributionInput {
    AttributionInput {
        branch_id: id(5),
        campaign_id: None,
        segment_id: None,
        contacts_scanned: 0,
        recipients_scanned: 0,
        touches: Vec::new(),
        suppressions: Vec::new(),
        deals: Vec::new(),
        segment_ids: Vec::new(),
        memberships: Vec::new(),
    }
}

#[test]
fn email_normalization_is_case_and_space_insensitive() {
    assert_eq!(normalize_email("  Ada@Example.COM "), "ada@example.com");
    assert_eq!(normalize_email(""), "");
}

#[test]
fn event_taxonomy_maps_known_values_and_preserves_unknown() {
    assert_eq!(classify_event("Sent"), EventKind::Send);
    assert_eq!(classify_event("hard_bounce"), EventKind::Bounce);
    assert_eq!(classify_event("spam_complaint"), EventKind::Complaint);
    assert_eq!(classify_event("unsubscribed"), EventKind::Unsubscribe);
    assert_eq!(classify_event("mystery"), EventKind::Unknown);
    assert_eq!(classify_recipient_status(Some("read")), EventKind::Delivery);
    assert_eq!(classify_recipient_status(None), EventKind::Unknown);
}

#[test]
fn recipient_touches_are_derived_only_from_persisted_columns() {
    let facts = recipient(1, "delivered", Some(id(1)), Some("ada@example.com"));
    let touches = recipient_touches(&facts);
    assert_eq!(touches.len(), 2);
    assert_eq!(touches[0].kind, EventKind::Delivery);
    assert_eq!(touches[0].source, TouchSource::RecipientState);
    assert_eq!(touches[0].occurred_at, at(10));
    assert_eq!(touches[1].kind, EventKind::Open);
    assert_eq!(touches[1].source, TouchSource::RecipientTimestamp);

    let mut no_timestamps = facts;
    no_timestamps.sent_at = None;
    no_timestamps.delivered_at = None;
    no_timestamps.opened_at = None;
    let touches = recipient_touches(&no_timestamps);
    assert_eq!(touches.len(), 1);
    assert_eq!(touches[0].occurred_at, at(0));
    assert_eq!(touches[0].source, TouchSource::RecipientState);
}

#[test]
fn counts_are_distinct_recipients_not_event_rows() {
    let first = recipient(1, "sent", Some(id(1)), None);
    let second = recipient(2, "sent", Some(id(1)), None);
    let mut touches = recipient_touches(&first);
    touches.extend(recipient_touches(&second));
    let counts = build_event_counts(&touches);
    assert_eq!(counts.send, 1);
    assert_eq!(counts.total(), 1);
}

#[test]
fn event_touch_keeps_raw_type_and_resolved_contact() {
    let facts = CampaignEventFacts {
        id: id(8),
        campaign_id: Some(id(90)),
        channel: Some("whatsapp".to_string()),
        event_type: "Reply".to_string(),
        recipient_email: Some("Grace@Example.com".to_string()),
        contact_id: Some(id(2)),
        occurred_at: at(5),
    };
    let touch = event_touch(&facts);
    assert_eq!(touch.kind, EventKind::Reply);
    assert_eq!(touch.raw_event_type, "Reply");
    assert_eq!(touch.source, TouchSource::CampaignEvent);
    assert_eq!(touch.contact_id, Some(id(2)));
    assert_eq!(touch.recipient_key(), format!("contact:{}", id(2)));
}

#[test]
fn empty_scope_has_no_denominator_and_no_fabricated_zero() {
    let report = build_report(&empty_input());
    assert_eq!(report.touches.total(), 0);
    assert_eq!(report.conversion_rate, None);
    assert_eq!(report.revenue_influenced, None);
    assert_eq!(report.roi, None);
    assert!(report
        .data_quality
        .notes
        .contains(&DataQualityNote::NoDealCohort));
    assert!(report
        .data_quality
        .notes
        .contains(&DataQualityNote::RoiNotReconciled));
}

#[test]
fn revenue_requires_same_currency_and_complete_values() {
    let mut input = empty_input();
    input.deals = vec![deal(1, Some(100.0), Some("EUR"), true), deal(2, Some(50.0), Some("EUR"), true)];
    let report = build_report(&input);
    let money = report.revenue_influenced.expect("same-currency won deals");
    assert_eq!(money.amount, 150.0);
    assert_eq!(money.currency, "EUR");
    assert_eq!(report.conversion_rate, Some(100.0));

    input.deals.push(deal(3, Some(25.0), Some("USD"), false));
    let rate = build_report(&input).conversion_rate.expect("deal cohort");
    assert!((rate - 200.0 / 3.0).abs() < 1e-9);

    input.deals.push(deal(4, None, None, true));
    let report = build_report(&input);
    assert_eq!(report.revenue_influenced, None);
    assert_eq!(report.data_quality.deals_without_money, 1);
    assert!(report
        .data_quality
        .notes
        .contains(&DataQualityNote::IncompleteDealMoney));
}

#[test]
fn unattributed_touches_are_counted_but_never_attributed() {
    let mut input = empty_input();
    let matched = recipient(1, "sent", Some(id(1)), None);
    let unknown = recipient(2, "sent", None, Some("someone@example.com"));
    input.touches.extend(recipient_touches(&matched));
    input.touches.extend(recipient_touches(&unknown));
    input.touches.push(event_touch(&CampaignEventFacts {
        id: id(3),
        campaign_id: Some(id(90)),
        channel: Some("email".to_string()),
        event_type: "mystery".to_string(),
        recipient_email: None,
        contact_id: None,
        occurred_at: at(3),
    }));
    // A click from a different address must not inflate the matched contact.
    let mut second_unknown = recipient(4, "sent", None, Some("other@example.com"));
    second_unknown.clicked_at = Some(at(30));
    input.touches.extend(recipient_touches(&second_unknown));

    let report = build_report(&input);
    assert_eq!(report.touches.send, 3);
    assert_eq!(report.touches.unknown, 1);
    assert_eq!(report.attributed_recipients, 1);
    // "someone", "other" and the e-mail-less unknown event are three keys.
    assert_eq!(report.unattributed_recipients, 3);
    assert_eq!(report.data_quality.unknown_event_types, 1);
    assert!(report
        .data_quality
        .notes
        .contains(&DataQualityNote::UnattributedTouches));
    assert!(report
        .data_quality
        .notes
        .contains(&DataQualityNote::UnknownEventTypes));
}

#[test]
fn suppressions_are_visible_on_memberships() {
    let memberships = vec![SegmentMembership {
        segment_id: id(77),
        segment_name: None,
        source: MembershipSource::AudienceList,
        contact_id: id(1),
        suppressed_channels: Vec::new(),
    }];
    let suppressions = vec![
        SuppressionRecord {
            address: "ada@example.com".to_string(),
            channel: "email".to_string(),
            reason: "unsubscribe".to_string(),
            source: SuppressionSource::DurableList,
            contact_id: Some(id(1)),
        },
        SuppressionRecord {
            address: "ada@example.com".to_string(),
            channel: "email".to_string(),
            reason: "unsubscribe".to_string(),
            source: SuppressionSource::CampaignEvent,
            contact_id: Some(id(1)),
        },
        SuppressionRecord {
            address: "ada@example.com".to_string(),
            channel: "whatsapp".to_string(),
            reason: "bounce".to_string(),
            source: SuppressionSource::CampaignEvent,
            contact_id: Some(id(1)),
        },
    ];
    let result = apply_suppressions(memberships, &suppressions);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].suppressed_channels, vec!["email", "whatsapp"]);
}

#[test]
fn suppression_events_become_durable_evidence() {
    assert!(EventKind::Unsubscribe.is_suppression());
    assert!(EventKind::Bounce.is_suppression());
    assert!(EventKind::Complaint.is_suppression());
    assert!(!EventKind::Open.is_suppression());
    let touches: Vec<AttributionTouch> = recipient_touches(&recipient(1, "sent", None, None));
    assert!(touches
        .iter()
        .all(|item| !classify_event(&item.raw_event_type).is_suppression()));
}
