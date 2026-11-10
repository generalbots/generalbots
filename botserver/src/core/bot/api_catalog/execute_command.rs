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
        // #2063 — the prompt advertises `name: description`; the package body
        // is pulled on demand, exactly like apps.find / api.find.
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
        // #1441 P3 — pipeline/forecast/report commands so chat and WhatsApp can
        // answer CRM questions and capture leads without opening the suite.
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
        // #1441 C8 — operate the pipeline from chat: create a deal, move a
        // stage, capture a contact.
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
        "products.items.list" => {
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
        // AutoTask: the command is advertised to the model, so it must be
        // executable here — it used to fall through to the navigation
        // catch-all and answer with a deep link while creating nothing.
        #[cfg(feature = "automations")]
        #[cfg(feature = "database")]
        "database.rows.insert" => {
            let table = str_of("table").ok_or_else(|| "params.table is required".to_string())?;
            let values = obj
                .get("values")
                .cloned()
                .ok_or_else(|| "params.values is required".to_string())?;
            let result = botdatabase::handlers::insert_row(
                axum::extract::State(Arc::clone(state)),
                database_bot_headers(bot_uuid),
                axum::extract::Path(table),
                axum::extract::Json(values),
            )
            .await
            .map_err(database_error)?;
            Ok(json!({ "success": result.0.success, "message": result.0.message }))
        }
        #[cfg(feature = "database")]
        "database.rows.update" => {
            let table = str_of("table").ok_or_else(|| "params.table is required".to_string())?;
            let id = str_of("id").ok_or_else(|| "params.id is required".to_string())?;
            let data = obj
                .get("values")
                .cloned()
                .ok_or_else(|| "params.values is required".to_string())?;
            let result = botdatabase::handlers::update_row(
                axum::extract::State(Arc::clone(state)),
                database_bot_headers(bot_uuid),
                axum::extract::Path(table),
                axum::extract::Json(botdatabase::handlers::UpdateRowRequest {
                    pk_value: id,
                    data,
                }),
            )
            .await
            .map_err(database_error)?;
            Ok(json!({ "success": result.0.success, "message": result.0.message }))
        }
        #[cfg(feature = "database")]
        "database.rows.delete" => {
            let table = str_of("table").ok_or_else(|| "params.table is required".to_string())?;
            let id = str_of("id").ok_or_else(|| "params.id is required".to_string())?;
            let result = botdatabase::handlers::delete_row(
                axum::extract::State(Arc::clone(state)),
                database_bot_headers(bot_uuid),
                axum::extract::Path((table, id)),
            )
            .await
            .map_err(database_error)?;
            Ok(json!({ "success": result.0.success, "message": result.0.message }))
        }
        #[cfg(feature = "database")]
        "database.rows.batch_delete" => {
            let table = str_of("table").ok_or_else(|| "params.table is required".to_string())?;
            let ids: Vec<String> = serde_json::from_value(
                obj.get("ids")
                    .cloned()
                    .ok_or_else(|| "params.ids is required".to_string())?,
            )
            .map_err(|_| "params.ids must be a JSON array of row ids".to_string())?;
            let result = botdatabase::handlers::batch_delete(
                axum::extract::State(Arc::clone(state)),
                database_bot_headers(bot_uuid),
                axum::extract::Path(table),
                axum::extract::Json(botdatabase::handlers::BatchDeleteRequest { ids }),
            )
            .await
            .map_err(database_error)?;
            Ok(json!({ "success": result.0.success, "message": result.0.message }))
        }
        #[cfg(feature = "database")]
        "database.table.create" => {
            let name = str_of("name").ok_or_else(|| "params.name is required".to_string())?;
            let columns: Vec<botdatabase::handlers::ColumnDef> = serde_json::from_value(
                obj.get("columns")
                    .cloned()
                    .ok_or_else(|| "params.columns is required".to_string())?,
            )
            .map_err(|_| "params.columns must be a JSON array of {name, type}".to_string())?;
            let result = botdatabase::handlers::create_table(
                axum::extract::State(Arc::clone(state)),
                database_bot_headers(bot_uuid),
                axum::extract::Json(botdatabase::handlers::CreateTableRequest {
                    name,
                    columns,
                    table_name: None,
                }),
            )
            .await
            .map_err(database_error)?;
            Ok(json!({ "success": result.0.success, "message": result.0.message }))
        }
        #[cfg(feature = "database")]
        "database.column.add" => {
            let table = str_of("table").ok_or_else(|| "params.table is required".to_string())?;
            let name = str_of("name").ok_or_else(|| "params.name is required".to_string())?;
            let data_type = str_of("type").ok_or_else(|| "params.type is required".to_string())?;
            let result = botdatabase::handlers::add_column(
                axum::extract::State(Arc::clone(state)),
                database_bot_headers(bot_uuid),
                axum::extract::Path(table),
                axum::extract::Json(botdatabase::handlers::AddColumnRequest {
                    name,
                    data_type: Some(data_type),
                    col_type: None,
                    nullable: None,
                    not_null: None,
                    default: None,
                }),
            )
            .await
            .map_err(database_error)?;
            Ok(json!({ "success": result.0.success, "message": result.0.message }))
        }
        #[cfg(feature = "database")]
        "database.rows.import" => {
            let table = str_of("table").ok_or_else(|| "params.table is required".to_string())?;
            let rows = import_rows(obj)?;
            if rows.is_empty() {
                return Err("params.csv or params.json must carry at least one row".to_string());
            }
            let mut inserted = 0usize;
            for row in rows {
                botdatabase::handlers::insert_row(
                    axum::extract::State(Arc::clone(state)),
                    database_bot_headers(bot_uuid),
                    axum::extract::Path(table.clone()),
                    axum::extract::Json(row),
                )
                .await
                .map_err(database_error)?;
                inserted += 1;
            }
            Ok(json!({
                "success": true,
                "table": table,
                "inserted": inserted,
                "message": format!("{inserted} row(s) imported into {table}"),
            }))
        }
        "tasks.autotask.create" | "tasks.autotask.run" => {
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
        _ => {
            // Unknown/mostly-navigation commands resolve to an in-app deep link
            // when the command declares one, otherwise a clear error.
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

/// #2034 — the database handlers authorize on the `X-Bot-Id` header. The chat
/// command already runs for a resolved bot, so synthesize the same scope header
/// instead of trusting client input.
#[cfg(feature = "database")]
fn database_bot_headers(bot_id: Uuid) -> axum::http::HeaderMap {
    let mut headers = axum::http::HeaderMap::new();
    if let Ok(value) = axum::http::HeaderValue::from_str(&bot_id.to_string()) {
        headers.insert("X-Bot-Id", value);
    }
    headers
}

#[cfg(feature = "database")]
fn database_error(
    (_status, axum::Json(body)): (axum::http::StatusCode, axum::Json<Value>),
) -> String {
    body.get("error")
        .and_then(Value::as_str)
        .unwrap_or("database operation failed")
        .to_string()
}

/// #2034 — accepts either a JSON array of objects or CSV text with a header
/// row, so a chat import does not depend on the caller guessing the shape.
#[cfg(feature = "database")]
fn import_rows(params: &serde_json::Map<String, Value>) -> Result<Vec<Value>, String> {
    if let Some(rows) = params.get("json") {
        let rows: Vec<Value> = serde_json::from_value(rows.clone())
            .map_err(|_| "params.json must be an array of objects".to_string())?;
        return Ok(rows);
    }

    let Some(csv) = params.get("csv").and_then(Value::as_str) else {
        return Err("params.csv or params.json is required".to_string());
    };

    let mut lines = csv.lines().filter(|line| !line.trim().is_empty());
    let header_line = lines
        .next()
        .ok_or_else(|| "CSV is empty (a header row is required)".to_string())?;
    let headers = split_csv_line(header_line);
    if headers.is_empty() {
        return Err("CSV header row is empty".to_string());
    }

    let mut rows = Vec::new();
    for line in lines {
        let values = split_csv_line(line);
        if values.len() != headers.len() {
            return Err(format!(
                "CSV row has {} value(s) but the header has {}",
                values.len(),
                headers.len()
            ));
        }
        let mut object = serde_json::Map::new();
        for (key, value) in headers.iter().zip(values) {
            object.insert(key.clone(), Value::String(value));
        }
        rows.push(Value::Object(object));
    }

    Ok(rows)
}

/// Splits one CSV line, honouring double-quoted fields with `""` escapes.
#[cfg(feature = "database")]
fn split_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' if quoted && chars.peek() == Some(&'"') => {
                current.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => {
                fields.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    fields.push(current.trim().to_string());
    fields
}

#[cfg(all(test, feature = "database"))]
mod database_import_tests {
    use super::{import_rows, split_csv_line};
    use serde_json::json;

    fn params(value: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
        match value {
            serde_json::Value::Object(map) => map,
            other => panic!("expected object, got {other}"),
        }
    }

    #[test]
    fn csv_import_maps_the_header_to_each_row() {
        let rows = import_rows(&params(json!({
            "csv": "name,email\nJoão,joao@example.com\nMaria,maria@example.com\n"
        })))
        .expect("rows");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["name"], json!("João"));
        assert_eq!(rows[1]["email"], json!("maria@example.com"));
    }

    #[test]
    fn csv_import_honours_quoted_fields_and_escaped_quotes() {
        let rows = import_rows(&params(json!({
            "csv": "name,note\n\"Silva, João\",\"said \"\"hi\"\"\"\n"
        })))
        .expect("rows");

        assert_eq!(rows[0]["name"], json!("Silva, João"));
        assert_eq!(rows[0]["note"], json!("said \"hi\""));
    }

    #[test]
    fn csv_import_rejects_ragged_rows() {
        let error = import_rows(&params(json!({ "csv": "a,b\n1\n" }))).unwrap_err();
        assert!(error.contains("header has 2"));
    }

    #[test]
    fn json_import_accepts_an_array_of_objects() {
        let rows = import_rows(&params(json!({
            "json": [{ "name": "João" }, { "name": "Maria" }]
        })))
        .expect("rows");

        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn import_requires_csv_or_json() {
        assert!(import_rows(&params(json!({ "table": "clientes" }))).is_err());
    }

    #[test]
    fn csv_split_trims_and_keeps_empty_fields() {
        assert_eq!(split_csv_line(" a , ,c "), vec!["a", "", "c"]);
    }
}
