//! #2069 CLD-6 — `slugify` is the only thing keeping signup and the drive
//! monitor on the same bucket, so its exact rule is pinned here.
use super::*;

#[test]
fn cld6_lowercases_and_dashes_spaces() {
    assert_eq!(slugify("ACME Corp"), "acme-corp");
    assert_eq!(slugify("ACME  Corp"), "acme--corp");
    assert_eq!(slugify("acme_corp"), "acme-corp");
    assert_eq!(slugify("Acme_Corp Ltda"), "acme-corp-ltda");
}

#[test]
fn cld6_strips_control_characters_instead_of_dashing_them() {
    // Tabs and newlines are dropped, not converted: "acme\tcorp" is ONE word.
    assert_eq!(slugify("acme\tcorp"), "acmecorp");
    assert_eq!(slugify("acme\ncorp"), "acmecorp");
    assert_eq!(slugify("acme\r\ncorp"), "acmecorp");
    assert_eq!(slugify("acme.corp"), "acmecorp");
    assert_eq!(slugify("acme/corp"), "acmecorp");
}

#[test]
fn cld6_keeps_leading_and_trailing_dashes() {
    // Documented, not desired: the filter keeps every dash, so a name that
    // starts with a dash produces a slug that starts with one. Signup and the
    // drive monitor agree on it, which is what matters for bucket identity.
    assert_eq!(slugify("-acme"), "-acme");
    assert_eq!(slugify("acme-"), "acme-");
    assert_eq!(slugify(" acme "), "-acme-");
}

#[test]
fn cld6_preserves_unicode_letters() {
    // Bucket risk: `is_alphanumeric()` is Unicode-aware, so an accented or
    // CJK name keeps those characters in the slug. MinIO accepts them, but a
    // name and its ASCII transliteration are DIFFERENT buckets. Changing this
    // rule would orphan every existing bucket, so it stays as documented.
    assert_eq!(slugify("Ação"), "ação");
    assert_eq!(slugify("日本"), "日本");
    assert_eq!(slugify("Café 24h"), "café-24h");
}

#[test]
fn cld6_is_idempotent() {
    for name in ["ACME Corp", "Ação Ltda", "-x-", "a_b c", ""] {
        let once = slugify(name);
        assert_eq!(slugify(&once), once, "slugify({name:?}) must be stable");
    }
}

#[test]
fn cld6_empty_and_symbol_only_names_collapse_to_empty() {
    assert_eq!(slugify(""), "");
    assert_eq!(slugify("!!!"), "");
    assert_eq!(slugify("   "), "---");
    assert_eq!(slugify("_"), "-");
}

#[test]
fn cld6_digits_survive() {
    assert_eq!(slugify("Bot 2.0"), "bot-20");
}
