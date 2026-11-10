// Tests for the SAVE → Rhai rewriter's pure helpers. Split out of
// `convert_save_statements.rs` to keep that file under the AGENTS.md size
// limit; the DB-backed column lookup is out of scope (no PostgreSQL offline).

use super::*;

#[test]
fn split_by_commas_outside_parens_ignores_nested_commas() {
    assert_eq!(split_by_commas_outside_parens("a, b"), vec!["a", "b"]);
    assert_eq!(
        split_by_commas_outside_parens("(a, b), c"),
        vec!["(a, b)", "c"]
    );
    assert_eq!(
        split_by_commas_outside_parens("f(x, g(y, z)), w"),
        vec!["f(x, g(y, z))", "w"]
    );
    assert_eq!(split_by_commas_outside_parens("[1, 2], {a, b}"), vec!["[1, 2]", "{a, b}"]);
}

#[test]
fn split_by_commas_outside_parens_drops_empty_slots() {
    assert_eq!(split_by_commas_outside_parens("a,,b"), vec!["a", "b"]);
    assert_eq!(split_by_commas_outside_parens("a,"), vec!["a"]);
    assert_eq!(split_by_commas_outside_parens(""), Vec::<&str>::new());
    assert_eq!(split_by_commas_outside_parens("  "), Vec::<&str>::new());
}

#[test]
fn parse_save_statement_splits_the_table_from_the_values() {
    assert_eq!(
        BasicCompiler::parse_save_statement("\"customers\", name, email").expect("parsed"),
        vec!["\"customers\"", "name", "email"]
    );
}

#[test]
fn parse_save_statement_keeps_commas_inside_literals() {
    assert_eq!(
        BasicCompiler::parse_save_statement("\"t\", \"a,b\"").expect("parsed"),
        vec!["\"t\"", "\"a,b\""]
    );
}

#[test]
fn parse_save_statement_drops_a_trailing_empty_slot() {
    assert_eq!(
        BasicCompiler::parse_save_statement("\"t\", a,").expect("parsed"),
        vec!["\"t\"", "a"]
    );
    assert_eq!(
        BasicCompiler::parse_save_statement("").expect("parsed"),
        Vec::<String>::new()
    );
}

#[test]
fn save_with_where_builds_a_map_with_the_id() {
    let mut counter = 0usize;
    let out = BasicCompiler::convert_save_with_where(
        "(key = val, other = 2) TO \"t\" WHERE id = 5",
        &mut counter,
    )
    .expect("converted")
    .expect("converted");
    assert_eq!(
        out,
        "let __save_data_0__ = #{id: 5, key: val, other: 2}; SAVE \"t\", __save_data_0__"
    );
    assert_eq!(counter, 1);
}

#[test]
fn save_with_where_passes_a_variable_through() {
    let mut counter = 0usize;
    let out = BasicCompiler::convert_save_with_where("var TO t WHERE id = expr", &mut counter)
        .expect("converted")
        .expect("converted");
    assert_eq!(out, "SAVE \"t\", var");
    // The variable form does not need a temporary map.
    assert_eq!(counter, 0);
}

#[test]
fn save_with_where_accepts_a_compact_id_condition() {
    let mut counter = 0usize;
    let out = BasicCompiler::convert_save_with_where("(k = 1) TO t WHERE id=7", &mut counter)
        .expect("converted")
        .expect("converted");
    assert_eq!(out, "let __save_data_0__ = #{id: 7, k: 1}; SAVE \"t\", __save_data_0__");
}

#[test]
fn save_with_where_numbers_each_temporary_map() {
    let mut counter = 3usize;
    let out = BasicCompiler::convert_save_with_where("(k = 1) TO t WHERE id = 1", &mut counter)
        .expect("converted")
        .expect("converted");
    assert!(out.contains("__save_data_3__"), "got: {out}");
    assert_eq!(counter, 4);
}

#[test]
fn save_with_where_rejects_a_statement_without_to_or_where() {
    let mut counter = 0usize;
    assert!(BasicCompiler::convert_save_with_where("x WHERE y = 1", &mut counter).is_err());
    assert!(BasicCompiler::convert_save_with_where("x TO t", &mut counter).is_err());
}

const TABLES_BAS: &str = "BEGIN TABLE customers\n    FIELD id UUID\n    FIELD name VARCHAR(255)\nEND TABLE\n\nBEGIN TABLE orders\n    FIELD id UUID\n    FIELD total DECIMAL(10,2)\nEND TABLE\n";

#[test]
fn table_definition_columns_come_from_the_matching_table() {
    assert_eq!(
        BasicCompiler::parse_table_definition_for_fields(TABLES_BAS, "customers").expect("parsed"),
        vec!["id", "name"]
    );
    assert_eq!(
        BasicCompiler::parse_table_definition_for_fields(TABLES_BAS, "orders").expect("parsed"),
        vec!["id", "total"]
    );
}

#[test]
fn table_definition_matches_the_name_exactly() {
    // A prefix of another table name must not select it.
    assert_eq!(
        BasicCompiler::parse_table_definition_for_fields(TABLES_BAS, "order").expect("parsed"),
        Vec::<String>::new()
    );
    assert_eq!(
        BasicCompiler::parse_table_definition_for_fields(TABLES_BAS, "customers_archive")
            .expect("parsed"),
        Vec::<String>::new()
    );
}

#[test]
fn table_definition_name_match_is_case_insensitive() {
    assert_eq!(
        BasicCompiler::parse_table_definition_for_fields(TABLES_BAS, "CUSTOMERS").expect("parsed"),
        vec!["id", "name"]
    );
}

#[test]
fn table_definition_accepts_the_bare_table_header() {
    let source = "TABLE legacy\n    FIELD code TEXT\nEND TABLE\n";
    assert_eq!(
        BasicCompiler::parse_table_definition_for_fields(source, "legacy").expect("parsed"),
        vec!["code"]
    );
}

#[test]
fn table_definition_ignores_fields_outside_the_table() {
    let source = "FIELD orphan TEXT\nBEGIN TABLE t\n    FIELD kept TEXT\nEND TABLE\nFIELD after TEXT\n";
    assert_eq!(
        BasicCompiler::parse_table_definition_for_fields(source, "t").expect("parsed"),
        vec!["kept"]
    );
}
