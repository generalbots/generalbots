//! IP reputation scoring used to pick the sending address. A pool with no
//! traffic must score as healthy, and a bad bounce/complaint record must floor
//! the score at zero instead of going negative.
use super::*;

fn reputation(delivered: i64, bounced: i64, complained: i64) -> IpReputation {
    IpReputation {
        id: Uuid::nil(),
        org_id: Uuid::nil(),
        ip: "203.0.113.10".to_string(),
        provider: "test".to_string(),
        delivered,
        bounced,
        complained,
        window_start: Utc::now(),
        updated_at: None,
    }
}

#[test]
fn an_unused_address_has_a_perfect_delivery_rate_and_no_bad_rates() {
    let fresh = reputation(0, 0, 0);

    assert_eq!(IpRouter::calculate_delivery_rate(&fresh), 1.0);
    assert_eq!(IpRouter::calculate_bounce_rate(&fresh), 0.0);
    assert_eq!(IpRouter::calculate_complaint_rate(&fresh), 0.0);
    assert_eq!(IpRouter::calculate_score(&fresh), 1.0);
}

#[test]
fn a_clean_record_keeps_the_full_score() {
    let clean = reputation(100, 0, 0);

    assert_eq!(IpRouter::calculate_delivery_rate(&clean), 1.0);
    assert_eq!(IpRouter::calculate_score(&clean), 1.0);
}

#[test]
fn rates_are_shares_of_the_attempted_volume() {
    let mixed = reputation(90, 10, 0);

    assert!((IpRouter::calculate_delivery_rate(&mixed) - 0.9).abs() < 1e-9);
    assert!((IpRouter::calculate_bounce_rate(&mixed) - 0.1).abs() < 1e-9);
}

#[test]
fn complaints_are_a_share_of_delivered_mail_only() {
    assert_eq!(IpRouter::calculate_complaint_rate(&reputation(0, 5, 3)), 0.0);
    assert!((IpRouter::calculate_complaint_rate(&reputation(100, 0, 5)) - 0.05).abs() < 1e-9);
}

#[test]
fn a_bounce_heavy_address_scores_below_a_clean_one() {
    let clean = reputation(990, 10, 0);
    let bad = reputation(700, 300, 0);

    assert!(IpRouter::calculate_score(&clean) > IpRouter::calculate_score(&bad));
    assert!((IpRouter::calculate_score(&clean) - 0.89).abs() < 1e-9);
}

#[test]
fn the_score_never_goes_negative() {
    assert_eq!(IpRouter::calculate_score(&reputation(1, 9, 0)), 0.0);
    assert_eq!(IpRouter::calculate_score(&reputation(100, 0, 50)), 0.0);
}
