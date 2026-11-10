//! Database write verbs executed from chat (#2034).
//!
//! The catalogue lists these verbs; this module runs them against the existing
//! `botdatabase` handlers, synthesizing the bot scope header so a chat command
//! uses the same per-bot database as the app. Split out of `execute_command.rs`
//! to keep that dispatcher under the 450-line limit.

use super::*;
use axum::extract::{Json, Path, State};
use axum::http::HeaderMap;

/// True when `name` is one of the database verbs this module executes.
pub(super) fn handles(name: &str) -> bool {
    matches!(
        name,
        "database.rows.insert"
            | "database.rows.update"
            | "database.rows.delete"
            | "database.rows.batch_delete"
            | "database.table.create"
            | "database.column.add"
            | "database.rows.import"
    )
}

/// Executes one database verb with the caller's bot scope.
pub(super) async fn execute(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    name: &str,
    params: &serde_json::Map<String, Value>,
) -> Result<Value, String> {
    let obj = params;
    let str_of = |key: &str| obj.get(key).and_then(|v| v.as_str()).map(|s| s.to_string());
    match name {
        // AutoTask: the command is advertised to the model, so it must be
        // executable here — it used to fall through to the navigation
        // catch-all and answer with a deep link while creating nothing.
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
        _ => Err(format!("unknown database command '{name}'")),
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

#[cfg(test)]
mod database_import_tests {
    #![cfg(feature = "database")]
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
