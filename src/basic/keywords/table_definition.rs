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

//! TABLE keyword implementation for dynamic table definitions
//!
//! Parses and creates database tables from BASIC syntax:
//!
//! ```basic
//! TABLE Contacts ON maria
//!     Id number key
//!     Nome string(150)
//!     Email string(255)
//!     Telefone string(20)
//! END TABLE
//! ```
//!
//! Connection names (e.g., "maria") are configured in config.csv with:
//! - conn-maria-Server
//! - conn-maria-Name (database name)
//! - conn-maria-Username
//! - conn-maria-Port
//! - conn-maria-Password
//! - conn-maria-Driver

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use log::{error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

/// Represents a field definition in a TABLE block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,
    pub field_type: String,
    pub length: Option<i32>,
    pub precision: Option<i32>,
    pub is_key: bool,
    pub is_nullable: bool,
    pub default_value: Option<String>,
    pub reference_table: Option<String>,
    pub field_order: i32,
}

/// Represents a complete TABLE definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDefinition {
    pub name: String,
    pub connection_name: String,
    pub fields: Vec<FieldDefinition>,
}

/// External database connection configuration
#[derive(Debug, Clone)]
pub struct ExternalConnection {
    pub name: String,
    pub driver: String,
    pub server: String,
    pub port: Option<i32>,
    pub database: String,
    pub username: String,
    pub password: String,
}

/// Parse a TABLE...END TABLE block from BASIC source
pub fn parse_table_definition(
    source: &str,
) -> Result<Vec<TableDefinition>, Box<dyn Error + Send + Sync>> {
    let mut tables = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Look for TABLE keyword
        if line.starts_with("TABLE ") {
            let table_def = parse_single_table(&lines, &mut i)?;
            tables.push(table_def);
        } else {
            i += 1;
        }
    }

    Ok(tables)
}

/// Parse a single TABLE block
fn parse_single_table(
    lines: &[&str],
    index: &mut usize,
) -> Result<TableDefinition, Box<dyn Error + Send + Sync>> {
    let header_line = lines[*index].trim();

    // Parse: TABLE TableName ON connection
    let parts: Vec<&str> = header_line.split_whitespace().collect();

    if parts.len() < 2 {
        return Err(format!(
            "Invalid TABLE syntax at line {}: {}",
            index + 1,
            header_line
        )
        .into());
    }

    let table_name = parts[1].to_string();

    // Check for ON clause
    let connection_name = if parts.len() >= 4 && parts[2].eq_ignore_ascii_case("ON") {
        parts[3].to_string()
    } else {
        "default".to_string()
    };

    trace!("Parsing TABLE {} ON {}", table_name, connection_name);

    *index += 1;
    let mut fields = Vec::new();
    let mut field_order = 0;

    // Parse fields until END TABLE
    while *index < lines.len() {
        let line = lines[*index].trim();

        // Skip empty lines and comments
        if line.is_empty()
            || line.starts_with("'")
            || line.starts_with("REM")
            || line.starts_with("//")
        {
            *index += 1;
            continue;
        }

        // Check for END TABLE
        if line.eq_ignore_ascii_case("END TABLE") {
            *index += 1;
            break;
        }

        // Parse field definition
        if let Ok(field) = parse_field_definition(line, field_order) {
            fields.push(field);
            field_order += 1;
        } else {
            warn!("Could not parse field definition: {}", line);
        }

        *index += 1;
    }

    Ok(TableDefinition {
        name: table_name,
        connection_name,
        fields,
    })
}

