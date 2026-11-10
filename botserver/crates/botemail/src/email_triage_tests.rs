//! Triage contract: the category vocabulary, the fallback keyword ladder and
//! the skip rule for machine-generated mail.

use chrono::Utc;

use super::{should_skip_triage, EmailCategory, EmailHeaders, EmailTriagingResult};

fn headers(subject: &str, from: &str, to: &[&str]) -> EmailHeaders {
    EmailHeaders {
        message_id: "<test@example.com>".into(),
        from: from.into(),
        to: to.iter().map(|addr| (*addr).to_string()).collect(),
        cc: vec![],
        subject: subject.into(),
        date: Utc::now(),
        in_reply_to: None,
        references: vec![],
    }
}

#[test]
fn every_category_parses_its_aliases() {
    let cases = [
        ("urgent", EmailCategory::Urgent),
        (" URGENT ", EmailCategory::Urgent),
        ("meeting_request", EmailCategory::MeetingRequest),
        ("meeting request", EmailCategory::MeetingRequest),
        ("Meeting", EmailCategory::MeetingRequest),
        ("informational", EmailCategory::Informational),
        ("info", EmailCategory::Informational),
        ("spam", EmailCategory::Spam),
    ];
    for (text, expected) in cases {
        assert_eq!(text.parse::<EmailCategory>().ok(), Some(expected), "{text:?}");
    }
}

#[test]
fn an_unknown_category_names_the_input() {
    let error = "billing".parse::<EmailCategory>().expect_err("must fail");
    assert_eq!(error, "Unknown email category: billing");
}

#[test]
fn the_category_vocabulary_round_trips_through_serde_and_display() {
    for (category, wire) in [
        (EmailCategory::Urgent, "urgent"),
        (EmailCategory::MeetingRequest, "meeting_request"),
        (EmailCategory::Informational, "informational"),
        (EmailCategory::Spam, "spam"),
    ] {
        assert_eq!(category.to_string(), wire);
        let value = serde_json::to_value(&category).expect("serializes");
        let back: EmailCategory = serde_json::from_value(value).expect("deserializes");
        assert_eq!(back, category);
    }
}

#[test]
fn a_triage_result_round_trips() {
    let result = EmailTriagingResult {
        category: EmailCategory::Urgent,
        confidence: 0.75,
        reasoning: "keyword".to_string(),
    };
    let value = serde_json::to_value(&result).expect("serializes");
    let back: EmailTriagingResult = serde_json::from_value(value).expect("deserializes");
    assert_eq!(back.category, EmailCategory::Urgent);
    assert!((back.confidence - 0.75).abs() < f32::EPSILON);
}

#[test]
fn an_empty_recipient_list_is_skipped() {
    assert!(should_skip_triage(&headers("Hi", "a@b.example", &[])));
}

#[test]
fn machine_senders_are_skipped() {
    for recipient in [
        "noreply@service.example",
        "no-reply@service.example",
        "notifications@service.example",
    ] {
        assert!(
            should_skip_triage(&headers("Hi", "a@b.example", &[recipient])),
            "{recipient}"
        );
    }
}

#[test]
fn a_human_recipient_is_not_skipped() {
    assert!(!should_skip_triage(&headers(
        "Hi",
        "a@b.example",
        &["user@example.com", "team@example.com"]
    )));
    // The marker must appear in the local part of an address we accept.
    assert!(!should_skip_triage(&headers(
        "Hi",
        "a@b.example",
        &["noreply-fan@example.com"]
    )));
}

#[test]
fn the_skip_rule_reads_every_recipient() {
    let mixed = headers(
        "Hi",
        "a@b.example",
        &["user@example.com", "notifications@service.example"],
    );
    assert!(should_skip_triage(&mixed));
}
