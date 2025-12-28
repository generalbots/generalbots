/*****************************************************************************\
|  █████  █████ ██    █ █████ █████   ████  ██      ████   █████ █████  ███ ® |
| ██      █     ███   █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █   █      |
| ██  ███ ████  █ ██  █ ████  █████  ██████ ██      ████   █   █   █    ██    |
| ██   ██ █     █  ██ █ █     ██  ██ ██  ██ ██      ██  █ ██   ██  █      █   |
|  █████  █████ █   ███ █████ ██  ██ ██  ██ █████   ████   █████   █   ███    |
|                                                                             |
| General Bots Copyright (c) pragmatismo.com.br. All rights reserved.         |
| Licensed under the AGPL-3.0.                                                |
|                                                                             |
| According to our dual licensing model, this program can be used either      |
| under the terms of the GNU Affero General Public License, version 3,        |
| or under a proprietary license.                                             |
|                                                                             |
| The texts of the GNU Affero General Public License with an additional       |
| permission and of our proprietary license can be found at and               |
| in the LICENSE file you have received along with this program.              |
|                                                                             |
| This program is distributed in the hope that it will be useful,             |
| but WITHOUT ANY WARRANTY, without even the implied warranty of              |
| MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the                |
| GNU Affero General Public License for more details.                         |
|                                                                             |
| "General Bots" is a registered trademark of pragmatismo.com.br.             |
| The licensing of the program under the AGPLv3 does not imply a              |
| trademark license. Therefore any rights, title and interest in              |
| our trademarks remain entirely with us.                                     |
|                                                                             |
\*****************************************************************************/

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, trace};
use rhai::{Array, Dynamic, Engine, Map};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::Arc;

pub fn register_import_export(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    register_import_keyword(Arc::clone(&state), user.clone(), engine);
    register_export_keyword(state, user, engine);
}

pub fn register_import_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(["IMPORT", "$expr$"], false, move |context, inputs| {
            let file_path = context.eval_expression_tree(&inputs[0])?.to_string();

            trace!("IMPORT: Loading data from {}", file_path);

            let state_for_task = Arc::clone(&state_clone);
            let user_for_task = user.clone();

            let (tx, rx) = std::sync::mpsc::channel::<Result<Value, String>>();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build();

                let send_err = if let Ok(_rt) = rt {
                    let result = execute_import_json(&state_for_task, &user_for_task, &file_path);
                    tx.send(result).err()
                } else {
                    tx.send(Err("Failed to build tokio runtime".into())).err()
                };

                if send_err.is_some() {
                    error!("Failed to send IMPORT result from thread");
                }
            });

            match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                Ok(Ok(json_result)) => Ok(json_to_dynamic(&json_result)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("IMPORT failed: {}", e).into(),
                    rhai::Position::NONE,
                ))),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "IMPORT timed out".into(),
                        rhai::Position::NONE,
                    )))
                }
                Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("IMPORT thread failed: {}", e).into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .unwrap();
}

