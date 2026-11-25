use super::*;
use super::command_auth::{authorize_admin_command, requires_admin, resolve_gated_role};

/// Executes a catalog command in-process against the backend state. Enforces
/// the RBAC matrix: admin-only commands and admin-only endpoints are denied
/// to non-admin users.
///
/// Consent is approved ONCE, upstream in `handle_api_call`, for the command
/// the user actually saw (name + params). Every loopback hop here uses
/// `exec_endpoint_trusted`: a gated hop would raise a *second* consent prompt
/// for a sub-request the user never reviewed, and the whole command failed
/// with `consent_required:<id>` even after approval (#2260). RBAC and the
/// registered-route check still run on every hop.
pub async fn execute_command(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    user_id: Uuid,
    name: &str,
    params: &Value,
) -> Result<Value, String> {
    // #2448 — the catalog's `admin_only` flag is enforced here, in the trusted
    // in-process path. An unresolved role is denied, never treated as a user.
    let gated = requires_admin(name);
    let role = gated.then(|| resolve_gated_role(state, user_id)).flatten();
    if let Err(error) = authorize_admin_command(role.as_deref(), gated) {
        botlog::safe_info!("command '{name}' denied: {}", error.sanitized_message());
        return Err(error.into());
    }
    let obj = match params {
        Value::Object(map) => map,
        _ => return Err("'params' must be a JSON object".to_string()),
    };

    // #2372 — command usage stats (catalog command name).
    let cmd_meta = commands_list().iter().find(|c| c.name == name).copied();
    crate::apps::record_command_usage(name, cmd_meta.map(|c| c.app), cmd_meta.map(|c| c.label));
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
            // #2146 — the model builds the digest from this JSON, so the
            // outlet and the publish date travel with every hit instead of
            // being guessed later.
            Ok(json!({
                "query": query,
                "results": results.iter().map(|r| json!({
                    "title": r.title,
                    "url": r.url,
                    "snippet": r.snippet,
                    "outlet": r.source,
                    "published_date": r.published_date,
                })).collect::<Vec<_>>(),
            }))
        }
        "apps.find" => {
            Ok(json!({ "apps": find_apps(str_of("query").map_or_else(Default::default, |v| v).as_str()) }))
        }
        // #2063 — the prompt advertises `name: description`; the body is on demand.
        #[cfg(feature = "sources")]
        "skills.list" => super::skills::list_skills(state, bot_uuid),
        #[cfg(feature = "sources")]
        "skills.find" => super::skills::find_skills(
            state,
            bot_uuid,
            str_of("query").map_or_else(Default::default, |v| v).as_str(),
        ),
        "api.find" => Ok(
            json!({ "matches": find_api_entries(state, user_id, str_of("query").map_or_else(Default::default, |v| v).as_str()) }),
        ),
        // Without the `sources` build the source-backed arms above are not
        // compiled, so the installed-skill listing is the reachable one.
        #[cfg(not(feature = "sources"))]
        "skills.list" => Ok(installed_skills(state, bot_uuid, None)),
        #[cfg(not(feature = "sources"))]
        "skills.find" => Ok(installed_skills(state, bot_uuid, str_of("query"))),
        name if super::execute_integrations::handles(name) => {
            super::execute_integrations::execute(state, bot_uuid, user_id, name, obj).await
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
                ("Username", str_of("username").map_or_else(Default::default, |v| v)),
                ("Password", str_of("password").map_or_else(Default::default, |v| v)),
                ("Port", str_of("port").map_or_else(Default::default, |v| v)),
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
            let intent_text = str_of("intent").map_or_else(Default::default, |v| v);
            let (project_id, project_name) = match str_of("project_id") {
                Some(pid) => (pid, str_of("project_name").map_or_else(Default::default, |v| v)),
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
            crate::core::bot::api_exec::exec_endpoint_trusted(
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
            // #2448 — a prompt injection can name the REST route of an
            // admin-only verb instead of the verb. The catalog flag is the
            // authority here too, so the loopback hop is denied in-process
            // before any HTTP call is made.
            if crate::apps::commands::admin_only_verb_binding(&method, &path) {
                let role = resolve_gated_role(state, user_id);
                if let Err(error) = authorize_admin_command(role.as_deref(), true) {
                    botlog::safe_info!(
                        "api.exec '{method} {path}' denied: {}",
                        error.sanitized_message()
                    );
                    return Err(error.into());
                }
            }
            let mut params = serde_json::Map::new();
            if let Some(Value::Object(map)) = obj.get("params") {
                params = map.clone();
            }
            crate::core::bot::api_exec::exec_endpoint_trusted(
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
            let query = str_of("query").map_or_else(Default::default, |v| v);
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
        "billing.invoice.create" => create_invoice(state, &bot_uuid, obj).await,
        "billing.payment.list" | "billing.payments.list" => list_payments(state, &bot_uuid).await,
        "billing.quote.list" | "billing.quotes.list" => list_quotes(state, &bot_uuid).await,
        "products.items.list" | "store.catalog.list" => {
            list_products(state, &bot_uuid, str_of("category").as_deref()).await
        }
        "products.items.create" => create_product_item(state, &bot_uuid, obj).await,
        "products.items.update" => update_product_item(state, &bot_uuid, obj).await,
        "tickets.list" => list_tickets(state, &bot_uuid).await,
        "drive.list" | "drive.search" => {
            list_drive_files(
                state,
                &bot_uuid,
                str_of("path").as_deref(),
                str_of("query")
                    .or_else(|| str_of("q"))
                    .or_else(|| str_of("name"))
                    .as_deref(),
            )
            .await
        }
        "drive.archive" => {
            let source = str_of("source").map_or_else(Default::default, |v| v);
            let dest = str_of("destination")
                .ok_or_else(|| "params.destination is required".to_string())?;
            archive_drive_files(state, &bot_uuid, &source, &dest).await
        }
        "payroll.diagnosis" => {
            let period = str_of("period");
            payroll_diagnosis(state, &bot_uuid, period.as_deref()).await
        }
        // #2448 — chat-native "undo my last agent batch". `params` are
        // deliberately not read: the shared service derives user, bot and
        // session from the verified command identity only, so a prompt cannot
        // name a foreign scope or batch. The route itself is consent-gated.
        #[cfg(feature = "consent")]
        "docs.undo" => {
            let result =
                crate::main_module::routes::collab_routes::undo_last_batch_for_chat(
                    state, user_id, bot_uuid,
                )
                .await
                .map_err(|error| error.to_string())?;
            serde_json::to_value(result).map_err(|error| error.to_string())
        }
        "monitoring.health" => Ok(json!({ "health": "ok", "note": "suite services running" })),
        #[cfg(feature = "automations")]
        "tasks.autotask.create"
        | "tasks.autotask.run"
        | "automations.create"
        | "automations.run" => {
            let intent = str_of("intent").map_or_else(Default::default, |v| v);
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
        // Values plus styling in one call: the generic loopback below writes
        // cells and drops `styles`/`formats`/`conditional` (#2260, #2263).
        name if sheet_verbs::handles(name) => {
            sheet_verbs::execute(state, user_id, bot_uuid, name, obj).await
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
                let result = crate::core::bot::api_exec::exec_endpoint_trusted(
                    state,
                    user_id,
                    Some(bot_uuid),
                    binding.method,
                    binding.path,
                    &mapped,
                )
                .await?;
                // The endpoint answered with the artifact it created or
                // updated; the deep link is resolved from that id (#2143).
                return Ok(crate::apps::commands::with_navigation(name, obj, result));
            }
            // Navigation commands resolve to a deep link, else a clear error.
            if let Some(cmd) = command_by_name(name) {
                if cmd.deep_link.is_some() {
                    return crate::apps::commands::navigation_result(cmd, obj);
                }
                Err(format!(
                    "command '{name}' is not executable from chat; open the {app} app instead",
                    app = if cmd.app.is_empty() { "suite" } else { cmd.app }
                ))
            } else {
                Err(format!("unknown command '{name}'"))
            }
        }
    }
}
