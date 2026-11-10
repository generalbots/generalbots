use super::*;
use chrono::TimeZone;

fn product(price_cents: i64, cost_cents: i64) -> Product {
    Product {
        id: Uuid::new_v4(),
        sku: "SKU-1".into(),
        name: "Test".into(),
        description: Some("Acme brand coffee".into()),
        category: Some("food".into()),
        barcode: None,
        unit: "un".into(),
        weight_grams: None,
        cost_cents,
        price_cents,
        tax_rate: 0.0,
        active: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn rule(strategy: PricingStrategy) -> PriceRule {
    PriceRule {
        id: Uuid::new_v4(),
        price_list_id: Uuid::new_v4(),
        product_id: None,
        category: None,
        strategy,
        markup_percent: None,
        fixed_price_cents: None,
        min_price_cents: None,
        max_price_cents: None,
        min_quantity: 1,
        active: true,
    }
}

fn now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 10, 12, 0, 0)
        .single()
        .expect("valid timestamp")
}

fn promo(discount_type: DiscountType, discount_value: f64) -> Promotion {
    Promotion {
        id: Uuid::new_v4(),
        name: "promo".into(),
        code: None,
        discount_type,
        discount_value,
        applies_to: PromotionTarget::AllProducts,
        min_purchase_cents: None,
        valid_from: now() - chrono::Duration::days(1),
        valid_until: now() + chrono::Duration::days(1),
        max_uses: None,
        current_uses: 0,
        active: true,
    }
}

#[test]
fn without_rules_the_list_price_is_the_base_price() {
    let engine = PricingEngine::new();
    let p = product(1000, 600);
    assert_eq!(engine.base_price(&p, &[], 1), 1000);
}

#[test]
fn rules_targeting_another_product_or_category_are_ignored() {
    let engine = PricingEngine::new();
    let p = product(1000, 600);

    let mut other_product = rule(PricingStrategy::Fixed);
    other_product.fixed_price_cents = Some(100);
    other_product.product_id = Some(Uuid::new_v4());
    assert_eq!(engine.base_price(&p, &[other_product], 1), 1000);

    let mut other_category = rule(PricingStrategy::Fixed);
    other_category.fixed_price_cents = Some(100);
    other_category.category = Some("drinks".into());
    assert_eq!(engine.base_price(&p, &[other_category], 1), 1000);

    let mut inactive = rule(PricingStrategy::Fixed);
    inactive.fixed_price_cents = Some(100);
    inactive.active = false;
    assert_eq!(engine.base_price(&p, &[inactive], 1), 1000);

    let mut matching = rule(PricingStrategy::Fixed);
    matching.fixed_price_cents = Some(100);
    matching.category = Some("food".into());
    assert_eq!(engine.base_price(&p, &[matching], 1), 100);
}

#[test]
fn a_rule_below_its_minimum_quantity_is_skipped() {
    let engine = PricingEngine::new();
    let p = product(1000, 600);
    let mut wholesale = rule(PricingStrategy::Fixed);
    wholesale.fixed_price_cents = Some(700);
    wholesale.min_quantity = 10;

    assert_eq!(engine.base_price(&p, &[wholesale.clone()], 9), 1000);
    assert_eq!(engine.base_price(&p, &[wholesale], 10), 700);
}

#[test]
fn cost_plus_markup_is_applied_to_the_unit_cost() {
    let engine = PricingEngine::new();
    let p = product(1000, 600);
    let mut cost_plus = rule(PricingStrategy::CostPlus);
    cost_plus.markup_percent = Some(25.0);
    assert_eq!(engine.base_price(&p, &[cost_plus], 1), 750);

    let mut no_markup = rule(PricingStrategy::CostPlus);
    no_markup.markup_percent = None;
    assert_eq!(engine.base_price(&p, &[no_markup], 1), 600);
}

