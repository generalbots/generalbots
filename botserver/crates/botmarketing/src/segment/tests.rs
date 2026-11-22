//! Unit tests for segment evaluation, validation, suppression and drift.

use chrono::{TimeZone, Utc};
use serde_json::{json, Value};
use uuid::Uuid;

use super::evaluate::{candidate_matches, consent_flags, is_suppressed, SegmentCandidate};
use super::model::{
    DriftReport, Segment, SegmentCondition, SegmentError, SegmentMatch, SegmentMeta, FIELD_CATALOG,
};
use super::validate::{
    compile_legacy_query, detect_drift, parse_segment, segment_fingerprint, ValidSegment,
};

fn condition(field: &str, op: &str, value: Value) -> SegmentCondition {
    SegmentCondition {
        field: field.to_string(),
        op: op.to_string(),
        value,
    }
}

fn segment(match_mode: SegmentMatch, groups: Vec<SegmentCondition>) -> Segment {
    Segment {
        match_mode,
        groups,
        meta: None,
    }
}

fn candidate(status: &str, company: Option<&str>, tags: &[&str]) -> SegmentCandidate {
    SegmentCandidate {
        id: Uuid::new_v4(),
        first_name: Some("Ada".to_string()),
        last_name: Some("Lovelace".to_string()),
        email: Some("ada@example.com".to_string()),
        status: status.to_string(),
        company: company.map(String::from),
        city: None,
        country: None,
        tags: tags.iter().map(|tag| tag.to_string()).collect(),
        owner_id: None,
        last_activity: Utc
            .with_ymd_and_hms(2026, 1, 1, 0, 0, 0)
            .single()
            .unwrap_or_else(|| Utc::now()),
        deal_value: Some(1500.0),
        deal_stages: vec!["proposal".to_string()],
        list_ids: Vec::new(),
        consent: Default::default(),
        marketing_subscribed: None,
    }
}

fn eligible_ids(candidates: &[SegmentCandidate], validated: &ValidSegment) -> Vec<Uuid> {
    candidates
        .iter()
        .filter(|item| candidate_matches(item, validated) && !is_suppressed(item))
        .map(|item| item.id)
        .collect()
}

#[test]
fn and_requires_every_condition() {
    let query = segment(
        SegmentMatch::All,
        vec![
            condition("status", "eq", json!("active")),
            condition("company", "contains", json!("acme")),
        ],
    );
    let validated = parse_segment(&query).unwrap();
    assert!(candidate_matches(&candidate("active", Some("Acme Ltd"), &[]), &validated));
    assert!(!candidate_matches(&candidate("active", Some("Globex"), &[]), &validated));
    assert!(!candidate_matches(&candidate("lead", Some("Acme Ltd"), &[]), &validated));
}

#[test]
fn any_accepts_one_condition() {
    let query = segment(
        SegmentMatch::Any,
        vec![
            condition("status", "eq", json!("active")),
            condition("tags", "contains", json!("vip")),
        ],
    );
    let validated = parse_segment(&query).unwrap();
    assert!(candidate_matches(&candidate("lead", None, &["vip"]), &validated));
    assert!(candidate_matches(&candidate("active", None, &[]), &validated));
    assert!(!candidate_matches(&candidate("lead", None, &["regular"]), &validated));
}

#[test]
fn numeric_and_date_conditions_use_typed_comparisons() {
    let query = segment(
        SegmentMatch::All,
        vec![
            condition("deal_value", "gte", json!(1000)),
            condition("last_activity", "after", json!("2025-01-01T00:00:00Z")),
        ],
    );
    let validated = parse_segment(&query).unwrap();
    let mut contact = candidate("active", None, &[]);
    contact.deal_value = Some(900.0);
    assert!(!candidate_matches(&contact, &validated));
    contact.deal_value = Some(1500.0);
    assert!(candidate_matches(&contact, &validated));
}

#[test]
fn consent_false_excludes_recipient() {
    let query = segment(SegmentMatch::All, vec![condition("status", "eq", json!("active"))]);
    let validated = parse_segment(&query).unwrap();
    let mut opted_out = candidate("active", None, &[]);
    opted_out.consent =
        consent_flags(Some(&json!({ "consent": { "email": false } })));
    let mut subscribed = candidate("active", None, &[]);
    subscribed.marketing_subscribed = Some(false);
    let bounced = candidate("bounced", None, &[]);
    let allowed = candidate("active", None, &[]);

    let members = eligible_ids(
        &[opted_out, subscribed, bounced, allowed.clone()],
        &validated,
    );
    assert_eq!(members, vec![allowed.id]);
}

