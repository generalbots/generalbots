use rhai::Dynamic;
use serde_json::Value;
use std::path::PathBuf;

pub fn to_array(dynamic: &Dynamic) -> Vec<Dynamic> {
    if let Some(array) = dynamic.clone().try_cast::<Vec<Dynamic>>() {
        array
    } else if let Some(map) = dynamic.clone().try_cast::<rhai::Map>() {
        map.into_values().collect()
    } else {
        vec![dynamic.clone()]
    }
}

pub fn json_value_to_dynamic(value: &Value) -> Dynamic {
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
            let vec: Vec<Dynamic> = arr.iter().map(json_value_to_dynamic).collect();
            Dynamic::from(vec)
        }
        Value::Object(map) => {
            let mut rhai_map = rhai::Map::new();
            for (k, v) in map {
                rhai_map.insert(k.clone().into(), json_value_to_dynamic(v));
            }
            Dynamic::from(rhai_map)
        }
    }
}

pub fn dynamic_to_json(value: &Dynamic) -> Value {
    if value.is_unit() {
        Value::Null
    } else if let Ok(b) = value.as_bool() {
        Value::Bool(b)
    } else if let Ok(i) = value.as_int() {
        Value::from(i)
    } else if let Ok(f) = value.as_float() {
        Value::from(f)
    } else if let Some(s) = value.clone().try_cast::<String>() {
        Value::String(s)
    } else if let Some(arr) = value.clone().try_cast::<Vec<Dynamic>>() {
        Value::Array(arr.iter().map(dynamic_to_json).collect())
    } else if let Some(map) = value.clone().try_cast::<rhai::Map>() {
        let mut json_map = serde_json::Map::new();
        for (k, v) in map {
            json_map.insert(k.to_string(), dynamic_to_json(&v));
        }
        Value::Object(json_map)
    } else {
        Value::String(value.to_string())
    }
}

pub fn convert_date_to_iso_format(date_str: &str) -> String {
    let trimmed = date_str.trim();
    // Already ISO format (yyyy-MM-dd or yyyy-MM-dd HH:mm:ss)
    if trimmed.len() >= 10 && trimmed.as_bytes()[4] == b'-' && trimmed.as_bytes()[7] == b'-' {
        return trimmed.to_string();
    }
    // Brazilian format: dd/mm/aaaa or dd/mm/aaaa HH:mm:ss
    if trimmed.contains('/') {
        let parts: Vec<&str> = trimmed.splitn(3, '/').collect();
        if parts.len() == 3 {
            let day = parts[0];
            let month = parts[1];
            let rest = parts[2];
            let year = if rest.contains(' ') {
                rest.split(' ').next().unwrap_or(rest)
            } else {
                rest
            };
            // Validate numbers
            if day.chars().all(|c| c.is_ascii_digit()) && month.chars().all(|c| c.is_ascii_digit()) {
                let d: i32 = day.parse().unwrap_or(0);
                let m: i32 = month.parse().unwrap_or(0);
                if (1..=31).contains(&d) && (1..=12).contains(&m) {
                    let time_part = if rest.contains(' ') {
                        " ".to_string() + rest.split(' ').nth(1).unwrap_or("")
                    } else {
                        String::new()
                    };
                    return format!("{:04}-{:02}-{:02}{}", year, m, d, time_part);
                }
            }
        }
    }
    date_str.to_string()
}

pub fn get_work_path() -> String {
    std::env::var("GBO_WORK_PATH").unwrap_or_else(|_| "/opt/gbo/work".to_string())
}

/// Returns the current org ID for bot file path isolation.
/// Always returns Uuid::nil() until real multi-tenant auth is implemented.
pub fn current_org_id() -> uuid::Uuid {
    uuid::Uuid::nil()
}

/// Build a relative bot path with org isolation (.gborg wrapping).
/// Returns: "{org_id}.gborg/{bot_bucket}.gbai/{sub_path}"
pub fn build_bot_path(org_id: impl std::fmt::Display, bot_bucket: &str, sub_path: &str) -> String {
    format!("{org_id}.gborg/{bot_bucket}.gbai/{sub_path}")
}

/// Build an absolute bot path with org isolation.
/// Returns: "{work_root}/{org_id}.gborg/{bot_bucket}.gbai/{sub_path}"
pub fn build_absolute_bot_path(
    work_root: &str,
    org_id: impl std::fmt::Display,
    bot_bucket: &str,
    sub_path: &str,
) -> String {
    format!("{work_root}/{org_id}.gborg/{bot_bucket}.gbai/{sub_path}")
}

/// Get work path with org isolation suffix.
/// Returns: "{work_path}/{org_id}.gborg/"
pub fn get_org_work_path(org_id: impl std::fmt::Display) -> String {
    format!("{}/{org_id}.gborg/", get_work_path())
}

pub fn get_content_type(path: &str) -> String {
    let p = PathBuf::from(path);
    match p.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "zip" => "application/zip",
        _ => "application/octet-stream",
    }.to_string()
}

pub fn get_default_bot(state: &std::sync::Arc<dyn botbasic_types::BasicRuntime>) -> Result<serde_json::Value, String> {
    let mut conn = state.db_pool().get().map_err(|e| e.to_string())?;
    use diesel::prelude::*;
    use diesel::sql_query;
    use diesel::sql_types::Text;
    #[derive(diesel::QueryableByName)]
    struct BotRow {
        #[diesel(sql_type = Text)]
        bot_id: String,
    }
    let rows = sql_query("SELECT bot_id FROM bots LIMIT 1")
        .load::<BotRow>(&mut conn)
        .map_err(|e| e.to_string())?;
    rows.into_iter()
        .next()
        .map(|r| serde_json::json!({"bot_id": r.bot_id}))
        .ok_or_else(|| "No bots found".to_string())
}

pub fn parse_filter(filter_str: &str) -> Result<(String, Vec<String>), Box<dyn std::error::Error + Send + Sync>> {
    let trimmed = filter_str.trim();
    if trimmed == "1=1" {
        return Ok(("TRUE".to_string(), vec![]));
    }
    let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err("Invalid filter format. Expected 'KEY=VALUE'".into());
    }
    let column = parts[0].trim();
    let value = parts[1].trim();
    if !column.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("Invalid column name in filter".into());
    }
    Ok((format!("{} = $1", column), vec![value.to_string()]))
}
