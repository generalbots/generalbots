use super::*;

/// Executes a catalog command in-process against the backend state. Enforces
/// the RBAC matrix: admin-only commands and admin-only endpoints are denied
/// to non-admin users.
pub async fn execute_command(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    user_id: Uuid,
    name: &str,
    params: &Value,
) -> Result<Value, String> {
    let role = crate::security::user_role::resolve_user_role(&state.conn, user_id);
    if role != crate::security::user_role::ROLE_ADMIN {
        let denied = commands_list()
            .iter()
            .any(|c| c.name == name && c.admin_only);
        if denied {
            return Err(format!("command '{name}' requires the admin role"));
        }
    }
    let obj = match params {
        Value::Object(map) => map,
        _ => return Err("'params' must be a JSON object".to_string()),
    };
    let str_of = |key: &str| obj.get(key).and_then(|v| v.as_str()).map(|s| s.to_string());
    match name {
        "service.tax" | "tax.calculate" => {
            execute_tax(state, &bot_uuid, str_of("service"), str_of("value")).await
        }
        #[cfg(feature = "banking")]
        "banking.diagnosis" => {
            let period = str_of("period");
            let resp = botbanking::cashflow::cashflow_diagnosis_inner(
                &state.conn,
                &bot_uuid,
                None,
                period.as_deref(),
            )
            .await
            .map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(resp).map_err(|e| e.to_string())?)
        }
        #[cfg(feature = "banking")]
        "banking.import" => {
            let file_key =
                str_of("file_key").ok_or_else(|| "params.file_key is required".to_string())?;
            let period = str_of("period");
            let resp = botbanking::cashflow::cashflow_import_inner(
                &state.conn,
                state.drive.as_ref(),
                &bot_uuid,
                None,
                Some(&file_key),
                None,
                period,
            )
            .await
            .map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(resp).map_err(|e| e.to_string())?)
        }
        "drive.write" => {
            let path = str_of("path").ok_or_else(|| "params.path is required".to_string())?;
            let content = str_of("content_base64")
                .ok_or_else(|| "params.content_base64 is required".to_string())?;
            write_drive_file(state, &bot_uuid, &path, &content).await
        }
        "drive.file" => {
            let from = str_of("from").ok_or_else(|| "params.from is required".to_string())?;
            let to = str_of("to").ok_or_else(|| "params.to is required".to_string())?;
            move_drive_file(state, &bot_uuid, &from, &to).await
        }
        #[cfg(any(feature = "research", feature = "web-search"))]
        "web.search" => {
            let query = str_of("query").ok_or_else(|| "params.query is required".to_string())?;
            let max: usize = obj
                .get("max_results")
                .and_then(|v| v.as_u64())
                .unwrap_or(5)
                .min(25) as usize;
            let results = botresearch::web_search::search_web(&query, max, "wt-wt")
                .await
                .map_err(|e| e.to_string())?;
            Ok(json!({
                "query": query,
                "results": results.iter().map(|r| json!({
                    "title": r.title,
                    "url": r.url,
                    "snippet": r.snippet,
                })).collect::<Vec<_>>(),
            }))
        }
        "apps.find" => {
            Ok(json!({ "apps": find_apps(str_of("query").unwrap_or_default().as_str()) }))
        }
        // #2063 — the prompt advertises `name: description`; the body is on demand.
        #[cfg(feature = "sources")]
        "skills.list" => super::skills::list_skills(state, bot_uuid),
        #[cfg(feature = "sources")]
        "skills.find" => super::skills::find_skills(
            state,
            bot_uuid,
            str_of("query").unwrap_or_default().as_str(),
        ),
        "api.find" => Ok(
            json!({ "matches": find_api_entries(state, user_id, str_of("query").unwrap_or_default().as_str()) }),
        ),
        "skills.list" => Ok(installed_skills(state, bot_uuid, None)),
        "skills.find" => Ok(installed_skills(state, bot_uuid, str_of("query"))),
        "integrations.catalog.search" => {
            let providers = crate::apps::integration_catalog::llm_search(
                str_of("q").as_deref(),
                str_of("category").as_deref(),
                str_of("status").as_deref(),
            );
            Ok(json!({ "provider_count": providers.len(), "providers": providers }))
        }
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
        "database.connections.create" => {
            // #1425 — external database connections (conn-*) are
            // provisionable from chat: the keys ride the ConfigManager
            // (sensitive `Password` → Vault per-bot path, the rest →
            // bot_configuration), so the TABLE keyword authenticates the
            // connection and no secret lands in config.csv.
            let conn_name = str_of("name")
                .map(|n| n.trim().to_string())
                .filter(|n| !n.is_empty())
                .ok_or_else(|| "params.name is required".to_string())?;
            if !conn_name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            {
                return Err(
                    "params.name must be alphanumeric (dashes/underscores allowed)".to_string(),
                );
            }
            let server = str_of("server")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "params.server is required".to_string())?;
            let database = str_of("database")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .ok_or_else(|| "params.database is required".to_string())?;
            let prefix = format!("conn-{bot_uuid}-{conn_name}-");
            let config_manager = botcore::config::ConfigManager::new(state.conn.clone());
            let entries: Vec<(&str, String)> = vec![
                ("Server", server),
                ("Name", database),
                ("Username", str_of("username").unwrap_or_default()),
                ("Password", str_of("password").unwrap_or_default()),
                ("Port", str_of("port").unwrap_or_default()),
                (
                    "Driver",
                    str_of("driver").unwrap_or_else(|| "postgres".to_string()),
                ),
            ];
            let mut stored = 0usize;
            for (field, value) in entries {
                if value.is_empty() {
                    continue;
                }
                config_manager
                    .set_config_with_branch(&bot_uuid, &format!("{prefix}{field}"), &value, None)
                    .map_err(|e| format!("persist conn-{conn_name}-{field}: {e}"))?;
                stored += 1;
            }
            Ok(serde_json::json!({
                "success": true,
                "connection": conn_name,
                "stored_fields": stored,
                "note": "the password is stored in the per-bot Vault path (sensitive key); config.csv is never used for secrets",
            }))
        }
        "vibe.project.change" => {
            // The model may call the command without a project_id. Resolve the
            // target in this order:
            //   1. explicit params.project_id
            //   2. an @project mention inside the intent text (the chat
            //      composer prefills "@contato-calc-e2e ..." — the mention IS
            //      the target)
            //   3. the caller's most recently created project in the bot's
            //      branch scope (#1196)
            // Without step 2 the fallback silently landed the edit in whichever
            // project was created last (e.g. an accidental "deploy-the-selected"
            // auto-project) instead of the project the user typed.
            let intent_text = str_of("intent").unwrap_or_default();
            let (project_id, project_name) = match str_of("project_id") {
                Some(pid) => (pid, str_of("project_name").unwrap_or_default()),
                None => match mentioned_project(state, &bot_uuid, &intent_text)? {
                    Some((pid, name)) => (pid, name),
                    None => {
                        // 3) latest-project fallback
                        let branch = branch_scope(state, &bot_uuid).unwrap_or_else(|_| Uuid::nil());
                        let mut conn = state.conn.get().map_err(|e| format!("DB error: {e}"))?;
                        // Projects created from the suite land in the caller's real
                        // branch; the default-bot scope may be the nil branch. Try
                        // both so the fallback finds the user's latest project
                        // regardless of which scope it was created under.
                        let latest = diesel::sql_query(
                            "SELECT id AS project_id, name AS project_name FROM vibe_projects WHERE branch_id = ANY($1) ORDER BY created_at DESC LIMIT 1",
                        )
                        .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(vec![branch, Uuid::nil()])
                        .get_result::<ProjectNameRow>(&mut conn)
                        .ok();
                        match latest {
                            Some(row) => (row.project_id.to_string(), row.project_name),
                            None => {
                                if let Some(name) = str_of("project_name").filter(|n| !n.is_empty())
                                {
                                    (name.clone(), name)
                                } else {
                                    return Err(
                                        "params.project_id is required (no Vibe project found)"
                                            .to_string(),
                                    );
                                }
                            }
                        }
                    }
                },
            };
            if intent_text.trim().is_empty() {
                return Err("params.intent is required".to_string());
            }
            let mut run_params = serde_json::Map::new();
            run_params.insert("intent".to_string(), Value::String(intent_text));
            run_params.insert("project_id".to_string(), Value::String(project_id));
            if !project_name.is_empty() {
                run_params.insert("project_name".to_string(), Value::String(project_name));
            }
            run_params.insert(
                "use_case".to_string(),
                Value::String("software_development".to_string()),
            );
            crate::core::bot::api_exec::exec_endpoint(
                state,
                user_id,
                Some(bot_uuid),
                "POST",
                "/api/vibe/run",
                &run_params,
            )
            .await
        }
        "api.exec" => {
            let method = str_of("method").ok_or_else(|| "params.method is required".to_string())?;
            let path = str_of("path").ok_or_else(|| "params.path is required".to_string())?;
            let mut params = serde_json::Map::new();
            if let Some(Value::Object(map)) = obj.get("params") {
                params = map.clone();
            }
            crate::core::bot::api_exec::exec_endpoint(
                state,
                user_id,
                Some(bot_uuid),
                &method,
                &path,
                &params,
            )
            .await
        }
        "crm.people.list" | "people.list" => list_people(state, &bot_uuid, None).await,
        "crm.people.search" | "people.search" => {
            let query = str_of("query").unwrap_or_default();
            list_people(state, &bot_uuid, Some(&query)).await
        }
        // #1441 P3 — CRM pipeline/forecast/report from chat, no suite needed.
        "crm.pipeline.forecast" => {
            let periods = obj
                .get("periods")
                .and_then(|v| v.as_u64())
                .unwrap_or(3)
                .min(12) as i64;
            crate::core::bot::crm_commands::crm_forecast_command(state, &bot_uuid, periods).await
        }
        "crm.leads.create" => {
            crate::core::bot::crm_commands::crm_create_lead_command(state, &bot_uuid, obj).await
        }
        "crm.leads.report" => {
            crate::core::bot::crm_commands::crm_pipeline_report_command(state, &bot_uuid).await
        }
        // #1441 C8 — operate the pipeline from chat: deal, stage, contact.
        "crm.deal.create" => {
            crate::core::bot::crm_commands::crm_create_deal_command(state, &bot_uuid, obj).await
        }
        "crm.deal.move-stage" => {
            crate::core::bot::crm_commands::crm_move_deal_stage_command(state, &bot_uuid, obj).await
        }
        "crm.contact.create" => {
            crate::core::bot::crm_commands::crm_create_contact_command(state, &bot_uuid, obj).await
        }
        "billing.invoice.list" => list_invoices(state, &bot_uuid).await,
        "products.items.list" | "store.catalog.list" => {
            list_products(state, &bot_uuid, str_of("category").as_deref()).await
        }
        "tickets.list" => list_tickets(state, &bot_uuid).await,
        "drive.list" => list_drive_files(state, &bot_uuid, str_of("path").as_deref()).await,
        "drive.archive" => {
            let source = str_of("source").unwrap_or_default();
            let dest = str_of("destination")
                .ok_or_else(|| "params.destination is required".to_string())?;
            archive_drive_files(state, &bot_uuid, &source, &dest).await
        }
        "payroll.diagnosis" => {
            let period = str_of("period");
            payroll_diagnosis(state, &bot_uuid, period.as_deref()).await
        }
        "monitoring.health" => Ok(json!({ "health": "ok", "note": "suite services running" })),
        "tasks.autotask.create" | "tasks.autotask.run" | "automations.create" | "automations.run" => {
            let intent = str_of("intent").unwrap_or_default();
            if intent.trim().is_empty() {
                return Err(
                    "the 'intent' parameter is required (what should be automated)".to_string(),
                );
            }
            let api = crate::autotask_host::ops::build_api(state);
            let response =
                botautotask::handlers::create_and_execute_for(&api, intent.trim(), bot_uuid).await;
            if !response.success {
                return Err(response.message.clone());
            }
            Ok(serde_json::to_value(&response).unwrap_or_else(|_| json!({})))
        }
        name if docs_verbs::handles(name) => {
            docs_verbs::execute(state, user_id, bot_uuid, name, obj).await
        }
        #[cfg(feature = "database")]
        name if database_verbs::handles(name) => {
            database_verbs::execute(state, bot_uuid, name, obj).await
        }
        _ => {
            // #2037-#2040 — a verb with an endpoint runs through api.exec's
            // loopback path instead of answering with a deep link.
            if let Some(binding) = crate::apps::commands::verb_endpoint(name) {
                let mapped = binding.map_params(obj);
                return crate::core::bot::api_exec::exec_endpoint(
                    state,
                    user_id,
                    Some(bot_uuid),
                    binding.method,
                    binding.path,
                    &mapped,
                )
                .await;
            }
            // Navigation commands resolve to a deep link, else a clear error.
            if let Some(cmd) = command_by_name(name) {
                if let Some(link) = cmd.deep_link {
                    Ok(
                        json!({ "navigate": true, "app": cmd.app, "label": cmd.label, "deep_link": link }),
                    )
                } else {
                    Err(format!(
                        "command '{name}' is not executable from chat; open the {app} app instead",
                        app = if cmd.app.is_empty() { "suite" } else { cmd.app }
                    ))
                }
            } else {
                Err(format!("unknown command '{name}'"))
            }
        }
    }
}

