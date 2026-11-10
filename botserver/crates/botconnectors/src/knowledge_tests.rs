//! Unit tests for the REST-configurable connector helpers in `knowledge.rs`.
//!
//! The credential JSON is the whole contract: an operator writes `base_url`,
//! `items_path` and the field names in Vault, and these helpers decide what is
//! requested and how the response is read. They are pure — no HTTP here.

use super::*;
use serde_json::json;

#[test]
fn percent_encode_escapes_per_utf8_byte_and_keeps_the_unreserved_set() {
    let cases = [
        ("a b", "a%20b"),
        ("a+b", "a%2Bb"),
        ("ä", "%C3%A4"),
        ("a/b?c=d&e", "a%2Fb%3Fc%3Dd%26e"),
        ("-_.~", "-_.~"),
        ("AZaz09", "AZaz09"),
        ("", ""),
    ];
    for (input, expected) in cases {
        assert_eq!(percent_encode(input), expected, "input={input:?}");
    }
}

#[test]
fn percent_encode_never_emits_a_plus_for_a_space() {
    // A `+` in a query string means "plus", not "space" — the container id
    // must survive as a literal space.
    assert!(!percent_encode("a b").contains('+'));
}

#[test]
fn response_items_returns_a_bare_array_untouched() {
    let root = json!([{"id": "1"}, {"id": "2"}]);
    assert_eq!(response_items(&root).len(), 2);
}

#[test]
fn response_items_unwraps_the_documented_envelopes_in_declaration_order() {
    assert_eq!(response_items(&json!({"items": [1, 2]})).len(), 2);
    assert_eq!(response_items(&json!({"data": [1]})).len(), 1);
    assert_eq!(response_items(&json!({"files": [1, 2, 3]})).len(), 3);
    // `items` is declared before `data`, so it wins when both are present.
    let both = json!({"items": [1], "data": [1, 2, 3]});
    assert_eq!(response_items(&both).len(), 1);
}

#[test]
fn response_items_returns_empty_for_scalars_and_objects() {
    assert!(response_items(&json!({"value": "x"})).is_empty());
    assert!(response_items(&json!({})).is_empty());
    assert!(response_items(&json!("text")).is_empty());
    assert!(response_items(&json!({"items": "not-an-array"})).is_empty());
}

#[test]
fn pick_skips_empty_strings_and_uses_the_declaration_order() {
    let cases = [
        (json!({"title": "", "name": "n"}), Some("n")),
        (json!({"title": "t", "name": "n"}), Some("t")),
        (json!({"name": 5}), None),
        (json!({}), None),
        (json!({"title": null}), None),
    ];
    for (obj, expected) in cases {
        assert_eq!(pick(&obj, &["title", "name"]), expected, "obj={obj}");
    }
}

#[test]
fn parse_time_accepts_the_three_documented_shapes() {
    let rfc3339 = parse_time("2026-01-02T03:04:05+02:00").expect("rfc3339");
    assert_eq!(rfc3339.to_rfc3339(), "2026-01-02T01:04:05+00:00");

    let naive = parse_time("2026-01-02 03:04:05").expect("naive");
    assert_eq!(naive.to_rfc3339(), "2026-01-02T03:04:05+00:00");

    let date_only = parse_time("2026-01-02").expect("date");
    assert_eq!(date_only.to_rfc3339(), "2026-01-02T00:00:00+00:00");

    assert!(parse_time("nope").is_none());
    assert!(parse_time("").is_none());
    assert!(parse_time("2026-13-40").is_none());
}

#[test]
fn base_url_is_required_and_trailing_slashes_are_trimmed() {
    assert_eq!(
        base_url(&json!({"base_url": "https://chat.example.com/"})),
        Ok("https://chat.example.com".to_string())
    );
    assert_eq!(
        base_url(&json!({"base_url": "   "})),
        Err("credentials.base_url is required".to_string())
    );
    assert_eq!(base_url(&json!({})), Err("credentials.base_url is required".to_string()));
}

