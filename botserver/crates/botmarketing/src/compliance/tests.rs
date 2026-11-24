//! Unit tests for the campaign delivery compliance gates (#2536).
//! No database or running server is required.

use super::*;
use serde_json::json;
use uuid::Uuid;

fn unique_address(prefix: &str) -> String {
    format!("{prefix}-{}@example.test", Uuid::new_v4())
}

#[test]
fn opt_out_blocks_send() {
    let branch_id = Uuid::new_v4();
    let address = unique_address("optout");
    let key = SuppressionKey::new(branch_id, MarketingChannel::Email, &address);

    let before = evaluate_send_gate(
        branch_id,
        MarketingChannel::Email,
        &address,
        None,
        Some(true),
        None,
    );
    assert_eq!(before, SendGateDecision::Allow);

    let added = SuppressionStore::global().add(key.clone());
    assert!(added.is_ok());
    assert!(added.map(|outcome| outcome.newly_added).unwrap_or(false));
    assert!(SuppressionStore::global().is_suppressed(&key));

    let after = evaluate_send_gate(
        branch_id,
        MarketingChannel::Email,
        &address,
        None,
        Some(true),
        None,
    );
    assert_eq!(after, SendGateDecision::Suppressed);
}

#[test]
fn suppression_is_channel_and_branch_scoped() {
    let branch_id = Uuid::new_v4();
    let other_branch = Uuid::new_v4();
    let address = unique_address("scope");
    let store = SuppressionStore::default();
    let key = SuppressionKey::new(branch_id, MarketingChannel::Email, &address);
    assert!(store.add(key).is_ok());

    assert!(store.is_suppressed(&SuppressionKey::new(
        branch_id,
        MarketingChannel::Email,
        &address.to_ascii_uppercase()
    )));
    assert!(!store.is_suppressed(&SuppressionKey::new(
        other_branch,
        MarketingChannel::Email,
        &address
    )));
    assert!(!store.is_suppressed(&SuppressionKey::new(
        branch_id,
        MarketingChannel::Whatsapp,
        &address
    )));
}

#[test]
fn consent_gate_blocks_send() {
    let branch_id = Uuid::new_v4();
    let address = unique_address("consent");
    let gate = |subscribed, metadata: Option<&serde_json::Value>| {
        evaluate_send_gate(
            branch_id,
            MarketingChannel::Email,
            &address,
            None,
            subscribed,
            metadata,
        )
    };

    assert_eq!(gate(None, None), SendGateDecision::NoConsent);
    assert_eq!(gate(Some(false), None), SendGateDecision::NoConsent);
    assert_eq!(gate(Some(true), None), SendGateDecision::Allow);

    let opted_out = json!({ "consent": { "email": false } });
    assert_eq!(gate(Some(true), Some(&opted_out)), SendGateDecision::NoConsent);

    let opted_in = json!({ "consent_email": "opted_in" });
    assert_eq!(gate(None, Some(&opted_in)), SendGateDecision::Allow);

    assert_eq!(
        evaluate_send_gate(
            branch_id,
            MarketingChannel::Email,
            &address,
            Some("unsubscribed"),
            Some(true),
            None,
        ),
        SendGateDecision::Suppressed
    );
    assert_eq!(channel_consent(None, None, MarketingChannel::Sms), ConsentState::Unknown);
}

#[test]
fn consent_missing_is_recorded_as_skip_event() {
    // A recipient with no consent flag and no `subscribed` value is Unknown,
    // never OptedIn. The worker writes `decision.status()` as the event status.
    let branch_id = Uuid::new_v4();
    let address = unique_address("missing-consent");
    let decision = evaluate_send_gate(
        branch_id,
        MarketingChannel::Sms,
        &address,
        None,
        None,
        None,
    );
    assert_eq!(decision, SendGateDecision::NoConsent);
    assert_eq!(decision.status(), "consent_skipped");

    assert!(recipient_status_blocks_send(Some("dead_letter")));
    assert!(recipient_status_blocks_send(Some(" BOUNCED ")));
    assert!(recipient_status_blocks_send(Some("complaint")));
    assert!(!recipient_status_blocks_send(Some("sent")));
    assert!(!recipient_status_blocks_send(None));
}

