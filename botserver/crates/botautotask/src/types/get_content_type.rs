

pub fn get_content_type(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" | "mjs" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "txt" => "text/plain",
        "csv" => "text/csv",
        "md" => "text/markdown",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "eot" => "application/vnd.ms-fontobject",
        "bas" => "text/plain",
        _ => "application/octet-stream",
    }
}
pub fn generate_create_table_sql(table: &crate::task_manifest::TableDefinition, driver: &str) -> String {
    if driver != "postgres" {
        return String::new();
    }

    let mut sql = format!("CREATE TABLE IF NOT EXISTS {} (\n", table.name);

    let mut field_lines = Vec::new();
    for field in &table.fields {
        let mut line = format!("  {}", field.name);

        let pg_type = match field.field_type.to_lowercase().as_str() {
            "guid" | "uuid" => "UUID".to_string(),
            "string" | "varchar" => "VARCHAR(255)".to_string(),
            "text" => "TEXT".to_string(),
            "integer" | "int" => "INTEGER".to_string(),
            "decimal" | "numeric" | "float" | "double" => "DECIMAL(10,2)".to_string(),
            "boolean" | "bool" => "BOOLEAN".to_string(),
            "date" => "DATE".to_string(),
            "datetime" | "timestamp" => "TIMESTAMPTZ".to_string(),
            "json" | "jsonb" => "JSONB".to_string(),
            "bigint" => "BIGINT".to_string(),
            "serial" | "autoincrement" => "SERIAL".to_string(),
            other => other.to_string(),
        };
        line.push_str(&format!(" {}", pg_type));

        if field.is_key {
            line.push_str(" PRIMARY KEY");
            if field.field_type.to_lowercase() == "guid" || field.field_type.to_lowercase() == "uuid" {
                line.push_str(" DEFAULT gen_random_uuid()");
            }
        }

        if !field.is_nullable && !field.is_key {
            line.push_str(" NOT NULL");
        }

        if let Some(ref default) = field.default_value {
            if !field.is_key {
                line.push_str(&format!(" DEFAULT {}", default));
            }
        }

        if let Some(ref _refs) = field.reference_table {
            // References handled via foreign key constraints separately
        }

        field_lines.push(line);
    }

    sql.push_str(&field_lines.join(",\n"));
    sql.push_str("\n)");
    sql
}

#[cfg(test)]
mod tests {
    use super::{generate_create_table_sql, get_content_type};
    use crate::task_manifest::{FieldDefinition, TableDefinition};

    fn field(name: &str, field_type: &str) -> FieldDefinition {
        FieldDefinition {
            name: name.to_string(),
            field_type: field_type.to_string(),
            nullable: false,
            is_key: false,
            is_nullable: false,
            default_value: None,
            reference_table: None,
        }
    }

    #[test]
    fn content_types_are_case_insensitive_and_default_to_octet_stream() {
        assert_eq!(get_content_type("index.html"), "text/html");
        assert_eq!(get_content_type("LOGO.A.PNG"), "image/png");
        assert_eq!(get_content_type("photo.jpeg"), "image/jpeg");
        assert_eq!(get_content_type("module.mjs"), "application/javascript");
        assert_eq!(get_content_type("README"), "application/octet-stream");
        assert_eq!(get_content_type("trailing."), "application/octet-stream");
        assert_eq!(get_content_type(""), "application/octet-stream");
    }

    #[test]
    fn create_table_sql_is_postgres_only_and_deterministic() {
        assert_eq!(generate_create_table_sql(&TableDefinition {
            name: "customers".to_string(),
            fields: vec![field("id", "uuid")],
        }, "mysql"), "");

        let mut id = field("id", "guid");
        id.is_key = true;
        let mut total = field("total", "decimal");
        total.default_value = Some("0".to_string());
        let table = TableDefinition {
            name: "customers".to_string(),
            fields: vec![id, field("name", "string"), total],
        };
        let sql = generate_create_table_sql(&table, "postgres");
        assert!(sql.starts_with("CREATE TABLE IF NOT EXISTS customers (\n"));
        assert!(sql.contains("  id UUID PRIMARY KEY DEFAULT gen_random_uuid()"));
        assert!(sql.contains("  name VARCHAR(255) NOT NULL"));
        assert!(sql.contains("  total DECIMAL(10,2) NOT NULL DEFAULT 0"));
        assert!(sql.ends_with("\n)"));
        assert_eq!(sql, generate_create_table_sql(&table, "postgres"));
    }
}
