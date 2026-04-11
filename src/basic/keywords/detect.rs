use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use diesel::prelude::*;
use diesel::sql_types::*;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, QueryableByName)]
struct ColumnRow {
    #[diesel(sql_type = Text)]
    column_name: String,
}

pub fn detect_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let bot_id = user.bot_id;

    engine
        .register_custom_syntax(["DETECT", "$expr$"], false, move |context, inputs| {
            let first_input = inputs.first().ok_or_else(|| {
                Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "DETECT requires a table name".into(),
                    rhai::Position::NONE,
                ))
            })?;
            let table_name = context.eval_expression_tree(first_input)?.to_string();

            let state_for_thread = Arc::clone(&state_clone);
            let bot_id_for_thread = bot_id;
            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build();

                let send_err = if let Ok(rt) = rt {
                    let result = rt.block_on(async move {
                        detect_anomalies_in_table(state_for_thread, &table_name, bot_id_for_thread).await
                    });
                    tx.send(result).err()
                } else {
                    tx.send(Err("failed to build tokio runtime".into())).err()
                };

                if send_err.is_some() {
                    error!("Failed to send DETECT thread result");
                }
            });

            match rx.recv_timeout(std::time::Duration::from_secs(300)) {
                Ok(Ok(result)) => Ok(Dynamic::from(result)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.to_string().into(),
                    rhai::Position::NONE,
                ))),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "Detection timed out".into(),
                        rhai::Position::NONE,
                    )))
                }
                Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("DETECT thread failed: {e}").into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .expect("valid syntax registration");
}

async fn detect_anomalies_in_table(
    state: Arc<AppState>,
    table_name: &str,
    bot_id: Uuid,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let columns = get_table_columns(&state, table_name, bot_id)?;
    let value_field = find_numeric_field(&columns);

    trace!("DETECT: columns = {:?}, value_field = {}", columns, value_field);

    #[derive(QueryableByName)]
    struct JsonRow {
        #[diesel(sql_type = Text)]
        data: String,
    }

    let column_list = columns.join(", ");
    let query = format!(
        "SELECT row_to_json(t)::text as data FROM (SELECT {} FROM {} LIMIT 1500) t",
        column_list, table_name
    );

    let pool = state.bot_database_manager.get_bot_pool(bot_id)?;
    let rows: Vec<JsonRow> = diesel::sql_query(&query)
        .load(&mut pool.get()?)?;

    let records: Vec<Value> = rows
        .into_iter()
        .filter_map(|row| serde_json::from_str(&row.data).ok())
        .collect();

    if records.is_empty() {
        return Err(format!("No data found in table {}", table_name).into());
    }

    let botmodels_host =
        std::env::var("BOTMODELS_HOST").unwrap_or_else(|_| "".to_string());
    let botmodels_key =
        std::env::var("BOTMODELS_API_KEY").unwrap_or_else(|_| "starter".to_string());

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/detect", botmodels_host))
        .header("X-API-Key", &botmodels_key)
        .json(&serde_json::json!({
            "data": records,
            "value_field": value_field
        }))
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("BotModels error: {}", error_text).into());
    }

    let result: Value = response.json().await?;
    Ok(result.to_string())
}

fn get_table_columns(
    state: &Arc<AppState>,
    table_name: &str,
    bot_id: Uuid,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let query = format!(
        "SELECT column_name FROM information_schema.columns WHERE table_name = '{}' ORDER BY ordinal_position",
        table_name
    );

    let pool = state.bot_database_manager.get_bot_pool(bot_id)?;
    let rows: Vec<ColumnRow> = diesel::sql_query(&query)
        .load(&mut pool.get()?)?;

    Ok(rows.into_iter().map(|r| r.column_name).collect())
}

fn find_numeric_field(columns: &[String]) -> String {
    let numeric_keywords = ["salario", "salary", "valor", "value", "amount", "preco", "price", 
                           "temperatura", "temp", "pressao", "pressure", "quantidade", "quantity",
                           "decimal", "numerico", "numeric", "base", "liquido", "bruto",
                           "desconto", "vantagem", "gratificacao"];
    
    for col in columns {
        let col_lower = col.to_lowercase();
        for keyword in &numeric_keywords {
            if col_lower.contains(keyword) {
                return col.clone();
            }
        }
    }
    
    let skip_keywords = ["id", "nome", "cpf", "matricula", "orgao", "cargo", "nivel", 
                        "mes", "ano", "tipo", "categoria", "data", "status", "email", 
                        "telefone", "protocolo", "servidor"];
    
    for col in columns {
        let col_lower = col.to_lowercase();
        let is_string = skip_keywords.iter().any(|kw| col_lower.contains(kw));
        if !is_string {
            return col.clone();
        }
    }
    
    columns.last().cloned().unwrap_or_else(|| "value".to_string())
}
