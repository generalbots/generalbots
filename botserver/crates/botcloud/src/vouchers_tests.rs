//! #2069 CLD-7/CLD-8 — voucher code shape and the redemption policy that
//! decides whether a trial is granted. Both used to be reachable only through
//! PostgreSQL, so neither was ever asserted.
use super::*;
use rand::SeedableRng;

fn seeded(seed: u64) -> rand::rngs::StdRng {
    rand::rngs::StdRng::seed_from_u64(seed)
}

#[test]
fn cld7_code_is_gb_days_dash_eight_alphanumerics() {
    for days in [1, 14, 30, 180] {
        let code = voucher_code_with(days, &mut seeded(7));
        let expected_prefix = format!("GB{days}-");
        assert!(code.starts_with(&expected_prefix), "{code}");

        let suffix = &code[expected_prefix.len()..];
        assert_eq!(suffix.chars().count(), 8, "{code}");
        assert!(
            suffix.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()),
            "{code} must stay A-Z0-9"
        );
    }
}

#[test]
fn cld7_the_same_seed_yields_the_same_code() {
    assert_eq!(voucher_code_with(14, &mut seeded(42)), voucher_code_with(14, &mut seeded(42)));
}

#[test]
fn cld7_different_seeds_yield_different_suffixes() {
    let a = voucher_code_with(14, &mut seeded(1));
    let b = voucher_code_with(14, &mut seeded(2));
    assert_ne!(a, b);
}

#[test]
fn cld7_out_of_range_day_counts_still_format() {
    // The range check lives in the request handler; the formatter never panics.
    assert!(voucher_code_with(0, &mut seeded(1)).starts_with("GB0-"));
    assert!(voucher_code_with(-5, &mut seeded(1)).starts_with("GB-5-"));
    assert!(voucher_code_with(i32::MAX, &mut seeded(1)).starts_with("GB2147483647-"));
}

#[test]
fn cld7_the_public_generator_keeps_the_shape() {
    let code = generate_voucher_code(30);
    assert!(code.starts_with("GB30-"), "{code}");
    assert_eq!(code.chars().count(), "GB30-".len() + 8, "{code}");
}

#[test]
fn cld8_trial_days_are_bounded_to_one_through_one_eighty() {
    assert!(TRIAL_DAYS_RANGE.contains(&1));
    assert!(TRIAL_DAYS_RANGE.contains(&180));
    assert!(!TRIAL_DAYS_RANGE.contains(&0));
    assert!(!TRIAL_DAYS_RANGE.contains(&181));
    assert!(!TRIAL_DAYS_RANGE.contains(&-1));
    assert!(!TRIAL_DAYS_RANGE.contains(&i32::MAX));
}

#[test]
fn cld8_a_valid_voucher_is_accepted() {
    let now = Utc::now();
    assert_eq!(redemption_rejection(0, 1, None, false, now), None);
    assert_eq!(redemption_rejection(1, 3, Some(now + Duration::days(1)), false, now), None);
}

#[test]
fn cld8_an_exhausted_voucher_is_refused() {
    let now = Utc::now();
    let reason = redemption_rejection(1, 1, None, false, now);
    assert_eq!(reason, Some("Voucher has reached its maximum uses"));
    // Over-use (a race that slipped through) is still refused.
    assert_eq!(redemption_rejection(5, 1, None, false, now), reason);
    assert_eq!(redemption_rejection(1, 0, None, false, now), reason);
}

#[test]
fn cld8_an_expired_voucher_is_refused_but_the_boundary_second_is_not() {
    let now = Utc::now();
    assert_eq!(
        redemption_rejection(0, 1, Some(now - Duration::seconds(1)), false, now),
        Some("Voucher has expired")
    );
    // `now > expiry` — exactly at expiry the voucher still works.
    assert_eq!(redemption_rejection(0, 1, Some(now), false, now), None);
}

#[test]
fn cld8_a_duplicate_redemption_is_refused() {
    let now = Utc::now();
    assert_eq!(
        redemption_rejection(0, 5, None, true, now),
        Some("Voucher already redeemed by this user")
    );
}

#[test]
fn cld8_exhaustion_wins_over_expiry_and_duplication() {
    // The endpoint reports the first applicable reason; the order is part of
    // the contract because it is what the user sees.
    let now = Utc::now();
    let expired = Some(now - Duration::days(1));
    assert_eq!(
        redemption_rejection(1, 1, expired, true, now),
        Some("Voucher has reached its maximum uses")
    );
    assert_eq!(
        redemption_rejection(0, 1, expired, true, now),
        Some("Voucher has expired")
    );
}
