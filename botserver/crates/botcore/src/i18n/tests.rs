    use super::*;

    #[test]
    fn test_parse_accept_language_simple() {
        let result = parse_accept_language("en-US");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "en-US");
        assert!((result[0].1 - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_accept_language_with_quality() {
        let result = parse_accept_language("pt-BR,pt;q=0.9,en;q=0.8");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].0, "pt-BR");
        assert_eq!(result[1].0, "pt");
        assert_eq!(result[2].0, "en");
    }

    #[test]
    fn test_parse_accept_language_sorted_by_quality() {
        let result = parse_accept_language("en;q=0.5,pt-BR;q=0.9,es;q=0.7");
        assert_eq!(result[0].0, "pt-BR");
        assert_eq!(result[1].0, "es");
        assert_eq!(result[2].0, "en");
    }

    #[test]
    fn test_negotiate_locale_exact_match() {
        let requested = vec![("pt-BR".to_string(), 1.0)];
        let result = negotiate_locale(&requested);
        assert!(result.is_some());
        assert_eq!(
            result.as_ref().map(|l| l.to_string()),
            Some("pt-BR".to_string())
        );
    }

    #[test]
    fn test_negotiate_locale_language_match() {
        let requested = vec![("pt-PT".to_string(), 1.0)];
        let result = negotiate_locale(&requested);
        assert!(result.is_some());
        assert_eq!(result.as_ref().map(|l| l.language()), Some("pt"));
    }

    #[test]
    fn test_negotiate_locale_fallback() {
        // A language outside AVAILABLE_LOCALES falls back to the default.
        let requested = vec![("sv".to_string(), 1.0)];
        let result = negotiate_locale(&requested);
        assert!(result.is_some());
        assert_eq!(result.as_ref().map(|l| l.language()), Some("en"));

        // A supported language is honoured rather than falling back.
        let requested = vec![("ja".to_string(), 1.0)];
        assert_eq!(
            negotiate_locale(&requested).as_ref().map(|l| l.language()),
            Some("ja")
        );
    }

    #[test]
    fn test_locale_default() {
        let locale = Locale::default();
        assert_eq!(locale.language(), "en");
        assert_eq!(locale.region(), None);
    }

    #[test]
    fn test_locale_display() {
        let locale = Locale::new("pt-BR").unwrap();
        assert_eq!(locale.to_string(), "pt-BR");
    }

    #[test]
    fn test_localized_error_not_found() {
        let locale = Locale::default();
        let error = LocalizedError::not_found(&locale, "User");
        assert_eq!(error.code, "error-http-404");
    }

    #[test]
    fn test_localized_error_with_details() {
        let locale = Locale::default();
        let error =
            LocalizedError::internal(&locale).with_details(serde_json::json!({"trace_id": "abc123"}));
        assert!(error.details.is_some());
    }

    #[test]
    fn test_available_locales_without_init() {
        let locales = available_locales();
        assert!(!locales.is_empty());
    }

    #[test]
    fn test_locale_catalogue_reports_full_metadata() {
        // #2087/#2096 — every locale the endpoint lists must carry the five
        // fields a picker and a formatter need, in the catalogue's own order.
        let locales = available_locales();
        let details = metadata_all(&locales);

        assert_eq!(details.len(), locales.len());
        for (code, meta) in locales.iter().zip(details.iter()) {
            assert_eq!(&meta.code, code, "same order as the catalogue");
            assert!(!meta.native_name.is_empty(), "native name for {code}");
            assert!(meta.dir == "ltr" || meta.dir == "rtl", "direction for {code}");
            assert!(!meta.date_format.is_empty(), "date format for {code}");
            assert!(!meta.number_format.is_empty(), "number format for {code}");
        }

        // The eight shipped locales are all present and named in their script.
        for expected in ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"] {
            let meta = metadata_for(expected);
            assert_eq!(meta.code, expected);
            assert_ne!(meta.native_name, expected, "native name for {expected}");
        }

        // Serialization keeps the documented snake_case field names.
        let payload = serde_json::to_value(metadata_all(&locales)).expect("serializable");
        let first = payload
            .as_array()
            .and_then(|items| items.first())
            .expect("at least one locale");
        for field in ["code", "native_name", "dir", "date_format", "number_format"] {
            assert!(first.get(field).is_some(), "missing {field}");
        }
    }

    #[test]
    fn test_parse_accept_language_drops_zero_quality() {
        // `q=0` means "not acceptable": the language must not be negotiated.
        let result = parse_accept_language("pt-BR;q=0, en;q=0.5");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "en");
    }

    #[test]
    fn test_parse_accept_language_malformed_header() {
        // Junk quality values fall back to 1.0 and never panic.
        let result = parse_accept_language("pt-BR;q=abc");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "pt-BR");
        assert!((result[0].1 - 1.0).abs() < f32::EPSILON);

        let empty = parse_accept_language("");
        assert!(empty.is_empty());
    }

    fn headers(pairs: &[(&str, &str)]) -> axum::http::HeaderMap {
        let mut map = axum::http::HeaderMap::new();
        for (name, value) in pairs {
            if let (Ok(name), Ok(value)) = (
                axum::http::HeaderName::from_bytes(name.as_bytes()),
                axum::http::HeaderValue::from_str(value),
            ) {
                map.insert(name, value);
            }
        }
        map
    }

    #[test]
    fn test_locale_from_headers_explicit_header_wins() {
        let map = headers(&[
            ("x-locale", "pt-BR"),
            ("accept-language", "en-US,en;q=0.9"),
        ]);
        assert_eq!(locale_from_headers(&map).to_string(), "pt-BR");
    }

    #[test]
    fn test_locale_from_headers_accept_language() {
        let map = headers(&[("accept-language", "pt-BR,pt;q=0.9,en;q=0.8")]);
        assert_eq!(locale_from_headers(&map).to_string(), "pt-BR");

        let language_only = headers(&[("accept-language", "pt-PT")]);
        assert_eq!(locale_from_headers(&language_only).language(), "pt");

        let english = headers(&[("accept-language", "en-US")]);
        assert_eq!(locale_from_headers(&english).language(), "en");
    }

    #[test]
    fn test_locale_from_headers_fallbacks() {
        // Unsupported explicit locale is ignored in favour of Accept-Language.
        let unsupported = headers(&[("x-locale", "sv"), ("accept-language", "es")]);
        assert_eq!(locale_from_headers(&unsupported).language(), "es");

        // No headers at all resolves to the default locale.
        assert_eq!(locale_from_headers(&headers(&[])).language(), "en");

        // A malformed Accept-Language header still resolves to a locale.
        let malformed = headers(&[("accept-language", ";;;q=")]);
        assert_eq!(locale_from_headers(&malformed).language(), "en");
    }

    #[test]
    fn test_error_body_is_backward_compatible() {
        let locale = Locale::default();
        let body = error_body(&locale, "error-http-404");
        assert_eq!(body["code"], "error-http-404");
        // Legacy clients read `error` as a string.
        assert!(body["error"].is_string());
        assert_eq!(body["message"], body["error"]);
    }

    #[test]
    fn test_localized_error_body_with_details() {
        let locale = Locale::default();
        let body = LocalizedError::internal(&locale)
            .with_details(serde_json::json!({ "trace_id": "abc" }))
            .body();
        assert_eq!(body["code"], "error-http-500");
        assert_eq!(body["details"]["trace_id"], "abc");
    }

    #[test]
    fn test_error_contract_codes_are_localized() {
        // The catalogue is embedded, so initializing with any path loads the
        // shipped .ftl files. A second call in the same process is expected to
        // fail (OnceCell) and is ignored.
        let _ = init_i18n("locales");

        let en = Locale::new("en").expect("en locale");
        let pt = Locale::new("pt-BR").expect("pt-BR locale");

        let plain = [
            ("error-http-400", 400),
            ("error-http-401", 401),
            ("error-http-403", 403),
            ("error-http-500", 500),
        ];

        for (code, status) in plain {
            let english = error_body(&en, code);
            let portuguese = error_body(&pt, code);

            assert_eq!(english["code"], code, "stable code for {status}");
            assert_eq!(portuguese["code"], code, "stable code for {status}");
            assert!(
                english["message"].as_str().is_some_and(|m| !m.is_empty()),
                "localized message for {status}"
            );
            assert_ne!(english["message"], format!("[{code}]"));
            assert_ne!(portuguese["message"], format!("[{code}]"));
            assert_ne!(
                english["message"], portuguese["message"],
                "pt-BR differs from en for {code}"
            );
        }

        let mut args = MessageArgs::new();
        args.insert("entity".to_string(), "Bot".to_string());
        let not_found = error_body_with_args(&pt, "error-http-404", &args);
        assert_eq!(not_found["code"], "error-http-404");
        assert!(not_found["message"]
            .as_str()
            .is_some_and(|m| m.contains("Bot")));

        let mut conflict_args = MessageArgs::new();
        conflict_args.insert("message".to_string(), "duplicate key".to_string());
        let conflict = error_body_with_args(&en, "error-http-409", &conflict_args);
        assert!(conflict["message"]
            .as_str()
            .is_some_and(|m| m.contains("duplicate key")));

        let mut rate_args = MessageArgs::new();
        rate_args.insert("seconds".to_string(), "30".to_string());
        let rate = error_body_with_args(&en, "error-http-429", &rate_args);
        assert!(rate["message"].as_str().is_some_and(|m| m.contains("30")));
    }
