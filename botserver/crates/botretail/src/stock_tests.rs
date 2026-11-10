//! Tests for the stock-level arithmetic the POS relies on.

use super::*;

fn level(quantity: i64, reserved: i64) -> StockLevel {
    StockLevel {
        branch_id: Uuid::new_v4(),
        variation_id: Uuid::new_v4(),
        quantity,
        reserved,
    }
}

#[test]
fn available_is_quantity_minus_reserved() {
    assert_eq!(level(100, 12).available(), 88);
    assert_eq!(level(0, 0).available(), 0);
    // An over-reserved shelf reports a negative availability, never a wrap.
    assert_eq!(level(2, 5).available(), -3);
}

#[test]
fn fulfilment_is_inclusive_at_the_available_boundary() {
    let shelf = level(10, 4);
    assert!(shelf.can_fulfill(6));
    assert!(!shelf.can_fulfill(7));
    assert!(shelf.can_fulfill(0));
}

#[test]
fn an_empty_shelf_fulfils_nothing_positive() {
    let empty = level(0, 0);
    assert!(empty.can_fulfill(0));
    assert!(!empty.can_fulfill(1));
}

#[test]
fn a_negative_request_is_never_fulfillable() {
    assert!(!level(10, 0).can_fulfill(-1));
}

#[test]
fn reservations_can_make_a_stocked_shelf_unfulfillable() {
    let shelf = level(10, 10);
    assert!(!shelf.can_fulfill(1));
}