#[test]
fn resolve_path_passes_absolute_urls_through_and_joins_relative_ones() {
    let cred = json!({"base_url": "https://api.example.com/"});
    assert_eq!(
        resolve_path(&cred, "items_path", "/default"),
        Ok("https://api.example.com/default".to_string())
    );
    assert_eq!(
        resolve_path(&cred, "items_path", "/items/{container}"),
        Ok("https://api.example.com/items/{container}".to_string())
    );
    assert_eq!(
        resolve_path(&cred, "items_path", "https://other.example.com/x/"),
        Ok("https://other.example.com/x".to_string())
    );
    assert_eq!(
        resolve_path(&json!({}), "items_path", "/items"),
        Err("credentials.base_url is required".to_string())
    );
}

#[test]
fn cred_string_trims_and_falls_back_to_the_default() {
    let cred = json!({"limit": " 50 ", "blank": "  ", "n": 5});
    assert_eq!(cred_string(&cred, "limit", "200"), "50");
    assert_eq!(cred_string(&cred, "blank", "200"), "200");
    assert_eq!(cred_string(&cred, "missing", "200"), "200");
    assert_eq!(cred_string(&cred, "n", "200"), "200");
}

#[test]
fn cred_token_tries_token_access_token_then_api_key() {
    assert_eq!(
        cred_token(&json!({"token": "t", "access_token": "a"})),
        Some("t".to_string())
    );
    assert_eq!(
        cred_token(&json!({"token": "", "access_token": "a"})),
        Some("a".to_string())
    );
    assert_eq!(cred_token(&json!({"api_key": "k"})), Some("k".to_string()));
    assert_eq!(cred_token(&json!({})), None);
    assert_eq!(cred_token(&json!({"token": 5})), None);
}

#[test]
fn item_acl_reads_only_string_entries_and_prefers_acl() {
    assert_eq!(
        item_acl(&json!({"acl": ["user:a", "group:b"]})),
        vec!["user:a".to_string(), "group:b".to_string()]
    );
    assert_eq!(
        item_acl(&json!({"acl_principals": ["user:c"]})),
        vec!["user:c".to_string()]
    );
    // Non-string entries are dropped rather than stringified.
    assert_eq!(item_acl(&json!({"acl": [1, "user:d", null]})), vec!["user:d".to_string()]);
    // `acl` wins over `acl_principals`.
    assert_eq!(
        item_acl(&json!({"acl": ["user:a"], "acl_principals": ["user:z"]})),
        vec!["user:a".to_string()]
    );
    assert!(item_acl(&json!({"acl": "user:a"})).is_empty());
    assert!(item_acl(&json!({})).is_empty());
}

#[test]
fn item_updated_at_follows_the_documented_field_order() {
    let both = json!({"updated_at": "2026-01-02", "edited_at": "2026-03-04"});
    assert_eq!(
        item_updated_at(&both).map(|t| t.to_rfc3339()),
        Some("2026-01-02T00:00:00+00:00".to_string())
    );
    assert_eq!(
        item_updated_at(&json!({"date": "2026-05-06"})).map(|t| t.to_rfc3339()),
        Some("2026-05-06T00:00:00+00:00".to_string())
    );
    assert!(item_updated_at(&json!({"updated_at": "not-a-date"})).is_none());
    assert!(item_updated_at(&json!({})).is_none());
}

#[test]
fn the_three_connectors_report_their_kind() {
    assert_eq!(ChatConnector.kind(), "chat");
    assert_eq!(MailConnector.kind(), "mail");
    assert_eq!(DriveConnector.kind(), "drive");
}

#[test]
fn container_defaults_are_per_connector() {
    // A connector with no credentials fails on `base_url` before any request,
    // proving the defaults are looked up per kind (never hardcoded).
    let empty = json!({});
    let container = Container {
        id: "c1".to_string(),
        name: "c1".to_string(),
    };
    for connector in [
        &ChatConnector as &dyn KnowledgeConnector,
        &MailConnector,
        &DriveConnector,
    ] {
        assert_eq!(
            connector.list_containers(&empty).err(),
            Some("credentials.base_url is required".to_string()),
            "kind={}",
            connector.kind()
        );
        assert_eq!(
            connector.iter_items(&empty, &container, None).err(),
            Some("credentials.base_url is required".to_string()),
            "kind={}",
            connector.kind()
        );
    }
}
