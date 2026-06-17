use uuid::Uuid;
use diesel::prelude::*;
use diesel::sql_query;

#[derive(Debug, Clone)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnSchema>,
}

#[derive(Debug, Clone)]
pub struct ColumnSchema {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_key: bool,
}

pub fn get_bot_table_schemas(conn: &mut PgConnection, bot_id: Uuid) -> Result<Vec<TableSchema>, String> {
    #[derive(QueryableByName, Debug)]
    struct TableRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        table_name: String,
    }

    #[derive(QueryableByName, Debug)]
    struct ColumnRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        table_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        column_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        data_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        is_nullable: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        character_maximum_length: String,
    }

    let tables: Vec<TableRow> = sql_query(
        "SELECT table_name \
         FROM dynamic_table_definitions \
         WHERE bot_id = $1 AND is_active = true \
         ORDER BY table_name"
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .load(conn)
    .map_err(|e| format!("Failed to query bot table definitions: {e}"))?;

    if tables.is_empty() {
        return Ok(Vec::new());
    }

    let table_names: Vec<String> = tables.iter().map(|t| t.table_name.clone()).collect();
    let table_list = table_names
        .iter()
        .map(|n| format!("'{}'", n.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(",");

    let columns_raw: Vec<ColumnRow> = sql_query(format!(
        "SELECT c.table_name::text, c.column_name::text, \
                c.data_type::text, c.is_nullable::text, \
                COALESCE(c.character_maximum_length::text, '') as character_maximum_length \
         FROM information_schema.columns c \
         WHERE c.table_schema = 'public' \
           AND c.table_name IN ({table_list}) \
         ORDER BY c.table_name, c.ordinal_position"
    ))
    .load(conn)
    .map_err(|e| format!("Failed to query materialized columns: {e}"))?;

    let result: Vec<TableSchema> = table_names
        .iter()
        .map(|name| {
            let cols: Vec<ColumnSchema> = columns_raw
                .iter()
                .filter(|c| c.table_name == *name)
                .map(|c| {
                    let pg_type = &c.data_type;
                    let display_type = if pg_type == "character varying" || pg_type == "character" {
                        if c.character_maximum_length.is_empty() {
                            "VARCHAR".to_string()
                        } else {
                            format!("VARCHAR({})", c.character_maximum_length)
                        }
                    } else if pg_type == "numeric" {
                        "NUMERIC".to_string()
                    } else if pg_type == "timestamp with time zone" || pg_type == "timestamp without time zone" {
                        "TIMESTAMP".to_string()
                    } else {
                        pg_type.to_uppercase()
                    };
                    ColumnSchema {
                        name: c.column_name.clone(),
                        data_type: display_type,
                        nullable: c.is_nullable == "YES",
                        is_key: c.column_name == "id",
                    }
                })
                .collect();
            TableSchema {
                name: name.clone(),
                columns: cols,
            }
        })
        .filter(|t| !t.columns.is_empty())
        .collect();

    Ok(result)
}

pub fn format_schemas_as_prompt(tables: &[TableSchema]) -> String {
    let mut result = String::new();
    result.push_str("Database tables defined in tables.bas (materialized):\n\n");
    for table in tables {
        let key_cols: Vec<&str> = table.columns
            .iter()
            .filter(|c| c.is_key)
            .map(|c| c.name.as_str())
            .collect();
        let key_info = if key_cols.is_empty() {
            String::new()
        } else {
            format!(" [primary key: {}]", key_cols.join(", "))
        };
        result.push_str(&format!("Table: {}{}\n", table.name, key_info));
        for col in &table.columns {
            let null_str = if col.nullable { "nullable" } else { "not null" };
            let key_marker = if col.is_key { " KEY" } else { "" };
            result.push_str(&format!(
                "  - {}{} ({}, {})\n",
                col.name, key_marker, col.data_type, null_str
            ));
        }
        result.push('\n');
    }
    result
}
