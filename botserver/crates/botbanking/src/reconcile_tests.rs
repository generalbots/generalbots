//! Tests for the reconciliation window and the bank↔platform matcher.

use super::*;
use super::super::platform::DeliveryPlatform;
use super::super::transaction::{ReconcileStatus, TransactionKind};
use rust_decimal_macros::dec;

fn bank(
    amount: Decimal,
    when: DateTime<Utc>,
    platform: Option<DeliveryPlatform>,
) -> BankTransaction {
    BankTransaction {
        id: Uuid::new_v4(),
        tenant_id: "t1".to_string(),
        side: TransactionSide::Bank,
        platform,
        external_id: "bank".to_string(),
        kind: TransactionKind::Credit,
        amount,
        currency: "BRL".to_string(),
        booked_at: when,
        counterparty: None,
        memo: None,
        status: ReconcileStatus::Unmatched,
        matched_with: None,
    }
}

fn platform_tx(amount: Decimal, when: DateTime<Utc>, platform: DeliveryPlatform) -> BankTransaction {
    let mut tx = bank(amount, when, Some(platform));
    tx.side = TransactionSide::Platform;
    tx.external_id = "platform".to_string();
    tx
}

#[test]
fn a_window_is_valid_when_it_ends_after_it_starts() {
    let start = Utc::now();
    let end = start + Duration::days(3);
    assert_eq!(window_duration(start, end), Ok(Duration::days(3)));
    assert_eq!(window_duration(start, start), Ok(Duration::zero()));
}

#[test]
fn an_inverted_window_is_rejected() {
    let start = Utc::now();
    let end = start - Duration::seconds(1);
    assert_eq!(
        window_duration(start, end),
        Err(ReconcileError::InvalidWindow { start, end })
    );
}

#[test]
fn an_exact_amount_on_the_same_day_is_an_exact_match() {
    let now = Utc::now();
    let bank_tx = bank(dec!(100.00), now, Some(DeliveryPlatform::IFood));
    let platform = platform_tx(dec!(100.00), now, DeliveryPlatform::IFood);
    let matches = match_transactions(&[bank_tx], &[platform], Duration::days(14));
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].confidence, MatchConfidence::Exact);
    assert_eq!(matches[0].delta, Decimal::ZERO);
}

#[test]
fn the_same_amount_a_day_later_drops_to_high_confidence() {
    let now = Utc::now();
    let bank_tx = bank(dec!(100.00), now, Some(DeliveryPlatform::Rappi));
    let platform = platform_tx(dec!(100.00), now - Duration::hours(30), DeliveryPlatform::Rappi);
    let matches = match_transactions(&[bank_tx], &[platform], Duration::days(14));
    assert_eq!(matches[0].confidence, MatchConfidence::High);
}

#[test]
fn a_cent_level_difference_is_medium_confidence() {
    let now = Utc::now();
    let bank_tx = bank(dec!(100.00), now, Some(DeliveryPlatform::Loggi));
    let platform = platform_tx(dec!(100.04), now, DeliveryPlatform::Loggi);
    let matches = match_transactions(&[bank_tx], &[platform], Duration::days(14));
    assert_eq!(matches[0].confidence, MatchConfidence::Medium);
    assert_eq!(matches[0].delta, dec!(0.04));
}

#[test]
fn a_difference_of_five_cents_is_low_confidence() {
    let now = Utc::now();
    let bank_tx = bank(dec!(100.00), now, Some(DeliveryPlatform::Loggi));
    let platform = platform_tx(dec!(100.05), now, DeliveryPlatform::Loggi);
    let matches = match_transactions(&[bank_tx], &[platform], Duration::days(14));
    assert_eq!(matches[0].confidence, MatchConfidence::Low);
}

#[test]
fn transactions_outside_the_window_never_match() {
    let now = Utc::now();
    let bank_tx = bank(dec!(100.00), now, Some(DeliveryPlatform::IFood));
    let too_far = platform_tx(dec!(100.00), now - Duration::days(15), DeliveryPlatform::IFood);
    assert!(match_transactions(&[bank_tx], &[too_far], Duration::days(14)).is_empty());
}

#[test]
fn the_window_boundary_itself_matches() {
    let now = Utc::now();
    let bank_tx = bank(dec!(100.00), now, Some(DeliveryPlatform::IFood));
    let edge = platform_tx(dec!(100.00), now - Duration::days(14), DeliveryPlatform::IFood);
    assert_eq!(
        match_transactions(&[bank_tx], &[edge], Duration::days(14)).len(),
        1
    );
}

#[test]
fn different_platforms_are_never_matched() {
    let now = Utc::now();
    let bank_tx = bank(dec!(100.00), now, Some(DeliveryPlatform::IFood));
    let other = platform_tx(dec!(100.00), now, DeliveryPlatform::Rappi);
    assert!(match_transactions(&[bank_tx], &[other], Duration::days(14)).is_empty());

    // A bank row without a platform has nothing to match against.
    let unlabelled = bank(dec!(100.00), now, None);
    let platform = platform_tx(dec!(100.00), now, DeliveryPlatform::IFood);
    assert!(match_transactions(&[unlabelled], &[platform], Duration::days(14)).is_empty());
}

#[test]
fn a_platform_settlement_is_consumed_once() {
    let now = Utc::now();
    let first = bank(dec!(100.00), now, Some(DeliveryPlatform::IFood));
    let second = bank(dec!(100.00), now, Some(DeliveryPlatform::IFood));
    let platform = platform_tx(dec!(100.00), now, DeliveryPlatform::IFood);
    let matches = match_transactions(&[first, second], &[platform], Duration::days(14));
    assert_eq!(matches.len(), 1);
}

#[test]
fn the_closest_amount_wins_when_several_settlements_are_in_range() {
    let now = Utc::now();
    let bank_tx = bank(dec!(100.00), now, Some(DeliveryPlatform::IFood));
    let near = platform_tx(dec!(100.01), now, DeliveryPlatform::IFood);
    let exact = platform_tx(dec!(100.00), now, DeliveryPlatform::IFood);
    let matches = match_transactions(&[bank_tx], &[near, exact], Duration::days(14));
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].confidence, MatchConfidence::Exact);
}

#[test]
fn rows_on_the_wrong_side_are_ignored() {
    let now = Utc::now();
    let platform_side = platform_tx(dec!(100.00), now, DeliveryPlatform::IFood);
    let bank_side = bank(dec!(100.00), now, Some(DeliveryPlatform::IFood));
    // The bank slice holds a platform row and vice versa: nothing matches.
    assert!(match_transactions(&[platform_side], &[bank_side], Duration::days(14)).is_empty());
}

#[test]
fn empty_inputs_produce_no_matches() {
    assert!(match_transactions(&[], &[], Duration::days(1)).is_empty());
}
