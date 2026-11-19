use super::*;

/// Whether `name` is one of the integration-catalog commands handled here.
/// Kept in sync with the arms in `execute` below.
pub(crate) fn handles(name: &str) -> bool {
    #[cfg(feature = "integrations")]
    if name == "integrations.decline" {
        return true;
    }
    matches!(
        name,
        "integrations.catalog.search"
            | "integrations.catalog.provider"
            | "integrations.connect.start"
            | "integrations.actions.list"
            | "integrations.invoke"
    )
}

/// Executes an integration-catalog command. Split out of `execute_command`
/// (#2369) so the dispatcher stays below the 450-line hard limit; the arms
/// are byte-for-byte the same behaviour as before the split.
pub(crate) async fn execute(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    user_id: Uuid,
    name: &str,
    obj: &serde_json::Map<String, Value>,
) -> Result<Value, String> {
    let str_of = |key: &str| obj.get(key).and_then(|v| v.as_str()).map(|s| s.to_string());
    match name {
        "integrations.catalog.search" => {
            let providers = crate::apps::integration_catalog::llm_search(
                str_of("q").as_deref(),
                str_of("category").as_deref(),
                str_of("status").as_deref(),
            );
            Ok(json!({ "provider_count": providers.len(), "providers": providers }))
        }
        #[cfg(feature = "integrations")]
        "integrations.decline" => {
            // #2061 — the user declined a setup offer; remember it so the
            // assistant does not repeat the offer on every turn.
            let provider =
                str_of("provider").ok_or_else(|| "params.provider is required".to_string())?;
            crate::core::bot::pipeline::connector_hints::remember_decline(user_id, &provider);
            Ok(json!({ "provider": provider, "declined": true }))
        }
        "integrations.catalog.provider" => {
            // #2060 — everything needed to act or start the setup, on demand:
            // auth method, field labels (never values), numbered setup guide.
            let provider =
                str_of("provider").ok_or_else(|| "params.provider is required".to_string())?;
            let item = crate::apps::integration_catalog::provider_by_id(&provider)
                .ok_or_else(|| "integration provider not found".to_string())?;
            Ok(serde_json::to_value(&item).unwrap_or_else(|_| json!({})))
        }
        "integrations.connect.start" => {
            let provider =
                str_of("provider").ok_or_else(|| "params.provider is required".to_string())?;
            let item = crate::apps::integration_catalog::provider_by_id(&provider)
                .ok_or_else(|| "integration provider not found".to_string())?;
            let fields: Vec<serde_json::Value> = item
                .auth
                .fields
                .iter()
                .map(|field| {
                    json!({
                        "key": field.key,
                        "label": field.label,
                        "secret": field.secret,
                        "required": field.required,
                    })
                })
                .collect();
            Ok(json!({
                "provider": item.id,
                "name": item.name,
                "status": item.status,
                "auth_method": item.auth.method,
                "fields": fields,
                "setup_steps": item.auth.setup_steps,
                "least_privilege": item.auth.least_privilege,
                "action_count": item.action_count,
                "connect_path": "/api/bots/{bot_id}/integration-connections",
                "ui_hint": format!("Settings -> Integrations -> {}", item.name),
                "note": "Ask the user for each credential, then submit it through the connect form. Never echo a secret back.",
            }))
        }
        "integrations.actions.list" => {
            let provider =
                str_of("provider").ok_or_else(|| "params.provider is required".to_string())?;
            let actions = crate::apps::integration_catalog::llm_actions(&provider)
                .ok_or_else(|| "integration provider not found".to_string())?;
            Ok(json!({
                "provider": provider,
                "action_count": actions.len(),
                "actions": actions,
            }))
        }
        "integrations.invoke" => {
            #[cfg(feature = "integrations")]
            {
                let provider =
                    str_of("provider").ok_or_else(|| "params.provider is required".to_string())?;
                let action =
                    str_of("action").ok_or_else(|| "params.action is required".to_string())?;
                let mut action_params = serde_json::Map::new();
                if let Some(Value::Object(map)) = obj.get("params") {
                    action_params = map.clone();
                }
                integrations_invoke(
                    state,
                    bot_uuid,
                    user_id,
                    &provider,
                    &action,
                    &Value::Object(action_params),
                )
                .await
            }
            #[cfg(not(feature = "integrations"))]
            {
                Err("integration actions are not available in this build".to_string())
            }
        }
        _ => Err(format!("unknown integration command '{name}'")),
    }
}
