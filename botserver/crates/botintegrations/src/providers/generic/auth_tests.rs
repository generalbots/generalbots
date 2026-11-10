//! Unit tests for credential-derived origins and per-style auth headers.
//!
//! Each `AuthStyle` variant is exercised once: the header set and query pairs
//! are the only thing standing between a stored envelope and a live provider
//! call, and a silently dropped `user-agent` (or a downgraded origin) is not
//! visible from the chat surface.

use super::*;
use serde_json::json;

const EMPTY_ACTIONS: &[ActionSpec] = &[];
const EMPTY_KEYS: &[&str] = &[];

fn spec_with(auth: AuthStyle, origin: Origin) -> ProviderSpec {
    ProviderSpec {
        slug: "widgets",
        origin,
        auth,
        actions: EMPTY_ACTIONS,
        action_keys: EMPTY_KEYS,
    }
}

fn auth_spec(auth: AuthStyle) -> ProviderSpec {
    spec_with(auth, Origin::Static("https://api.widgets.test"))
}

fn header_value<'a>(headers: &'a [(&'static str, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.as_str())
}

fn expected_basic(raw: &str) -> String {
    format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode(raw)
    )
}

#[test]
fn basic_header_encodes_user_and_password_verbatim() {
    // The separator inside the password must survive: only the first colon is
    // structural for HTTP Basic.
    assert_eq!(basic_header("user", "pa:ss"), expected_basic("user:pa:ss"));
    assert_eq!(basic_header("user", ""), expected_basic("user:"));
    assert_eq!(basic_header("", ""), expected_basic(":"));
}

#[test]
fn a_static_origin_is_used_as_declared() {
    let spec = auth_spec(AuthStyle::Bearer { token_field: "token" });
    assert_eq!(
        resolve_origin(&spec, &json!({})),
        Ok("https://api.widgets.test".to_string())
    );
}

#[test]
fn a_zendesk_subdomain_must_be_a_bare_host_label() {
    let spec = spec_with(
        AuthStyle::Bearer { token_field: "token" },
        Origin::ZendeskSubdomain,
    );
    assert_eq!(
        resolve_origin(&spec, &json!({"subdomain": "acme"})),
        Ok("https://acme.api.zendesk.com/api/v2".to_string())
    );
    assert!(resolve_origin(&spec, &json!({"subdomain": "ac me"})).is_err());
    assert!(resolve_origin(&spec, &json!({"subdomain": "../etc"})).is_err());
    assert!(resolve_origin(&spec, &json!({"subdomain": ""})).is_err());
    assert!(resolve_origin(&spec, &json!({})).is_err());
}

#[test]
fn a_mailchimp_key_selects_its_data_center() {
    let spec = spec_with(
        AuthStyle::Bearer { token_field: "api_key" },
        Origin::MailchimpDataCenter,
    );
    assert_eq!(
        resolve_origin(&spec, &json!({"api_key": "abc123-us5"})),
        Ok("https://us5.api.mailchimp.com/3.0".to_string())
    );
    // A key without a data-center suffix cannot address any host.
    assert!(resolve_origin(&spec, &json!({"api_key": "abc123"})).is_err());
    // ... and an empty suffix is just as unusable.
    assert!(resolve_origin(&spec, &json!({"api_key": "abc123-"})).is_err());
    assert!(resolve_origin(&spec, &json!({"api_key": ""})).is_err());
}

#[test]
fn a_from_field_origin_refuses_a_plain_http_downgrade() {
    let spec = spec_with(
        AuthStyle::Bearer { token_field: "token" },
        Origin::FromField {
            field: "base_url",
            pattern: "{value}",
        },
    );
    assert_eq!(
        resolve_origin(&spec, &json!({"base_url": "https://api.acme.test"})),
        Ok("https://api.acme.test".to_string())
    );
    assert!(resolve_origin(&spec, &json!({"base_url": "http://api.acme.test"})).is_err());
    assert!(resolve_origin(&spec, &json!({"base_url": "api.acme.test"})).is_err());
}

#[test]
fn bearer_sends_an_authorization_header_and_the_user_agent_last() {
    let spec = auth_spec(AuthStyle::Bearer { token_field: "token" });
    let (headers, query) = auth_headers_and_query(&spec, &json!({"token": " t "})).expect("built");

    assert!(query.is_empty());
    assert_eq!(
        headers,
        vec![
            ("authorization", "Bearer t".to_string()),
            ("user-agent", "generalbots-botintegrations".to_string()),
        ]
    );
}

#[test]
fn an_api_key_header_style_sends_the_named_header() {
    let spec = auth_spec(AuthStyle::ApiKeyHeader {
        header: "x-api-key",
        field: "api_key",
    });
    let (headers, _) = auth_headers_and_query(&spec, &json!({"api_key": "k-1"})).expect("built");
    assert_eq!(header_value(&headers, "x-api-key"), Some("k-1"));
    assert_eq!(headers.len(), 2);
}

#[test]
fn a_basic_template_style_substitutes_every_placeholder() {
    let spec = auth_spec(AuthStyle::BasicTemplate {
        user_template: "{email}/token",
        password_field: "token",
    });
    let credentials = json!({"email": "a@b.c", "token": "secret"});
    let (headers, _) = auth_headers_and_query(&spec, &credentials).expect("built");
    let expected = expected_basic("a@b.c/token:secret");
    assert_eq!(header_value(&headers, "authorization"), Some(expected.as_str()));
}