pub fn register_export_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);

    engine
        .register_custom_syntax(
            ["EXPORT", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let file_path = context.eval_expression_tree(&inputs[0])?.to_string();
                let data = context.eval_expression_tree(&inputs[1])?;

                trace!("EXPORT: Saving data to {}", file_path);

                let data_json = dynamic_to_json_value(&data);

                let state_for_task = Arc::clone(&state_clone);
                let user_for_task = user.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(_rt) = rt {
                        let result = execute_export_json(
                            &state_for_task,
                            &user_for_task,
                            &file_path,
                            data_json,
                        );
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".into())).err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send EXPORT result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(60)) {
                    Ok(Ok(result)) => Ok(Dynamic::from(result)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("EXPORT failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                            "EXPORT timed out".into(),
                            rhai::Position::NONE,
                        )))
                    }
                    Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("EXPORT thread failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .unwrap();
}

fn execute_import_json(
    state: &AppState,
    user: &UserSession,
    file_path: &str,
) -> Result<Value, String> {
    match execute_import(state, user, file_path) {
        Ok(dynamic) => Ok(dynamic_to_json(&dynamic)),
        Err(e) => Err(e.to_string()),
    }
}

fn execute_export_json(
    state: &AppState,
    user: &UserSession,
    file_path: &str,
    data_json: Value,
) -> Result<String, String> {
    let data = json_to_dynamic(&data_json);
    match execute_export(state, user, file_path, data) {
        Ok(result) => Ok(result),
        Err(e) => Err(e.to_string()),
    }
}

fn dynamic_to_json_value(data: &Dynamic) -> Value {
    dynamic_to_json(data)
}

fn execute_import(
    state: &AppState,
    user: &UserSession,
    file_path: &str,
) -> Result<Dynamic, Box<dyn std::error::Error + Send + Sync>> {
    let full_path = resolve_file_path(state, user, file_path)?;
    let extension = Path::new(&full_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "csv" => import_csv(&full_path),
        "json" => import_json(&full_path),
        "xlsx" | "xls" => import_excel(&full_path),
        "tsv" => import_tsv(&full_path),
        _ => Err(format!("Unsupported file format: .{}", extension).into()),
    }
}

fn execute_export(
    state: &AppState,
    user: &UserSession,
    file_path: &str,
    data: Dynamic,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let full_path = resolve_export_path(state, user, file_path)?;
    let extension = Path::new(&full_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "csv" => export_csv(&full_path, data),
        "json" => export_json(&full_path, data),
        "xlsx" => export_excel(&full_path, data),
        "tsv" => export_tsv(&full_path, data),
        _ => Err(format!("Unsupported export format: .{}", extension).into()),
    }
}

fn resolve_file_path(
    state: &AppState,
    user: &UserSession,
    file_path: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if file_path.starts_with("http://") || file_path.starts_with("https://") {
        return Ok(file_path.to_string());
    }

    if Path::new(file_path).is_absolute() {
        return Ok(file_path.to_string());
    }

    let data_dir = state
        .config
        .as_ref()
        .map(|c| c.data_dir.as_str())
        .unwrap_or("./botserver-stack/data");
    let base_path = format!("{}/bots/{}/gbdrive", data_dir, user.bot_id);

    let full_path = format!("{}/{}", base_path, file_path);

    if Path::new(&full_path).exists() {
        Ok(full_path)
    } else {
        let alt_path = format!("{}/bots/{}/{}", data_dir, user.bot_id, file_path);
        if Path::new(&alt_path).exists() {
            Ok(alt_path)
        } else {
            Err(format!("File not found: {}", file_path).into())
        }
    }
}

fn resolve_export_path(
    state: &AppState,
    user: &UserSession,
    file_path: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if Path::new(file_path).is_absolute() {
        return Ok(file_path.to_string());
    }

    let data_dir = state
        .config
        .as_ref()
        .map(|c| c.data_dir.as_str())
        .unwrap_or("./botserver-stack/data");
    let base_path = format!("{}/bots/{}/gbdrive", data_dir, user.bot_id);

    std::fs::create_dir_all(&base_path)?;

    Ok(format!("{}/{}", base_path, file_path))
}

fn import_csv(file_path: &str) -> Result<Dynamic, Box<dyn std::error::Error + Send + Sync>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let headers: Vec<String> = match lines.next() {
        Some(Ok(line)) => parse_csv_line(&line),
        _ => return Err("CSV file is empty or has no header".into()),
    };

    let mut results: Array = Array::new();

    for line in lines.map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }

        let values = parse_csv_line(&line);
        let mut row_map: Map = Map::new();

        for (i, header) in headers.iter().enumerate() {
            let value = values.get(i).map(|s| s.as_str()).unwrap_or("");
            row_map.insert(header.clone().into(), Dynamic::from(value.to_string()));
        }

        results.push(Dynamic::from(row_map));
    }

    trace!("Imported {} rows from CSV", results.len());
    Ok(Dynamic::from(results))
}