#[test]
fn bool_consent_condition_reads_custom_fields() {
    let query = segment(
        SegmentMatch::All,
        vec![condition("consent_whatsapp", "eq", json!(true))],
    );
    let validated = parse_segment(&query).unwrap();
    let mut contact = candidate("active", None, &[]);
    contact.consent = consent_flags(Some(&json!({ "consent_whatsapp": "opted_in" })));
    assert!(candidate_matches(&contact, &validated));
    contact.consent = consent_flags(Some(&json!({ "consent_whatsapp": false })));
    assert!(!candidate_matches(&contact, &validated));
}

#[test]
fn materialization_filter_is_idempotent() {
    let query = segment(
        SegmentMatch::All,
        vec![condition("status", "eq", json!("active"))],
    );
    let validated = parse_segment(&query).unwrap();
    let contacts = vec![
        candidate("active", Some("Acme"), &[]),
        candidate("active", Some("Globex"), &[]),
        candidate("unsubscribed", Some("Initech"), &[]),
    ];
    let first = eligible_ids(&contacts, &validated);
    let second = eligible_ids(&contacts, &validated);
    assert_eq!(first, second);
    assert_eq!(first.len(), 2);
    assert!(!first.contains(&contacts[2].id));
}

#[test]
fn validation_rejects_unknown_fields_and_ops() {
    let unknown_field = segment(SegmentMatch::All, vec![condition("nope", "eq", json!("x"))]);
    assert_eq!(
        parse_segment(&unknown_field).unwrap_err(),
        SegmentError::UnknownField {
            field: "nope".to_string()
        }
    );

    let unknown_op = segment(SegmentMatch::All, vec![condition("status", "regex", json!("x"))]);
    assert_eq!(
        parse_segment(&unknown_op).unwrap_err(),
        SegmentError::UnknownOp {
            op: "regex".to_string()
        }
    );

    let wrong_op = segment(
        SegmentMatch::All,
        vec![condition("deal_value", "contains", json!("1"))],
    );
    assert_eq!(
        parse_segment(&wrong_op).unwrap_err(),
        SegmentError::OpNotAllowed {
            field: "deal_value".to_string(),
            op: "contains".to_string(),
        }
    );

    let wrong_value = segment(
        SegmentMatch::All,
        vec![condition("consent_email", "eq", json!("maybe"))],
    );
    assert_eq!(
        parse_segment(&wrong_value).unwrap_err(),
        SegmentError::InvalidValue {
            field: "consent_email".to_string(),
            expected: "bool",
        }
    );

    assert_eq!(
        parse_segment(&segment(SegmentMatch::All, Vec::new())).unwrap_err(),
        SegmentError::EmptySegment
    );
}

#[test]
fn legacy_query_compiles_to_structured_conditions() {
    let legacy = compile_legacy_query("status=active company=Acme");
    assert_eq!(legacy.match_mode, SegmentMatch::All);
    assert_eq!(legacy.groups.len(), 2);
    assert_eq!(legacy.groups[0].field, "status");
    assert_eq!(legacy.groups[0].op, "eq");
    assert_eq!(legacy.groups[1].field, "company");
    parse_segment(&legacy).unwrap();
}

#[test]
fn drift_detects_query_and_member_changes() {
    let query = segment(SegmentMatch::All, vec![condition("status", "eq", json!("active"))]);
    let meta = SegmentMeta {
        last_refreshed_at: Some("2026-01-01T00:00:00Z".to_string()),
        member_count: Some(3),
        query_hash: Some(segment_fingerprint(&query)),
    };
    assert_eq!(
        detect_drift(&query, Some(&meta), 3),
        DriftReport {
            query_hash_changed: false,
            member_count_changed: false,
        }
    );
    let changed = segment(SegmentMatch::Any, vec![condition("company", "eq", json!("Acme"))]);
    assert!(detect_drift(&changed, Some(&meta), 3).query_hash_changed);
    assert!(detect_drift(&query, Some(&meta), 2).member_count_changed);
}

#[test]
fn catalog_covers_every_documented_field() {
    let expected = [
        "status",
        "company",
        "tags",
        "consent_email",
        "consent_sms",
        "consent_whatsapp",
        "owner",
        "deal_value",
        "deal_stage",
        "last_activity",
        "list_membership",
    ];
    for field in expected {
        assert!(FIELD_CATALOG.iter().any(|spec| spec.field == field), "{field} is missing");
    }
}
