//! Grid helpers shared by every CRM fragment: query-string encoding (a value
//! must never break out of its parameter), the filter suffix carried through
//! paging, the "never render an empty cell" fallback and the money format.
use super::*;

use std::cmp::Ordering;

fn query(stage: Option<&str>, search: Option<&str>, sort: Option<&str>, dir: Option<&str>) -> GridQuery {
    GridQuery {
        stage: stage.map(str::to_string),
        search: search.map(str::to_string),
        sort: sort.map(str::to_string),
        dir: dir.map(str::to_string),
        ..GridQuery::default()
    }
}

#[test]
fn urlencode_leaves_unreserved_characters_alone() {
    assert_eq!(urlencode("Ana_Silva-01.v2~x"), "Ana_Silva-01.v2~x");
}

#[test]
fn urlencode_uses_plus_for_spaces() {
    assert_eq!(urlencode("Ana Silva"), "Ana+Silva");
}

#[test]
fn urlencode_escapes_parameter_separators() {
    assert_eq!(urlencode("a&b=c"), "a%26b%3Dc");
    assert_eq!(urlencode("a/b"), "a%2Fb");
    assert_eq!(urlencode("50%"), "50%25");
}

#[test]
fn urlencode_escapes_non_ascii_as_utf8_bytes() {
    assert_eq!(urlencode("João"), "Jo%C3%A3o");
}

#[test]
fn no_filters_produce_no_suffix() {
    assert_eq!(filter_suffix(&GridQuery::default()), "");
    assert_eq!(filter_suffix(&query(Some("all"), None, None, None)), "");
}

#[test]
fn a_stage_filter_is_carried() {
    assert_eq!(
        filter_suffix(&query(Some("proposal"), None, None, None)),
        "&stage=proposal"
    );
}

#[test]
fn a_blank_search_is_dropped_and_a_real_one_is_trimmed_and_encoded() {
    assert_eq!(filter_suffix(&query(None, Some("   "), None, None)), "");
    assert_eq!(
        filter_suffix(&query(None, Some(" Ana & Bo "), None, None)),
        "&search=Ana+%26+Bo"
    );
}

#[test]
fn sorting_carries_its_direction_only_with_a_column() {
    assert_eq!(
        filter_suffix(&query(None, None, Some("name"), Some("desc"))),
        "&sort=name&dir=desc"
    );
    assert_eq!(
        filter_suffix(&query(None, None, Some("name"), None)),
        "&sort=name"
    );
    assert_eq!(filter_suffix(&query(None, None, None, Some("desc"))), "");
}

#[test]
fn a_missing_value_renders_as_a_dash() {
    assert_eq!(or_dash(None), "-");
    assert_eq!(or_dash(Some("   ")), "-");
    assert_eq!(or_dash(Some(" Ana ")), "Ana");
}

#[test]
fn a_value_is_escaped_before_it_reaches_the_grid() {
    assert_eq!(or_dash(Some("<b>x</b>")), "&lt;b&gt;x&lt;/b&gt;");
}

#[test]
fn money_uses_the_record_currency_symbol() {
    assert_eq!(format_money(1234.5, Some("USD")), "$1234.50");
    assert_eq!(format_money(1234.5, Some("EUR")), "€1234.50");
    assert_eq!(format_money(1234.5, Some("brl")), "R$1234.50");
    assert_eq!(format_money(1234.5, None), "$1234.50");
}

#[test]
fn yen_is_rendered_without_decimals_and_unknown_codes_keep_their_prefix() {
    assert_eq!(format_money(1234.5, Some("JPY")), "¥1234");
    assert_eq!(format_money(10.0, Some("chf")), "CHF 10.00");
}

#[test]
fn sorting_is_ascending_unless_desc_is_requested() {
    assert!(is_ascending(None));
    assert!(is_ascending(Some("asc")));
    assert!(is_ascending(Some("ASC")));
    assert!(!is_ascending(Some("desc")));
    assert!(!is_ascending(Some("DESC")));
}

#[test]
fn ordering_puts_missing_values_last_in_ascending_order() {
    let asc = is_ascending(Some("asc"));

    assert_eq!(order_by(Some(&"a"), Some(&"b"), asc), Ordering::Less);
    assert_eq!(order_by(Some(&"b"), Some(&"a"), asc), Ordering::Greater);
    assert_eq!(order_by(Some(&"a"), None, asc), Ordering::Greater);
    assert_eq!(order_by(None, Some(&"a"), asc), Ordering::Less);
    assert_eq!(order_by(None, None, asc), Ordering::Equal);
}

#[test]
fn descending_order_reverses_the_comparison() {
    assert_eq!(order_by(Some(&"a"), Some(&"b"), false), Ordering::Greater);
}

#[test]
fn no_pager_is_rendered_for_a_single_page() {
    assert!(pager_bar("contacts", "", PAGE_SIZE, 0).is_empty());
    assert!(pager_bar("contacts", "", 0, 0).is_empty());
}

#[test]
fn the_pager_offers_next_then_prev_as_the_window_moves() {
    let first = pager_bar("contacts", "&stage=won", PAGE_SIZE * 3, 0);
    assert!(first.contains("Page 1 of 3"));
    assert!(first.contains("Next"));
    assert!(!first.contains("Prev"));
    assert!(first.contains("?offset=50&stage=won"));

    let middle = pager_bar("contacts", "", PAGE_SIZE * 3, PAGE_SIZE);
    assert!(middle.contains("Page 2 of 3"));
    assert!(middle.contains("Prev"));
    assert!(middle.contains("Next"));

    let last = pager_bar("contacts", "", PAGE_SIZE * 3, PAGE_SIZE * 2);
    assert!(last.contains("Page 3 of 3"));
    assert!(last.contains("Prev"));
    assert!(!last.contains("Next"));
}
