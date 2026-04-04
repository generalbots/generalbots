/*****************************************************************************\
|  Table Schema Migration Module
|  Automatically syncs table.bas definitions with database schema
\*****************************************************************************/

use crate::core::shared::sanitize_identifier;
use crate::core::shared::state::AppState;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use log::{error, info, warn};
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

use super::table_definition::{
    map_type_to_sql, parse_table_definition, FieldDefinition, TableDefinition,
};

/// Schema migration result
#[derive(Debug, Default)]
pub struct MigrationResult {
    pub tables_created: usize,
    pub tables_altered: usize,
    pub columns_added: usize,
    pub columns_dropped: usize,
    pub errors: Vec<String>,
}

/// Column metadata from database
#[derive(Debug, Clone)]
pub struct DbColumn {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
}

/// Columns that should never be dropped automatically
const PROTECTED_COLUMNS: &[&str] = &[
    "id",
    "bot_id",
    "org_id",
    "user_id",
    "created_at",
    "updated_at",
    "deleted_at",
    "is_deleted",
    "version",
    "tenant_id",
];

/// Compare and sync table schema with definition
pub fn sync_table_schema(
    table: &TableDefinition,
    existing_columns: &[DbColumn],
    create_sql: &str,
    conn: &mut diesel::PgConnection,
) -> Result<MigrationResult, Box<dyn Error + Send + Sync>> {
    let mut result = MigrationResult::default();

    // If no columns exist, create the table
    if existing_columns.is_empty() {
        info!("Creating new table: {}", table.name);
        sql_query(create_sql)
            .execute(conn)
            .map_err(|e| format!("Failed to create table {}: {}", table.name, e))?;
        result.tables_created += 1;
        return Ok(result);
    }

    let table_name = sanitize_identifier(&table.name);
    let defined_col_names: std::collections::HashSet<String> =
        table.fields.iter().map(|f| f.name.clone()).collect();
    let existing_col_names: std::collections::HashSet<String> =
        existing_columns.iter().map(|c| c.name.clone()).collect();

    // Add missing columns
    let mut missing_columns: Vec<&FieldDefinition> = Vec::new();
    for field in &table.fields {
        if !existing_col_names.contains(&field.name) {
            missing_columns.push(field);
        }
    }

    if !missing_columns.is_empty() {
        info!(
            "Table {} is missing {} columns, adding them",
            table.name,
            missing_columns.len()
        );

        for field in &missing_columns {
            let sql_type = map_type_to_sql(field, "postgres");
            let column_sql = format!(
                "ALTER TABLE {} ADD COLUMN IF NOT EXISTS {} {}",
                &table_name,
                sanitize_identifier(&field.name),
                sql_type
            );

            info!(
                "Adding column: {}.{} ({})",
                table.name, field.name, sql_type
            );
            match sql_query(&column_sql).execute(conn) {
                Ok(_) => {
                    result.columns_added += 1;
                    info!("Successfully added column {}.{}", table.name, field.name);
                }
                Err(e) => {
                    let err_str = e.to_string();
                    if !err_str.contains("already exists") && !err_str.contains("duplicate column")
                    {
                        let error_msg =
                            format!("Failed to add column {}.{}: {}", table.name, field.name, e);
                        error!("{}", error_msg);
                        result.errors.push(error_msg);
                    } else {
                        info!(
                            "Column {}.{} already exists, skipping",
                            table.name, field.name
                        );
                    }
                }
            }
        }
        result.tables_altered += 1;
    }

    // Drop columns that were removed from definition (with protection)
    let mut orphaned_columns: Vec<&DbColumn> = Vec::new();
    for col in existing_columns {
        if !defined_col_names.contains(&col.name) && !PROTECTED_COLUMNS.contains(&col.name.as_str())
        {
            orphaned_columns.push(col);
        }
    }

    if !orphaned_columns.is_empty() {
        warn!(
            "Table {} has {} orphaned columns not in definition:",
            table.name,
            orphaned_columns.len()
        );

        for col in &orphaned_columns {
            info!("Dropping orphaned column: {}.{}", table.name, col.name);
            let drop_sql = format!(
                "ALTER TABLE {} DROP COLUMN IF EXISTS {}",
                &table_name,
                sanitize_identifier(&col.name)
            );

            match sql_query(&drop_sql).execute(conn) {
                Ok(_) => {
                    result.columns_dropped += 1;
                    info!("Successfully dropped column {}.{}", table.name, col.name);
                }
                Err(e) => {
                    let error_msg =
                        format!("Failed to drop column {}.{}: {}", table.name, col.name, e);
                    error!("{}", error_msg);
                    result.errors.push(error_msg);
                }
            }
        }
        if result.columns_dropped > 0 {
            result.tables_altered += 1;
        }
    }

    Ok(result)
}

