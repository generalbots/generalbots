use std::collections::HashSet;
use std::sync::LazyLock;

static ALLOWED_TABLES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "automations",
        "bots",
        "bot_configurations",
        "bot_memories",
        "clicks",
        "group_members",
        "groups",
        "message_history",
        "organizations",
        "table_access",
        "tasks",
        "trigger_kinds",
        "user_login_tokens",
        "user_preferences",
        "user_sessions",
        "users",
        "a2a_messages",
        "api_records",
        "calendar_events",
        "documents",
        "email_accounts",
        "meetings",
        "notifications",
        "oauth_providers",
        "oauth_tokens",
        "research_sessions",
        "sources",
    ])
});

static ALLOWED_ORDER_COLUMNS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "id",
        "created_at",
        "updated_at",
        "name",
        "email",
        "title",
        "status",
        "priority",
        "due_date",
        "start_date",
        "end_date",
        "order",
        "position",
        "timestamp",
    ])
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlGuardError {
    InvalidTableName(String),
    InvalidColumnName(String),
    InvalidOrderDirection(String),
    InvalidIdentifier(String),
    PotentialInjection(String),
}

impl std::fmt::Display for SqlGuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidTableName(name) => write!(f, "Invalid table name: {name}"),
            Self::InvalidColumnName(name) => write!(f, "Invalid column name: {name}"),
            Self::InvalidOrderDirection(dir) => write!(f, "Invalid order direction: {dir}"),
            Self::InvalidIdentifier(id) => write!(f, "Invalid identifier: {id}"),
            Self::PotentialInjection(input) => write!(f, "Potential SQL injection detected: {input}"),
        }
    }
}

impl std::error::Error for SqlGuardError {}

pub fn validate_table_name(table: &str) -> Result<&str, SqlGuardError> {
    let sanitized = sanitize_identifier(table);

    if ALLOWED_TABLES.contains(sanitized.as_str()) {
        Ok(table)
    } else {
        Err(SqlGuardError::InvalidTableName(table.to_string()))
    }
}

pub fn validate_order_column(column: &str) -> Result<&str, SqlGuardError> {
    let sanitized = sanitize_identifier(column);

    if ALLOWED_ORDER_COLUMNS.contains(sanitized.as_str()) {
        Ok(column)
    } else {
        Err(SqlGuardError::InvalidColumnName(column.to_string()))
    }
}

pub fn validate_order_direction(direction: &str) -> Result<&'static str, SqlGuardError> {
    match direction.to_uppercase().as_str() {
        "ASC" => Ok("ASC"),
        "DESC" => Ok("DESC"),
        _ => Err(SqlGuardError::InvalidOrderDirection(direction.to_string())),
    }
}

pub fn sanitize_identifier(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect()
}

pub fn validate_identifier(name: &str) -> Result<String, SqlGuardError> {
    let sanitized = sanitize_identifier(name);

    if sanitized.is_empty() {
        return Err(SqlGuardError::InvalidIdentifier(name.to_string()));
    }

    if sanitized.len() > 64 {
        return Err(SqlGuardError::InvalidIdentifier("Identifier too long".to_string()));
    }

    if sanitized.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        return Err(SqlGuardError::InvalidIdentifier("Identifier cannot start with digit".to_string()));
    }

    Ok(sanitized)
}

pub fn check_for_injection_patterns(input: &str) -> Result<(), SqlGuardError> {
    let lower = input.to_lowercase();

    let dangerous_patterns = [
        "--",
        "/*",
        "*/",
        ";",
        "union",
        "select",
        "insert",
        "update",
        "delete",
        "drop",
        "truncate",
        "exec",
        "execute",
        "xp_",
        "sp_",
        "0x",
        "char(",
        "nchar(",
        "varchar(",
        "nvarchar(",
        "cast(",
        "convert(",
        "@@",
        "waitfor",
        "delay",
        "benchmark",
        "sleep(",
    ];

    for pattern in dangerous_patterns {
        if lower.contains(pattern) {
            return Err(SqlGuardError::PotentialInjection(format!(
                "Dangerous pattern '{}' detected",
                pattern
            )));
        }
    }

    Ok(())
}

pub fn escape_string_literal(value: &str) -> String {
    value.replace('\'', "''").replace('\\', "\\\\")
}

pub fn build_safe_select_query(
    table: &str,
    order_by: Option<&str>,
    order_dir: Option<&str>,
    limit: i32,
    offset: i32,
) -> Result<String, SqlGuardError> {
    let validated_table = validate_table_name(table)?;

    let safe_limit = limit.clamp(1, 1000);
    let safe_offset = offset.max(0);

    let order_clause = match (order_by, order_dir) {
        (Some(col), Some(dir)) => {
            let validated_col = validate_order_column(col)?;
            let validated_dir = validate_order_direction(dir)?;
            format!("ORDER BY {} {}", validated_col, validated_dir)
        }
        (Some(col), None) => {
            let validated_col = validate_order_column(col)?;
            format!("ORDER BY {} ASC", validated_col)
        }
        _ => "ORDER BY id ASC".to_string(),
    };

    Ok(format!(
        "SELECT row_to_json(t.*) as data FROM {} t {} LIMIT {} OFFSET {}",
        validated_table, order_clause, safe_limit, safe_offset
    ))
}