#[test]
fn unsubscribe_is_idempotent() {
    let claims = UnsubscribeClaims {
        branch_id: Uuid::new_v4(),
        channel: MarketingChannel::Whatsapp,
        address: unique_address("unsub"),
    };
    let first = record_unsubscribe(&claims);
    assert!(first.is_ok());
    assert!(first.map(|result| result.success && result.newly_added).unwrap_or(false));

    let second = record_unsubscribe(&claims);
    assert!(second.is_ok());
    assert!(!second.map(|result| result.newly_added).unwrap_or(true));

    let decision = evaluate_send_gate(
        claims.branch_id,
        MarketingChannel::Whatsapp,
        &claims.address,
        Some("sent"),
        Some(true),
        None,
    );
    assert_eq!(decision, SendGateDecision::Suppressed);
}

#[test]
fn unsubscribe_token_is_signed_and_bound_to_claims() {
    let key = "unit-test-signing-key";
    let claims = UnsubscribeClaims {
        branch_id: Uuid::new_v4(),
        channel: MarketingChannel::Email,
        address: unique_address("token"),
    };
    let token = sign_claims_with_key(key, &claims);
    assert!(token.is_some());
    let token = token.unwrap_or_else(String::new);

    let verified = verify_unsubscribe_token(key, &token);
    assert!(verified.is_ok());
    assert_eq!(
        verified.map(|decoded| decoded.address).unwrap_or_else(String::new),
        claims.address
    );
    assert_eq!(
        verify_unsubscribe_token("another-key", &token).err(),
        Some(UnsubscribeError::BadSignature)
    );
    assert!(verify_unsubscribe_token(key, "not-a-token").is_err());
    assert_eq!(
        build_unsubscribe_url("", &claims),
        None
    );
}

#[test]
fn bounce_classification() {
    assert_eq!(classify_bounce("550 5.1.1 mailbox does not exist"), BounceClass::Hard);
    assert_eq!(classify_bounce("Spam complaint reported by recipient"), BounceClass::Complaint);
    assert_eq!(classify_bounce("mailbox full 4.2.2"), BounceClass::Soft);
    assert_eq!(classify_bounce("connection timed out"), BounceClass::Transient);
    assert_eq!(classify_bounce("unrecognised provider text"), BounceClass::Unknown);

    assert!(BounceClass::Hard.is_terminal());
    assert!(BounceClass::Complaint.is_terminal());
    assert!(BounceClass::Transient.is_retryable());
    assert!(BounceClass::Soft.is_retryable());
    assert_eq!(BounceClass::parse("hard_bounce"), Some(BounceClass::Hard));
}

#[test]
fn retry_exhausts_after_bounded_attempts() {
    let mut state = AttemptState::default();
    for _ in 0..MAX_SEND_ATTEMPTS {
        assert!(!state.is_exhausted());
        state.record("timeout", BounceClass::Transient);
    }
    assert!(state.is_exhausted());
    assert_eq!(state.attempts, MAX_SEND_ATTEMPTS);

    let restored = AttemptState::from_json(Some(&state.to_json()));
    assert_eq!(restored, state);
    assert_eq!(retry_delay_millis(0), 2_000);
    assert_eq!(retry_delay_millis(1), 4_000);

    // A partially consumed budget is restored across passes: only the
    // remaining attempts may be spent, never a fresh full budget.
    let partial = json!({ "attempts": MAX_SEND_ATTEMPTS - 1 });
    let restored = AttemptState::from_json(Some(&partial));
    assert_eq!(MAX_SEND_ATTEMPTS - restored.attempts, 1);
    let exhausted = json!({ "attempts": MAX_SEND_ATTEMPTS });
    assert!(AttemptState::from_json(Some(&exhausted)).is_exhausted());
}

#[test]
fn terminal_bounce_exhausts_budget_immediately() {
    let mut state = AttemptState::default();
    state.record("550 user unknown", BounceClass::Hard);
    assert!(state.is_exhausted());
    assert_eq!(state.bounce_class, Some(BounceClass::Hard));
}

#[test]
fn unsubscribe_headers_are_rfc8058_one_click_pair() {
    let url = "https://example.test/api/marketing/unsubscribe/abc.def";
    let headers = unsubscribe_headers(url);
    assert!(headers.contains(&format!("List-Unsubscribe: <{url}>")));
    assert!(headers.contains("List-Unsubscribe-Post: List-Unsubscribe=One-Click"));
    assert!(headers.contains("\r\n"));
}

