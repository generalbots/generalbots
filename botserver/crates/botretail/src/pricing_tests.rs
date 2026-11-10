//! Tests for price-list break selection and the effective window.

use super::*;
use rust_decimal_macros::dec;

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
}

fn entry(variation_id: Uuid, tier: PriceTier, price: Decimal, min_qty: u32) -> PriceListEntry {
    PriceListEntry { variation_id, tier, price, min_qty }
}

fn list(entries: Vec<PriceListEntry>, effective_until: Option<NaiveDate>) -> PriceList {
    PriceList {
        id: Uuid::new_v4(),
        tenant_id: "t1".to_string(),
        name: "Default".to_string(),
        currency: "BRL".to_string(),
        effective_from: date(2026, 1, 1),
        effective_until,
        entries,
        created_at: Utc::now(),
    }
}

fn wholesale_breaks(variation: Uuid) -> Vec<PriceListEntry> {
    vec![
        entry(variation, PriceTier::Wholesale, dec!(10.00), 100),
        entry(variation, PriceTier::Wholesale, dec!(12.00), 1),
        entry(variation, PriceTier::Retail, dec!(20.00), 1),
    ]
}

#[test]
fn quantity_selects_the_highest_break_it_reaches() {
    let variation = Uuid::new_v4();
    let price_list = list(wholesale_breaks(variation), None);

    let cases = [
        (1u32, dec!(12.00)),
        (99, dec!(12.00)),
        (100, dec!(10.00)),
        (1000, dec!(10.00)),
    ];
    for (quantity, expected) in cases {
        assert_eq!(
            price_list.price_for_quantity(variation, PriceTier::Wholesale, quantity),
            Some(expected),
            "quantity {quantity}"
        );
    }
}

#[test]
fn a_quantity_below_every_break_falls_back_to_the_cheapest_break_entry() {
    let variation = Uuid::new_v4();
    // Only a bulk break exists: a single unit still gets the list price.
    let price_list = list(
        vec![entry(variation, PriceTier::Wholesale, dec!(9.00), 50)],
        None,
    );
    assert_eq!(
        price_list.price_for_quantity(variation, PriceTier::Wholesale, 0),
        Some(dec!(9.00))
    );
}

#[test]
fn tiers_and_variations_do_not_leak_into_each_other() {
    let variation = Uuid::new_v4();
    let price_list = list(wholesale_breaks(variation), None);

    assert_eq!(
        price_list.price_for_quantity(variation, PriceTier::Retail, 500),
        Some(dec!(20.00))
    );
    assert_eq!(price_list.price_for_quantity(variation, PriceTier::Vip, 500), None);
    assert_eq!(
        price_list.price_for_quantity(Uuid::new_v4(), PriceTier::Wholesale, 500),
        None
    );
}

#[test]
fn the_effective_window_is_inclusive_on_both_bounds() {
    let variation = Uuid::new_v4();
    let price_list = list(wholesale_breaks(variation), Some(date(2026, 12, 31)));

    assert!(!price_list.is_effective_on(date(2025, 12, 31)));
    assert!(price_list.is_effective_on(date(2026, 1, 1)));
    assert!(price_list.is_effective_on(date(2026, 6, 15)));
    assert!(price_list.is_effective_on(date(2026, 12, 31)));
    assert!(!price_list.is_effective_on(date(2027, 1, 1)));
}

#[test]
fn an_open_ended_window_never_expires() {
    let variation = Uuid::new_v4();
    let price_list = list(wholesale_breaks(variation), None);
    assert!(price_list.is_effective_on(date(2030, 5, 20)));
}
