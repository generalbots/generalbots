//! Case-insensitive replace expectations.

use super::*;

#[test]
fn every_ascii_case_variant_is_replaced_and_counted() {
    let cases = [
        ("alpha ALPHA Alpha", "alpha", "x", "x x x", 3),
        ("a-b-a", "a", "z", "z-b-z", 2),
        ("unchanged", "needle", "x", "unchanged", 0),
        ("", "needle", "x", "", 0),
        ("abc", "", "x", "abc", 0),
    ];
    for (haystack, needle, replacement, expected, count) in cases {
        assert_eq!(
            replace_ascii_case(haystack, needle, replacement),
            (expected.to_string(), count),
            "haystack {haystack:?} needle {needle:?}"
        );
    }
}

#[test]
fn an_empty_needle_never_loops_or_replaces() {
    let (text, count) = replace_ascii_case("aaaa", "", "b");
    assert_eq!(text, "aaaa");
    assert_eq!(count, 0);
}

#[test]
fn multibyte_haystacks_are_sliced_on_character_boundaries() {
    let (text, count) = replace_ascii_case("relatório relatório", "tório", "-");
    assert_eq!(text, "rela- rela-");
    assert_eq!(count, 2);
}

#[test]
fn only_ascii_letters_are_case_folded() {
    // The helper is explicitly ASCII-only: `AÇÃO` does not fold to `ação`,
    // so an accented needle matches its exact form only.
    let (text, count) = replace_ascii_case("ação AÇÃO", "ação", "x");
    assert_eq!(text, "x AÇÃO");
    assert_eq!(count, 1);
}

#[test]
fn replacement_text_is_not_re_scanned() {
    let (text, count) = replace_ascii_case("aaa", "a", "aa");
    assert_eq!(text, "aaaaaa");
    assert_eq!(count, 3);
}