fn import_tsv(file_path: &str) -> Result<Dynamic, Box<dyn std::error::Error + Send + Sync>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let headers: Vec<String> = match lines.next() {
        Some(Ok(line)) => line.split('\t').map(|s| s.trim().to_string()).collect(),
        _ => return Err("TSV file is empty or has no header".into()),
    };

    let mut results: Array = Array::new();

    for line in lines.map_while(Result::ok) {
        if line.trim().is_empty() {
            continue;
        }

        let values: Vec<String> = line.split('\t').map(|s| s.trim().to_string()).collect();
        let mut row_map: Map = Map::new();

        for (i, header) in headers.iter().enumerate() {
            let value = values.get(i).map(|s| s.as_str()).unwrap_or("");
            row_map.insert(header.clone().into(), Dynamic::from(value.to_string()));
        }

        results.push(Dynamic::from(row_map));
    }

    trace!("Imported {} rows from TSV", results.len());
    Ok(Dynamic::from(results))
}

fn import_json(file_path: &str) -> Result<Dynamic, Box<dyn std::error::Error + Send + Sync>> {
    let content = std::fs::read_to_string(file_path)?;
    let json_value: Value = serde_json::from_str(&content)?;

    let result = json_to_dynamic(&json_value);
    trace!("Imported JSON data");
    Ok(result)
}

fn import_excel(file_path: &str) -> Result<Dynamic, Box<dyn std::error::Error + Send + Sync>> {
    use calamine::{open_workbook, Reader, Xlsx};

    let mut workbook: Xlsx<_> = open_workbook(file_path)?;

    let sheet_names: Vec<String> = workbook.sheet_names();
    if sheet_names.is_empty() {
        return Err("Excel file has no sheets".into());
    }

    let range = workbook
        .worksheet_range(&sheet_names[0])
        .map_err(|e| format!("Failed to read sheet: {}", e))?;

    let mut rows_iter = range.rows();

    let headers: Vec<String> = match rows_iter.next() {
        Some(row) => row
            .iter()
            .map(|cell| cell.to_string().trim().to_string())
            .collect(),
        None => return Err("Excel sheet is empty".into()),
    };

    let mut results: Array = Array::new();

    for row in rows_iter {
        let mut row_map: Map = Map::new();

        for (i, header) in headers.iter().enumerate() {
            let value = row.get(i).map(|cell| cell.to_string()).unwrap_or_default();
            row_map.insert(header.clone().into(), Dynamic::from(value));
        }

        results.push(Dynamic::from(row_map));
    }

    trace!("Imported {} rows from Excel", results.len());
    Ok(Dynamic::from(results))
}

fn export_csv(
    file_path: &str,
    data: Dynamic,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let array = to_array(data);
    if array.is_empty() {
        return Err("No data to export".into());
    }

    let headers = get_headers_from_array(&array);

    let mut file = File::create(file_path)?;

    writeln!(file, "{}", headers.join(","))?;

    for item in array {
        let map = dynamic_to_map(&item);
        let values: Vec<String> = headers
            .iter()
            .map(|h| {
                let val = map.get(h).map(|v| v.to_string()).unwrap_or_default();
                escape_csv_value(&val)
            })
            .collect();
        writeln!(file, "{}", values.join(","))?;
    }

    trace!("Exported data to CSV: {}", file_path);
    Ok(file_path.to_string())
}

fn export_tsv(
    file_path: &str,
    data: Dynamic,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let array = to_array(data);
    if array.is_empty() {
        return Err("No data to export".into());
    }

    let headers = get_headers_from_array(&array);

    let mut file = File::create(file_path)?;

    writeln!(file, "{}", headers.join("\t"))?;

    for item in array {
        let map = dynamic_to_map(&item);
        let values: Vec<String> = headers
            .iter()
            .map(|h| map.get(h).map(|v| v.to_string()).unwrap_or_default())
            .collect();
        writeln!(file, "{}", values.join("\t"))?;
    }

    trace!("Exported data to TSV: {}", file_path);
    Ok(file_path.to_string())
}