#[test]
fn a_malformed_basic_template_is_rejected() {
    let spec = auth_spec(AuthStyle::BasicTemplate {
        user_template: "{email/token",
        password_field: "token",
    });
    assert!(auth_headers_and_query(&spec, &json!({"email": "a@b.c", "token": "s"})).is_err());
}

#[test]
fn a_basic_join_style_joins_two_fields_and_tolerates_a_missing_second() {
    let spec = auth_spec(AuthStyle::BasicJoin {
        first_field: "api_key",
        separator: ':',
        second_field: Some("secret"),
    });
    let (headers, _) =
        auth_headers_and_query(&spec, &json!({"api_key": "k", "secret": "s"})).expect("built");
    let expected = expected_basic("k:s");
    assert_eq!(header_value(&headers, "authorization"), Some(expected.as_str()));

    let optional = auth_spec(AuthStyle::BasicJoin {
        first_field: "api_key",
        separator: ':',
        second_field: None,
    });
    let (headers, _) = auth_headers_and_query(&optional, &json!({"api_key": "k"})).expect("built");
    let expected = expected_basic("k:");
    assert_eq!(header_value(&headers, "authorization"), Some(expected.as_str()));
}

#[test]
fn a_second_field_containing_the_separator_is_rejected() {
    let spec = auth_spec(AuthStyle::BasicJoin {
        first_field: "api_key",
        separator: ':',
        second_field: Some("secret"),
    });
    assert!(auth_headers_and_query(&spec, &json!({"api_key": "k", "secret": "a:b"})).is_err());
}

#[test]
fn query_pair_credentials_never_become_headers() {
    const PAIRS: &[(&str, &str)] = &[("key", "api_key"), ("token", "token")];
    let spec = auth_spec(AuthStyle::QueryPairs { pairs: PAIRS });
    let (headers, query) =
        auth_headers_and_query(&spec, &json!({"api_key": "k", "token": "t"})).expect("built");

    assert_eq!(
        query,
        vec![("key".to_string(), "k".to_string()), ("token".to_string(), "t".to_string())]
    );
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].0, "user-agent");
}

#[test]
fn a_body_field_style_contributes_no_headers() {
    let spec = auth_spec(AuthStyle::BodyField { field: "api_key" });
    let (headers, query) = auth_headers_and_query(&spec, &json!({"api_key": "k"})).expect("built");
    assert!(query.is_empty());
    assert_eq!(headers, vec![("user-agent", "generalbots-botintegrations".to_string())]);
}

#[test]
fn an_api_key_headers_style_sends_every_pair_in_order() {
    const PAIRS: &[(&str, &str)] = &[
        ("x-algolia-application-id", "app_id"),
        ("x-algolia-api-key", "api_key"),
    ];
    let spec = auth_spec(AuthStyle::ApiKeyHeaders { pairs: PAIRS });
    let (headers, _) =
        auth_headers_and_query(&spec, &json!({"app_id": "A1", "api_key": "K1"})).expect("built");

    assert_eq!(
        headers,
        vec![
            ("x-algolia-application-id", "A1".to_string()),
            ("x-algolia-api-key", "K1".to_string()),
            ("user-agent", "generalbots-botintegrations".to_string()),
        ]
    );
}

#[test]
fn a_bearer_headers_style_appends_its_pinned_headers_before_the_user_agent() {
    const EXTRA: &[(&str, &str)] = &[("notion-version", "2022-06-28")];
    let spec = auth_spec(AuthStyle::BearerHeaders {
        token_field: "token",
        headers: EXTRA,
    });
    let (headers, _) = auth_headers_and_query(&spec, &json!({"token": "t"})).expect("built");

    assert_eq!(
        headers,
        vec![
            ("authorization", "Bearer t".to_string()),
            ("notion-version", "2022-06-28".to_string()),
            ("user-agent", "generalbots-botintegrations".to_string()),
        ]
    );
}

#[test]
fn every_style_requires_its_credential_fields() {
    let specs = [
        auth_spec(AuthStyle::Bearer { token_field: "token" }),
        auth_spec(AuthStyle::ApiKeyHeader {
            header: "x-api-key",
            field: "api_key",
        }),
        auth_spec(AuthStyle::ApiKeyHeaders {
            pairs: &[("x-key", "api_key")],
        }),
        auth_spec(AuthStyle::BearerHeaders {
            token_field: "token",
            headers: &[],
        }),
        auth_spec(AuthStyle::BasicJoin {
            first_field: "api_key",
            separator: ':',
            second_field: None,
        }),
    ];
    for spec in specs {
        assert!(
            auth_headers_and_query(&spec, &json!({})).is_err(),
            "slug={}",
            spec.slug
        );
    }
}

#[test]
fn require_param_str_accepts_strings_and_numbers() {
    assert_eq!(
        require_param_str(&json!({"id": " 42 "}), "id"),
        Ok("42".to_string())
    );
    assert_eq!(require_param_str(&json!({"id": 5}), "id"), Ok("5".to_string()));
    assert_eq!(
        require_param_str(&json!({"id": -1.5}), "id"),
        Ok("-1.5".to_string())
    );
}

#[test]
fn require_param_str_rejects_missing_blank_and_non_scalar_values() {
    for params in [json!({}), json!({"id": "   "}), json!({"id": ""}), json!({"id": null})] {
        let error = require_param_str(&params, "id").expect_err("rejected");
        assert!(error.contains("id"), "error names the param: {error}");
    }
    assert!(require_param_str(&json!({"id": {"a": 1}}), "id").is_err());
    assert!(require_param_str(&json!({"id": [1]}), "id").is_err());
    assert!(require_param_str(&json!({"id": true}), "id").is_err());
}
