use super::*;


    use serde_json::json;

    const TEST_ACTIONS: &[ActionSpec] = &[ActionSpec {
        key: "widgets.items.list",
        method: "GET",
        path: "/items/{item_id}",
        summary: "List items.",
        path_params: &["item_id"],
        query: &[("limit", "limit")],
        body_param: None,
        body_wrapper: None,
        risk: Risk::Low,
        params: &[
            ParamSpec {
                name: "item_id",
                kind: ParamKind::Str,
                required: true,
            },
            ParamSpec {
                name: "limit",
                kind: ParamKind::Str,
                required: false,
            },
        ],
    }];

    const TEST_KEYS: &[&str] = &["widgets.items.list"];

    const TEST_SPEC: ProviderSpec = ProviderSpec {
        slug: "widgets",
        origin: Origin::Static("https://api.widgets.test/v1"),
        auth: AuthStyle::Bearer {
            token_field: "token",
        },
        actions: TEST_ACTIONS,
        action_keys: TEST_KEYS,
    };

    #[test]
    fn url_templating_encodes_placeholders_and_query() {
        let action = &TEST_ACTIONS[0];
        let url = build_url_from_parts(
            "https://api.widgets.test/v1",
            action,
            &json!({"item_id": "a b/c", "limit": "5"}),
            vec![("limit".to_string(), "5".to_string())],
        )
        .unwrap();
        assert_eq!(url, "https://api.widgets.test/v1/items/a%20b%2Fc?limit=5");
    }

    #[test]
    fn missing_required_path_param_is_rejected_before_network() {
        let action = &TEST_ACTIONS[0];
        assert!(build_url_from_parts(
            "https://api.widgets.test/v1",
            action,
            &json!({"limit": "5"}),
            vec![]
        )
        .is_err());
    }

    #[test]
    fn basic_template_substitutes_envelope_fields() {
        let spec = ProviderSpec {
            slug: "t",
            origin: Origin::Static("https://x"),
            auth: AuthStyle::BasicTemplate {
                user_template: "{email}/token",
                password_field: "token",
            },
            actions: &[],
            action_keys: &[],
        };
        let credentials = json!({"email": "a@b.c", "token": "secret"});
        let (headers, _) = auth_headers_and_query(&spec, &credentials).unwrap();
        let (_, value) = headers.iter().find(|(n, _)| *n == "authorization").unwrap();
        assert_eq!(
            *value,
            format!(
                "Basic {}",
                base64::engine::general_purpose::STANDARD.encode("a@b.c/token:secret")
            )
        );
    }

    #[test]
    fn zendesk_origin_validates_subdomain_charset() {
        let spec = ProviderSpec {
            slug: "z",
            origin: Origin::ZendeskSubdomain,
            auth: AuthStyle::Bearer {
                token_field: "token",
            },
            actions: &[],
            action_keys: &[],
        };
        assert_eq!(
            resolve_origin(&spec, &json!({"subdomain": "acme"})).unwrap(),
            "https://acme.api.zendesk.com/api/v2"
        );
        assert!(resolve_origin(&spec, &json!({"subdomain": "../etc"})).is_err());
    }

    #[test]
    fn mailchimp_origin_extracts_data_center() {
        let spec = ProviderSpec {
            slug: "m",
            origin: Origin::MailchimpDataCenter,
            auth: AuthStyle::Bearer {
                token_field: "api_key",
            },
            actions: &[],
            action_keys: &[],
        };
        let key = ["abcd1234", "-", "us19"].concat();
        let origin = resolve_origin(&spec, &json!({"api_key": key})).unwrap();
        assert_eq!(origin, "https://us19.api.mailchimp.com/3.0");
    }

    #[test]
    fn outcomes_are_capped_and_summarized() {
        let big: Vec<Value> = (0..40).map(|i| json!({"id": i})).collect();
        let outcome = shape_outcome(&TEST_ACTIONS[0], 200, &serde_json::to_vec(&big).unwrap());
        assert!(outcome.truncated);
        assert_eq!(outcome.data.as_array().unwrap().len(), MAX_LIST_ITEMS);
        assert!(outcome.summary.contains("(status 200)"));
    }

    #[test]
    fn generic_adapter_metadata_matches_declared_keys() {
        let adapter = GenericAdapter::new(&TEST_SPEC);
        assert_eq!(adapter.provider(), "widgets");
        let catalog = adapter.safe_action_catalog();
        assert_eq!(catalog.len(), adapter.implemented_actions().len());
        assert_eq!(catalog[0].risk, "low");
        assert!(!catalog[0].requires_approval);
    }