/// Parse a single field definition line
/// Format: FieldName type[(length[,precision])] [key] [references TableName]
fn parse_field_definition(
    line: &str,
    order: i32,
) -> Result<FieldDefinition, Box<dyn Error + Send + Sync>> {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.is_empty() {
        return Err("Empty field definition".into());
    }

    let field_name = parts[0].to_string();
    let mut field_type = String::new();
    let mut length: Option<i32> = None;
    let mut precision: Option<i32> = None;
    let mut is_key = false;
    let mut reference_table: Option<String> = None;

    if parts.len() >= 2 {
        let type_part = parts[1];

        // Parse type with optional length: string(150) or number(10,2)
        if let Some(paren_start) = type_part.find('(') {
            field_type = type_part[..paren_start].to_lowercase();
            let params = &type_part[paren_start + 1..type_part.len() - 1];
            let param_parts: Vec<&str> = params.split(',').collect();

            if !param_parts.is_empty() {
                length = param_parts[0].trim().parse().ok();
            }
            if param_parts.len() > 1 {
                precision = param_parts[1].trim().parse().ok();
            }
        } else {
            field_type = type_part.to_lowercase();
        }
    }

    // Check for additional modifiers
    for i in 2..parts.len() {
        let part = parts[i].to_lowercase();
        match part.as_str() {
            "key" => is_key = true,
            _ if parts
                .get(i - 1)
                .map(|p| p.eq_ignore_ascii_case("references"))
                .unwrap_or(false) =>
            {
                // This is the reference table name, already handled
            }
            "references" => {
                if i + 1 < parts.len() {
                    reference_table = Some(parts[i + 1].to_string());
                }
            }
            _ => {}
        }
    }

    Ok(FieldDefinition {
        name: field_name,
        field_type,
        length,
        precision,
        is_key,
        is_nullable: !is_key, // Keys are not nullable by default
        default_value: None,
        reference_table,
        field_order: order,
    })
}

/// Map BASIC types to SQL types
fn map_type_to_sql(field: &FieldDefinition, driver: &str) -> String {
    let base_type = match field.field_type.as_str() {
        "string" => {
            let len = field.length.unwrap_or(255);
            format!("VARCHAR({})", len)
        }
        "number" | "integer" | "int" => {
            if field.precision.is_some() {
                let len = field.length.unwrap_or(10);
                let prec = field.precision.unwrap_or(2);
                format!("DECIMAL({},{})", len, prec)
            } else if field.length.is_some() {
                "BIGINT".to_string()
            } else {
                "INTEGER".to_string()
            }
        }
        "double" | "float" => {
            if let (Some(len), Some(prec)) = (field.length, field.precision) {
                format!("DECIMAL({},{})", len, prec)
            } else {
                "DOUBLE PRECISION".to_string()
            }
        }
        "date" => "DATE".to_string(),
        "datetime" | "timestamp" => match driver {
            "mysql" | "mariadb" => "DATETIME".to_string(),
            _ => "TIMESTAMP".to_string(),
        },
        "boolean" | "bool" => match driver {
            "mysql" | "mariadb" => "TINYINT(1)".to_string(),
            _ => "BOOLEAN".to_string(),
        },
        "text" => "TEXT".to_string(),
        "guid" | "uuid" => match driver {
            "mysql" | "mariadb" => "CHAR(36)".to_string(),
            _ => "UUID".to_string(),
        },
        _ => format!("VARCHAR({})", field.length.unwrap_or(255)),
    };

    base_type
}

/// Generate CREATE TABLE SQL statement
pub fn generate_create_table_sql(table: &TableDefinition, driver: &str) -> String {
    let mut sql = format!(
        "CREATE TABLE IF NOT EXISTS {} (\n",
        sanitize_identifier(&table.name)
    );

    let mut column_defs = Vec::new();
    let mut primary_keys = Vec::new();

    for field in &table.fields {
        let sql_type = map_type_to_sql(field, driver);
        let mut col_def = format!("    {} {}", sanitize_identifier(&field.name), sql_type);

        if field.is_key {
            primary_keys.push(sanitize_identifier(&field.name));
        }

        if !field.is_nullable {
            col_def.push_str(" NOT NULL");
        }

        if let Some(ref default) = field.default_value {
            col_def.push_str(&format!(" DEFAULT {}", default));
        }

        column_defs.push(col_def);
    }

    sql.push_str(&column_defs.join(",\n"));

    // Add primary key constraint
    if !primary_keys.is_empty() {
        sql.push_str(&format!(",\n    PRIMARY KEY ({})", primary_keys.join(", ")));
    }

    sql.push_str("\n)");

    // Add engine for MySQL/MariaDB
    if driver == "mysql" || driver == "mariadb" {
        sql.push_str(" ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci");
    }

    sql.push(';');
    sql
}

