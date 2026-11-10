//! Contract tests for the in-memory data operations (#2064).
//!
//! `FILTER`, `SUM`, `JOIN`, `PIVOT` and `GROUP BY` run over a `Dynamic` array
//! that came from a query; they are the only place a BASIC script compares or
//! aggregates rows, and they never touch the database.

use super::*;
use botbasic_core::utils::json_value_to_dynamic;

fn rows(value: Value) -> Dynamic {
    json_value_to_dynamic(&value)
}

fn sample() -> Dynamic {
    rows(json!([
        {"name": "ana", "amount": 10, "city": "recife"},
        {"name": "bruno", "amount": 25, "city": "recife"},
        {"name": "carla", "amount": 5, "city": "sp"},
    ]))
}

fn names(result: Dynamic) -> Vec<String> {
    to_array(&result)
        .iter()
        .map(|row| {
            dynamic_to_map(row)
                .get("name")
                .map(ToString::to_string)
                .unwrap_or_default()
        })
        .collect()
}

#[test]
fn equality_operators_compare_as_text() {
    for operator in ["=", "=="] {
        let condition = format!("name {operator} ana");
        let filtered = execute_filter(&sample(), &condition).expect("valid filter");
        assert_eq!(names(filtered), ["ana"], "operator {operator}");
    }

    for operator in ["!=", "<>"] {
        let condition = format!("city {operator} recife");
        let filtered = execute_filter(&sample(), &condition).expect("valid filter");
        assert_eq!(names(filtered), ["carla"], "operator {operator}");
    }
}

#[test]
fn numeric_operators_compare_numbers_not_strings() {
    // A lexicographic comparison would put "10" before "5"; the numeric
    // operators must not.
    let filtered = execute_filter(&sample(), "amount > 9").expect("valid filter");
    assert_eq!(names(filtered), ["ana", "bruno"]);

    let filtered = execute_filter(&sample(), "amount <= 10").expect("valid filter");
    assert_eq!(names(filtered), ["ana", "carla"]);

    let filtered = execute_filter(&sample(), "amount >= 25").expect("valid filter");
    assert_eq!(names(filtered), ["bruno"]);
}

#[test]
fn non_numeric_operands_degrade_to_zero() {
    // The documented behaviour: an unparseable side of a `>` comparison is 0,
    // so `city > 0` matches nothing and `city <= 0` matches everything.
    let filtered = execute_filter(&sample(), "city > 0").expect("valid filter");
    assert!(to_array(&filtered).is_empty());

    let filtered = execute_filter(&sample(), "city <= 0").expect("valid filter");
    assert_eq!(to_array(&filtered).len(), 3);
}

#[test]
fn like_is_a_case_insensitive_substring_match() {
    let filtered = execute_filter(&sample(), "name LIKE AN").expect("valid filter");
    assert_eq!(names(filtered), ["ana"]);

    let filtered = execute_filter(&sample(), "city like eci").expect("valid filter");
    assert_eq!(names(filtered), ["ana", "bruno"]);
}

#[test]
fn rows_without_the_field_are_skipped() {
    let data = rows(json!([{"name": "ana"}, {"other": 1}, {"name": "bruno"}]));
    let filtered = execute_filter(&data, "name != nobody").expect("valid filter");
    assert_eq!(names(filtered), ["ana", "bruno"]);
}

#[test]
fn an_invalid_condition_is_an_error() {
    assert!(execute_filter(&sample(), "no operator here").is_err());
}

#[test]
fn aggregates_cover_the_documented_operations() {
    assert_eq!(execute_aggregate("SUM", &sample(), "amount").expect("sum").as_float().expect("float"), 40.0);
    assert_eq!(execute_aggregate("AVG", &sample(), "amount").expect("avg").as_float().expect("float"), 40.0 / 3.0);
    assert_eq!(execute_aggregate("COUNT", &sample(), "amount").expect("count").as_float().expect("float"), 3.0);
    assert_eq!(execute_aggregate("MIN", &sample(), "amount").expect("min").as_float().expect("float"), 5.0);
    assert_eq!(execute_aggregate("MAX", &sample(), "amount").expect("max").as_float().expect("float"), 25.0);
    // The operation name is case-insensitive.
    assert_eq!(execute_aggregate("sum", &sample(), "amount").expect("sum").as_float().expect("float"), 40.0);
}