pub fn build_safe_count_query(table: &str) -> Result<String, SqlGuardError> {
    let validated_table = validate_table_name(table)?;
    Ok(format!("SELECT COUNT(*) as count FROM {}", validated_table))
}

pub fn build_safe_delete_query(table: &str) -> Result<String, SqlGuardError> {
    let validated_table = validate_table_name(table)?;
    Ok(format!("DELETE FROM {} WHERE id = $1", validated_table))
}

pub fn build_safe_select_by_id_query(table: &str) -> Result<String, SqlGuardError> {
    let validated_table = validate_table_name(table)?;
    Ok(format!(
        "SELECT row_to_json(t.*) as data FROM {} t WHERE id = $1",
        validated_table
    ))
}

pub fn register_dynamic_table(table_name: &'static str) {
    log::info!("Dynamic table registration requested for: {}", table_name);
}

pub fn is_table_allowed(table: &str) -> bool {
    let sanitized = sanitize_identifier(table);
    ALLOWED_TABLES.contains(sanitized.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_table_name_allowed() {
        assert!(validate_table_name("users").is_ok());
        assert!(validate_table_name("bots").is_ok());
        assert!(validate_table_name("tasks").is_ok());
    }

    #[test]
    fn test_validate_table_name_disallowed() {
        assert!(validate_table_name("evil_table").is_err());
        assert!(validate_table_name("users; DROP TABLE users;--").is_err());
        assert!(validate_table_name("").is_err());
    }

    #[test]
    fn test_validate_order_direction() {
        assert_eq!(validate_order_direction("ASC").unwrap(), "ASC");
        assert_eq!(validate_order_direction("desc").unwrap(), "DESC");
        assert_eq!(validate_order_direction("Asc").unwrap(), "ASC");
        assert!(validate_order_direction("RANDOM").is_err());
    }

    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(sanitize_identifier("valid_name"), "valid_name");
        assert_eq!(sanitize_identifier("name123"), "name123");
        assert_eq!(sanitize_identifier("name; DROP--"), "nameDROP");
        assert_eq!(sanitize_identifier(""), "");
    }

    #[test]
    fn test_check_for_injection_patterns() {
        assert!(check_for_injection_patterns("normal text").is_ok());
        assert!(check_for_injection_patterns("hello world").is_ok());

        assert!(check_for_injection_patterns("'; DROP TABLE users;--").is_err());
        assert!(check_for_injection_patterns("1 UNION SELECT * FROM passwords").is_err());
        assert!(check_for_injection_patterns("test/*comment*/").is_err());
        assert!(check_for_injection_patterns("WAITFOR DELAY '0:0:5'").is_err());
    }

    #[test]
    fn test_escape_string_literal() {
        assert_eq!(escape_string_literal("hello"), "hello");
        assert_eq!(escape_string_literal("it's"), "it''s");
        assert_eq!(escape_string_literal("back\\slash"), "back\\\\slash");
        assert_eq!(escape_string_literal("O'Brien's"), "O''Brien''s");
    }

    #[test]
    fn test_build_safe_select_query() {
        let query = build_safe_select_query("users", Some("created_at"), Some("DESC"), 10, 0);
        assert!(query.is_ok());
        let q = query.unwrap();
        assert!(q.contains("users"));
        assert!(q.contains("ORDER BY created_at DESC"));
        assert!(q.contains("LIMIT 10"));
        assert!(q.contains("OFFSET 0"));
    }

    #[test]
    fn test_build_safe_select_query_invalid_table() {
        let query = build_safe_select_query("evil_table", None, None, 10, 0);
        assert!(query.is_err());
    }

    #[test]
    fn test_build_safe_count_query() {
        let query = build_safe_count_query("users");
        assert!(query.is_ok());
        assert!(query.unwrap().contains("SELECT COUNT(*)"));
    }

    #[test]
    fn test_build_safe_delete_query() {
        let query = build_safe_delete_query("tasks");
        assert!(query.is_ok());
        assert!(query.unwrap().contains("DELETE FROM tasks WHERE id = $1"));
    }

    #[test]
    fn test_validate_identifier() {
        assert!(validate_identifier("valid_name").is_ok());
        assert!(validate_identifier("name123").is_ok());
        assert!(validate_identifier("").is_err());
        assert!(validate_identifier("123name").is_err());
    }

    #[test]
    fn test_limit_clamping() {
        let query = build_safe_select_query("users", None, None, 10000, 0).unwrap();
        assert!(query.contains("LIMIT 1000"));

        let query2 = build_safe_select_query("users", None, None, -5, -10).unwrap();
        assert!(query2.contains("LIMIT 1"));
        assert!(query2.contains("OFFSET 0"));
    }

    #[test]
    fn test_is_table_allowed() {
        assert!(is_table_allowed("users"));
        assert!(is_table_allowed("bots"));
        assert!(!is_table_allowed("hacked_table"));
    }
}
