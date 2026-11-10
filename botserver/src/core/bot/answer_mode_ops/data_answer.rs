//! Data answer mode: natural language → SQL → HTML table.

use std::sync::Arc;

use botcore::shared::state::AppState;
use botlib::models::BotResponse;
use uuid::Uuid;

use super::super::answer_mode_config;
use super::html_render::{escape_html, format_results_as_html_table};
use super::llm_text::{call_llm_for_text, extract_sql_from_response};
use super::response_builder::make_bot_response;
use super::sql_exec::execute_sql_query;

pub async fn generate_data_response(
    state: &Arc<AppState>,
    user_text: &str,
    bot_uuid: Uuid,
    _bot_name: &str,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<BotResponse, String> {
    let mut conn = state
        .conn
        .get()
        .map_err(|e| format!("DB connection error: {e}"))?;
    let tables = answer_mode_config::get_bot_table_schemas(&mut conn, bot_uuid)?;
    if tables.is_empty() {
        return Ok(make_bot_response(
            bot_uuid,
            user_id,
            session_id,
            "<p>No tables found in the bot database.</p>".to_string(),
        ));
    }

    let schema_text = answer_mode_config::format_schemas_as_prompt(&tables);

    let sql_prompt = format!(
        "You are a PostgreSQL SQL query generator for a business database.\n\n\
         {schema_text}\n\
         The user asks: \"{user_text}\"\n\n\
         Generate ONLY a valid PostgreSQL SQL query that answers this question.\n\
         Rules:\n\
         1. Use ONLY SELECT queries (no INSERT, UPDATE, DELETE, DROP, TRUNCATE, ALTER, CREATE)\n\
         2. Use row_to_json() for each row to get JSON output\n\
         3. Limit results to 100 rows max\n\
         4. Return ONLY the SQL query, no explanations, no markdown formatting\n\
         5. Use single quotes for string literals\n\
         6. Table and column names should be lowercase\n\
         7. If using COUNT, SUM, AVG, MIN, MAX or any aggregate function, you MUST include GROUP BY for all non-aggregated columns in SELECT\n\
         8. Always include GROUP BY when counting or aggregating by a column\n\n\
         SQL:"
    );

    let llm_result = call_llm_for_text(state, bot_uuid, &sql_prompt).await?;
    let sql = extract_sql_from_response(&llm_result);

    if sql.is_empty() || sql.to_uppercase() == "SQL:" {
        return Ok(make_bot_response(
            bot_uuid,
            user_id,
            session_id,
            format!(
                "<p>Could not generate a SQL query for: <strong>{}</strong></p>\
                     <p style=\"font-size:13px;color:#666;\">LLM raw response: {}</p>",
                escape_html(user_text),
                escape_html(&llm_result)
            ),
        ));
    }

    let bot_pool = state
        .bot_database_manager
        .get_bot_pool(bot_uuid)
        .ok_or_else(|| format!("No database pool for bot {}", bot_uuid))?;
    let mut bot_conn = bot_pool
        .get()
        .map_err(|e| format!("Bot DB connection error: {e}"))?;
    let results = execute_sql_query(&mut bot_conn, &sql)?;
    let table_html = format_results_as_html_table(&results);

    let content = format!(
        "<div class=\"data-mode-response\">\
         <p><strong>Query:</strong> <code style=\"background:#f0f0f0;padding:2px 6px;border-radius:4px;font-size:13px;\">{}</code></p>\
         {table_html}\
         </div>",
        escape_html(&sql)
    );

    Ok(make_bot_response(bot_uuid, user_id, session_id, content))
}