#[test]
fn aggregates_over_an_empty_set_use_the_documented_identity() {
    let empty = rows(json!([]));
    assert_eq!(execute_aggregate("SUM", &empty, "amount").expect("sum").as_float().expect("float"), 0.0);
    assert_eq!(execute_aggregate("AVG", &empty, "amount").expect("avg").as_float().expect("float"), 0.0);
    assert_eq!(execute_aggregate("COUNT", &empty, "amount").expect("count").as_float().expect("float"), 0.0);
    // MIN/MAX of nothing keep their f64 identity, not 0: a caller can tell
    // "no rows" from "a row of zero".
    assert!(execute_aggregate("MIN", &empty, "amount").expect("min").as_float().expect("float").is_infinite());
    assert!(execute_aggregate("MAX", &empty, "amount").expect("max").as_float().expect("float").is_infinite());
}

#[test]
fn an_unknown_aggregate_is_an_error() {
    assert!(execute_aggregate("MEDIAN", &sample(), "amount").is_err());
}

#[test]
fn join_matches_on_the_key_and_keeps_left_fields() {
    let left = rows(json!([
        {"id": 1, "name": "ana"},
        {"id": 2, "name": "bruno"},
        {"id": 3, "name": "carla"},
    ]));
    let right = rows(json!([
        {"id": 1, "city": "recife"},
        {"id": 2, "city": "sp"},
    ]));

    let joined = execute_join(&left, &right, "id").expect("join");
    let joined = to_array(&joined);
    assert_eq!(joined.len(), 2, "rows without a match are dropped");

    let first = dynamic_to_map(&joined[0]);
    assert_eq!(first.get("name").map(ToString::to_string), Some("ana".to_string()));
    assert_eq!(first.get("city").map(ToString::to_string), Some("recife".to_string()));
}

#[test]
fn join_does_not_overwrite_a_left_field_with_a_right_one() {
    let left = rows(json!([{"id": 1, "city": "left"}]));
    let right = rows(json!([{"id": 1, "city": "right"}]));

    let joined = to_array(&execute_join(&left, &right, "id").expect("join"));
    let row = dynamic_to_map(&joined[0]);
    assert_eq!(row.get("city").map(ToString::to_string), Some("left".to_string()));
}

#[test]
fn join_expands_duplicate_keys_on_the_right() {
    let left = rows(json!([{"id": 1, "name": "ana"}]));
    let right = rows(json!([
        {"id": 1, "tag": "vip"},
        {"id": 1, "tag": "beta"},
    ]));

    assert_eq!(to_array(&execute_join(&left, &right, "id").expect("join")).len(), 2);
}

#[test]
fn pivot_sums_by_row_key_and_treats_junk_as_zero() {
    let data = rows(json!([
        {"city": "recife", "amount": 10},
        {"city": "recife", "amount": 5},
        {"city": "sp", "amount": "not a number"},
    ]));

    let pivot = to_array(&execute_pivot(&data, "city", "amount").expect("pivot"));
    let mut by_city: Vec<(String, f64)> = pivot
        .iter()
        .map(|row| {
            let map = dynamic_to_map(row);
            (
                map.get("city").map(ToString::to_string).unwrap_or_default(),
                map.get("amount")
                    .and_then(|value| value.as_float().ok())
                    .unwrap_or(-1.0),
            )
        })
        .collect();
    by_city.sort_by(|a, b| a.0.cmp(&b.0));

    assert_eq!(
        by_city,
        [("recife".to_string(), 15.0), ("sp".to_string(), 0.0)]
    );
}

#[test]
fn group_by_buckets_the_rows_under_their_key() {
    let grouped = execute_group_by(&sample(), "city").expect("group by");
    assert!(grouped.is_map());

    let map = grouped.cast::<Map>();
    assert_eq!(map.len(), 2);
    let recife = map.get("recife").expect("recife bucket");
    assert_eq!(to_array(recife).len(), 2);
    let sp = map.get("sp").expect("sp bucket");
    assert_eq!(to_array(sp).len(), 1);
}

#[test]
fn a_scalar_is_treated_as_a_single_row() {
    let scalar = Dynamic::from(42_i64);
    assert_eq!(to_array(&scalar).len(), 1);
    // `dynamic_to_map` of a non-map is empty, so the row has no fields.
    assert!(dynamic_to_map(&scalar).is_empty());
}
