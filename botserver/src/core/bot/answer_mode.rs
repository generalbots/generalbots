use std::sync::Arc;
use botcore::shared::state::AppState;
use botlib::models::BotResponse;
use uuid::Uuid;
use serde_json::json;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnswerMode {
    Default,
    Data,
    Chart,
}

impl AnswerMode {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "data" => AnswerMode::Data,
            "chart" => AnswerMode::Chart,
            _ => AnswerMode::Default,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AnswerMode::Default => "default",
            AnswerMode::Data => "data",
            AnswerMode::Chart => "chart",
        }
    }
}

pub async fn get_answer_mode(state: &Arc<AppState>, session_id: &Uuid) -> AnswerMode {
    #[cfg(feature = "cache")]
    if let Some(ref cache) = state.cache {
        let key = format!("answer_mode:{}", session_id);
        let mut conn = match cache.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(_) => return AnswerMode::Default,
        };
        let result: Result<String, _> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await;
        return match result {
            Ok(mode_str) => AnswerMode::from_str(&mode_str),
            Err(_) => AnswerMode::Default,
        };
    }
    AnswerMode::Default
}

pub async fn store_answer_mode(
    state: &Arc<AppState>,
    session_id: &Uuid,
    mode: &AnswerMode,
) -> Result<(), String> {
    #[cfg(feature = "cache")]
    if let Some(ref cache) = state.cache {
        let key = format!("answer_mode:{}", session_id);
        let mut conn = cache.get_multiplexed_async_connection().await
            .map_err(|e| format!("Redis error: {}", e))?;
        redis::cmd("SET")
            .arg(&key)
            .arg(mode.as_str())
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| format!("Redis SET error: {}", e))?;
        return Ok(());
    }
    log::warn!("SET ANSWER MODE: cache not available, mode change not persisted");
    Ok(())
}