fn export_json(
    file_path: &str,
    data: Dynamic,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let json_value = dynamic_to_json(&data);
    let json_string = serde_json::to_string_pretty(&json_value)?;

    std::fs::write(file_path, json_string)?;

    trace!("Exported data to JSON: {}", file_path);
    Ok(file_path.to_string())
}

fn export_excel(
    file_path: &str,
    data: Dynamic,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use rust_xlsxwriter::Workbook;

    let array = to_array(data);
    if array.is_empty() {
        return Err("No data to export".into());
    }

    let headers = get_headers_from_array(&array);

    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();

    for (col, header) in headers.iter().enumerate() {
        sheet.write_string(0, col as u16, header)?;
    }

    for (row_idx, item) in array.iter().enumerate() {
        let map = dynamic_to_map(item);
        for (col, header) in headers.iter().enumerate() {
            let value = map.get(header).map(|v| v.to_string()).unwrap_or_default();
            sheet.write_string((row_idx + 1) as u32, col as u16, &value)?;
        }
    }

    workbook.save(file_path)?;

    trace!("Exported data to Excel: {}", file_path);
    Ok(file_path.to_string())
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
            }
            ',' if !in_quotes => {
                result.push(current.trim().to_string());
                current = String::new();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    result.push(current.trim().to_string());
    result
}

fn escape_csv_value(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn to_array(data: Dynamic) -> Array {
    if data.is_array() {
        data.cast::<Array>()
    } else {
        vec![data]
    }
}

fn dynamic_to_map(data: &Dynamic) -> HashMap<String, Dynamic> {
    let mut result = HashMap::new();

    if data.is_map() {
        let map = data.clone().cast::<Map>();
        for (k, v) in map.iter() {
            result.insert(k.to_string(), v.clone());
        }
    }

    result
}

fn get_headers_from_array(array: &Array) -> Vec<String> {
    let mut headers = Vec::new();

    if let Some(first) = array.first() {
        let map = dynamic_to_map(first);
        headers = map.keys().cloned().collect();
        headers.sort();
    }

    headers
}

fn json_to_dynamic(value: &Value) -> Dynamic {
    match value {
        Value::Null => Dynamic::UNIT,
        Value::Bool(b) => Dynamic::from(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from(f)
            } else {
                Dynamic::from(n.to_string())
            }
        }
        Value::String(s) => Dynamic::from(s.clone()),
        Value::Array(arr) => {
            let rhai_arr: Array = arr.iter().map(json_to_dynamic).collect();
            Dynamic::from(rhai_arr)
        }
        Value::Object(obj) => {
            let mut map = Map::new();
            for (k, v) in obj {
                map.insert(k.clone().into(), json_to_dynamic(v));
            }
            Dynamic::from(map)
        }
    }
}

fn dynamic_to_json(data: &Dynamic) -> Value {
    if data.is_unit() {
        Value::Null
    } else if data.is_bool() {
        Value::Bool(data.as_bool().unwrap_or(false))
    } else if data.is_int() {
        Value::Number(serde_json::Number::from(data.as_int().unwrap_or(0)))
    } else if data.is_float() {
        if let Some(n) = serde_json::Number::from_f64(data.as_float().unwrap_or(0.0)) {
            Value::Number(n)
        } else {
            Value::Null
        }
    } else if data.is_string() {
        Value::String(data.to_string())
    } else if data.is_array() {
        let arr = data.clone().cast::<Array>();
        Value::Array(arr.iter().map(dynamic_to_json).collect())
    } else if data.is_map() {
        let map = data.clone().cast::<Map>();
        let mut obj = serde_json::Map::new();
        for (k, v) in map.iter() {
            obj.insert(k.to_string(), dynamic_to_json(v));
        }
        Value::Object(obj)
    } else {
        Value::String(data.to_string())
    }
}
