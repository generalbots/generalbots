//! Tests for the POS validation rules and stock check.

use super::*;
use crate::stock::StockLevel;
use rust_decimal_macros::dec;

fn line(variation_id: Uuid, quantity: i64, unit_price: Decimal, discount: Decimal) -> PosLineItem {
    let mut item = PosLineItem {
        id: Uuid::new_v4(),
        variation_id,
        quantity,
        unit_price,
        discount,
        total: Decimal::ZERO,
    };
    item.total = item.computed_total();
    item
}

fn payment(amount: Decimal) -> PosPayment {
    PosPayment {
        id: Uuid::new_v4(),
        method: PaymentMethod::Pix,
        amount,
        auth_code: None,
    }
}

fn sale(lines: Vec<PosLineItem>, payments: Vec<PosPayment>) -> PosSale {
    PosSale {
        id: Uuid::new_v4(),
        tenant_id: "t1".to_string(),
        branch_id: Uuid::new_v4(),
        cashier_id: "u1".to_string(),
        customer_id: None,
        lines,
        payments,
        finalized_at: Utc::now(),
        nfce_key: None,
    }
}

fn level(variation_id: Uuid, quantity: i64, reserved: i64) -> StockLevel {
    StockLevel { branch_id: Uuid::new_v4(), variation_id, quantity, reserved }
}

#[test]
fn a_line_total_is_quantity_times_price_minus_discount() {
    let item = line(Uuid::new_v4(), 2, dec!(10.00), dec!(3.00));
    assert_eq!(item.computed_total(), dec!(17.00));
    assert_eq!(item.total, dec!(17.00));
}

#[test]
fn a_sale_without_lines_or_value_is_rejected() {
    assert_eq!(sale(vec![], vec![]).validate(), Err(PosError::ZeroTotal));

    let zero = line(Uuid::new_v4(), 1, dec!(10.00), dec!(10.00));
    assert_eq!(sale(vec![zero], vec![payment(dec!(0.00))]).validate(), Err(PosError::ZeroTotal));

    let negative = line(Uuid::new_v4(), 1, dec!(10.00), dec!(25.00));
    assert_eq!(
        sale(vec![negative], vec![payment(dec!(1.00))]).validate(),
        Err(PosError::ZeroTotal)
    );
}

#[test]
fn an_underpaid_sale_reports_both_amounts() {
    let item = line(Uuid::new_v4(), 1, dec!(100.00), Decimal::ZERO);
    let result = sale(vec![item], vec![payment(dec!(60.00))]).validate();
    assert_eq!(
        result,
        Err(PosError::PaymentMismatch { paid: dec!(60.00), gross: dec!(100.00) })
    );
}

#[test]
fn a_sale_with_no_payment_at_all_is_a_mismatch_not_a_zero_total() {
    let item = line(Uuid::new_v4(), 1, dec!(100.00), Decimal::ZERO);
    let result = sale(vec![item], vec![]).validate();
    assert_eq!(
        result,
        Err(PosError::PaymentMismatch { paid: Decimal::ZERO, gross: dec!(100.00) })
    );
}

#[test]
fn an_exact_payment_validates_with_no_change() {
    let item = line(Uuid::new_v4(), 1, dec!(100.00), Decimal::ZERO);
    let pos = sale(vec![item], vec![payment(dec!(100.00))]);
    assert_eq!(pos.validate(), Ok(()));
    assert_eq!(pos.change(), Decimal::ZERO);
    assert_eq!(pos.balance(), Decimal::ZERO);
}

#[test]
fn overpaying_is_allowed_and_becomes_change() {
    let item = line(Uuid::new_v4(), 1, dec!(100.00), Decimal::ZERO);
    let pos = sale(
        vec![item],
        vec![payment(dec!(60.00)), payment(dec!(50.00))],
    );
    assert_eq!(pos.validate(), Ok(()));
    assert_eq!(pos.paid(), dec!(110.00));
    assert_eq!(pos.change(), dec!(10.00));
    assert_eq!(pos.balance(), dec!(-10.00));
}

#[test]
fn a_fully_stocked_sale_passes_the_stock_check() {
    let variation = Uuid::new_v4();
    let pos = sale(vec![line(variation, 3, dec!(10.00), Decimal::ZERO)], vec![]);
    assert_eq!(pos.ensure_stock(&[level(variation, 3, 0)]), Ok(()));
}

#[test]
fn reservations_reduce_what_the_sale_may_take() {
    let variation = Uuid::new_v4();
    let pos = sale(vec![line(variation, 3, dec!(10.00), Decimal::ZERO)], vec![]);
    assert_eq!(
        pos.ensure_stock(&[level(variation, 5, 3)]),
        Err(PosError::OutOfStock(variation))
    );
}

#[test]
fn a_variation_without_a_level_has_nothing_on_hand() {
    let variation = Uuid::new_v4();
    let pos = sale(vec![line(variation, 1, dec!(10.00), Decimal::ZERO)], vec![]);
    assert_eq!(pos.ensure_stock(&[]), Err(PosError::OutOfStock(variation)));
}

#[test]
fn lines_for_the_same_variation_are_aggregated() {
    let variation = Uuid::new_v4();
    let pos = sale(
        vec![
            line(variation, 3, dec!(10.00), Decimal::ZERO),
            line(variation, 3, dec!(9.00), Decimal::ZERO),
        ],
        vec![],
    );
    assert_eq!(
        pos.ensure_stock(&[level(variation, 5, 0)]),
        Err(PosError::OutOfStock(variation))
    );
    assert_eq!(pos.ensure_stock(&[level(variation, 6, 0)]), Ok(()));
}

#[test]
fn a_non_positive_line_requests_no_stock() {
    let variation = Uuid::new_v4();
    let pos = sale(vec![line(variation, 0, dec!(10.00), Decimal::ZERO)], vec![]);
    assert_eq!(pos.ensure_stock(&[]), Ok(()));
}
