use diesel::prelude::*;
use diesel::PgConnection;
use log::warn;

pub fn sanitize_identifier(name: &str) -> String {
    let sanitized: String = name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if sanitized != name {
        warn!("Identifier '{}' sanitized to '{}'", name, sanitized);
    }
    sanitized
}

pub fn validate_table_name(name: &str) -> Result<String, String> {
    let sanitized = sanitize_identifier(name);
    if sanitized.is_empty() {
        return Err("Table name cannot be empty".to_string());
    }
    if sanitized.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        return Err("Table name cannot start with a digit".to_string());
    }
    Ok(sanitized)
}

pub fn sanitize_sql_value(value: &str) -> String {
    value.replace('\'', "''").replace('\\', "\\\\")
}

pub fn sanitize_path_component(component: &str) -> String {
    component.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .collect()
}

pub fn build_safe_count_query(table: &str) -> String {
    let safe_table = sanitize_identifier(table);
    format!("SELECT COUNT(*) FROM {}", safe_table)
}

pub fn build_safe_select_query(table: &str, columns: &str, where_clause: &str) -> String {
    let safe_table = sanitize_identifier(table);
    if where_clause.is_empty() {
        format!("SELECT {} FROM {}", columns, safe_table)
    } else {
        format!("SELECT {} FROM {} WHERE {}", columns, safe_table, where_clause)
    }
}

pub fn build_safe_select_by_id_query(table: &str, columns: &str) -> String {
    let safe_table = sanitize_identifier(table);
    format!("SELECT {} FROM {} WHERE id = $1", columns, safe_table)
}

pub fn is_table_allowed_with_conn(conn: &mut PgConnection, table: &str) -> bool {
use diesel::sql_query;
use diesel::sql_types::BigInt;
#[derive(diesel::QueryableByName)]
struct CountRow { #[diesel(sql_type = BigInt)] count: i64 }
let safe_table = sanitize_identifier(table);
sql_query(format!("SELECT COUNT(*) as count FROM information_schema.tables WHERE table_name = '{}' LIMIT 1", safe_table))
.load::<CountRow>(conn)
.map(|rows| rows.first().map(|r| r.count > 0).unwrap_or(false))
.unwrap_or(false)
}

pub fn log_and_sanitize(error: &dyn std::error::Error, context: &str, _user_id: Option<uuid::Uuid>) -> String {
    let msg = error.to_string();
    let sanitized = if msg.contains("password") || msg.contains("token") || msg.contains("secret") {
        "[REDACTED]".to_string()
    } else {
        msg.chars().take(200).collect()
    };
    log::error!("{}: {}", context, sanitized);
    sanitized
}

pub fn get_stack_path() -> String {
    std::env::var("GBO_STACK_PATH").unwrap_or_else(|_| "/opt/gbo".to_string())
}
