use diesel::prelude::*;
use diesel::PgConnection;
use log::warn;

pub fn sanitize_identifier(name: &str) -> String {
    let sanitized: String = name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .to_lowercase();
    if sanitized != name.to_lowercase() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_identifier_strips_injection_characters() {
        assert_eq!(sanitize_identifier("users; DROP TABLE x--"), "usersdroptablex");
        assert_eq!(sanitize_identifier("a'b\"c"), "abc");
        assert_eq!(sanitize_identifier("tbl_1"), "tbl_1");
        assert_eq!(sanitize_identifier("UPPER"), "upper");
        assert_eq!(sanitize_identifier("  spaced  "), "spaced");
        assert_eq!(sanitize_identifier(""), "");
    }

    #[test]
    fn sanitize_identifier_keeps_unicode_alphanumerics() {
        assert_eq!(sanitize_identifier("café_1"), "café_1");
        assert_eq!(sanitize_identifier("ação"), "ação");
    }

    #[test]
    fn validate_table_name_rejects_empty_and_leading_digits() {
        assert_eq!(validate_table_name("customers").unwrap(), "customers");
        assert_eq!(validate_table_name("Customers").unwrap(), "customers");
        assert!(validate_table_name("").is_err());
        assert!(validate_table_name("!!!").is_err());
        assert!(validate_table_name("1users").is_err());
        assert_eq!(validate_table_name("_users").unwrap(), "_users");
    }

    #[test]
    fn validate_table_name_error_messages_are_stable() {
        assert_eq!(validate_table_name("").unwrap_err(), "Table name cannot be empty");
        assert_eq!(
            validate_table_name("9lives").unwrap_err(),
            "Table name cannot start with a digit"
        );
    }

    #[test]
    fn sanitize_sql_value_doubles_quotes_and_backslashes() {
        assert_eq!(sanitize_sql_value("O'Brien"), "O''Brien");
        assert_eq!(sanitize_sql_value("a\\b"), "a\\\\b");
        assert_eq!(sanitize_sql_value("' OR 1=1 --"), "'' OR 1=1 --");
        assert_eq!(sanitize_sql_value("plain"), "plain");
        assert_eq!(sanitize_sql_value(""), "");
    }

    #[test]
    fn sanitize_path_component_blocks_traversal() {
        assert_eq!(sanitize_path_component("../../etc/passwd"), "....etcpasswd");
        assert_eq!(sanitize_path_component("file-1_2.txt"), "file-1_2.txt");
        assert_eq!(sanitize_path_component("a b/c"), "abc");
        assert_eq!(sanitize_path_component(""), "");
    }

    #[test]
    fn build_safe_count_query_uses_sanitized_table() {
        assert_eq!(
            build_safe_count_query("users; DROP TABLE users"),
            "SELECT COUNT(*) FROM usersdroptableusers"
        );
        assert_eq!(build_safe_count_query("Users"), "SELECT COUNT(*) FROM users");
    }

    #[test]
    fn build_safe_select_query_omits_empty_where() {
        assert_eq!(
            build_safe_select_query("users", "*", ""),
            "SELECT * FROM users"
        );
        assert_eq!(
            build_safe_select_query("Users", "id, name", "id = 1"),
            "SELECT id, name FROM users WHERE id = 1"
        );
        assert_eq!(
            build_safe_select_query("users;--", "*", ""),
            "SELECT * FROM users"
        );
    }

    #[test]
    fn build_safe_select_by_id_query_always_parameterizes_id() {
        assert_eq!(
            build_safe_select_by_id_query("Orders", "id"),
            "SELECT id FROM orders WHERE id = $1"
        );
        assert_eq!(
            build_safe_select_by_id_query("o'rders", "*"),
            "SELECT * FROM orders WHERE id = $1"
        );
    }

    #[test]
    fn log_and_sanitize_redacts_secrets_and_truncates() {
        #[derive(Debug)]
        struct E(&'static str);
        impl std::fmt::Display for E {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        impl std::error::Error for E {}

        assert_eq!(log_and_sanitize(&E("bad password here"), "ctx", None), "[REDACTED]");
        assert_eq!(log_and_sanitize(&E("token expired"), "ctx", None), "[REDACTED]");
        assert_eq!(log_and_sanitize(&E("secret leaked"), "ctx", None), "[REDACTED]");

        let long = "x".repeat(500);
        assert_eq!(log_and_sanitize(&E(Box::leak(long.into_boxed_str())), "ctx", None).len(), 200);
        assert_eq!(log_and_sanitize(&E("plain failure"), "ctx", None), "plain failure");
    }

    #[test]
    fn get_stack_path_never_returns_empty() {
        assert!(!get_stack_path().is_empty());
    }
}