#[test]
fn tiered_rules_discount_by_quantity_band() {
    let engine = PricingEngine::new();
    let p = product(1000, 600);
    let mut tiered = rule(PricingStrategy::Tiered);
    tiered.fixed_price_cents = Some(1000);

    // The tier bands are 10 and 100 units; each subtracts a fixed amount.
    assert_eq!(engine.base_price(&p, &[tiered.clone()], 9), 1000);
    assert_eq!(engine.base_price(&p, &[tiered.clone()], 10), 950);
    assert_eq!(engine.base_price(&p, &[tiered.clone()], 99), 950);
    assert_eq!(engine.base_price(&p, &[tiered], 100), 900);
}

#[test]
fn the_cheapest_candidate_wins_and_prices_never_go_negative() {
    let engine = PricingEngine::new();
    let p = product(1000, 600);

    let mut ten_percent_off = rule(PricingStrategy::Fixed);
    ten_percent_off.fixed_price_cents = Some(900);
    let mut half_price = rule(PricingStrategy::Fixed);
    half_price.fixed_price_cents = Some(500);
    assert_eq!(engine.base_price(&p, &[ten_percent_off, half_price], 1), 500);

    let mut negative = rule(PricingStrategy::Fixed);
    negative.fixed_price_cents = Some(-250);
    assert_eq!(engine.base_price(&p, &[negative], 1), 0);
}

#[test]
fn a_fixed_rule_without_a_price_falls_back_to_the_list_price() {
    let engine = PricingEngine::new();
    let p = product(1000, 600);
    let mut fixed = rule(PricingStrategy::Fixed);
    fixed.fixed_price_cents = None;
    assert_eq!(engine.base_price(&p, &[fixed], 1), 1000);
}

#[test]
fn percent_and_fixed_amount_promotions_reduce_the_price() {
    let engine = PricingEngine::new();
    let p = product(1000, 600);
    assert_eq!(
        engine.apply_promotions(1000, &p, &[promo(DiscountType::Percent, 10.0)], 1, now()),
        900
    );
    assert_eq!(
        engine.apply_promotions(1000, &p, &[promo(DiscountType::FixedAmount, 250.0)], 1, now()),
        750
    );
    // Free shipping does not change the unit price.
    assert_eq!(
        engine.apply_promotions(1000, &p, &[promo(DiscountType::FreeShipping, 1.0)], 1, now()),
        1000
    );
}

#[test]
fn buy_x_get_y_only_applies_from_its_quantity_threshold() {
    let engine = PricingEngine::new();
    let p = product(1000, 600);
    // discount_value carries the minimum quantity for BuyXGetY.
    let buy_three = promo(DiscountType::BuyXGetY, 3.0);
    assert_eq!(engine.apply_promotions(1000, &p, &[buy_three.clone()], 2, now()), 1000);
    assert_eq!(engine.apply_promotions(1000, &p, &[buy_three], 3, now()), 500);
}

#[test]
fn promotions_outside_their_window_or_usage_cap_are_ignored() {
    let engine = PricingEngine::new();
    let p = product(1000, 600);

    let mut expired = promo(DiscountType::Percent, 50.0);
    expired.valid_until = now() - chrono::Duration::seconds(1);
    assert_eq!(engine.apply_promotions(1000, &p, &[expired], 1, now()), 1000);

    let mut future = promo(DiscountType::Percent, 50.0);
    future.valid_from = now() + chrono::Duration::seconds(1);
    assert_eq!(engine.apply_promotions(1000, &p, &[future], 1, now()), 1000);

    // Boundaries are inclusive.
    let mut starting_now = promo(DiscountType::Percent, 50.0);
    starting_now.valid_from = now();
    starting_now.valid_until = now();
    assert_eq!(engine.apply_promotions(1000, &p, &[starting_now], 1, now()), 500);

    let mut exhausted = promo(DiscountType::Percent, 50.0);
    exhausted.max_uses = Some(2);
    exhausted.current_uses = 2;
    assert_eq!(engine.apply_promotions(1000, &p, &[exhausted], 1, now()), 1000);

    let mut inactive = promo(DiscountType::Percent, 50.0);
    inactive.active = false;
    assert_eq!(engine.apply_promotions(1000, &p, &[inactive], 1, now()), 1000);
}