/// Sanitize identifier to prevent SQL injection
fn sanitize_identifier(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

/// Load external connection configuration from bot config
pub fn load_connection_config(
    state: &AppState,
    bot_id: Uuid,
    connection_name: &str,
) -> Result<ExternalConnection, Box<dyn Error + Send + Sync>> {
    let config_manager = crate::core::config::ConfigManager::new(state.conn.clone());

    let prefix = format!("conn-{}-", connection_name);

    let server = config_manager
        .get_config(&bot_id, &format!("{}Server", prefix), None)
        .ok_or_else(|| format!("Missing {prefix}Server in config"))?;

    let database = config_manager
        .get_config(&bot_id, &format!("{}Name", prefix), None)
        .ok_or_else(|| format!("Missing {prefix}Name in config"))?;

    let username = config_manager
        .get_config(&bot_id, &format!("{}Username", prefix), None)
        .unwrap_or_default();

    let password = config_manager
        .get_config(&bot_id, &format!("{}Password", prefix), None)
        .unwrap_or_default();

    let port = config_manager
        .get_config(&bot_id, &format!("{}Port", prefix), None)
        .and_then(|p| p.parse().ok());

    let driver = config_manager
        .get_config(&bot_id, &format!("{}Driver", prefix), None)
        .unwrap_or_else(|| "postgres".to_string());

    Ok(ExternalConnection {
        name: connection_name.to_string(),
        driver,
        server,
        port,
        database,
        username,
        password,
    })
}

/// Build connection string for external database
pub fn build_connection_string(conn: &ExternalConnection) -> String {
    let port = conn.port.unwrap_or(match conn.driver.as_str() {
        "mysql" | "mariadb" => 3306,
        "postgres" | "postgresql" => 5432,
        "mssql" | "sqlserver" => 1433,
        _ => 5432,
    });

    match conn.driver.as_str() {
        "mysql" | "mariadb" => {
            format!(
                "mysql://{}:{}@{}:{}/{}",
                conn.username, conn.password, conn.server, port, conn.database
            )
        }
        "postgres" | "postgresql" => {
            format!(
                "postgres://{}:{}@{}:{}/{}",
                conn.username, conn.password, conn.server, port, conn.database
            )
        }
        "mssql" | "sqlserver" => {
            format!(
                "mssql://{}:{}@{}:{}/{}",
                conn.username, conn.password, conn.server, port, conn.database
            )
        }
        _ => {
            format!(
                "postgres://{}:{}@{}:{}/{}",
                conn.username, conn.password, conn.server, port, conn.database
            )
        }
    }
}

/// Store table definition in metadata tables
pub fn store_table_definition(
    conn: &mut diesel::PgConnection,
    bot_id: Uuid,
    table: &TableDefinition,
) -> Result<Uuid, Box<dyn Error + Send + Sync>> {
    // Insert table definition
    let table_id: Uuid = diesel::sql_query(
        "INSERT INTO dynamic_table_definitions (bot_id, table_name, connection_name)
         VALUES ($1, $2, $3)
         ON CONFLICT (bot_id, table_name, connection_name)
         DO UPDATE SET updated_at = NOW()
         RETURNING id",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<Text, _>(&table.name)
    .bind::<Text, _>(&table.connection_name)
    .get_result::<IdResult>(conn)?
    .id;

    // Delete existing fields for this table
    diesel::sql_query("DELETE FROM dynamic_table_fields WHERE table_definition_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(table_id)
        .execute(conn)?;

    // Insert field definitions
    for field in &table.fields {
        diesel::sql_query(
            "INSERT INTO dynamic_table_fields
             (table_definition_id, field_name, field_type, field_length, field_precision,
              is_key, is_nullable, default_value, reference_table, field_order)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind::<diesel::sql_types::Uuid, _>(table_id)
        .bind::<Text, _>(&field.name)
        .bind::<Text, _>(&field.field_type)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Integer>, _>(field.length)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Integer>, _>(field.precision)
        .bind::<diesel::sql_types::Bool, _>(field.is_key)
        .bind::<diesel::sql_types::Bool, _>(field.is_nullable)
        .bind::<diesel::sql_types::Nullable<Text>, _>(&field.default_value)
        .bind::<diesel::sql_types::Nullable<Text>, _>(&field.reference_table)
        .bind::<diesel::sql_types::Integer, _>(field.field_order)
        .execute(conn)?;
    }

    Ok(table_id)
}

#[derive(QueryableByName)]
struct IdResult {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
}

/// Execute CREATE TABLE on external connection
pub async fn create_table_on_external_db(
    connection_string: &str,
    create_sql: &str,
    driver: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match driver {
        "mysql" | "mariadb" => create_table_mysql(connection_string, create_sql).await,
        "postgres" | "postgresql" => create_table_postgres(connection_string, create_sql).await,
        _ => {
            warn!("Unsupported driver: {}, attempting postgres", driver);
            create_table_postgres(connection_string, create_sql).await
        }
    }
}

#[cfg(feature = "dynamic-db")]
async fn create_table_mysql(
    connection_string: &str,
    sql: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    use sqlx::mysql::MySqlPoolOptions;
    use sqlx::Executor;

    let pool = MySqlPoolOptions::new()
        .max_connections(1)
        .connect(connection_string)
        .await?;

    pool.execute(sql).await?;
    info!("MySQL table created successfully");
    Ok(())
}

#[cfg(not(feature = "dynamic-db"))]
async fn create_table_mysql(
    _connection_string: &str,
    _sql: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    Err("MySQL support requires the 'dynamic-db' feature".into())
}

#[cfg(feature = "dynamic-db")]
async fn create_table_postgres(
    connection_string: &str,
    sql: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    use sqlx::postgres::PgPoolOptions;
    use sqlx::Executor;

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(connection_string)
        .await?;

    pool.execute(sql).await?;
    info!("PostgreSQL table created successfully");
    Ok(())
}

#[cfg(not(feature = "dynamic-db"))]
async fn create_table_postgres(
    _connection_string: &str,
    _sql: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    Err("PostgreSQL dynamic table support requires the 'dynamic-db' feature".into())
}

/// Process TABLE definitions during .bas file compilation
pub fn process_table_definitions(
    state: Arc<AppState>,
    bot_id: Uuid,
    source: &str,
) -> Result<Vec<TableDefinition>, Box<dyn Error + Send + Sync>> {
    let tables = parse_table_definition(source)?;

    if tables.is_empty() {
        return Ok(tables);
    }

    let mut conn = state.conn.get()?;

    for table in &tables {
        info!(
            "Processing TABLE {} ON {}",
            table.name, table.connection_name
        );

        // Store table definition in metadata
        store_table_definition(&mut conn, bot_id, table)?;

        // Load connection config and create table on external DB
        if table.connection_name != "default" {
            match load_connection_config(&state, bot_id, &table.connection_name) {
                Ok(ext_conn) => {
                    let create_sql = generate_create_table_sql(table, &ext_conn.driver);
                    let conn_string = build_connection_string(&ext_conn);

                    info!(
                        "Creating table {} on {} ({})",
                        table.name, table.connection_name, ext_conn.driver
                    );
                    trace!("SQL: {}", create_sql);

                    // Execute async in blocking context
                    let driver = ext_conn.driver.clone();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            if let Err(e) =
                                create_table_on_external_db(&conn_string, &create_sql, &driver)
                                    .await
                            {
                                error!("Failed to create table on external DB: {}", e);
                            }
                        })
                    });
                }
                Err(e) => {
                    error!(
                        "Failed to load connection config for {}: {}",
                        table.connection_name, e
                    );
                }
            }
        } else {
            // Create on default (internal) database
            let create_sql = generate_create_table_sql(table, "postgres");
            info!("Creating table {} on default connection", table.name);
            trace!("SQL: {}", create_sql);

            sql_query(&create_sql).execute(&mut conn)?;
        }
    }

    Ok(tables)
}

