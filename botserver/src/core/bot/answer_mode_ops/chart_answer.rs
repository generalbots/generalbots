//! Chart answer mode: natural language → SQL → Chart.js configuration.

use std::sync::Arc;

use botcore::shared::state::AppState;
use botlib::models::BotResponse;
use serde_json::json;
use uuid::Uuid;

use super::super::answer_mode_config;
use super::html_render::{escape_html, format_chart_html, format_results_as_html_table};
use super::llm_text::{call_llm_for_text, extract_json_from_response, extract_sql_from_response};
use super::response_builder::make_bot_response;
use super::sql_exec::execute_sql_query;

pub async fn generate_chart_response(
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
         1. Use ONLY SELECT queries\n\
         2. Limit results to 100 rows max\n\
         3. Return ONLY the SQL query, no explanations, no markdown\n\
         4. Use single quotes for string literals\n\
         5. Table and column names should be lowercase\n\
         6. If using COUNT, SUM, AVG, MIN, MAX or any aggregate function, you MUST include GROUP BY for all non-aggregated columns in SELECT\n\
         7. Always include GROUP BY when counting or aggregating by a column\n\
         8. DATE GROUPING: when grouping by a date column, ALWAYS format it for humans. Use EXTRACT(YEAR FROM <datecol>) or EXTRACT(MONTH FROM <datecol>) or TO_CHAR(<datecol>, 'YYYY-MM') and ALIAS the extracted value with a clear label (ex: 'mes', 'ano', 'periodo'). Never group by the raw date column when the user asks by month/year.\n\
         9. NEVER return null or empty labels. If a date/aggregate column can be null, wrap it with COALESCE({{col}}, 'Sem registro') or filter it out with WHERE {{col}} IS NOT NULL.\n\
         10. Do not fabricate data — if no rows match, the query still returns empty.\n\n\
         SQL:"
    );

    let llm_result = call_llm_for_text(state, bot_uuid, &sql_prompt).await?;
    let sql = extract_sql_from_response(&llm_result);

    if sql.is_empty() {
        return Ok(make_bot_response(
            bot_uuid,
            user_id,
            session_id,
            format!(
                "<p>Could not generate a SQL query for: <strong>{}</strong></p>",
                escape_html(user_text)
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
    let results_json = serde_json::to_value(&results).unwrap_or(json!([]));
    let results_str = serde_json::to_string_pretty(&results_json).unwrap_or_default();

    let chart_prompt = format!(
        "You are a Chart.js v4 configuration generator.\n\n\
         The user asked: \"{user_text}\"\n\n\
         The SQL query executed was:\n```sql\n{sql}\n```\n\n\
         The result data is:\n```json\n{results_str}\n```\n\n\
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
         Return ONLY valid JSON, no other text."
    );

    let chart_result = call_llm_for_text(state, bot_uuid, &chart_prompt).await?;
    let chart_config = extract_json_from_response(&chart_result)?;
    let chart_html = format_chart_html(&chart_config)?;
    let table_html = format_results_as_html_table(&results);

    let content = format!(
        "<div class=\"chart-mode-response\">\
         {chart_html}\
         {table_html}\
         </div>"
    );

    Ok(make_bot_response(bot_uuid, user_id, session_id, content))
}
