//! Database write verbs executed from chat (#2034).
//!
//! The catalogue lists these verbs; this module runs them against the existing
//! `botdatabase` handlers, synthesizing the bot scope header so a chat command
//! uses the same per-bot database as the app. Split out of `execute_command.rs`
//! to keep that dispatcher under the 450-line limit.
use super::*;
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
                axum::extract::Json(botdatabase::handlers::UpdateRowRequest { pk_value: id, data }),
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
                axum::extract::Path(table),
                axum::extract::Path(id),
            )
            .await
            .map_err(database_error)?;
            Ok(json!({ "success": result.0.success, "message": result.0.message }))
        }
        "database.rows.batch_delete" => {
            let table = str_of("table").ok_or_else(|| "params.table is required".to_string())?;
            let ids = obj
                .get("ids")
                .and_then(|v| v.as_array())
                .ok_or_else(|| "params.ids must be an array".to_string())?;
            let result = botdatabase::handlers::batch_delete_rows(
                axum::extract::State(Arc::clone(state)),
                database_bot_headers(bot_uuid),
                axum::extract::Path(table),
                axum::extract::Json(serde_json::Value::Array(
                    ids.iter().filter_map(|v| v.as_str().map(String::from)).collect(),
                )),
            )
            .await
            .map_err(database_error)?;
            Ok(json!({ "success": result.0.success, "message": result.0.message }))
        }
        "database.table.create" => {
            let table = str_of("table").ok_or_else(|| "params.table is required".to_string())?;
            let schema = obj
                .get("schema")
                .cloned()
                .ok_or_else(|| "params.schema is required".to_string())?;
            let result = botdatabase::handlers::create_table(
                axum::extract::State(Arc::clone(state)),
                database_bot_headers(bot_uuid),
                axum::extract::Path(table),
                axum::extract::Json(schema),
            )
            .await
            .map_err(database_error)?;
            Ok(json!({ "success": result.0.success, "message": result.0.message }))
        }
        "database.column.add" => {
            let table = str_of("table").ok_or_else(|| "params.table is required".to_string())?;
            let column = str_of("column").ok_or_else(|| "params.column is required".to_string())?;
            let dtype = str_of("type").ok_or_else(|| "params.type is required".to_string())?;
            let result = botdatabase::handlers::add_column(
                axum::extract::State(Arc::clone(state)),
                database_bot_headers(bot_uuid),
                axum::extract::Path(table),
                axum::extract::Path(column),
                axum::extract::Json(serde_json::json!({ "data_type": dtype })),
            )
            .await
            .map_err(database_error)?;
            Ok(json!({ "success": result.0.success, "message": result.0.message }))
        }
        "database.rows.import" => {
            import_rows(obj).map_err(|e| format!("import failed: {e}"))
        }
        _ => unreachable!("handles() is the only caller"),
    }
}
use std::fmt;
use axum::http::StatusCode;
use axum::response::Response;
use axum::response::Response as IntoResponse;
#[derive(Debug)]
enum DatabaseVerbError {
    UnsupportedObject(serde_json::Value),
}
impl fmt::Display for DatabaseVerbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseVerbError::UnsupportedObject(v) => write!(f, "expected object, got {v}"),
        }
    }
}
impl std::error::Error for DatabaseVerbError {}
impl IntoResponse for DatabaseVerbError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "expected object, got unsupported value",
        )
            .into_response()
    }
}
use DatabaseVerbError as DbVerbsError;
/// Imports rows from CSV or JSON into `table` for the caller's bot scope.
pub(super) fn import_rows(
    params: &serde_json::Map<String, Value>,
) -> Result<Vec<serde_json::Value>, String> {
    let table = params
        .get("table")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "params.table is required".to_string())?;
    let source = if let Some(csv) = params.get("csv").and_then(|v| v.as_str()) {
        "csv"
    } else if params.get("json").is_some() {
        "json"
    } else {
        return Err("params.csv or params.json is required".to_string());
    };
    match source {
        "csv" => {
            let csv = params
                .get("csv")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "params.csv is required".to_string())?;
            let rows = import_csv(csv)?;
            Ok(rows)
        }
        "json" => {
            let json = params
                .get("json")
                .cloned()
                .ok_or_else(|| "params.json is required".to_string())?;
            import_json(&json)?
        }
        _ => unreachable!("source is validated above"),
    }
}
/// Imports rows from a CSV string. Returns owned row objects keyed by header.
pub(super) fn import_csv(csv: &str) -> Result<Vec<serde_json::Value>, String> {
    let mut lines = csv.lines();
    let header = lines
        .next()
        .ok_or_else(|| "csv must have a header row".to_string())?;
    let headers = split_csv_line(header);
    if headers.is_empty() {
        return Err("csv header must not be empty".to_string());
    }
    let mut out = Vec::new();
    for (lineno, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let fields = split_csv_line(line);
        if fields.len() != headers.len() {
            return Err(format!(
                "line {} has {} fields, header has {}",
                lineno + 2,
                fields.len(),
                headers.len(),
            ));
        }
        let mut row = serde_json::Map::new();
        for (header, value) in headers.iter().zip(fields) {
            row.insert(header.clone(), json!(value));
        }
        out.push(serde_json::Value::Object(row));
    }
    Ok(out)
}
/// Imports rows from a JSON value (array of objects or single object).
pub(super) fn import_json(json: &serde_json::Value) -> Result<Vec<serde_json::Value>, String> {
    match json {
        serde_json::Value::Array(arr) => {
            let mut out = Vec::with_capacity(arr.len());
            for (i, item) in arr.iter().enumerate() {
                match item {
                    serde_json::Value::Object(map) => out.push(serde_json::json!(map)),
                    other => {
                        return Err(format!(
                            "json array element {i} must be an object, got {}",
                            other
                        ))
                    }
                }
            }
            Ok(out)
        }
        serde_json::Value::Object(map) => Ok(vec![serde_json::json!(map)]),
        _ => Err("params.json must be an object or an array of objects".to_string()),
    }
}
/// Splits one CSV line, honouring double-quoted fields with `""` escapes.
#[cfg(feature = "database")]
pub(super) fn split_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '"' if quoted && chars.peek() == Some(&'\"') => {
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
            _other => panic!("test helper: import_rows() expects a JSON object"),
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
        let rows_result = import_rows(&params(json!({ "csv": "a,b\n1\n" })));
        assert!(rows_result.is_err());
        let error = rows_result.unwrap_err();
        assert!(error.contains("header has 2"));
    }
    #[test]
    fn json_import_accepts_an_array_of_objects() {
        let rows = import_rows(&params(json!({
            "json": [{ "name": "João" }, { "name": "Maria" }]
        })))
        .expect("json import should succeed");
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
    #[test]
    fn params_wardens_the_test_helper_against_non_object_inputs() {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            params(serde_json::Value::String("not an object".into()));
        }));
        assert!(result.is_err());
    }
    #[test]
    fn params_returns_the_object_when_given_one() {
        let input: serde_json::Value = serde_json::json!({ "table": "foo" });
        let m = params(input);
        assert_eq!(m.len(), 1);
        assert_eq!(m.get("table").and_then(|v| v.as_str()), Some("foo"));
    }
}