/// Register TABLE keyword (no-op at runtime, processed at compile time)
pub fn register_table_keywords(
    _state: Arc<AppState>,
    _user: UserSession,
    _engine: &mut rhai::Engine,
) {
    // TABLE...END TABLE is processed at compile time, not runtime
    // This function exists for consistency with other keyword modules
    trace!("TABLE keyword registered (compile-time only)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_table_definition() {
        let source = r#"
TABLE Contacts ON maria
    Id number key
    Nome string(150)
    Email string(255)
    Telefone string(20)
END TABLE
"#;

        let tables = parse_table_definition(source).unwrap();
        assert_eq!(tables.len(), 1);

        let table = &tables[0];
        assert_eq!(table.name, "Contacts");
        assert_eq!(table.connection_name, "maria");
        assert_eq!(table.fields.len(), 4);

        assert_eq!(table.fields[0].name, "Id");
        assert_eq!(table.fields[0].field_type, "number");
        assert!(table.fields[0].is_key);

        assert_eq!(table.fields[1].name, "Nome");
        assert_eq!(table.fields[1].field_type, "string");
        assert_eq!(table.fields[1].length, Some(150));
    }

    #[test]
    fn test_parse_field_with_precision() {
        let field = parse_field_definition("Preco double(10,2)", 0).unwrap();
        assert_eq!(field.name, "Preco");
        assert_eq!(field.field_type, "double");
        assert_eq!(field.length, Some(10));
        assert_eq!(field.precision, Some(2));
    }

    #[test]
    fn test_generate_create_table_sql() {
        let table = TableDefinition {
            name: "TestTable".to_string(),
            connection_name: "default".to_string(),
            fields: vec![
                FieldDefinition {
                    name: "id".to_string(),
                    field_type: "number".to_string(),
                    length: None,
                    precision: None,
                    is_key: true,
                    is_nullable: false,
                    default_value: None,
                    reference_table: None,
                    field_order: 0,
                },
                FieldDefinition {
                    name: "name".to_string(),
                    field_type: "string".to_string(),
                    length: Some(100),
                    precision: None,
                    is_key: false,
                    is_nullable: true,
                    default_value: None,
                    reference_table: None,
                    field_order: 1,
                },
            ],
        };

        let sql = generate_create_table_sql(&table, "postgres");
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS TestTable"));
        assert!(sql.contains("id INTEGER NOT NULL"));
        assert!(sql.contains("name VARCHAR(100)"));
        assert!(sql.contains("PRIMARY KEY (id)"));
    }

    #[test]
    fn test_map_types() {
        let field = FieldDefinition {
            name: "test".to_string(),
            field_type: "string".to_string(),
            length: Some(50),
            precision: None,
            is_key: false,
            is_nullable: true,
            default_value: None,
            reference_table: None,
            field_order: 0,
        };
        assert_eq!(map_type_to_sql(&field, "postgres"), "VARCHAR(50)");

        let date_field = FieldDefinition {
            name: "created".to_string(),
            field_type: "datetime".to_string(),
            length: None,
            precision: None,
            is_key: false,
            is_nullable: true,
            default_value: None,
            reference_table: None,
            field_order: 0,
        };
        assert_eq!(map_type_to_sql(&date_field, "mysql"), "DATETIME");
        assert_eq!(map_type_to_sql(&date_field, "postgres"), "TIMESTAMP");
    }

    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(sanitize_identifier("valid_name"), "valid_name");
        assert_eq!(sanitize_identifier("DROP TABLE; --"), "DROPTABLE");
        assert_eq!(sanitize_identifier("name123"), "name123");
    }

    #[test]
    fn test_build_connection_string() {
        let conn = ExternalConnection {
            name: "test".to_string(),
            driver: "mysql".to_string(),
            server: "localhost".to_string(),
            port: Some(3306),
            database: "testdb".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
        };

        let conn_str = build_connection_string(&conn);
        assert_eq!(conn_str, "mysql://user:pass@localhost:3306/testdb");
    }
}
