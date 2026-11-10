//! Campaign rate arithmetic. Every rate is a percentage of delivered mail, so
//! a campaign that never delivered anything must report `0.0` instead of
//! dividing by zero.
use super::*;

#[test]
fn open_and_click_rates_are_percentages_of_delivered_mail() {
    assert_eq!(calculate_open_rate(200, 50), 25.0);
    assert_eq!(calculate_click_rate(200, 10), 5.0);
}

#[test]
fn a_full_house_reads_as_one_hundred_percent() {
    assert_eq!(calculate_open_rate(10, 10), 100.0);
    assert_eq!(calculate_click_rate(10, 10), 100.0);
}

#[test]
fn nothing_delivered_reports_zero_instead_of_dividing_by_zero() {
    assert_eq!(calculate_open_rate(0, 0), 0.0);
    assert_eq!(calculate_click_rate(0, 0), 0.0);
    assert_eq!(calculate_open_rate(0, 7), 0.0);
    assert_eq!(calculate_click_rate(0, 7), 0.0);
}

#[test]
fn a_negative_delivery_count_is_treated_as_nothing_delivered() {
    assert_eq!(calculate_open_rate(-5, 2), 0.0);
    assert_eq!(calculate_click_rate(-5, 2), 0.0);
}

#[test]
fn a_fractional_rate_is_preserved() {
    let rate = calculate_open_rate(3, 1);
    assert!((rate - 33.333_333).abs() < 1e-5, "rate was {rate}");
}
