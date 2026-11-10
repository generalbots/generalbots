//! Tests for promotion applicability and discount math.

use super::*;
use rust_decimal_macros::dec;

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
}

fn promotion(kind: DiscountKind, value: Decimal, spec: Option<&str>) -> Promotion {
    Promotion {
        id: Uuid::new_v4(),
        tenant_id: "t1".to_string(),
        name: "Promo".to_string(),
        kind,
        value,
        bundle_spec: spec.map(str::to_string),
        applies_to: vec![],
        window: PromotionWindow {
            start: date(2026, 3, 1),
            end: date(2026, 3, 31),
            branch_ids: vec![],
        },
        active: true,
        created_at: Utc::now(),
    }
}

#[test]
fn percentage_discounts_are_capped_and_rounded_to_cents() {
    let cases = [
        (dec!(10), dec!(100.00), 1i64, dec!(10.00)),
        (dec!(7.5), dec!(999.00), 1, dec!(74.93)),
        (dec!(0), dec!(100.00), 1, dec!(0.00)),
        // More than the whole subtotal can never exceed the subtotal.
        (dec!(150), dec!(100.00), 1, dec!(100.00)),
        (dec!(-5), dec!(100.00), 1, dec!(0.00)),
        (dec!(10), dec!(50.00), 3, dec!(15.00)),
    ];
    for (value, unit_price, quantity, expected) in cases {
        let promo = promotion(DiscountKind::Percentage, value, None);
        assert_eq!(
            promo.discount_for(quantity, unit_price),
            expected,
            "value {value} qty {quantity}"
        );
    }
}

#[test]
fn fixed_amount_discounts_never_exceed_the_line() {
    let cases = [
        (dec!(20), dec!(100.00), 1i64, dec!(20.00)),
        (dec!(200), dec!(100.00), 1, dec!(100.00)),
        (dec!(-20), dec!(100.00), 1, dec!(0.00)),
        (dec!(20), dec!(60.00), 2, dec!(20.00)),
    ];
    for (value, unit_price, quantity, expected) in cases {
        let promo = promotion(DiscountKind::FixedAmount, value, None);
        assert_eq!(promo.discount_for(quantity, unit_price), expected);
    }
}

#[test]
fn pay_for_nm_charges_only_the_paid_units_of_each_group() {
    let promo = promotion(DiscountKind::PayForNM, dec!(0), Some("3/2"));
    let cases = [
        (0i64, dec!(0.00)),
        (1, dec!(0.00)),
        (2, dec!(0.00)),
        (3, dec!(10.00)),
        (5, dec!(10.00)),
        (6, dec!(20.00)),
        (9, dec!(30.00)),
    ];
    for (quantity, expected) in cases {
        assert_eq!(promo.discount_for(quantity, dec!(10.00)), expected, "qty {quantity}");
    }
}

#[test]
fn a_malformed_pay_for_nm_spec_discounts_nothing() {
    for spec in [None, Some(""), Some("3-2"), Some("3/"), Some("0/0"), Some("2/3"), Some("x/y")] {
        let promo = promotion(DiscountKind::PayForNM, dec!(0), spec);
        assert_eq!(
            promo.discount_for(9, dec!(10.00)),
            Decimal::ZERO,
            "spec {spec:?}"
        );
    }
}

#[test]
fn bundle_pricing_is_resolved_before_the_line_is_discounted() {
    let promo = promotion(DiscountKind::Bundle, dec!(59.90), Some("A+B"));
    assert_eq!(promo.discount_for(2, dec!(40.00)), Decimal::ZERO);
}

#[test]
fn an_empty_or_negative_line_yields_no_discount() {
    for kind in [
        DiscountKind::Percentage,
        DiscountKind::FixedAmount,
        DiscountKind::PayForNM,
    ] {
        let promo = promotion(kind, dec!(10), Some("3/2"));
        assert_eq!(promo.discount_for(0, dec!(10.00)), Decimal::ZERO);
        assert_eq!(promo.discount_for(-3, dec!(10.00)), Decimal::ZERO);
        assert_eq!(promo.discount_for(3, dec!(0.00)), Decimal::ZERO);
    }
}

#[test]
fn an_empty_branch_list_applies_everywhere() {
    let promo = promotion(DiscountKind::Percentage, dec!(10), None);
    assert!(promo.applies_to_branch(Uuid::new_v4(), date(2026, 3, 15)));
}

#[test]
fn a_branch_list_restricts_the_promotion_to_those_branches() {
    let listed = Uuid::new_v4();
    let mut promo = promotion(DiscountKind::Percentage, dec!(10), None);
    promo.window.branch_ids = vec![listed];

    assert!(promo.applies_to_branch(listed, date(2026, 3, 15)));
    assert!(!promo.applies_to_branch(Uuid::new_v4(), date(2026, 3, 15)));
    // The branch restriction never widens the date window.
    assert!(!promo.applies_to_branch(listed, date(2026, 4, 1)));
}

#[test]
fn an_inactive_promotion_never_applies() {
    let mut promo = promotion(DiscountKind::Percentage, dec!(10), None);
    promo.active = false;
    assert!(!promo.applies_to_branch(Uuid::new_v4(), date(2026, 3, 15)));
    assert!(!promo.is_applicable(date(2026, 3, 15)));
}

#[test]
fn the_window_bounds_are_inclusive() {
    let promo = promotion(DiscountKind::Percentage, dec!(10), None);
    assert!(promo.is_applicable(date(2026, 3, 1)));
    assert!(promo.is_applicable(date(2026, 3, 31)));
    assert!(!promo.is_applicable(date(2026, 2, 28)));
    assert!(!promo.is_applicable(date(2026, 4, 1)));
}
