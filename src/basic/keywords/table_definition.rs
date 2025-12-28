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
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use log::{error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDefinition {
    pub name: String,
    pub connection_name: String,
    pub fields: Vec<FieldDefinition>,
}

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

pub fn parse_table_definition(
    source: &str,
) -> Result<Vec<TableDefinition>, Box<dyn Error + Send + Sync>> {
    let mut tables = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.starts_with("TABLE ") {
            let table_def = parse_single_table(&lines, &mut i)?;
            tables.push(table_def);
        } else {
            i += 1;
        }
    }

    Ok(tables)
}

fn parse_single_table(
    lines: &[&str],
    index: &mut usize,
) -> Result<TableDefinition, Box<dyn Error + Send + Sync>> {
    let header_line = lines[*index].trim();

    let parts: Vec<&str> = header_line.split_whitespace().collect();

    if parts.len() < 2 {
        return Err(format!(
            "Invalid TABLE syntax at line {}: {}",
            *index + 1,
            header_line
        )
        .into());
    }

    let table_name = parts[1].to_string();

    let connection_name = if parts.len() >= 4 && parts[2].eq_ignore_ascii_case("ON") {
        parts[3].to_string()
    } else {
        "default".to_string()
    };

    trace!("Parsing TABLE {} ON {}", table_name, connection_name);

    *index += 1;
    let mut fields = Vec::new();
    let mut field_order = 0;

    while *index < lines.len() {
        let line = lines[*index].trim();

        if line.is_empty()
            || line.starts_with('\'')
            || line.starts_with("REM")
            || line.starts_with("//")
        {
            *index += 1;
            continue;
        }

        if line.eq_ignore_ascii_case("END TABLE") {
            *index += 1;
            break;
        }

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

    for i in 2..parts.len() {
        let part = parts[i].to_lowercase();
        match part.as_str() {
            "key" => is_key = true,
            _ if parts
                .get(i - 1)
                .map(|p| p.eq_ignore_ascii_case("references"))
                .unwrap_or(false) => {}
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
        is_nullable: !is_key,
        default_value: None,
        reference_table,
        field_order: order,
    })
}

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
            use std::fmt::Write;
            let _ = write!(&mut col_def, " DEFAULT {}", default);
        }

        column_defs.push(col_def);
    }

    sql.push_str(&column_defs.join(",\n"));

    if !primary_keys.is_empty() {
        use std::fmt::Write;
        let _ = write!(&mut sql, ",\n    PRIMARY KEY ({})", primary_keys.join(", "));
    }

    sql.push_str("\n)");

    if driver == "mysql" || driver == "mariadb" {
        use std::fmt::Write;
        let _ = write!(
            &mut sql,
            " ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci"
        );
    }

    sql.push(';');
    sql
}

fn sanitize_identifier(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect()
}

pub fn load_connection_config(
    state: &AppState,
    bot_id: Uuid,
    connection_name: &str,
) -> Result<ExternalConnection, Box<dyn Error + Send + Sync>> {
    let config_manager = crate::core::config::ConfigManager::new(state.conn.clone());

    let prefix = format!("conn-{}-", connection_name);

    let server = config_manager
        .get_config(&bot_id, &format!("{}Server", prefix), None)
        .map_err(|_| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Missing {prefix}Server in config"),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;

    let database = config_manager
        .get_config(&bot_id, &format!("{}Name", prefix), None)
        .map_err(|_| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Missing {prefix}Name in config"),
            )) as Box<dyn std::error::Error + Send + Sync>
        })?;

    let username = config_manager
        .get_config(&bot_id, &format!("{}Username", prefix), None)
        .ok()
        .unwrap_or_default();

    let password = config_manager
        .get_config(&bot_id, &format!("{}Password", prefix), None)
        .ok()
        .unwrap_or_default();

    let port = config_manager
        .get_config(&bot_id, &format!("{}Port", prefix), None)
        .ok()
        .and_then(|p| p.parse().ok());

    let driver = config_manager
        .get_config(&bot_id, &format!("{}Driver", prefix), None)
        .unwrap_or_else(|_| "postgres".to_string());

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

pub fn build_connection_string(conn: &ExternalConnection) -> String {
    let port = conn.port.unwrap_or(match conn.driver.as_str() {
        "mysql" | "mariadb" => 3306,
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

pub fn store_table_definition(
    conn: &mut diesel::PgConnection,
    bot_id: Uuid,
    table: &TableDefinition,
) -> Result<Uuid, Box<dyn Error + Send + Sync>> {
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

    diesel::sql_query("DELETE FROM dynamic_table_fields WHERE table_definition_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(table_id)
        .execute(conn)?;

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

pub fn create_table_on_external_db(
    connection_string: &str,
    create_sql: &str,
    driver: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match driver {
        "mysql" | "mariadb" => create_table_mysql(connection_string, create_sql),
        "postgres" | "postgresql" => create_table_postgres(connection_string, create_sql),
        _ => {
            warn!("Unsupported driver: {}, attempting postgres", driver);
            create_table_postgres(connection_string, create_sql)
        }
    }
}

fn create_table_mysql(
    _connection_string: &str,
    _sql: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    Err("MySQL support is disabled. Please use PostgreSQL for dynamic tables.".into())
}

fn create_table_postgres(
    connection_string: &str,
    sql: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    use diesel::pg::PgConnection;
    use diesel::prelude::*;

    let mut conn = PgConnection::establish(connection_string)?;
    diesel::sql_query(sql).execute(&mut conn)?;
    info!("PostgreSQL table created successfully");
    Ok(())
}

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

        store_table_definition(&mut conn, bot_id, table)?;

        if table.connection_name == "default" {
            let create_sql = generate_create_table_sql(table, "postgres");
            info!("Creating table {} on default connection", table.name);
            trace!("SQL: {}", create_sql);

            sql_query(&create_sql).execute(&mut conn)?;
        } else {
            match load_connection_config(&state, bot_id, &table.connection_name) {
                Ok(ext_conn) => {
                    let create_sql = generate_create_table_sql(table, &ext_conn.driver);
                    let conn_string = build_connection_string(&ext_conn);

                    info!(
                        "Creating table {} on {} ({})",
                        table.name, table.connection_name, ext_conn.driver
                    );
                    trace!("SQL: {}", create_sql);

                    let driver = ext_conn.driver.clone();
                    if let Err(e) = create_table_on_external_db(&conn_string, &create_sql, &driver)
                    {
                        error!("Failed to create table on external DB: {}", e);
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to load connection config for {}: {}",
                        table.connection_name, e
                    );
                }
            }
        }
    }

    Ok(tables)
}

pub fn register_table_keywords(
    _state: Arc<AppState>,
    _user: UserSession,
    _engine: &mut rhai::Engine,
) {
    trace!("TABLE keyword registered (compile-time only)");
}