#[test]
fn the_minimum_purchase_is_a_strict_lower_bound() {
    let engine = PricingEngine::new();
    let p = product(1000, 600);
    let mut minimum = promo(DiscountType::FixedAmount, 100.0);
    minimum.min_purchase_cents = Some(1000);

    // Equal to the minimum is accepted; one cent below is not.
    assert_eq!(engine.apply_promotions(1000, &p, &[minimum.clone()], 1, now()), 900);
    assert_eq!(engine.apply_promotions(999, &p, &[minimum], 1, now()), 999);
}

#[test]
fn promotion_targets_are_matched_by_product_category_and_brand() {
    let engine = PricingEngine::new();
    let p = product(1000, 600);

    let mut by_category = promo(DiscountType::Percent, 10.0);
    by_category.applies_to = PromotionTarget::Category("food".into());
    assert_eq!(engine.apply_promotions(1000, &p, &[by_category], 1, now()), 900);

    let mut wrong_category = promo(DiscountType::Percent, 10.0);
    wrong_category.applies_to = PromotionTarget::Category("drinks".into());
    assert_eq!(engine.apply_promotions(1000, &p, &[wrong_category], 1, now()), 1000);

    let mut by_product = promo(DiscountType::Percent, 10.0);
    by_product.applies_to = PromotionTarget::Product(p.id);
    assert_eq!(engine.apply_promotions(1000, &p, &[by_product], 1, now()), 900);

    // Brand targeting is a substring match on the description.
    let mut by_brand = promo(DiscountType::Percent, 10.0);
    by_brand.applies_to = PromotionTarget::Brand("Acme".into());
    assert_eq!(engine.apply_promotions(1000, &p, &[by_brand], 1, now()), 900);
}

#[test]
fn the_best_discount_wins_and_the_price_never_goes_negative() {
    let engine = PricingEngine::new();
    let p = product(1000, 600);
    let ten = promo(DiscountType::Percent, 10.0);
    let thirty = promo(DiscountType::Percent, 30.0);
    assert_eq!(engine.apply_promotions(1000, &p, &[ten, thirty], 1, now()), 700);

    // A fixed discount larger than the price floors at zero.
    let huge = promo(DiscountType::FixedAmount, 5000.0);
    assert_eq!(engine.apply_promotions(1000, &p, &[huge], 1, now()), 0);
}

#[test]
fn a_quote_floors_the_promotion_at_the_unit_cost() {
    let engine = PricingEngine::new();
    let p = product(1000, 600);
    let half = promo(DiscountType::Percent, 50.0);
    let quote = engine.quote(&p, &[], &[half], 1, now());

    assert_eq!(quote.list_price_cents, 1000);
    assert_eq!(quote.final_price_cents, 600);
    assert_eq!(quote.discount_cents, 400);
    assert_eq!(quote.total_cents, 600);
}

#[test]
fn tax_is_rounded_to_the_nearest_cent_and_added_to_the_total() {
    let engine = PricingEngine::new();
    let mut p = product(999, 600);
    p.tax_rate = 0.075;

    let quote = engine.quote(&p, &[], &[], 1, now());
    // 999 * 0.075 = 74.925 -> 75 (round half up), not truncated to 74.
    assert_eq!(quote.tax_cents, 75);
    assert_eq!(quote.total_cents, 999 + 75);
}

#[test]
fn a_zero_tax_rate_produces_no_tax() {
    let engine = PricingEngine::new();
    let p = product(1000, 600);
    let quote = engine.quote(&p, &[], &[], 1, now());
    assert_eq!(quote.tax_cents, 0);
    assert_eq!(quote.total_cents, 1000);
    assert_eq!(quote.applied_rule_id, None);
    assert_eq!(quote.applied_promotion_id, None);
}