/// Get existing columns from a table
pub fn get_table_columns(
    table_name: &str,
    conn: &mut diesel::PgConnection,
) -> Result<Vec<DbColumn>, Box<dyn Error + Send + Sync>> {
    let query = format!(
        "SELECT column_name, data_type, is_nullable
         FROM information_schema.columns
         WHERE table_name = '{}' AND table_schema = 'public'
         ORDER BY ordinal_position",
        sanitize_identifier(table_name)
    );

    #[derive(QueryableByName)]
    struct ColumnRow {
        #[diesel(sql_type = Text)]
        column_name: String,
        #[diesel(sql_type = Text)]
        data_type: String,
        #[diesel(sql_type = Text)]
        is_nullable: String,
    }

    let rows: Vec<ColumnRow> = match sql_query(&query).load(conn) {
        Ok(r) => r,
        Err(e) => {
            // Table doesn't exist
            return Err(format!("Table {} does not exist: {}", table_name, e).into());
        }
    };

    Ok(rows
        .into_iter()
        .map(|row| DbColumn {
            name: row.column_name,
            data_type: row.data_type,
            is_nullable: row.is_nullable == "YES",
        })
        .collect())
}

/// Process table definitions with schema sync for a specific bot
pub fn sync_bot_tables(
    state: &Arc<AppState>,
    bot_id: Uuid,
    source: &str,
) -> Result<MigrationResult, Box<dyn Error + Send + Sync>> {
    let tables = parse_table_definition(source)?;
    let mut result = MigrationResult::default();

    info!(
        "Processing {} table definitions with schema sync for bot {}",
        tables.len(),
        bot_id
    );

    // Get bot's database connection
    let pool = state.bot_database_manager.get_bot_pool(bot_id)?;
    let mut conn = pool.get()?;

    for table in &tables {
        if table.connection_name != "default" {
            continue; // Skip external connections for now
        }

        info!("Syncing table: {}", table.name);

        // Get existing columns
        let existing_columns = get_table_columns(&table.name, &mut conn).unwrap_or_default();

        // Generate CREATE TABLE SQL
        let create_sql = super::table_definition::generate_create_table_sql(table, "postgres");

        // Sync schema
        match sync_table_schema(table, &existing_columns, &create_sql, &mut conn) {
            Ok(sync_result) => {
                result.tables_created += sync_result.tables_created;
                result.tables_altered += sync_result.tables_altered;
                result.columns_added += sync_result.columns_added;
                result.columns_dropped += sync_result.columns_dropped;
                result.errors.extend(sync_result.errors);
            }
            Err(e) => {
                let error_msg = format!("Failed to sync table {}: {}", table.name, e);
                error!("{}", error_msg);
                result.errors.push(error_msg);
            }
        }
    }

    // Log summary
    info!("Schema sync summary for bot {}: {} tables created, {} altered, {} columns added, {} columns dropped, {} errors",
        bot_id, result.tables_created, result.tables_altered, result.columns_added, result.columns_dropped, result.errors.len());

    if !result.errors.is_empty() {
        warn!("Schema sync completed with {} errors:", result.errors.len());
        for error in &result.errors {
            warn!("  - {}", error);
        }
    }

    Ok(result)
}

/// Validate that all required columns exist
pub fn validate_table_schema(
    table_name: &str,
    required_fields: &[FieldDefinition],
    conn: &mut diesel::PgConnection,
) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let existing_columns = get_table_columns(table_name, conn)?;
    let existing_col_names: std::collections::HashSet<String> =
        existing_columns.iter().map(|c| c.name.clone()).collect();

    let mut missing = Vec::new();
    for field in required_fields {
        if !existing_col_names.contains(&field.name) {
            missing.push(field.name.clone());
        }
    }

    if !missing.is_empty() {
        warn!("Table {} is missing columns: {:?}", table_name, missing);
        return Ok(false);
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_column_creation() {
        let col = DbColumn {
            name: "test_col".to_string(),
            data_type: "character varying".to_string(),
            is_nullable: true,
        };
        assert_eq!(col.name, "test_col");
        assert_eq!(col.is_nullable, true);
    }
}