#[test]
fn unsubscribe_token_rejects_tampered_payload_and_empty_key() {
    let key = "unit-test-signing-key";
    let claims = UnsubscribeClaims {
        branch_id: Uuid::new_v4(),
        channel: MarketingChannel::Email,
        address: unique_address("tamper"),
    };
    let token = match sign_claims_with_key(key, &claims) {
        Some(token) => token,
        None => {
            log::warn!("test setup: signing key rejected valid claims");
            return;
        }
    };
    assert!(sign_claims_with_key("", &claims).is_none());

    let (signature, payload) = token.split_once('.').unwrap_or(("", ""));
    let tampered = format!("{signature}.{payload}ff");
    assert_eq!(
        verify_unsubscribe_token(key, &tampered).err(),
        Some(UnsubscribeError::BadSignature)
    );
    assert_eq!(
        verify_unsubscribe_token("", &token).err(),
        Some(UnsubscribeError::SigningUnavailable)
    );
}

#[test]
fn suppression_key_normalizes_address_on_both_sides() {
    let branch_id = Uuid::new_v4();
    let store = SuppressionStore::default();
    let address = unique_address("normalize");
    let upper = address.to_ascii_uppercase();
    assert!(store
        .add(SuppressionKey::new(
            branch_id,
            MarketingChannel::Email,
            &upper
        ))
        .is_ok());
    assert!(store.is_suppressed(&SuppressionKey::new(
        branch_id,
        MarketingChannel::Email,
        &format!("  {address}  ")
    )));
}

#[test]
fn consent_flag_is_channel_scoped() {
    let metadata = json!({
        "consent_email": true,
        "consent_whatsapp": false,
    });
    assert_eq!(
        channel_consent(None, Some(&metadata), MarketingChannel::Email),
        ConsentState::OptedIn
    );
    assert_eq!(
        channel_consent(None, Some(&metadata), MarketingChannel::Whatsapp),
        ConsentState::OptedOut
    );
    assert_eq!(
        channel_consent(None, Some(&metadata), MarketingChannel::Instagram),
        ConsentState::Unknown
    );
}

#[test]
fn retry_delay_grows_exponentially_and_is_capped() {
    assert_eq!(retry_delay_millis(0), 2_000);
    assert_eq!(retry_delay_millis(1), 4_000);
    assert_eq!(retry_delay_millis(2), 8_000);
    assert_eq!(retry_delay_millis(u32::MAX), retry_delay_millis(6));
}

#[test]
fn all_channels_failed_body_falls_back_when_no_errors_given() {
    let body = all_channels_failed_body(&serde_json::Map::new());
    let parsed: serde_json::Value =
        serde_json::from_str(&body).unwrap_or_else(|_| json!({}));
    assert_eq!(parsed["success"], false);
    assert_eq!(parsed["error"], "channel_error");
}

#[test]
fn complaint_classification_takes_precedence_over_hard_bounce() {
    assert_eq!(
        classify_bounce("spam complaint 550 5.1.1 mailbox does not exist"),
        BounceClass::Complaint
    );
}

#[test]
fn unsubscribe_base_url_uses_safe_configured_or_first_party_fallback() {
    assert_eq!(
        resolve_unsubscribe_base("https://example.test/", None),
        "https://example.test"
    );
    assert_eq!(resolve_unsubscribe_base("", None), FIRST_PARTY_UNSUBSCRIBE_BASE);
    assert_eq!(resolve_unsubscribe_base("stub", None), FIRST_PARTY_UNSUBSCRIBE_BASE);
    assert_eq!(
        resolve_unsubscribe_base("javascript:alert(1)", None),
        FIRST_PARTY_UNSUBSCRIBE_BASE
    );
    assert_eq!(
        resolve_unsubscribe_base("https://user:pass@example.test", None),
        FIRST_PARTY_UNSUBSCRIBE_BASE
    );
    assert_eq!(
        resolve_unsubscribe_base("", Some("https://self.test//")),
        "https://self.test"
    );
}

#[test]
fn durable_suppression_row_maps_to_normalized_key() {
    let branch_id = Uuid::new_v4();
    let address = unique_address("durable");
    let stored = format!("  {}  ", address.to_ascii_uppercase());
    let key = suppression_key_from_record(branch_id, "EMAIL", &stored);
    assert_eq!(
        key,
        Some(SuppressionKey::new(
            branch_id,
            MarketingChannel::Email,
            &address
        ))
    );
    assert_eq!(
        suppression_key_from_record(branch_id, "carrier-pigeon", &address),
        None
    );
}

#[test]
fn attempt_state_truncates_overlong_provider_error() {
    let mut state = AttemptState::default();
    state.record(&"x".repeat(2_048), BounceClass::Transient);
    let stored = state.to_json();
    let last = stored
        .get("last_error")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    assert!(last.chars().count() <= 513);
    assert!(last.ends_with('…'));
}