pub fn get_bot_table_schemas(conn: &mut diesel::PgConnection, bot_id: Uuid) -> Result<Vec<TableSchema>, String> {
    #[derive(QueryableByName, Debug)]
    struct TableRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        table_name: String,
    }

    #[derive(QueryableByName, Debug)]
    struct ColumnRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        table_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        column_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        data_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        is_nullable: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        character_maximum_length: String,
    }

    let tables: Vec<TableRow> = sql_query(
        "SELECT table_name \
         FROM dynamic_table_definitions \
         WHERE bot_id = $1 AND is_active = true \
         ORDER BY table_name"
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .load(conn)
    .map_err(|e| format!("Failed to query bot table definitions: {}", e))?;

    if tables.is_empty() {
        return Ok(Vec::new());
    }

    let table_names: Vec<String> = tables.iter().map(|t| t.table_name.clone()).collect();
    let table_list = table_names
        .iter()
        .map(|n| format!("'{}'", n.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(",");

    let columns_raw: Vec<ColumnRow> = sql_query(&format!(
        "SELECT c.table_name::text, c.column_name::text, \
                c.data_type::text, c.is_nullable::text, \
                COALESCE(c.character_maximum_length::text, '') as character_maximum_length \
         FROM information_schema.columns c \
         WHERE c.table_schema = 'public' \
           AND c.table_name IN ({}) \
         ORDER BY c.table_name, c.ordinal_position",
        table_list
    ))
    .load(conn)
    .map_err(|e| format!("Failed to query materialized columns: {}", e))?;

    let result: Vec<TableSchema> = table_names
        .iter()
        .map(|name| {
            let cols: Vec<ColumnSchema> = columns_raw
                .iter()
                .filter(|c| c.table_name == *name)
                .map(|c| {
                    let pg_type = &c.data_type;
                    let display_type = if pg_type == "character varying" || pg_type == "character" {
                        if c.character_maximum_length.is_empty() {
                            format!("VARCHAR")
                        } else {
                            format!("VARCHAR({})", c.character_maximum_length)
                        }
                    } else if pg_type == "numeric" {
                        format!("NUMERIC")
                    } else if pg_type == "timestamp with time zone" || pg_type == "timestamp without time zone" {
                        format!("TIMESTAMP")
                    } else {
                        pg_type.to_uppercase()
                    };
                    ColumnSchema {
                        name: c.column_name.clone(),
                        data_type: display_type,
                        nullable: c.is_nullable == "YES",
                        is_key: c.column_name == "id",
                    }
                })
                .collect();
            TableSchema {
                name: name.clone(),
                columns: cols,
            }
        })
        .filter(|t| !t.columns.is_empty())
        .collect();

    Ok(result)
}

#[derive(Debug, Clone)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnSchema>,
}

#[derive(Debug, Clone)]
pub struct ColumnSchema {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_key: bool,
}

pub fn format_schemas_as_prompt(tables: &[TableSchema]) -> String {
    let mut result = String::new();
    result.push_str("Database tables defined in tables.bas (materialized):\n\n");
    for table in tables {
        let key_cols: Vec<&str> = table.columns
            .iter()
            .filter(|c| c.is_key)
            .map(|c| c.name.as_str())
            .collect();
        let key_info = if key_cols.is_empty() {
            String::new()
        } else {
            format!(" [primary key: {}]", key_cols.join(", "))
        };
        result.push_str(&format!("Table: {}{}\n", table.name, key_info));
        for col in &table.columns {
            let null_str = if col.nullable { "nullable" } else { "not null" };
            let key_marker = if col.is_key { " KEY" } else { "" };
            result.push_str(&format!(
                "  - {}{} ({}, {})\n",
                col.name, key_marker, col.data_type, null_str
            ));
        }
        result.push('\n');
    }
    result
}

fn extract_sql_from_response(response: &str) -> String {
    let trimmed = response.trim();
    if trimmed.starts_with("```") {
        let without_fence = trimmed.trim_start_matches("```sql")
            .trim_start_matches("```SQL")
            .trim_start_matches("```")
            .trim();
        if let Some(end) = without_fence.rfind("```") {
            without_fence[..end].trim().to_string()
        } else {
            without_fence.to_string()
        }
    } else {
        trimmed.to_string()
    }
}

fn extract_json_from_response(response: &str) -> Result<serde_json::Value, String> {
    let trimmed = response.trim();
    let json_str = if trimmed.starts_with("```") {
        let without_fence = trimmed.trim_start_matches("```json")
            .trim_start_matches("```JSON")
            .trim_start_matches("```")
            .trim();
        if let Some(end) = without_fence.rfind("```") {
            without_fence[..end].trim().to_string()
        } else {
            without_fence.to_string()
        }
    } else {
        trimmed.to_string()
    };
    serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse JSON: {}", e))
}

pub fn execute_sql_query(conn: &mut diesel::PgConnection, sql: &str) -> Result<Vec<serde_json::Value>, String> {
    #[derive(QueryableByName, Debug)]
    struct JsonRow {
        #[diesel(sql_type = Text)]
        row_data: String,
    }

    let safe_sql = sql.trim().trim_end_matches(';');
    let upper = safe_sql.to_uppercase();
    if !upper.starts_with("SELECT") && !upper.starts_with("WITH") {
        return Err("Only SELECT queries are allowed".to_string());
    }
    if upper.contains("INSERT") || upper.contains("UPDATE") || upper.contains("DELETE") || upper.contains("DROP") || upper.contains("TRUNCATE") || upper.contains("ALTER") || upper.contains("CREATE") {
        return Err("Only SELECT queries are allowed".to_string());
    }

    let wrapped_sql = format!(
        "SELECT row_to_json(t)::text as row_data FROM ({}) t LIMIT 100",
        safe_sql
    );

    let rows: Vec<JsonRow> = sql_query(&wrapped_sql)
        .load(conn)
        .map_err(|e| format!("SQL execution error: {}", e))?;

    let results: Vec<serde_json::Value> = rows
        .iter()
        .filter_map(|r| serde_json::from_str(&r.row_data).ok())
        .collect();

    Ok(results)
}

pub fn format_results_as_html_table(results: &[serde_json::Value]) -> String {
    if results.is_empty() {
        return "<p><em>No results found.</em></p>".to_string();
    }

    let first = &results[0];
    let keys: Vec<String> = match first {
        serde_json::Value::Object(map) => map.keys().cloned().collect(),
        _ => return format!("<p>{}</p>", escape_html(&first.to_string())),
    };

    let mut html = String::from(
        "<div style=\"overflow-x:auto;margin:12px 0;\"><table style=\"\
            border-collapse:collapse;width:100%;font-family:system-ui,sans-serif;\
            font-size:14px;box-shadow:0 1px 3px rgba(0,0,0,0.1);border-radius:8px;\
            overflow:hidden;\">"
    );

    html.push_str("<thead style=\"background:#f8f9fa;\"><tr>");
    for key in &keys {
        html.push_str(&format!(
            "<th style=\"padding:10px 12px;text-align:left;font-weight:600;\
             border-bottom:2px solid #dee2e6;white-space:nowrap;\">{}</th>",
            escape_html(key)
        ));
    }
    html.push_str("</tr></thead><tbody>");

    for (i, row) in results.iter().enumerate() {
        let bg = if i % 2 == 0 { "#ffffff" } else { "#f8f9fa" };
        html.push_str(&format!("<tr style=\"background:{};\">", bg));
        if let serde_json::Value::Object(map) = row {
            for key in &keys {
                let val = map.get(key).map(|v| match v {
                    serde_json::Value::Null => String::from("<em>null</em>"),
                    serde_json::Value::String(s) => escape_html(s),
                    other => escape_html(&other.to_string()),
                }).unwrap_or_default();
                html.push_str(&format!(
                    "<td style=\"padding:8px 12px;border-bottom:1px solid #eee;\">{}</td>",
                    val
                ));
            }
        }
        html.push_str("</tr>");
    }

    html.push_str("</tbody></table></div>");
    html.push_str(&format!(
        "<p style=\"font-size:13px;color:#666;margin-top:8px;\">{} row(s) returned</p>",
        results.len()
    ));
    html
}

pub fn format_chart_html(chart_config: &serde_json::Value) -> Result<String, String> {
    let chart_id = format!("chart_{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("main"));
    let config_json = serde_json::to_string_pretty(chart_config)
        .map_err(|e| format!("Failed to serialize chart config: {}", e))?;

    let html = format!(
        r#"<div style="margin:16px 0;padding:16px;background:#fff;border-radius:12px;box-shadow:0 2px 8px rgba(0,0,0,0.08);">
<canvas id="{}" style="max-width:100%;height:auto;"></canvas>
</div>
<script>
(function() {{
function renderChart_{}() {{
var ctx = document.getElementById('{}');
if (!ctx) return;
var config = {};
if (typeof Chart !== 'undefined') {{
new Chart(ctx, config);
}} else {{
var s = document.createElement('script');
s.src = '/suite/js/vendor/chart.umd.min.js';
s.onload = function() {{ new Chart(ctx, config); }};
document.head.appendChild(s);
}}
}}
renderChart_{}();
}})();
</script>"#,
        chart_id, chart_id, chart_id, config_json, chart_id
    );
    Ok(html)
}

pub async fn generate_data_response(
    state: &Arc<AppState>,
    user_text: &str,
    bot_uuid: Uuid,
    _bot_name: &str,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<BotResponse, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB connection error: {}", e))?;
    let tables = get_bot_table_schemas(&mut conn, bot_uuid)?;
    if tables.is_empty() {
        return Ok(BotResponse {
            bot_id: bot_uuid.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            channel: "web".to_string(),
            content: "<p>No tables found in the bot database.</p>".to_string(),
            message_type: botlib::message_types::MessageType(2),
            stream_token: None,
            is_complete: true,
            suggestions: vec![],
            switchers: vec![],
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        });
    }

    let schema_text = format_schemas_as_prompt(&tables);

    let sql_prompt = format!(
        "You are a PostgreSQL SQL query generator for a business database.\n\n\
         {}\n\
         The user asks: \"{}\"\n\n\
         Generate ONLY a valid PostgreSQL SQL query that answers this question.\n\
         Rules:\n\
         1. Use ONLY SELECT queries (no INSERT, UPDATE, DELETE, DROP, TRUNCATE, ALTER, CREATE)\n\
         2. Use row_to_json() for each row to get JSON output\n\
         3. Limit results to 100 rows max\n\
         4. Return ONLY the SQL query, no explanations, no markdown formatting\n\
         5. Use single quotes for string literals\n\
         6. Table and column names should be lowercase\n\n\
         SQL:",
        schema_text, user_text
    );

    let llm_result = call_llm_for_text(state, bot_uuid, &sql_prompt).await?;
    let sql = extract_sql_from_response(&llm_result);

    if sql.is_empty() || sql.to_uppercase() == "SQL:" {
        return Ok(BotResponse {
            bot_id: bot_uuid.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            channel: "web".to_string(),
            content: format!("<p>Could not generate a SQL query for: <strong>{}</strong></p>\
                             <p style=\"font-size:13px;color:#666;\">LLM raw response: {}</p>",
                             escape_html(user_text), escape_html(&llm_result)),
            message_type: botlib::message_types::MessageType(2),
            stream_token: None,
            is_complete: true,
            suggestions: vec![],
            switchers: vec![],
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        });
    }

    let results = execute_sql_query(&mut conn, &sql)?;
    let table_html = format_results_as_html_table(&results);

    let content = format!(
        "<div class=\"data-mode-response\">\
         <p><strong>Query:</strong> <code style=\"background:#f0f0f0;padding:2px 6px;border-radius:4px;font-size:13px;\">{}</code></p>\
         {}\
         </div>",
        escape_html(&sql), table_html
    );

    Ok(BotResponse {
        bot_id: bot_uuid.to_string(),
        user_id: user_id.to_string(),
        session_id: session_id.to_string(),
        channel: "web".to_string(),
        content,
        message_type: botlib::message_types::MessageType(2),
        stream_token: None,
        is_complete: true,
        suggestions: vec![],
        switchers: vec![],
        context_name: None,
        context_length: 0,
        context_max_length: 0,
    })
}

pub async fn generate_chart_response(
    state: &Arc<AppState>,
    user_text: &str,
    bot_uuid: Uuid,
    _bot_name: &str,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<BotResponse, String> {
    let mut conn = state.conn.get().map_err(|e| format!("DB connection error: {}", e))?;
    let tables = get_bot_table_schemas(&mut conn, bot_uuid)?;
    if tables.is_empty() {
        return Ok(BotResponse {
            bot_id: bot_uuid.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            channel: "web".to_string(),
            content: "<p>No tables found in the bot database.</p>".to_string(),
            message_type: botlib::message_types::MessageType(2),
            stream_token: None,
            is_complete: true,
            suggestions: vec![],
            switchers: vec![],
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        });
    }

    let schema_text = format_schemas_as_prompt(&tables);

    let sql_prompt = format!(
        "You are a PostgreSQL SQL query generator for a business database.\n\n\
         {}\n\
         The user asks: \"{}\"\n\n\
         Generate ONLY a valid PostgreSQL SQL query that answers this question.\n\
         Rules:\n\
         1. Use ONLY SELECT queries\n\
         2. Limit results to 100 rows max\n\
         3. Return ONLY the SQL query, no explanations, no markdown\n\
         4. Use single quotes for string literals\n\
         5. Table and column names should be lowercase\n\n\
         SQL:",
        schema_text, user_text
    );

    let llm_result = call_llm_for_text(state, bot_uuid, &sql_prompt).await?;
    let sql = extract_sql_from_response(&llm_result);

    if sql.is_empty() {
        return Ok(BotResponse {
            bot_id: bot_uuid.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            channel: "web".to_string(),
            content: format!("<p>Could not generate a SQL query for: <strong>{}</strong></p>", escape_html(user_text)),
            message_type: botlib::message_types::MessageType(2),
            stream_token: None,
            is_complete: true,
            suggestions: vec![],
            switchers: vec![],
            context_name: None,
            context_length: 0,
            context_max_length: 0,
        });
    }

    let results = execute_sql_query(&mut conn, &sql)?;
    let results_json = serde_json::to_value(&results).unwrap_or(json!([]));
    let results_str = serde_json::to_string_pretty(&results_json).unwrap_or_default();

    let chart_prompt = format!(
        "You are a Chart.js v4 configuration generator.\n\n\
         The user asked: \"{}\"\n\n\
         The SQL query executed was:\n```sql\n{}\n```\n\n\
         The result data is:\n```json\n{}\n```\n\n\
         Generate ONLY a valid Chart.js v4 configuration object (JSON only, no code fences, no explanations).\n\
         The configuration must have this structure:\n\
         {{\n\
           \"type\": \"bar\" | \"line\" | \"pie\" | \"doughnut\" | \"radar\" | \"polarArea\",\n\
           \"data\": {{\n\
             \"labels\": [...],\n\
             \"datasets\": [{{\"label\": \"...\", \"data\": [...]}}]\n\
           }},\n\
           \"options\": {{\n\
             \"responsive\": true,\n\
             \"plugins\": {{\"title\": {{\"display\": true, \"text\": \"...\"}}}}\n\
           }}\n\
         }}\n\n\
         Analyze the data columns and pick appropriate x-axis (labels) and y-axis (datasets) fields.\n\
         If the data has a date/time column, sort by it. Choose the best chart type for the data.\n\n\
         Return ONLY valid JSON, no other text.",
        user_text, &sql, &results_str
    );

    let chart_result = call_llm_for_text(state, bot_uuid, &chart_prompt).await?;
    let chart_config = extract_json_from_response(&chart_result)?;
    let chart_html = format_chart_html(&chart_config)?;
    let table_html = format_results_as_html_table(&results);

    let content = format!(
        "<div class=\"chart-mode-response\">\
         {}\
         {}\
         </div>",
        chart_html, table_html
    );

    Ok(BotResponse {
        bot_id: bot_uuid.to_string(),
        user_id: user_id.to_string(),
        session_id: session_id.to_string(),
        channel: "web".to_string(),
        content,
        message_type: botlib::message_types::MessageType(2),
        stream_token: None,
        is_complete: true,
        suggestions: vec![],
        switchers: vec![],
        context_name: None,
        context_length: 0,
        context_max_length: 0,
    })
}

async fn call_llm_for_text(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    prompt: &str,
) -> Result<String, String> {
    use botcore::config::ConfigManager;

    let cfg = ConfigManager::new(state.conn.clone());
    let llm_url = cfg.get_config(&bot_uuid, "llm-url", Some("")).unwrap_or_default();
    let llm_key = cfg.get_config(&bot_uuid, "llm-key", Some("")).unwrap_or_default();
    let llm_model = cfg.get_config(&bot_uuid, "llm-model", Some("")).unwrap_or_default();

    let provider: Arc<dyn botlib::traits::LLMProvider> = if !llm_url.is_empty() {
        let inner = crate::llm::create_llm_provider_from_url(
            &llm_url,
            if llm_model.is_empty() { None } else { Some(llm_model.clone()) },
            None, None,
        );
        Arc::new(crate::llm::BotlibLLMProviderWrapper::new(inner, llm_model, llm_key)) as Arc<dyn botlib::traits::LLMProvider>
    } else if let Some(ref global) = state.llm_provider {
        global.clone()
    } else {
        return Err("No LLM provider available".to_string());
    };

    provider.generate_simple(prompt).await
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
