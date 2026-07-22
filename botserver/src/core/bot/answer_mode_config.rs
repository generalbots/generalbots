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
        column_name: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        data_type: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        character_maximum_length: String,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_nullable: bool,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_key: bool,
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

    let result: Vec<TableSchema> = tables
        .iter()
        .map(|table| {
            let cols: Vec<ColumnSchema> = sql_query(
                "SELECT dtf.field_name::text as column_name, dtf.field_type::text as data_type, \
                        COALESCE(dtf.field_length::text, '') as character_maximum_length, \
                        dtf.is_nullable as is_nullable, dtf.is_key as is_key \
                 FROM dynamic_table_fields dtf \
                 JOIN dynamic_table_definitions dtd ON dtf.table_definition_id = dtd.id \
                 WHERE dtd.bot_id = $1 AND dtd.table_name = $2 \
                 ORDER BY dtf.field_order"
            )
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .bind::<diesel::sql_types::Varchar, _>(&table.table_name)
            .load::<ColumnRow>(conn)
            .unwrap_or_default()
            .into_iter()
            .map(|c| {
                let field_type = &c.data_type;
                let display_type = if field_type == "string" {
                    if c.character_maximum_length.is_empty() {
                        "VARCHAR".to_string()
                    } else {
                        format!("VARCHAR({})", c.character_maximum_length)
                    }
                } else if field_type == "integer" {
                    "INTEGER".to_string()
                } else if field_type == "date" {
                    "DATE".to_string()
                } else if field_type == "datetime" {
                    "TIMESTAMP".to_string()
                } else if field_type == "numeric" {
                    "NUMERIC".to_string()
                } else {
                    field_type.to_uppercase()
                };
                ColumnSchema {
                    name: c.column_name,
                    data_type: display_type,
                    nullable: c.is_nullable,
                    is_key: c.is_key,
                }
            })
            .collect();
            TableSchema {
                name: table.table_name.clone(),
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
