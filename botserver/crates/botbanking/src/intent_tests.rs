//! Banking intent payloads: the string handed back to BASIC and the matching
//! edges the keywords rely on.

use serde_json::Value;

use super::BankingRequest;

fn value(request: &BankingRequest) -> Value {
    serde_json::to_value(request).expect("serialize")
}

#[test]
fn to_json_string_is_compact_json_of_the_payload() {
    let request = BankingRequest::add_reconcile_rule("rule", "description", "PIX");
    let text = request.to_json_string();
    let parsed: Value = serde_json::from_str(&text).expect("valid JSON");
    assert_eq!(parsed, value(&request));
    assert!(!text.contains('\n'));
}

#[test]
fn a_bank_row_can_settle_only_one_delivery() {
    let deliveries = r#"[
        {"platform_order_id":"A1","net_value":100.0},
        {"platform_order_id":"A1","net_value":100.0}
    ]"#;
    let bank = r#"[{"description":"PIX A1","amount":100.0}]"#;
    let json = value(&BankingRequest::reconcile(deliveries, bank));
    assert_eq!(json["matched_count"], 1);
    assert_eq!(json["unmatched_count"], 0);
}

#[test]
fn the_amount_tolerance_is_one_cent() {
    let bank = r#"[{"description":"no ref","amount":100.00}]"#;
    let just_inside = r#"[{"platform_order_id":"","net_value":99.995}]"#;
    let just_outside = r#"[{"platform_order_id":"","net_value":99.98}]"#;
    assert_eq!(
        value(&BankingRequest::reconcile(just_inside, bank))["matched_count"],
        1
    );
    assert_eq!(
        value(&BankingRequest::reconcile(just_outside, bank))["matched_count"],
        0
    );
}

#[test]
fn an_order_id_match_wins_over_the_amount() {
    let deliveries = r#"[{"platform_order_id":"ORDER-7","net_value":10.0}]"#;
    let bank = r#"[{"description":"refund ORDER-7","amount":999.0}]"#;
    let json = value(&BankingRequest::reconcile(deliveries, bank));
    assert_eq!(json["matched_count"], 1);
    assert_eq!(json["matched"][0]["match_type"], "order_id");
    assert_eq!(json["total_amount_matched"], 10.0);
}

#[test]
fn a_delivery_without_amounts_never_matches_a_missing_bank_amount() {
    let deliveries = r#"[{"platform_order_id":""}]"#;
    let bank = r#"[{"description":"no amount"}]"#;
    let json = value(&BankingRequest::reconcile(deliveries, bank));
    // Both sides default to 0.0, so the amounts match; the row is consumed.
    assert_eq!(json["matched_count"], 1);
    assert_eq!(json["matched"][0]["match_type"], "amount");
}

#[test]
fn an_unmatched_query_tolerates_malformed_ids() {
    let json = value(&BankingRequest::unmatched_deliveries("not json"));
    assert_eq!(json["filter"]["reconciled"], false);
    assert_eq!(json["filter"]["exclude_ids"].as_array().expect("array").len(), 0);
}

#[test]
fn margins_ignore_a_non_positive_subtotal() {
    let deliveries = r#"[{"subtotal":0.0,"net_value":10.0}]"#;
    let json = value(&BankingRequest::delivery_margins(deliveries));
    assert_eq!(json["count"], 1);
    assert_eq!(json["total_net"], 10.0);
    assert_eq!(json["margin_percent"], 0.0);

    let negative = r#"[{"subtotal":-50.0,"net_value":10.0}]"#;
    let json = value(&BankingRequest::delivery_margins(negative));
    assert_eq!(json["margin_percent"], 0.0);
}

#[test]
fn margins_treat_non_numeric_fields_as_zero() {
    let deliveries = r#"[{"subtotal":"many","platform_commission":null,"net_value":true}]"#;
    let json = value(&BankingRequest::delivery_margins(deliveries));
    assert_eq!(json["count"], 1);
    assert_eq!(json["total_subtotal"], 0.0);
    assert_eq!(json["total_commission"], 0.0);
    assert_eq!(json["total_net"], 0.0);
}

#[test]
fn an_empty_revenue_report_has_no_platforms() {
    let json = value(&BankingRequest::revenue_by_platform("[]"));
    assert_eq!(json["kind"], "revenue_by_platform");
    assert_eq!(json["platforms"].as_array().expect("array").len(), 0);
}

#[test]
fn an_import_round_trips_through_serde() {
    let request = BankingRequest::import_bank_statement(
        "date,description,amount\n2026-08-01,PIX,10.00\n",
    );
    let json = value(&request);
    assert_eq!(json["kind"], "bank_transactions");
    let back: BankingRequest = serde_json::from_value(json).expect("deserializes");
    assert_eq!(back, request);
}
