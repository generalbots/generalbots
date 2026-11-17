#[cfg(test)]
mod prompt_assembly_tests {
    use super::{
        canvas_drawing_block, capability_block, client_locale, delivery_policy_block,
        response_language_block, with_structured_tools,
    };

    #[test]
    fn capability_block_uses_the_client_defaults() {
        let block = capability_block("web", None);
        assert!(block.starts_with("CLIENT CAPABILITIES\nchannel=web; surfaces="));
        assert!(block.contains("max_live_apps=2; motion=full; locale=en; input=text,file."));
        assert!(block.contains("Only use a surface listed above."));
        // A null/non-object caps payload must not panic and keeps the defaults.
        assert!(capability_block("whatsapp", Some(&serde_json::Value::Null))
            .contains("channel=whatsapp;"));
    }

    #[test]
    fn capability_block_reports_only_the_announced_surfaces() {
        let caps = serde_json::json!({
            "surfaces": ["text", "gb.app.v1", 7, null],
            "motion": "reduced",
            "locale": "pt-BR",
            "input": "text",
            "max_live_apps": 4,
        });
        let block = capability_block("web", Some(&caps));
        assert!(block.contains("surfaces=text, gb.app.v1; max_live_apps=4"));
        assert!(block.contains("motion=reduced; locale=pt-BR; input=text."));
    }

    #[test]
    fn canvas_prompt_is_only_appended_for_drawing_turns() {
        for drawing in ["desenhe um gato", "DRAW A LOGO", "svg of a skyline"] {
            let block = canvas_drawing_block(drawing);
            assert!(
                block.starts_with("CANVAS DRAWING SUBSYSTEM"),
                "turn {drawing}"
            );
            assert!(block.contains("Open in Canvas"));
        }
        for prose in ["qual é o faturamento do mês?", "send the invoice by email"] {
            assert_eq!(canvas_drawing_block(prose), "", "turn {prose}");
        }
    }

    #[test]
    fn delivery_policy_names_the_artifact_rule() {
        let block = delivery_policy_block();
        assert!(block.starts_with("DELIVERY POLICY (ask, done)"));
        assert!(block.contains("Never substitute a different format silently."));
        assert!(block.contains("Deliver quietly"));
    }

    #[test]
    fn client_locale_falls_back_to_english() {
        assert_eq!(client_locale(None), "en");
        assert_eq!(client_locale(Some(&serde_json::Value::Null)), "en");
        assert_eq!(client_locale(Some(&serde_json::json!({}))), "en");
        assert_eq!(
            client_locale(Some(&serde_json::json!({ "locale": "  " }))),
            "en"
        );
        assert_eq!(
            client_locale(Some(&serde_json::json!({ "locale": 7 }))),
            "en"
        );
    }

    #[test]
    fn client_locale_reads_the_announced_locale() {
        let caps = serde_json::json!({ "locale": " pt-BR " });
        assert_eq!(client_locale(Some(&caps)), "pt-BR");
        // The capability block and the language rule must never disagree.
        assert!(capability_block("web", Some(&caps)).contains("locale=pt-BR;"));
        assert!(response_language_block(&client_locale(Some(&caps)))
            .contains("client locale hint: pt-BR"));
    }

    #[test]
    fn response_language_block_states_the_rule_for_every_locale() {
        // #2096 — the rule is a platform property: it must name no locale of its
        // own, so all eight shipped locales get the same instruction.
        for locale in ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"] {
            let block = response_language_block(locale);
            assert!(block.starts_with("RESPONSE LANGUAGE"), "locale {locale}");
            assert!(
                block.contains("reply in the language of the user's message"),
                "locale {locale}"
            );
            assert!(
                block.contains(&format!("client locale hint: {locale}")),
                "locale {locale}"
            );
        }
        // An unknown/empty hint still produces a usable rule.
        let block = response_language_block("");
        assert!(block.contains("answer in the user's language"));
        assert!(!block.contains("client locale hint"));
    }

    #[test]
    fn response_language_block_protects_non_translatable_payloads() {
        let block = response_language_block("en");
        for kept in ["code", "file paths", "shell commands", "SQL", "URLs"] {
            assert!(block.contains(kept), "missing {kept}");
        }
        assert!(block.contains("MM/DD/YYYY"));
        assert!(block.contains("switch language"));
    }

    #[test]
    fn structured_tool_instructions_stay_off_non_web_channels() {
        // Only the web channel can render run events; every other channel gets
        // the prompt untouched regardless of the runtime feature flag.
        assert_eq!(
            with_structured_tools("base".to_string(), "whatsapp"),
            "base"
        );
    }
}

#[cfg(test)]
mod api_catalog_gate_tests {
    use super::api_catalog_enabled;

    #[test]
    fn catalogue_is_on_by_default_and_for_unset_values() {
        assert!(api_catalog_enabled(None));
        assert!(api_catalog_enabled(Some("")));
        assert!(api_catalog_enabled(Some("1")));
        assert!(api_catalog_enabled(Some("true")));
    }

    #[test]
    fn only_an_explicit_opt_out_hides_the_catalogue() {
        for value in ["0", "false", "off", "no", " FALSE "] {
            assert!(!api_catalog_enabled(Some(value)), "value {value}");
        }
    }
}
